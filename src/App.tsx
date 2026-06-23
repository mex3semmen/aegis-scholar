import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

type CorpusStatus = {
  source_count: number;
  registered_count: number;
  extracted_count: number;
  failed_count: number;
};

type CitationLocator = {
  label: string;
  section?: string | null;
  paragraph_index?: number | null;
  start_char: number;
  end_char: number;
  [key: string]: unknown;
};

type FinalAnswerStatement = {
  statement_id: string;
  grounded_statement_id: string;
  status: "supported" | "needs_evidence" | "unsupported";
  text: string;
  claim_ids: string[];
  evidence_ids: string[];
  chunk_ids: string[];
  locators: CitationLocator[];
  support_level: "direct_grounded_statement" | "missing_evidence";
};

type FinalAnswer = {
  final_answer_id: string;
  grounded_answer_id: string;
  source_id: string;
  version_id: string;
  query: string;
  created_at: string;
  answer_mode: "contract_only";
  statement_count: number;
  unsupported_count: number;
  statements: FinalAnswerStatement[];
  warnings: string[];
};

type FinalAnswerMetadata = {
  final_answer_id: string;
  grounded_answer_id: string;
  statement_count: number;
  unsupported_count: number;
  needs_evidence_count: number;
};

type AnswerArtifactOverview = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  final_answers: FinalAnswerMetadata[];
};

type AnswerArtifactSourceMetadata = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
};

type AnswerArtifactSourceHealth = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  malformed_final_answer_count: number;
  unsupported_statement_count: number;
  needs_evidence_statement_count: number;
};

type AnswerArtifactHealth = {
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  malformed_final_answer_count: number;
  unsupported_statement_count: number;
  needs_evidence_statement_count: number;
  sources: AnswerArtifactSourceHealth[];
};

type AnswerArtifactIssue = {
  source_id: string;
  issue_kind: "malformed_final_answer" | "unsupported_statement" | "needs_evidence_statement";
  final_answer_id?: string | null;
  grounded_answer_id?: string | null;
  statement_index?: number | null;
  statement_status?: string | null;
  message: string;
};

type AnswerArtifactExportSource = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  final_answers: FinalAnswerMetadata[];
  issue_count: number;
};

type AnswerArtifactExportManifest = {
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
  sources: AnswerArtifactExportSource[];
};

type ExportedArtifactFile = {
  relative_path: string;
  artifact_kind: "manifest" | "issues" | "answer_draft" | "grounded_answer" | "final_answer";
  source_id?: string | null;
  artifact_id?: string | null;
};

type AnswerArtifactExportResult = {
  manifest: AnswerArtifactExportManifest;
  exported_source_count: number;
  exported_draft_count: number;
  exported_grounded_answer_count: number;
  exported_final_answer_count: number;
  exported_issue_count: number;
  export_id: string;
  written_files: ExportedArtifactFile[];
};

function sanitizeBackendError(error: unknown) {
  const message = String(error);
  return message.replace(/[A-Za-z]:\\[^"]+/g, "[path hidden]").replace(/E:\\[^"]+/g, "[path hidden]");
}

function locatorSummary(locator: CitationLocator) {
  const section = locator.section ? `section=${locator.section}` : null;
  const paragraph = locator.paragraph_index !== null && locator.paragraph_index !== undefined ? `paragraph=${locator.paragraph_index}` : null;
  const range = `chars=${locator.start_char}-${locator.end_char}`;
  return [locator.label, section, paragraph, range].filter(Boolean).join(" | ");
}

