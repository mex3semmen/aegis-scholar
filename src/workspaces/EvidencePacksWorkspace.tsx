import { createEffect, createSignal, JSX } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

type EvidencePackActionStatus = "not_started" | "running" | "succeeded" | "failed";
type EvidencePackMaxResults = 5 | 10 | 25;

type EvidencePackSource = {
  source_id: string;
  version_id: string;
  title: string;
  source_type: string;
  ingestion_status: string;
};

type EvidencePackBuildResult = {
  evidence_pack_id: string;
  source_id: string;
  version_id: string;
  query: string;
  retrieval_index_version: string;
  result_count: number;
  item_count: number;
  warnings: string[];
};

type EvidencePackBuildSummary = {
  pack: EvidencePackBuildResult;
  sourceTitle: string;
  requestedMaxResults: EvidencePackMaxResults;
};

const ELIGIBLE_SOURCE_STATUSES = new Set([
  "indexed",
  "evidence_ready",
  "answer_drafted",
  "grounded_answer_ready",
]);

function actionStatusLabel(status: EvidencePackActionStatus) {
  switch (status) {
    case "not_started":
      return "Not started";
    case "running":
      return "Running";
    case "succeeded":
      return "Succeeded";
    case "failed":
      return "Failed";
  }
}

export default function EvidencePacksWorkspace(props: any): JSX.Element {
  const [selectedSourceId, setSelectedSourceId] = createSignal("");
  const [sourceSelectionTouched, setSourceSelectionTouched] = createSignal(false);
  const [query, setQuery] = createSignal("");
  const [maxResults, setMaxResults] = createSignal<EvidencePackMaxResults>(5);
  const [actionStatus, setActionStatus] = createSignal<EvidencePackActionStatus>("not_started");
  const [validationError, setValidationError] = createSignal<string | null>(null);
  const [actionError, setActionError] = createSignal<string | null>(null);
  const [buildSummary, setBuildSummary] = createSignal<EvidencePackBuildSummary | null>(null);

  const eligibleSources = () =>
    (props.sourceContext as EvidencePackSource[])
      .filter((source) => ELIGIBLE_SOURCE_STATUSES.has(source.ingestion_status))
      .sort((left, right) => left.source_id.localeCompare(right.source_id));

  function resetActionResult() {
    setActionStatus("not_started");
    setValidationError(null);
    setActionError(null);
    setBuildSummary(null);
  }

  createEffect(() => {
    const sources = eligibleSources();
    const currentSourceId = selectedSourceId();
    const selectedContextSource = sources.find((source) =>
      (props.sourceContextSelectedIds as string[]).includes(source.source_id),
    );
    if (sources.some((source) => source.source_id === currentSourceId)) {
      if (!sourceSelectionTouched() && selectedContextSource && selectedContextSource.source_id !== currentSourceId) {
        setSelectedSourceId(selectedContextSource.source_id);
        resetActionResult();
      }
      return;
    }

    const existingEvidenceSource = sources.find(
      (source) => source.source_id === props.selectedEvidencePackSourceId,
    );
    setSourceSelectionTouched(false);
    setSelectedSourceId(selectedContextSource?.source_id ?? existingEvidenceSource?.source_id ?? sources[0]?.source_id ?? "");
    resetActionResult();
  });

  function selectSource(sourceId: string) {
    setSourceSelectionTouched(true);
    setSelectedSourceId(sourceId);
    resetActionResult();
  }

  function updateQuery(value: string) {
    setQuery(value);
    resetActionResult();
  }

  function updateMaxResults(value: string) {
    setMaxResults(Number(value) as EvidencePackMaxResults);
    resetActionResult();
  }

  function explainEvidencePackError(error: unknown) {
    const sanitized = props.sanitizeBackendError(error);
    const normalized = sanitized.toLowerCase().replace(/[_\s-]+/g, "");
    if (normalized.includes("evidencepackempty")) {
      return "No matching evidence was found for this query.";
    }
    return sanitized;
  }

  async function buildEvidencePack() {
    if (actionStatus() === "running") {
      return;
    }

    const source = eligibleSources().find((item) => item.source_id === selectedSourceId());
    const trimmedQuery = query().trim();
    if (!source) {
      setValidationError("Select an indexed source.");
      setActionStatus("failed");
      setBuildSummary(null);
      return;
    }
    if (!trimmedQuery) {
      setValidationError("Query is required to build an Evidence Pack.");
      setActionStatus("failed");
      setBuildSummary(null);
      return;
    }

    const requestedMaxResults = maxResults();
    setActionStatus("running");
    setValidationError(null);
    setActionError(null);
    setBuildSummary(null);
    try {
      const pack = await invoke<EvidencePackBuildResult>("build_evidence_pack", {
        root: ".",
        source_id: source.source_id,
        query: trimmedQuery,
        max_results: requestedMaxResults,
      });
      setBuildSummary({
        pack,
        sourceTitle: source.title || source.source_id,
        requestedMaxResults,
      });
      setActionStatus("succeeded");
      await Promise.all([
        props.refreshCorpusStatus(),
        props.refreshSourceContext(true),
        props.loadEvidencePacksBySourceId(source.source_id),
      ]);
    } catch (error) {
      setActionStatus("failed");
      setActionError(explainEvidencePackError(error));
      setBuildSummary(null);
    }
  }

  const selectedSource = () =>
    eligibleSources().find((source) => source.source_id === selectedSourceId()) ?? null;

  return (
    <div class="artifact-overview workspace-panel" id="evidence-packs" data-workspace="evidence_packs">
      <div class="evidence-pack-action-header">
        <div>
          <h3>Evidence packs</h3>
          <p class="muted">Build and inspect source-grounded Evidence Packs from an already indexed local source.</p>
        </div>
        <span class={`status-pill status-${actionStatus()}`}>{actionStatusLabel(actionStatus())}</span>
      </div>

      <section class="warning-box evidence-pack-action">
        <h4>Build Evidence Pack</h4>
        <p class="muted">
          This action runs only after explicit confirmation. It retrieves local evidence and does not generate an answer.
        </p>

        {eligibleSources().length > 0 ? (
          <div class="form-row">
            <label>
              Indexed source
              <select
                value={selectedSourceId()}
                onChange={(event) => selectSource(event.currentTarget.value)}
                disabled={actionStatus() === "running"}
              >
                {eligibleSources().map((source) => (
                  <option value={source.source_id}>
                    {source.title || source.source_id} ({props.formatSnakeCaseLabel(source.ingestion_status)})
                  </option>
                ))}
              </select>
            </label>
            <label>
              Query
              <input
                type="text"
                value={query()}
                onInput={(event) => updateQuery(event.currentTarget.value)}
                placeholder="Which evidence supports the research question?"
                disabled={actionStatus() === "running"}
              />
            </label>
            <label>
              Maximum results
              <select
                value={maxResults()}
                onChange={(event) => updateMaxResults(event.currentTarget.value)}
                disabled={actionStatus() === "running"}
              >
                <option value={5}>5</option>
                <option value={10}>10</option>
                <option value={25}>25</option>
              </select>
            </label>
          </div>
        ) : (
          <p class="muted">Import and index a source before building an Evidence Pack.</p>
        )}

        {validationError() ? <p class="error">{validationError()}</p> : null}
        {actionError() ? <p class="error">{actionError()}</p> : null}

        <div class="hero-actions">
          <button onClick={buildEvidencePack} disabled={actionStatus() === "running" || !selectedSource()}>
            {actionStatus() === "running" ? "Building..." : "Build Evidence Pack"}
          </button>
        </div>

        {buildSummary() ? (
          <div class="contract-meta evidence-pack-build-summary">
            <div><span>Evidence Pack ID</span><strong>{buildSummary()!.pack.evidence_pack_id}</strong></div>
            <div><span>Source</span><strong>{buildSummary()!.sourceTitle}</strong></div>
            <div><span>Source ID</span><strong>{buildSummary()!.pack.source_id}</strong></div>
            <div><span>Query</span><strong>{buildSummary()!.pack.query}</strong></div>
            <div><span>Retrieval index</span><strong>{buildSummary()!.pack.retrieval_index_version}</strong></div>
            <div><span>Retrieval results</span><strong>{buildSummary()!.pack.result_count}</strong></div>
            <div><span>Evidence items</span><strong>{buildSummary()!.pack.item_count}</strong></div>
            <div><span>Warnings</span><strong>{buildSummary()!.pack.warnings.length}</strong></div>
            <div><span>Requested max results</span><strong>{buildSummary()!.requestedMaxResults}</strong></div>
          </div>
        ) : null}
      </section>

      <section class="evidence-pack-list">
        <h4>Existing Evidence Packs</h4>
        {selectedSource() ? (
          <>
            <div class="hero-actions">
              <button
                onClick={() => props.loadEvidencePacksBySourceId(selectedSourceId())}
                disabled={props.evidencePacksLoading || actionStatus() === "running"}
              >
                {props.evidencePacksLoading ? "Loading..." : "Load Evidence Packs"}
              </button>
            </div>
            {props.evidencePacksError && props.evidencePacksSourceId === selectedSourceId() ? (
              <p class="error">{props.evidencePacksError}</p>
            ) : null}
            {props.evidencePacksSourceId === selectedSourceId() ? (
              props.evidencePacks ? (
                <>
                  <div class="contract-meta">
                    <div><span>Source ID</span><strong>{selectedSourceId()}</strong></div>
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
                    <p>No Evidence Packs listed yet for this source.</p>
                  )}
                </>
              ) : props.evidencePacksLoading ? (
                <p>Loading Evidence Packs...</p>
              ) : props.evidencePacksError ? null : (
                <p>No Evidence Packs loaded yet for this source.</p>
              )
            ) : (
              <p>No Evidence Packs loaded yet for this source.</p>
            )}
          </>
        ) : (
          <p>Select an indexed source to load Evidence Packs.</p>
        )}
      </section>
    </div>
  );
}
