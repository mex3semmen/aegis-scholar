import { JSX } from "solid-js";

export default function ScholarChatWorkspace(props: any): JSX.Element {
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

  return (
    <section class="chat-workspace" id="scholar-chat" data-workspace="scholar_chat">
      <div class="chat-workspace-header">
        <p class="eyebrow">Scholar Chat</p>
        <h2>Ask, preview, and stay local</h2>
        <p class="muted">
          Preview the next safe local workflow step. Execution stays gated, and deeper diagnostics stay out of the way.
        </p>
      </div>

      <div class="chat-column">
        <div class="chat-surface">
          {!hasTranscript ? (
            <div class="chat-welcome-card">
              <div class="chat-welcome-copy">
                <h3>Your local research workspace starts here</h3>
                <p>
                  Ask a paper, lecture, method, or thesis question. Scholar Chat will preview the next local workflow step without turning preview into execution.
                </p>
              </div>
              <div class="chat-suggestion-grid" aria-label="Prompt suggestions">
                {props.suggestions.map((item: any) => (
                  <button class="chat-suggestion-chip" onClick={() => props.onApplySuggestion(item.prompt)}>
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

          <div class="chat-composer" aria-label="Scholar Chat composer">
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
    </section>
  );
}
