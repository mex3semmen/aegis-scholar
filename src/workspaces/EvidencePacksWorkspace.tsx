import { JSX } from "solid-js";

export default function EvidencePacksWorkspace(props: any): JSX.Element {
  return (
    <div class="artifact-overview workspace-panel" id="evidence-packs" data-workspace="evidence_packs">
      <h3>Evidence packs</h3>
      <p class="muted">Read-only evidence-pack metadata for the selected retrieval or answer-artifact source.</p>
      {props.selectedEvidencePackSourceId ? (
        <>
          <div class="hero-actions">
            <button onClick={() => props.loadEvidencePacks()} disabled={props.evidencePacksLoading}>
              {props.evidencePacksLoading ? "Loading..." : "Load evidence packs"}
            </button>
          </div>
          {props.evidencePacksError && props.evidencePacksSourceId === props.selectedEvidencePackSourceId ? (
            <p class="error">{props.evidencePacksError}</p>
          ) : null}
          {props.evidencePacksSourceId === props.selectedEvidencePackSourceId ? (
            props.evidencePacks ? (
              <>
                <div class="contract-meta">
                  <div><span>Source ID</span><strong>{props.selectedEvidencePackSourceId}</strong></div>
                  <div><span>Packs</span><strong>{props.evidencePacks.length}</strong></div>
                </div>
                {props.evidencePacks.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {props.evidencePacks.map((item: any) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.evidence_pack_id}</span>
                          <small>
                            version={item.version_id} | created={item.created_at} | items={item.item_count} | results={item.result_count} | warnings={item.warning_count}
                          </small>
                          <small>
                            query={item.query} | retrieval_index_version={item.retrieval_index_version} | pack_version={item.evidence_pack_version}
                          </small>
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No evidence packs listed yet for this source.</p>
                )}
              </>
            ) : props.evidencePacksLoading ? (
              <p>Loading evidence packs...</p>
            ) : props.evidencePacksError ? null : (
              <p>No evidence packs loaded yet for this source.</p>
            )
          ) : (
            <p>No evidence packs loaded yet for this source.</p>
          )}
        </>
      ) : (
        <p>Select a retrieval or answer-artifact source to load evidence packs.</p>
      )}
    </div>
  );
}
