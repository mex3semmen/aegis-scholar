import { expect, test, type Locator, type Page } from "@playwright/test";

type TauriInvokeHandler = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
type BrowserSmokeTauriInternals = {
  invoke: TauriInvokeHandler;
  convertFileSrc: (value: string) => string;
  transformCallback: () => string;
  unregisterCallback: () => void;
};

function installBrowserSmokeTauriShim() {
  const invoke: TauriInvokeHandler = async (cmd, _args = {}) => {
    switch (cmd) {
      case "get_corpus_status":
        return {
          source_count: 1,
          registered_count: 1,
          extracted_count: 1,
          failed_count: 0,
        };
      case "list_sources":
        return [
          {
            source_id: "browser-smoke-source",
            version_id: "browser-smoke-version",
            title: "Browser smoke source",
            source_type: "markdown_note",
            ingestion_status: "ready",
          },
        ];
      case "inspect_managed_llama_server_status":
        return {
          lifecycle_status: "not_started",
          health_status: "not_started",
          owns_active_server: false,
          port_occupied: false,
          port_occupied_by_unmanaged_process: false,
          port_occupancy_status: "free",
          host: null,
          port: null,
          alias: null,
          process_id: null,
          exit_code: null,
          safe_executable_file_name: null,
          safe_model_file_name: null,
          health_url: null,
          response_body_preview: "",
          response_body_truncated: false,
          blockers: [],
          warnings: [],
          next_required_actions: [],
          summary: "No managed llama-server is running.",
          preview_only: true,
          no_process_spawn: true,
          no_model_output_used: true,
          no_answer_generation: true,
          no_persistence: true,
          no_artifact_write: true,
          no_lan_binding_by_default: true,
        };
      case "list_scholar_chat_sessions":
        return [
          {
            session_id: "browser-smoke-session",
            title: "Browser smoke session",
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
            message_count: 1,
            last_message_at: 1_700_000_000_000,
          },
        ];
      case "load_scholar_chat_session_transcript":
        return [
          {
            id: 1,
            role: "user",
            kind: "prompt",
            prompt: "Browser smoke transcript",
            title: "Browser smoke transcript",
            content: "Browser smoke transcript",
            created_at: 1_700_000_000_000,
          },
        ];
      default:
        throw new Error(`Unexpected Tauri command in browser smoke: ${cmd}`);
    }
  };

  // Make the browser run look enough like Tauri for the mount-time invoke calls to succeed.
  // Only the known read-only commands are allowed to return data.
  const globalWindow = window as Window & {
    isTauri?: boolean;
    __TAURI_INTERNALS__?: BrowserSmokeTauriInternals;
  };
  globalWindow.isTauri = true;
  globalWindow.__TAURI_INTERNALS__ = {
    invoke,
    convertFileSrc: (value: string) => value,
    transformCallback: () => "browser-smoke-callback",
    unregisterCallback: () => {},
  };
}

async function expectWorkspaceReachable(
  page: Page,
  workspaceNav: Locator,
  options: {
    buttonName: string | RegExp;
    headingLevel: number;
    headingName: string;
    workspaceMarker?: string;
  },
) {
  const workspaceButton = workspaceNav.getByRole("button", { name: options.buttonName });
  await expect(workspaceButton).toBeVisible();
  await workspaceButton.click();
  await expect(page.getByRole("heading", { level: options.headingLevel, name: options.headingName })).toBeVisible();
  if (options.workspaceMarker) {
    await expect(page.locator(`[data-workspace="${options.workspaceMarker}"]`)).toBeVisible();
  }
}

type BrowserSmokeErrorCapture = {
  pageErrors: Error[];
  consoleErrors: string[];
  assertNoErrors: () => void;
  waitForCounts: (expected: { pageErrors: number; consoleErrors: number }) => Promise<void>;
};

function installBrowserErrorCapture(page: Page): BrowserSmokeErrorCapture {
  const pageErrors: Error[] = [];
  const consoleErrors: string[] = [];

  page.on("pageerror", (error) => {
    pageErrors.push(error);
  });
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });

  return {
    pageErrors,
    consoleErrors,
    assertNoErrors: () => {
      expect(pageErrors).toEqual([]);
      expect(consoleErrors).toEqual([]);
    },
    waitForCounts: async (expected) => {
      await expect.poll(() => ({
        pageErrors: pageErrors.length,
        consoleErrors: consoleErrors.length,
      })).toEqual(expected);
    },
  };
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installBrowserSmokeTauriShim);
});

test("loads the shell and Scholar Chat smoke surface", async ({ page }) => {
  const browserErrors = installBrowserErrorCapture(page);

  await page.goto("/");

  await expect(page.locator(".app-shell")).toBeVisible();
  await expect(page.getByRole("complementary", { name: "Workspace navigation" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Ask locally, preview first" })).toBeVisible();
  await expect(page.getByRole("complementary", { name: "Scholar Chat sessions" })).toBeVisible();
  await expect(page.getByText("Browser smoke session")).toBeVisible();
  await expect(page.getByText("Transcript loaded. Composer state stays in memory.")).toBeVisible();
  const workspaceNav = page.getByRole("complementary", { name: "Workspace navigation" });
  await expectWorkspaceReachable(page, workspaceNav, {
    buttonName: /Sources/,
    headingLevel: 2,
    headingName: "Sources",
    workspaceMarker: "sources",
  });
  await expectWorkspaceReachable(page, workspaceNav, {
    buttonName: /Evidence Packs/,
    headingLevel: 1,
    headingName: "Evidence Packs",
    workspaceMarker: "evidence_packs",
  });
  await expectWorkspaceReachable(page, workspaceNav, {
    buttonName: "Artifacts & Diagnostics",
    headingLevel: 1,
    headingName: "Artifacts & Diagnostics",
    workspaceMarker: "developer_diagnostics",
  });
  const hasTauriShim = await page.evaluate(() => {
    const globalWindow = window as Window & { isTauri?: boolean; __TAURI_INTERNALS__?: { invoke?: unknown } };
    return Boolean(globalWindow.isTauri && globalWindow.__TAURI_INTERNALS__?.invoke);
  });
  expect(hasTauriShim).toBe(true);
  browserErrors.assertNoErrors();
});

test("captures browser runtime errors", async ({ page }) => {
  const browserErrors = installBrowserErrorCapture(page);

  await page.setContent("<!doctype html><html><head><title>Browser smoke sentinel</title></head><body>Browser smoke sentinel</body></html>");
  await page.evaluate(() => {
    console.error("browser smoke sentinel console error");
    setTimeout(() => {
      throw new Error("browser smoke sentinel page error");
    }, 0);
  });

  await browserErrors.waitForCounts({ pageErrors: 1, consoleErrors: 1 });
  expect(browserErrors.consoleErrors).toEqual(["browser smoke sentinel console error"]);
  expect(browserErrors.pageErrors.map((error) => error.message)).toEqual(["browser smoke sentinel page error"]);
});
