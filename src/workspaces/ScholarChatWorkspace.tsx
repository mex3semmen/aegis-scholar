import { createSignal, type JSX } from "solid-js";
import type {
  GroundingPolicy,
  ScholarChatMode,
  ScholarChatSessionSummary,
  ScholarChatTranscriptMessage,
} from "../appTypes";

type PromptSuggestion = {
  label: string;
  prompt: string;
};

type ScholarChatWorkspaceProps = {
  transcript: ScholarChatTranscriptMessage[];
  sessions: ScholarChatSessionSummary[];
  activeSessionId: string | null;
  sessionRailError: string | null;
  sessionActionLoading: { kind: "rename" | "delete"; sessionId: string } | null;
  onSelectSession: (sessionId: string) => void;
  onNewSession: () => void;
  onRenameSession: (sessionId: string, title: string) => Promise<boolean>;
  onDeleteSession: (sessionId: string) => Promise<boolean>;
  suggestions: PromptSuggestion[];
  runtimeReadinessNote: string;
  prompt: string;
  validationError: string | null;
  error: string | null;
  previewLoading: boolean;
  executionGateLoading: boolean;
  selectedSourceSummary: string;
  mode: ScholarChatMode;
  groundingPolicy: GroundingPolicy;
  modes: { value: ScholarChatMode; label: string }[];
  groundingPolicies: { value: GroundingPolicy; label: string }[];
  onApplySuggestion: (prompt: string) => void;
  onPromptInput: (value: string) => void;
  onPreviewPlan: () => void;
  onCheckNextStep: () => void;
  onModeChange: (value: string) => void;
  onGroundingPolicyChange: (value: string) => void;
  renderMetricGrid: (entries: { label: string; value: string | number }[]) => JSX.Element;
  formatSnakeCaseLabel: (value: string) => string;
};

function formatSessionMessageCount(messageCount: number) {
  return `${messageCount} message${messageCount === 1 ? "" : "s"}`;
}

