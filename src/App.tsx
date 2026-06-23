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
    const trimmedSourceId = sourceId().trim();
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
        </div>
        {finalAnswerError() && <p class="error">{finalAnswerError()}</p>}
        {artifactOverviewError() && <p class="error">{artifactOverviewError()}</p>}
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