export default function App() {
  const [status, setStatus] = createSignal<CorpusStatus | null>(null);
  const [statusError, setStatusError] = createSignal<string | null>(null);
  const [sourceId, setSourceId] = createSignal("");
  const [finalAnswerId, setFinalAnswerId] = createSignal("");
  const [finalAnswer, setFinalAnswer] = createSignal<FinalAnswer | null>(null);
  const [finalAnswerError, setFinalAnswerError] = createSignal<string | null>(null);
  const [finalAnswerLoading, setFinalAnswerLoading] = createSignal(false);
  const [artifactOverview, setArtifactOverview] = createSignal<AnswerArtifactOverview | null>(null);
  const [artifactOverviewError, setArtifactOverviewError] = createSignal<string | null>(null);
  const [artifactOverviewLoading, setArtifactOverviewLoading] = createSignal(false);
  const [artifactSources, setArtifactSources] = createSignal<AnswerArtifactSourceMetadata[]>([]);
  const [artifactSourcesError, setArtifactSourcesError] = createSignal<string | null>(null);
  const [artifactSourcesLoading, setArtifactSourcesLoading] = createSignal(false);
  const [artifactHealth, setArtifactHealth] = createSignal<AnswerArtifactHealth | null>(null);
  const [artifactHealthError, setArtifactHealthError] = createSignal<string | null>(null);
  const [artifactHealthLoading, setArtifactHealthLoading] = createSignal(false);
  const [artifactIssues, setArtifactIssues] = createSignal<AnswerArtifactIssue[]>([]);
  const [artifactIssuesError, setArtifactIssuesError] = createSignal<string | null>(null);
  const [artifactIssuesLoading, setArtifactIssuesLoading] = createSignal(false);
  const [artifactManifest, setArtifactManifest] = createSignal<AnswerArtifactExportManifest | null>(null);
  const [artifactManifestError, setArtifactManifestError] = createSignal<string | null>(null);
  const [artifactManifestLoading, setArtifactManifestLoading] = createSignal(false);
  const [exportRoot, setExportRoot] = createSignal("");
  const [artifactExportResult, setArtifactExportResult] = createSignal<AnswerArtifactExportResult | null>(null);
  const [artifactExportError, setArtifactExportError] = createSignal<string | null>(null);
  const [artifactExportLoading, setArtifactExportLoading] = createSignal(false);

  async function loadStatus() {
    setStatusError(null);
    try {
      const result = await invoke<CorpusStatus>("get_corpus_status", {
        root: ".",
      });
      setStatus(result);
    } catch (err) {
      setStatusError(String(err));
    }
  }

  async function loadFinalAnswer() {
    await loadFinalAnswerByIds(sourceId().trim(), finalAnswerId().trim());
  }

  async function loadFinalAnswerByIds(trimmedSourceId: string, trimmedFinalAnswerId: string) {
    if (!trimmedSourceId || !trimmedFinalAnswerId) {
      setFinalAnswerError("Source ID and final answer ID are required.");
      return;
    }

    if (finalAnswerLoading()) {
      return;
    }

    setFinalAnswerLoading(true);
    setFinalAnswerError(null);
    try {
      const result = await invoke<FinalAnswer>("get_final_answer", {
        root: ".",
        source_id: trimmedSourceId,
        final_answer_id: trimmedFinalAnswerId,
      });
      setFinalAnswer(result);
    } catch (err) {
      setFinalAnswerError(sanitizeBackendError(err));
    } finally {
      setFinalAnswerLoading(false);
    }
  }

  async function loadArtifactOverview() {
    await loadArtifactOverviewBySourceId(sourceId().trim());
  }

  async function loadArtifactSources() {
    if (artifactSourcesLoading()) {
      return;
    }
    setArtifactSourcesLoading(true);
    setArtifactSourcesError(null);
    try {
      const result = await invoke<AnswerArtifactSourceMetadata[]>("list_answer_artifact_sources", {
        root: ".",
      });
      setArtifactSources(result);
    } catch (err) {
      setArtifactSources([]);
      setArtifactSourcesError(sanitizeBackendError(err));
    } finally {
      setArtifactSourcesLoading(false);
    }
  }

  async function loadArtifactHealth() {
    if (artifactHealthLoading()) {
      return;
    }
    setArtifactHealthLoading(true);
    setArtifactHealthError(null);
    try {
      const result = await invoke<AnswerArtifactHealth>("get_answer_artifact_health", {
        root: ".",
      });
      setArtifactHealth(result);
    } catch (err) {
      setArtifactHealth(null);
      setArtifactHealthError(sanitizeBackendError(err));
    } finally {
      setArtifactHealthLoading(false);
    }
  }

  async function loadArtifactIssues() {
    if (artifactIssuesLoading()) {
      return;
    }
    setArtifactIssuesLoading(true);
    setArtifactIssuesError(null);
    try {
      const result = await invoke<AnswerArtifactIssue[]>("list_answer_artifact_issues", {
        root: ".",
      });
      setArtifactIssues(result);
    } catch (err) {
      setArtifactIssues([]);
      setArtifactIssuesError(sanitizeBackendError(err));
    } finally {
      setArtifactIssuesLoading(false);
    }
  }

  async function loadArtifactManifest() {
    if (artifactManifestLoading()) {
      return;
    }
    setArtifactManifestLoading(true);
    setArtifactManifestError(null);
    try {
      const result = await invoke<AnswerArtifactExportManifest>("get_answer_artifact_export_manifest", {
        root: ".",
      });
      setArtifactManifest(result);
    } catch (err) {
      setArtifactManifest(null);
      setArtifactManifestError(sanitizeBackendError(err));
    } finally {
      setArtifactManifestLoading(false);
    }
  }

  async function exportArtifacts() {
    const trimmedExportRoot = exportRoot().trim();
    if (!trimmedExportRoot) {
      setArtifactExportError("Export destination is required.");
      return;
    }
    if (artifactExportLoading()) {
      return;
    }
    setArtifactExportLoading(true);
    setArtifactExportError(null);
    try {
      const result = await invoke<AnswerArtifactExportResult>("export_answer_artifacts", {
        root: ".",
        export_root: trimmedExportRoot,
      });
      setArtifactExportResult(result);
    } catch (err) {
      setArtifactExportResult(null);
      setArtifactExportError(sanitizeBackendError(err));
    } finally {
      setArtifactExportLoading(false);
    }
  }

  async function selectArtifactSource(item: AnswerArtifactSourceMetadata) {
    setSourceId(item.source_id);
    await loadArtifactOverviewBySourceId(item.source_id);
  }

  async function selectArtifactSourceId(source_id: string) {
    setSourceId(source_id);
    await loadArtifactOverviewBySourceId(source_id);
  }

  async function loadArtifactOverviewBySourceId(trimmedSourceId: string) {
    if (!trimmedSourceId) {
      setArtifactOverviewError("Source ID is required to load the artifact overview.");
      return;
    }
    if (artifactOverviewLoading()) {
      return;
    }
    setArtifactOverviewLoading(true);
    setArtifactOverviewError(null);
    try {
      const result = await invoke<AnswerArtifactOverview>("get_answer_artifact_overview", {
        root: ".",
        source_id: trimmedSourceId,
      });
      setArtifactOverview(result);
    } catch (err) {
      setArtifactOverview(null);
      setArtifactOverviewError(sanitizeBackendError(err));
    } finally {
      setArtifactOverviewLoading(false);
    }
  }

  async function selectFinalAnswer(finalAnswerMetadata: FinalAnswerMetadata) {
    setFinalAnswerId(finalAnswerMetadata.final_answer_id);
    await loadFinalAnswerByIds(sourceId().trim(), finalAnswerMetadata.final_answer_id);
  }

  return (
    <main class="app-shell">
      <section class="hero">
        <p class="eyebrow">AEGIS Scholar</p>
        <h1>Scientific knowledge OS</h1>
        <p>
          Minimal Phase 1 scaffold for Corpus Authority and Source Registry.
          No RAG, no model runtime, no coding-agent behavior.
        </p>
        <div class="hero-actions">
          <button onClick={loadStatus}>Check corpus status</button>
        </div>
      </section>

      <section class="card">
        <h2>Corpus status</h2>
        {status() ? (
          <pre>{JSON.stringify(status(), null, 2)}</pre>
        ) : (
          <p>No status loaded yet.</p>
        )}
        {statusError() && <p class="error">{statusError()}</p>}
      </section>

      <section class="card">
        <h2>Final answer inspector</h2>
        <p class="muted">
          Read-only display for an already-built FinalAnswer contract.
        </p>
        <div class="form-row">
          <label>
            Source ID
            <input
              type="text"
              value={sourceId()}
              onInput={(event) => setSourceId(event.currentTarget.value)}
              placeholder="src_..."
            />
          </label>
          <label>
            Final answer ID
            <input
              type="text"
              value={finalAnswerId()}
              onInput={(event) => setFinalAnswerId(event.currentTarget.value)}
              placeholder="fan_..."
            />
          </label>
        </div>
        <div class="hero-actions">
          <button onClick={loadFinalAnswer} disabled={finalAnswerLoading()}>
            {finalAnswerLoading() ? "Loading..." : "Load final answer"}
          </button>
          <button onClick={loadArtifactOverview} disabled={artifactOverviewLoading()}>
            {artifactOverviewLoading() ? "Loading..." : "Load artifact overview"}
          </button>
          <button onClick={loadArtifactSources} disabled={artifactSourcesLoading()}>
            {artifactSourcesLoading() ? "Loading..." : "Load source index"}
          </button>
          <button onClick={loadArtifactHealth} disabled={artifactHealthLoading()}>
            {artifactHealthLoading() ? "Loading..." : "Load artifact health"}
          </button>
          <button onClick={loadArtifactIssues} disabled={artifactIssuesLoading()}>
            {artifactIssuesLoading() ? "Loading..." : "Load artifact issues"}
          </button>
          <button onClick={loadArtifactManifest} disabled={artifactManifestLoading()}>
            {artifactManifestLoading() ? "Loading..." : "Load export manifest"}
          </button>
          <label class="inline-field">
            Export destination
            <input
              type="text"
              value={exportRoot()}
              onInput={(event) => setExportRoot(event.currentTarget.value)}
              placeholder="E:\\path\\to\\export"
            />
          </label>
          <button onClick={exportArtifacts} disabled={artifactExportLoading()}>
            {artifactExportLoading() ? "Loading..." : "Export artifacts"}
          </button>
        </div>
        {finalAnswerError() && <p class="error">{finalAnswerError()}</p>}
        {artifactOverviewError() && <p class="error">{artifactOverviewError()}</p>}
        {artifactSourcesError() && <p class="error">{artifactSourcesError()}</p>}
        {artifactHealthError() && <p class="error">{artifactHealthError()}</p>}
        {artifactIssuesError() && <p class="error">{artifactIssuesError()}</p>}
        {artifactManifestError() && <p class="error">{artifactManifestError()}</p>}
        {artifactExportError() && <p class="error">{artifactExportError()}</p>}
        <div class="artifact-overview">
          <h3>Sources with artifacts</h3>
          {artifactSources().length > 0 ? (
            <ul class="final-answer-list-items">
              {artifactSources().map((item) => (
                <li>
                  <button class="final-answer-list-item" onClick={() => selectArtifactSource(item)}>
                    <span>{item.source_id}</span>
                    <small>
                      drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count}
                    </small>
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p>No sources with artifacts listed yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Artifact health</h3>
          {artifactHealth() ? (
            <>
              <div class="contract-meta">
                <div><span>Sources</span><strong>{artifactHealth()!.source_count}</strong></div>
                <div><span>Drafts</span><strong>{artifactHealth()!.draft_count}</strong></div>
                <div><span>Grounded answers</span><strong>{artifactHealth()!.grounded_answer_count}</strong></div>
                <div><span>Final answers</span><strong>{artifactHealth()!.final_answer_count}</strong></div>
                <div><span>Malformed finals</span><strong>{artifactHealth()!.malformed_final_answer_count}</strong></div>
                <div><span>Unsupported statements</span><strong>{artifactHealth()!.unsupported_statement_count}</strong></div>
                <div><span>Needs evidence</span><strong>{artifactHealth()!.needs_evidence_statement_count}</strong></div>
              </div>
              {artifactHealth()!.sources.length > 0 ? (
                <ul class="final-answer-list-items">
                  {artifactHealth()!.sources.map((item) => (
                    <li>
                      <button class="final-answer-list-item" onClick={() => selectArtifactSource(item)}>
                        <span>{item.source_id}</span>
                        <small>
                          drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count} | malformed={item.malformed_final_answer_count} | needs_evidence={item.needs_evidence_statement_count} | unsupported={item.unsupported_statement_count}
                        </small>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No artifact health entries yet.</p>
              )}
            </>
          ) : (
            <p>No artifact health loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Artifact issues</h3>
          {artifactIssues().length > 0 ? (
            <>
              <p class="muted">Issues: {artifactIssues().length}</p>
              <ul class="final-answer-list-items">
                {artifactIssues().map((item) => (
                  <li>
                    <div class="final-answer-list-item">
                      <span>
                        <button class="link-button" onClick={() => selectArtifactSourceId(item.source_id)}>
                          {item.source_id}
                        </button>
                      </span>
                      <small>
                        {item.issue_kind}
                        {item.final_answer_id ? ` | ${item.final_answer_id}` : ""}
                        {item.statement_index !== null && item.statement_index !== undefined ? ` | statement=${item.statement_index}` : ""}
                        {item.statement_status ? ` | status=${item.statement_status}` : ""}
                        {item.grounded_answer_id ? ` | grounded=${item.grounded_answer_id}` : ""}
                      </small>
                      <p>{item.message}</p>
                    </div>
                  </li>
                ))}
              </ul>
            </>
          ) : (
            <p>No artifact issues loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Export manifest</h3>
          {artifactManifest() ? (
            <>
              <div class="contract-meta">
                <div><span>Sources</span><strong>{artifactManifest()!.source_count}</strong></div>
                <div><span>Drafts</span><strong>{artifactManifest()!.draft_count}</strong></div>
                <div><span>Grounded answers</span><strong>{artifactManifest()!.grounded_answer_count}</strong></div>
                <div><span>Final answers</span><strong>{artifactManifest()!.final_answer_count}</strong></div>
                <div><span>Issues</span><strong>{artifactManifest()!.issue_count}</strong></div>
              </div>
              {artifactManifest()!.sources.length > 0 ? (
                <ul class="final-answer-list-items">
                  {artifactManifest()!.sources.map((item) => (
                    <li>
                      <button class="final-answer-list-item" onClick={() => selectArtifactSourceId(item.source_id)}>
                        <span>{item.source_id}</span>
                        <small>
                          drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count} | issues={item.issue_count}
                        </small>
                        {item.final_answers.length > 0 && (
                          <small>
                            {item.final_answers.map((answer) => answer.final_answer_id).join(", ")}
                          </small>
                        )}
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No export manifest entries yet.</p>
              )}
            </>
          ) : (
            <p>No export manifest loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Export result</h3>
          {artifactExportResult() ? (
            <>
              <div class="contract-meta">
                <div><span>Export ID</span><strong>{artifactExportResult()!.export_id}</strong></div>
                <div><span>Sources</span><strong>{artifactExportResult()!.exported_source_count}</strong></div>
                <div><span>Drafts</span><strong>{artifactExportResult()!.exported_draft_count}</strong></div>
                <div><span>Grounded answers</span><strong>{artifactExportResult()!.exported_grounded_answer_count}</strong></div>
                <div><span>Final answers</span><strong>{artifactExportResult()!.exported_final_answer_count}</strong></div>
                <div><span>Issues</span><strong>{artifactExportResult()!.exported_issue_count}</strong></div>
              </div>
              {artifactExportResult()!.written_files.length > 0 ? (
                <ul class="final-answer-list-items">
                  {artifactExportResult()!.written_files.map((item) => (
                    <li>
                      <div class="final-answer-list-item">
                        <span>{item.relative_path}</span>
                        <small>
                          {item.artifact_kind}
                          {item.source_id ? ` | ${item.source_id}` : ""}
                          {item.artifact_id ? ` | ${item.artifact_id}` : ""}
                        </small>
                      </div>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No exported files listed yet.</p>
              )}
            </>
          ) : (
            <p>No export result loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Artifact overview</h3>
          {artifactOverview() ? (
            <div class="contract-meta">
              <div><span>Source ID</span><strong>{artifactOverview()!.source_id}</strong></div>
              <div><span>Answer drafts</span><strong>{artifactOverview()!.draft_count}</strong></div>
              <div><span>Grounded answers</span><strong>{artifactOverview()!.grounded_answer_count}</strong></div>
              <div><span>Final answers</span><strong>{artifactOverview()!.final_answer_count}</strong></div>
            </div>
          ) : (
            <p>No artifact overview loaded yet.</p>
          )}
          {artifactOverview() && artifactOverview()!.final_answers.length > 0 ? (
            <ul class="final-answer-list-items">
              {artifactOverview()!.final_answers.map((item) => (
                <li>
                  <button class="final-answer-list-item" onClick={() => selectFinalAnswer(item)}>
                    <span>{item.final_answer_id}</span>
                    <small>
                      {item.grounded_answer_id} | statements={item.statement_count} | needs_evidence={item.needs_evidence_count} | unsupported={item.unsupported_count}
                    </small>
                  </button>
                </li>
              ))}
            </ul>
          ) : artifactOverview() ? (
            <p>No final answers listed yet.</p>
          ) : null}
        </div>
        {finalAnswer() ? (
          <div class="contract-view">
            <div class="contract-meta">
              <div><span>Final answer ID</span><strong>{finalAnswer()!.final_answer_id}</strong></div>
              <div><span>Grounded answer ID</span><strong>{finalAnswer()!.grounded_answer_id}</strong></div>
              <div><span>Mode</span><strong>{finalAnswer()!.answer_mode}</strong></div>
              <div><span>Statements</span><strong>{finalAnswer()!.statement_count}</strong></div>
              <div><span>Needs evidence</span><strong>{finalAnswer()!.statements.filter((statement) => statement.status === "needs_evidence").length}</strong></div>
              <div><span>Unsupported</span><strong>{finalAnswer()!.unsupported_count}</strong></div>
            </div>
            {finalAnswer()!.statements.length > 0 ? (
              <div class="statement-list">
                {finalAnswer()!.statements.map((statement, index) => (
                <article class="statement-card">
                  <div class="statement-header">
                    <h3>
                      Statement {index + 1}
                    </h3>
                    <span class={`status-pill status-${statement.status}`}>
                      {statement.status}
                    </span>
                  </div>
                  <p>{statement.text}</p>
                  <div class="reference-grid">
                    <div><span>Statement ID</span><code>{statement.statement_id}</code></div>
                    <div><span>Grounded statement ID</span><code>{statement.grounded_statement_id}</code></div>
                    <div><span>Support level</span><code>{statement.support_level}</code></div>
                    <div><span>Claim IDs</span><code>{statement.claim_ids.join(", ") || "none"}</code></div>
                    <div><span>Evidence IDs</span><code>{statement.evidence_ids.join(", ") || "none"}</code></div>
                    <div><span>Chunk IDs</span><code>{statement.chunk_ids.join(", ") || "none"}</code></div>
                    <div class="full-span">
                      <span>Locators</span>
                      <div class="locator-list">
                        {statement.locators.length > 0 ? (
                          statement.locators.map((locator) => <code>{locatorSummary(locator)}</code>)
                        ) : (
                          <code>none</code>
                        )}
                      </div>
                    </div>
                  </div>
                </article>
              ))}
              </div>
            ) : (
              <p>No statements in this final answer.</p>
            )}
            {finalAnswer()!.warnings.length > 0 && (
              <div class="warning-box">
                <h3>Warnings</h3>
                <ul>
                  {finalAnswer()!.warnings.map((warning) => (
                    <li>{warning}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ) : (
          <p>No final answer loaded yet.</p>
        )}
      </section>
    </main>
  );
}
