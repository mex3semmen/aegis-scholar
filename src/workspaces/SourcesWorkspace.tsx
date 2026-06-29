import { JSX } from "solid-js";

export default function SourcesWorkspace(props: any): JSX.Element {
  return (
    <section class="card workspace-panel" id="sources" data-workspace="sources">
      <h2>Sources</h2>
      <p class="muted">
        Source registration and readiness are still early. This workspace keeps the corpus state visible while the import and readiness flow stays manual.
      </p>
      {props.status ? (
        <>
          <pre>{JSON.stringify(props.status, null, 2)}</pre>
          {props.status.source_count === 0 ? (
            <p class="muted">No local sources yet. See the source readiness panel below for supported source types and next steps.</p>
          ) : null}
        </>
      ) : (
        <p>No status loaded yet.</p>
      )}
      {props.statusError ? <p class="error">{props.statusError}</p> : null}
      <div class="compact-note">
        <h3>Source context</h3>
        <p class="muted">
          No local sources selected. AEGIS can still preview the workflow, but grounded answers require sources.
        </p>
        {props.sourceContextLoading ? (
          <p>Loading registered sources...</p>
        ) : props.sourceContextError ? (
          <p class="error">{props.sourceContextError}</p>
        ) : props.sourceContext.length === 0 ? (
          props.renderFirstRunSourceReadiness()
        ) : (
          <>
            <p class="muted">Selected source count: {props.sourceContextSelectedIds.length}</p>
            <ul class="final-answer-list-items">
              {props.sourceContext.map((item: any) => (
                <li>
                  <label class="final-answer-list-item">
                    <span>
                      <input
                        type="checkbox"
                        checked={props.sourceContextSelectedIds.includes(item.source_id)}
                        onChange={() => {
                          props.toggleSourceContext(item.source_id);
                          props.setScholarChatPreview(null);
                          props.setScholarChatExecutionGatePreview(null);
                        }}
                      />
                      <strong> {item.title || item.source_id}</strong>
                    </span>
                    <small>
                      source_id={item.source_id} | type={props.formatSnakeCaseLabel(item.source_type)} | version={item.version_id} | status={props.formatSnakeCaseLabel(item.ingestion_status)}
                    </small>
                  </label>
                </li>
              ))}
            </ul>
          </>
        )}
        {props.sourceContext.length > 0 ? props.renderSourceWorkflowActionHints() : null}
        <p class="muted">{props.selectedSourceSummary}</p>
      </div>
    </section>
  );
}
