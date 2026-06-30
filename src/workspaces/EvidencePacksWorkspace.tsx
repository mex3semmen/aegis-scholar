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

type EvidencePackMetadata = {
  source_id: string;
  version_id: string;
  evidence_pack_id: string;
  query: string;
  created_at: string;
  retrieval_index_version: string;
  result_count: number;
  item_count: number;
  warning_count: number;
  evidence_pack_version: string;
};

type DraftClaim = {
  claim_id: string;
  status: string;
  text: string;
  evidence_ids: string[];
  chunk_ids: string[];
  locators: unknown[];
  confidence: string;
};

type AnswerDraft = {
  answer_draft_id: string;
  evidence_pack_id: string;
  source_id: string;
  version_id: string;
  query: string;
  created_at: string;
  draft_mode: string;
  claim_count: number;
  unsupported_count: number;
  claims: DraftClaim[];
  warnings: string[];
};

type AnswerDraftBuildSummary = {
  draft: AnswerDraft;
  sourceTitle: string;
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

function compactClaimPreview(text: string, maxChars = 180) {
  const compacted = text.split(/\s+/).filter(Boolean).join(" ").trim();
  if (compacted.length <= maxChars) {
    return compacted;
  }
  return `${compacted.slice(0, Math.max(0, maxChars - 3)).trimEnd()}...`;
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
  const [answerDraftStatus, setAnswerDraftStatus] = createSignal<EvidencePackActionStatus>("not_started");
  const [answerDraftRunningPackId, setAnswerDraftRunningPackId] = createSignal<string | null>(null);
  const [answerDraftError, setAnswerDraftError] = createSignal<string | null>(null);
  const [answerDraftSummary, setAnswerDraftSummary] = createSignal<AnswerDraftBuildSummary | null>(null);
  let previousExternalSourceSelection: string | null = null;
  let answerDraftRequestVersion = 0;

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

  function resetAnswerDraftResult() {
    answerDraftRequestVersion += 1;
    setAnswerDraftStatus("not_started");
    setAnswerDraftRunningPackId(null);
    setAnswerDraftError(null);
    setAnswerDraftSummary(null);
  }

  const workspaceMutationRunning = () =>
    actionStatus() === "running" || answerDraftStatus() === "running";

  createEffect(() => {
    const externalSelection = [
      props.selectedEvidencePackSourceId,
      ...(props.sourceContextSelectedIds as string[]),
    ].join("|");
    if (previousExternalSourceSelection !== null && externalSelection !== previousExternalSourceSelection) {
      if (answerDraftStatus() === "running") {
        answerDraftRequestVersion += 1;
        setAnswerDraftError(null);
        setAnswerDraftSummary(null);
      } else {
        resetAnswerDraftResult();
      }
    }
    previousExternalSourceSelection = externalSelection;
  });

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
        resetAnswerDraftResult();
      }
      return;
    }

    const existingEvidenceSource = sources.find(
      (source) => source.source_id === props.selectedEvidencePackSourceId,
    );
    setSourceSelectionTouched(false);
    setSelectedSourceId(selectedContextSource?.source_id ?? existingEvidenceSource?.source_id ?? sources[0]?.source_id ?? "");
    resetActionResult();
    resetAnswerDraftResult();
  });

  function selectSource(sourceId: string) {
    setSourceSelectionTouched(true);
    setSelectedSourceId(sourceId);
    resetActionResult();
    resetAnswerDraftResult();
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
    if (workspaceMutationRunning()) {
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

  function explainAnswerDraftError(error: unknown) {
    const sanitized = props.sanitizeBackendError(error);
    const normalized = sanitized.toLowerCase().replace(/[_\s-]+/g, "");
    if (normalized.includes("answerdraftemptyevidence") || normalized.includes("evidencepackempty")) {
      return "The selected Evidence Pack contains no evidence items.";
    }
    if (normalized.includes("evidencepackmissing")) {
      return "The selected Evidence Pack is no longer available. Reload the Evidence Pack list.";
    }
    if (normalized.includes("answerdraftinvalidid") || normalized.includes("evidencepackinvalidid")) {
      return "The selected Evidence Pack ID is invalid.";
    }
    if (normalized.includes("evidencepackreadfailed")) {
      return "The selected Evidence Pack could not be read.";
    }
    return sanitized;
  }

  async function buildAnswerDraft(item: EvidencePackMetadata) {
    if (workspaceMutationRunning()) {
      return;
    }

    const source = (props.sourceContext as EvidencePackSource[]).find(
      (candidate) => candidate.source_id === item.source_id,
    );
    const requestVersion = ++answerDraftRequestVersion;
    setAnswerDraftStatus("running");
    setAnswerDraftRunningPackId(item.evidence_pack_id);
    setAnswerDraftError(null);
    setAnswerDraftSummary(null);
    try {
      const draft = await invoke<AnswerDraft>("build_answer_draft", {
        root: ".",
        source_id: item.source_id,
        evidence_pack_id: item.evidence_pack_id,
      });
      await Promise.all([
        props.refreshCorpusStatus(),
        props.refreshSourceContext(true),
        props.loadEvidencePacksBySourceId(item.source_id),
      ]);
      if (requestVersion !== answerDraftRequestVersion) {
        setAnswerDraftStatus("not_started");
        setAnswerDraftRunningPackId(null);
        return;
      }
      setAnswerDraftSummary({
        draft,
        sourceTitle: source?.title || draft.source_id,
      });
      setAnswerDraftStatus("succeeded");
      setAnswerDraftRunningPackId(null);
    } catch (error) {
      if (requestVersion !== answerDraftRequestVersion) {
        setAnswerDraftStatus("not_started");
        setAnswerDraftRunningPackId(null);
        return;
      }
      setAnswerDraftStatus("failed");
      setAnswerDraftRunningPackId(null);
      setAnswerDraftError(explainAnswerDraftError(error));
      setAnswerDraftSummary(null);
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
                disabled={workspaceMutationRunning()}
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
                disabled={workspaceMutationRunning()}
              />
            </label>
            <label>
              Maximum results
              <select
                value={maxResults()}
                onChange={(event) => updateMaxResults(event.currentTarget.value)}
                disabled={workspaceMutationRunning()}
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
          <button onClick={buildEvidencePack} disabled={workspaceMutationRunning() || !selectedSource()}>
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
        <div class="answer-draft-action-header">
          <h4>Existing Evidence Packs</h4>
          <span class={`status-pill status-${answerDraftStatus()}`}>
            Answer draft: {actionStatusLabel(answerDraftStatus())}
          </span>
        </div>
        {selectedSource() ? (
          <>
            <div class="hero-actions">
              <button
                onClick={() => props.loadEvidencePacksBySourceId(selectedSourceId())}
                disabled={props.evidencePacksLoading || workspaceMutationRunning()}
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
                      {(props.evidencePacks as EvidencePackMetadata[]).map((item) => (
                        <li>
                          <div class="final-answer-list-item">
                            <span>{item.evidence_pack_id}</span>
                            <small>
                              version={item.version_id} | created={item.created_at} | items={item.item_count} | results={item.result_count} | warnings={item.warning_count}
                            </small>
                            <small>
                              query={item.query} | retrieval_index_version={item.retrieval_index_version} | pack_version={item.evidence_pack_version}
                            </small>
                            <div class="hero-actions evidence-pack-row-actions">
                              <button
                                onClick={() => buildAnswerDraft(item)}
                                disabled={workspaceMutationRunning() || props.evidencePacksLoading}
                              >
                                {answerDraftRunningPackId() === item.evidence_pack_id
                                  ? "Building..."
                                  : "Build answer draft"}
                              </button>
                            </div>
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

        {answerDraftError() ? <p class="error">{answerDraftError()}</p> : null}
        {answerDraftSummary() ? (
          <section class="compact-note answer-draft-result">
            <h4>Answer Draft</h4>
            <p class="muted">
              Mechanical evidence-only claim scaffold. This is not a grounded answer or final prose.
            </p>
            <div class="contract-meta">
              <div><span>Answer Draft ID</span><strong>{answerDraftSummary()!.draft.answer_draft_id}</strong></div>
              <div><span>Evidence Pack ID</span><strong>{answerDraftSummary()!.draft.evidence_pack_id}</strong></div>
              <div><span>Source</span><strong>{answerDraftSummary()!.sourceTitle}</strong></div>
              <div><span>Source ID</span><strong>{answerDraftSummary()!.draft.source_id}</strong></div>
              <div><span>Query</span><strong>{answerDraftSummary()!.draft.query}</strong></div>
              <div><span>Draft mode</span><strong>{props.formatSnakeCaseLabel(answerDraftSummary()!.draft.draft_mode)}</strong></div>
              <div><span>Claims</span><strong>{answerDraftSummary()!.draft.claim_count}</strong></div>
              <div><span>Unsupported</span><strong>{answerDraftSummary()!.draft.unsupported_count}</strong></div>
              <div><span>Warnings</span><strong>{answerDraftSummary()!.draft.warnings.length}</strong></div>
            </div>
            {answerDraftSummary()!.draft.claims.length > 0 ? (
              <>
                {answerDraftSummary()!.draft.claims.length > 3 ? (
                  <p class="muted">Showing 3 of {answerDraftSummary()!.draft.claim_count} claims.</p>
                ) : null}
                <ul class="final-answer-list-items answer-draft-claim-list">
                  {answerDraftSummary()!.draft.claims.slice(0, 3).map((claim) => (
                    <li>
                      <article class="final-answer-list-item answer-draft-claim">
                        <div class="answer-draft-claim-header">
                          <span>{props.formatSnakeCaseLabel(claim.status)}</span>
                          <small>{props.formatSnakeCaseLabel(claim.confidence)}</small>
                        </div>
                        <p>{compactClaimPreview(claim.text)}</p>
                        <small>
                          evidence={claim.evidence_ids.length} | chunks={claim.chunk_ids.length} | locators={claim.locators.length}
                        </small>
                      </article>
                    </li>
                  ))}
                </ul>
              </>
            ) : (
              <p>No claims returned.</p>
            )}
          </section>
        ) : null}
      </section>
    </div>
  );
}
