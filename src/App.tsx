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

export default function App() {
  const [status, setStatus] = createSignal<CorpusStatus | null>(null);
  const [statusError, setStatusError] = createSignal<string | null>(null);
  const [sourceId, setSourceId] = createSignal("");
  const [finalAnswerId, setFinalAnswerId] = createSignal("");
  const [finalAnswer, setFinalAnswer] = createSignal<FinalAnswer | null>(null);
  const [finalAnswerError, setFinalAnswerError] = createSignal<string | null>(null);

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
    setFinalAnswerError(null);
    setFinalAnswer(null);
    try {
      const result = await invoke<FinalAnswer>("get_final_answer", {
        root: ".",
        source_id: sourceId(),
        final_answer_id: finalAnswerId(),
      });
      setFinalAnswer(result);
    } catch (err) {
      setFinalAnswerError(String(err));
    }
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
          <button onClick={loadFinalAnswer}>Load final answer</button>
        </div>
        {finalAnswerError() && <p class="error">{finalAnswerError()}</p>}
        {finalAnswer() ? (
          <div class="contract-view">
            <div class="contract-meta">
              <div><span>Final answer ID</span><strong>{finalAnswer()!.final_answer_id}</strong></div>
              <div><span>Grounded answer ID</span><strong>{finalAnswer()!.grounded_answer_id}</strong></div>
              <div><span>Mode</span><strong>{finalAnswer()!.answer_mode}</strong></div>
              <div><span>Statements</span><strong>{finalAnswer()!.statement_count}</strong></div>
              <div><span>Unsupported</span><strong>{finalAnswer()!.unsupported_count}</strong></div>
            </div>
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
                    <div class="full-span"><span>Locators</span><code>{statement.locators.length > 0 ? JSON.stringify(statement.locators, null, 2) : "none"}</code></div>
                  </div>
                </article>
              ))}
            </div>
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