export default function ScholarChatWorkspace(props: ScholarChatWorkspaceProps): JSX.Element {
  const [editingSessionId, setEditingSessionId] = createSignal<string | null>(null);
  const [editingSessionTitle, setEditingSessionTitle] = createSignal("");
  const [renameValidationError, setRenameValidationError] = createSignal<string | null>(null);
  const [deleteConfirmSessionId, setDeleteConfirmSessionId] = createSignal<string | null>(null);
  const hasTranscript = props.transcript.length > 0;

  function renderTranscriptMessage(message: any) {
    if (message.role === "user") {
      return (
        <div class="chat-transcript-message user-message">
          <p class="chat-message-label">You</p>
          <p>{message.content}</p>
        </div>
      );
    }

    if (message.kind === "workflow_preview" && message.workflow_preview) {
      const preview = message.workflow_preview;
      return (
        <div class="assistant-card chat-transcript-card">
          <p class="chat-message-label">AEGIS</p>
          <h3>Workflow preview</h3>
          <p class="chat-message-summary">{preview.summary}</p>
          <p class="chat-message-summary">
            It recognized this as {props.formatSnakeCaseLabel(preview.recognized_intent)} and will stay preview-only.
          </p>
          <div class="chat-message-actions">
            {preview.next_required_actions.length > 0 ? <p><strong>Next:</strong> {preview.next_required_actions[0]}</p> : null}
            {preview.blockers.length > 0 ? <p><strong>Blocker:</strong> {preview.blockers[0]}</p> : null}
            {preview.warnings.length > 0 ? <p><strong>Note:</strong> {preview.warnings[0]}</p> : null}
          </div>
          <details class="chat-transcript-details">
            <summary>Preview details</summary>
            {props.renderMetricGrid([
              { label: "Selected sources", value: preview.selected_source_count },
              { label: "Execution allowed", value: preview.execution_allowed ? "yes" : "no" },
              { label: "Preview only", value: preview.preview_only ? "yes" : "no" },
            ])}
            <p><strong>Prompt:</strong> {preview.normalized_prompt}</p>
            <div class="contract-meta">
              <div>
                <span>Required local context</span>
                <strong>{preview.required_local_context.map((item: string) => props.formatSnakeCaseLabel(item)).join(", ") || "none"}</strong>
              </div>
            </div>
            <h4>Planned steps</h4>
            <ul>
              {preview.planned_steps.map((step: string) => (
                <li>{step}</li>
              ))}
            </ul>
            {preview.next_required_actions.length > 1 ? (
              <div class="warning-box">
                <h4>Additional next actions</h4>
                <ul>
                  {preview.next_required_actions.slice(1).map((action: string) => (
                    <li>{action}</li>
                  ))}
                </ul>
              </div>
            ) : null}
            {preview.blockers.length > 1 ? (
              <div class="warning-box">
                <h4>Additional blockers</h4>
                <ul>
                  {preview.blockers.slice(1).map((blocker: string) => (
                    <li>{blocker}</li>
                  ))}
                </ul>
              </div>
            ) : null}
            {preview.warnings.length > 1 ? (
              <div class="warning-box">
                <h4>Additional warnings</h4>
                <ul>
                  {preview.warnings.slice(1).map((warning: string) => (
                    <li>{warning}</li>
                  ))}
                </ul>
              </div>
            ) : null}
          </details>
        </div>
      );
    }

    if (message.kind === "execution_gate" && message.execution_gate_preview) {
      const gate = message.execution_gate_preview;
      return (
        <div class="assistant-card chat-transcript-card">
          <p class="chat-message-label">AEGIS</p>
          <h3>Safe next step</h3>
          <p class="chat-message-summary">{gate.blocked_reason || "The next safe step is ready to review."}</p>
          <p class="chat-message-summary">
            {props.formatSnakeCaseLabel(gate.gate_decision)} for {props.formatSnakeCaseLabel(gate.allowed_future_action)}.
          </p>
          <div class="chat-message-actions">
            {gate.next_required_actions.length > 0 ? <p><strong>Next:</strong> {gate.next_required_actions[0]}</p> : null}
            {gate.blockers.length > 0 ? <p><strong>Blocker:</strong> {gate.blockers[0]}</p> : null}
            {gate.warnings.length > 0 ? <p><strong>Note:</strong> {gate.warnings[0]}</p> : null}
          </div>
          <details class="chat-transcript-details">
            <summary>Gate details</summary>
            {props.renderMetricGrid([
              { label: "Status", value: props.formatSnakeCaseLabel(gate.status) },
              { label: "Consent needed", value: gate.consent_required ? "yes" : "no" },
              { label: "Preview only", value: gate.preview_only ? "yes" : "no" },
            ])}
            <p>
              <strong>Required local context:</strong>{" "}
              {gate.required_local_context.map((item: string) => props.formatSnakeCaseLabel(item)).join(", ") || "none"}
            </p>
            <h4>Planned steps</h4>
            <ul>
              {gate.planned_steps.map((step: string) => (
                <li>{step}</li>
              ))}
            </ul>
            <div class="contract-meta">
              <div>
                <span>Planned intent</span>
                <strong>{props.formatSnakeCaseLabel(gate.planned_intent)}</strong>
              </div>
              <div>
                <span>Selected sources</span>
                <strong>{gate.selected_source_count}</strong>
              </div>
              <div>
                <span>Execution allowed now</span>
                <strong>{gate.execution_allowed_now ? "yes" : "no"}</strong>
              </div>
              <div>
                <span>User consent present</span>
                <strong>{gate.user_consent_present ? "yes" : "no"}</strong>
              </div>
            </div>
            <div class="contract-meta">
              {gate.safety_invariants.map((item: string) => (
                <div>
                  <span>Safety</span>
                  <strong>{props.formatSnakeCaseLabel(item)}</strong>
                </div>
              ))}
            </div>
            <div class="contract-meta">
              <div>
                <span>No filesystem write</span>
                <strong>{gate.no_filesystem_write ? "yes" : "no"}</strong>
              </div>
              <div>
                <span>No backend mutation</span>
                <strong>{gate.no_backend_mutation ? "yes" : "no"}</strong>
              </div>
              <div>
                <span>No runtime execution</span>
                <strong>{gate.no_runtime_execution ? "yes" : "no"}</strong>
              </div>
              <div>
                <span>No LLM call</span>
                <strong>{gate.no_llm_call ? "yes" : "no"}</strong>
              </div>
              <div>
                <span>No network call</span>
                <strong>{gate.no_network_call ? "yes" : "no"}</strong>
              </div>
            </div>
            {gate.next_required_actions.length > 1 ? (
              <div class="warning-box">
                <h4>Additional next actions</h4>
                <ul>
                  {gate.next_required_actions.slice(1).map((action: string) => (
                    <li>{action}</li>
                  ))}
                </ul>
              </div>
            ) : null}
            {gate.blockers.length > 1 ? (
              <div class="warning-box">
                <h4>Additional blockers</h4>
                <ul>
                  {gate.blockers.slice(1).map((blocker: string) => (
                    <li>{blocker}</li>
                  ))}
                </ul>
              </div>
            ) : null}
            {gate.warnings.length > 1 ? (
              <div class="warning-box">
                <h4>Additional warnings</h4>
                <ul>
                  {gate.warnings.slice(1).map((warning: string) => (
                    <li>{warning}</li>
                  ))}
                </ul>
              </div>
            ) : null}
          </details>
        </div>
      );
    }

    return (
      <div class="assistant-card chat-transcript-card">
        <p class="chat-message-label">AEGIS</p>
        <h3>{message.title}</h3>
        <p>{message.content}</p>
      </div>
    );
  }

  function beginRenameSession(session: ScholarChatSessionSummary) {
    setDeleteConfirmSessionId(null);
    setRenameValidationError(null);
    setEditingSessionTitle(session.title);
    setEditingSessionId(session.session_id);
    queueMicrotask(() => {
      const input = document.getElementById(`session-rename-${session.session_id}`) as HTMLInputElement | null;
      input?.focus();
      input?.select();
    });
  }

  function cancelRenameSession() {
    setEditingSessionId(null);
    setEditingSessionTitle("");
    setRenameValidationError(null);
  }

  async function commitRenameSession(sessionId: string) {
    const trimmedTitle = editingSessionTitle().trim();
    if (!trimmedTitle) {
      setRenameValidationError("Session title cannot be blank.");
      return;
    }

    const success = await props.onRenameSession(sessionId, trimmedTitle);
    if (success) {
      cancelRenameSession();
    }
  }

  function beginDeleteSession(session: ScholarChatSessionSummary) {
    setEditingSessionId(null);
    setEditingSessionTitle("");
    setRenameValidationError(null);
    setDeleteConfirmSessionId(session.session_id);
  }

  function cancelDeleteSession() {
    setDeleteConfirmSessionId(null);
  }

  async function commitDeleteSession(sessionId: string) {
    const success = await props.onDeleteSession(sessionId);
    if (success) {
      cancelDeleteSession();
    }
  }

  return (
    <section class="chat-workspace" id="scholar-chat" data-workspace="scholar_chat">
      <div class="chat-workspace-header">
        <p class="eyebrow">Scholar Chat</p>
        <h2>Ask locally, preview first</h2>
        <p class="muted">
          Preview the next safe local workflow step. Execution stays gated, and the session rail stays nested here.
        </p>
      </div>

      <div class="chat-workspace-body">
        <aside class="chat-session-rail" aria-label="Scholar Chat sessions">
          <div class="chat-session-rail-header">
            <div class="chat-session-rail-copy">
              <p class="eyebrow">Sessions</p>
              <p class="muted">Saved per project. History loads here only when you choose it.</p>
            </div>
            <button
              type="button"
              class="secondary-action chat-session-new-action"
              onClick={() => {
                cancelRenameSession();
                cancelDeleteSession();
                props.onNewSession();
              }}
            >
              New session
            </button>
          </div>

          <div class="chat-session-rail-status" classList={{ "chat-session-rail-status--active": props.activeSessionId !== null }}>
            <span>{props.activeSessionId ? "Active session" : "Current draft"}</span>
            <strong>
              {props.activeSessionId
                ? "Transcript loaded. Composer state stays in memory."
                : "Will save on the first preview or check action."}
            </strong>
          </div>

          {props.sessionActionLoading ? (
            <p class="chat-session-rail-loading muted">
              {props.sessionActionLoading.kind === "rename" ? "Renaming session..." : "Deleting session..."}
            </p>
          ) : null}

          {props.sessionRailError ? <p class="chat-session-rail-error">{props.sessionRailError}</p> : null}

          {props.sessions.length > 0 ? (
            <div class="chat-session-list" aria-label="Saved Scholar Chat sessions">
              {props.sessions.map((session) => (
                <article
                  class="chat-session-item"
                  classList={{
                    active: props.activeSessionId === session.session_id,
                    editing: editingSessionId() === session.session_id,
                    confirming: deleteConfirmSessionId() === session.session_id,
                  }}
                >
                  <button
                    type="button"
                    class="chat-session-select"
                    classList={{ active: props.activeSessionId === session.session_id }}
                    aria-current={props.activeSessionId === session.session_id ? "true" : undefined}
                    disabled={Boolean(props.sessionActionLoading)}
                    onClick={() => {
                      cancelRenameSession();
                      cancelDeleteSession();
                      props.onSelectSession(session.session_id);
                    }}
                  >
                    <span>{session.title}</span>
                    <small>{formatSessionMessageCount(session.message_count)}</small>
                  </button>

                  {!editingSessionId() && !deleteConfirmSessionId() ? (
                    <div class="chat-session-item-actions">
                      <button
                        type="button"
                        class="chat-session-secondary-action"
                        disabled={Boolean(props.sessionActionLoading)}
                        onClick={() => beginRenameSession(session)}
                      >
                        Rename
                      </button>
                      <button
                        type="button"
                        class="chat-session-danger-action"
                        disabled={Boolean(props.sessionActionLoading)}
                        onClick={() => beginDeleteSession(session)}
                      >
                        Delete
                      </button>
                    </div>
                  ) : null}

                  {editingSessionId() === session.session_id ? (
                    <div class="chat-session-inline-editor">
                      <label class="chat-session-inline-field">
                        Rename session
                        <input
                          id={`session-rename-${session.session_id}`}
                          type="text"
                          value={editingSessionTitle()}
                          onInput={(event) => {
                            setEditingSessionTitle(event.currentTarget.value);
                            setRenameValidationError(null);
                          }}
                          onKeyDown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void commitRenameSession(session.session_id);
                            }
                            if (event.key === "Escape") {
                              event.preventDefault();
                              cancelRenameSession();
                            }
                          }}
                          disabled={Boolean(props.sessionActionLoading)}
                          placeholder="Enter a new session title"
                        />
                      </label>
                      {renameValidationError() ? <p class="chat-session-inline-error">{renameValidationError()}</p> : null}
                      <div class="chat-session-inline-actions">
                        <button
                          type="button"
                          class="chat-session-save-action"
                          disabled={Boolean(props.sessionActionLoading) || !editingSessionTitle().trim()}
                          onClick={() => void commitRenameSession(session.session_id)}
                        >
                          {props.sessionActionLoading?.kind === "rename" && props.sessionActionLoading.sessionId === session.session_id ? "Saving..." : "Save"}
                        </button>
                        <button
                          type="button"
                          class="chat-session-cancel-action"
                          disabled={Boolean(props.sessionActionLoading)}
                          onClick={cancelRenameSession}
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  ) : null}

                  {deleteConfirmSessionId() === session.session_id ? (
                    <div class="chat-session-delete-panel">
                      <p class="chat-session-delete-copy">Delete removes saved session history for this project.</p>
                      <div class="chat-session-inline-actions">
                        <button
                          type="button"
                          class="chat-session-danger-action"
                          disabled={Boolean(props.sessionActionLoading)}
                          onClick={() => void commitDeleteSession(session.session_id)}
                        >
                          {props.sessionActionLoading?.kind === "delete" && props.sessionActionLoading.sessionId === session.session_id ? "Deleting..." : "Delete session"}
                        </button>
                        <button
                          type="button"
                          class="chat-session-cancel-action"
                          disabled={Boolean(props.sessionActionLoading)}
                          onClick={cancelDeleteSession}
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  ) : null}
                </article>
              ))}
            </div>
          ) : (
            <div class="chat-session-empty-state">
              <h3>No saved sessions yet</h3>
              <p>Your first preview or check will create a session for this project.</p>
            </div>
          )}
        </aside>

        <div class="chat-column">
          <div class="chat-surface chat-surface--calm">
            {!hasTranscript ? (
              <div class="chat-empty-state chat-welcome-card">
                <div class="chat-welcome-copy">
                  <h3>Your local research workspace starts here</h3>
                  <p>
                    Ask a paper, lecture, method, or thesis question. Scholar Chat will preview the next local workflow step without turning preview into execution.
                  </p>
                </div>
                <div class="chat-suggestion-grid" aria-label="Prompt suggestions">
                  {props.suggestions.map((item: any) => (
                    <button type="button" class="chat-suggestion-chip" onClick={() => props.onApplySuggestion(item.prompt)}>
                      <span>{item.label}</span>
                      <small>{item.prompt}</small>
                    </button>
                  ))}
                </div>
              </div>
            ) : null}

            {hasTranscript ? (
              <div class="chat-transcript" aria-label="Scholar Chat transcript" aria-live="polite">
                {props.transcript.map((message: any) => renderTranscriptMessage(message))}
              </div>
            ) : null}

            <div class="chat-composer chat-composer--anchored" aria-label="Scholar Chat composer">
              <label class="composer-field">
                Prompt
                <textarea
                  rows={5}
                  value={props.prompt}
                  onInput={(event) => props.onPromptInput(event.currentTarget.value)}
                  placeholder="Ask Scholar Chat about a paper, lecture, method, or thesis problem..."
                />
              </label>
              {props.validationError ? <p class="error">{props.validationError}</p> : null}
              {props.error ? <p class="error">{props.error}</p> : null}
              <div class="composer-actions">
                <button class="primary-action" onClick={props.onPreviewPlan} disabled={props.previewLoading}>
                  {props.previewLoading ? "Previewing..." : "Preview plan"}
                </button>
                <button class="secondary-action" onClick={props.onCheckNextStep} disabled={props.executionGateLoading}>
                  {props.executionGateLoading ? "Checking..." : "Check next step"}
                </button>
              </div>
              <p class="chat-inline-note muted">
                {props.selectedSourceSummary} Open Sources when you want to adjust source readiness or selection.
              </p>
              {props.runtimeReadinessNote ? <p class="chat-runtime-note muted">{props.runtimeReadinessNote}</p> : null}
              <details class="planning-options">
                <summary>Advanced planning options</summary>
                <div class="form-row">
                  <label>
                    Mode
                    <select value={props.mode} onChange={(event) => props.onModeChange(event.currentTarget.value)}>
                      {props.modes.map((item: any) => (
                        <option value={item.value}>{item.label}</option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Grounding policy
                    <select value={props.groundingPolicy} onChange={(event) => props.onGroundingPolicyChange(event.currentTarget.value)}>
                      {props.groundingPolicies.map((item: any) => (
                        <option value={item.value}>{item.label}</option>
                      ))}
                    </select>
                  </label>
                </div>
              </details>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
