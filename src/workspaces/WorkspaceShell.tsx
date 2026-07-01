import { JSX, ParentProps } from "solid-js";

type WorkspaceSection = {
  value: string;
  label: string;
  targetId: string;
  description: string;
};

type WorkspaceShellProps = ParentProps<{
  activeWorkspace: string;
  activeLabel: string;
  activeDescription: string;
  workspaceSections: WorkspaceSection[];
  onActivate: (workspace: string) => void;
}>;

export default function WorkspaceShell(props: WorkspaceShellProps): JSX.Element {
  return (
    <main class="app-shell" data-active-workspace={props.activeWorkspace}>
      <aside class="workspace-nav" aria-label="Workspace navigation">
        <p class="eyebrow">Workspaces</p>
        <div class="workspace-nav-list">
          {props.workspaceSections.map((item) => (
            <button
              type="button"
              classList={{ active: props.activeWorkspace === item.value }}
              onClick={() => props.onActivate(item.value)}
            >
              <span>{item.label}</span>
              <small>{item.description}</small>
            </button>
          ))}
        </div>
      </aside>

      <div class="workspace-content">
        <section class="workspace-banner">
          <h1>{props.activeLabel || "Scholar Chat"}</h1>
          <p class="muted">{props.activeDescription || "Chat-first academic workflow"}</p>
        </section>
        {props.children}
      </div>
    </main>
  );
}
