import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

type CorpusStatus = {
  source_count: number;
  registered_count: number;
  extracted_count: number;
  failed_count: number;
};

type RetrievalIndexEntry = {
  chunk_id: string;
  source_id: string;
  version_id: string;
  locator: CitationLocator;
  text_hash: string;
  normalized_terms: string[];
};

type RetrievalIndex = {
  source_id: string;
  version_id: string;
  indexed_at: string;
  chunk_count: number;
  index_version: string;
  chunk_report_hash: string;
  entries: RetrievalIndexEntry[];
  warnings: string[];
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
  schema_version: string;
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
  sources: AnswerArtifactExportSource[];
};

type ExportedArtifactFile = {
  relative_path: string;
  artifact_kind: "manifest" | "issues" | "summary" | "integrity" | "answer_draft" | "grounded_answer" | "final_answer";
  source_id?: string | null;
  artifact_id?: string | null;
};

type AnswerArtifactExportIntegrityFile = {
  relative_path: string;
  byte_count: number;
  sha256: string;
};

type AnswerArtifactExportIntegrity = {
  schema_version: string;
  algorithm: string;
  files: AnswerArtifactExportIntegrityFile[];
};

type AnswerArtifactExportResult = {
  schema_version: string;
  manifest: AnswerArtifactExportManifest;
  integrity: AnswerArtifactExportIntegrity;
  exported_source_count: number;
  exported_draft_count: number;
  exported_grounded_answer_count: number;
  exported_final_answer_count: number;
  exported_issue_count: number;
  export_id: string;
  written_files: ExportedArtifactFile[];
};

type AnswerArtifactExportIssueKindCount = {
  issue_kind: "malformed_final_answer" | "needs_evidence_statement" | "unsupported_statement";
  count: number;
};

type AnswerArtifactExportBundleInspectionIssueKindCount = {
  kind:
    | "missing_manifest"
    | "manifest_read_failed"
    | "missing_issues"
    | "issues_read_failed"
    | "missing_summary"
    | "summary_read_failed"
    | "missing_integrity"
    | "integrity_read_failed"
    | "integrity_schema_version_missing"
    | "integrity_schema_version_unsupported"
    | "integrity_algorithm_missing"
    | "integrity_algorithm_unsupported"
    | "integrity_duplicate_path"
    | "integrity_path_invalid"
    | "integrity_missing_file"
    | "integrity_byte_count_mismatch"
    | "integrity_digest_mismatch"
    | "schema_version_missing"
    | "schema_version_unsupported"
    | "schema_version_mismatch"
    | "summary_counts_mismatch"
    | "summary_issue_count_mismatch"
    | "summary_issue_kind_counts_mismatch"
    | "summary_export_id_mismatch"
    | "summary_metadata_mismatch";
  count: number;
};

type AnswerArtifactExportBundleInspectionSummary = {
  is_consistent: boolean;
  schema_supported: boolean;
  integrity_verified: boolean;
  issue_count: number;
  warning_count: number;
  issue_counts_by_kind: AnswerArtifactExportBundleInspectionIssueKindCount[];
  checked_file_count: number;
  integrity_file_count: number;
};

type AnswerArtifactExportBundleInspectionReportSection = {
  heading: string;
  lines: string[];
};

type AnswerArtifactExportBundleInspectionReportPreview = {
  title: string;
  schema_version: string;
  is_consistent: boolean;
  integrity_verified: boolean;
  issue_count: number;
  warning_count: number;
  issue_counts_by_kind: AnswerArtifactExportBundleInspectionIssueKindCount[];
  sections: AnswerArtifactExportBundleInspectionReportSection[];
};

type AnswerArtifactExportBundleInspectionIssueGroup = {
  kind: AnswerArtifactExportBundleInspectionIssueKind;
  count: number;
  lines: string[];
};

type AnswerArtifactExportBundleInspectionStatus = {
  code: string;
  label: string;
  severity: string;
  reason: string;
};

type AnswerArtifactExportBundleFileStatus = {
  file_label: string;
  present: boolean;
  parsed: boolean;
  malformed: boolean;
  schema_version?: string | null;
  schema_status: string;
  integrity_status: string;
  issue_count: number;
  status: string;
};

type AnswerArtifactExportSummarySource = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
};

type AnswerArtifactExportSummary = {
  schema_version: string;
  export_id: string;
  generated_from: string;
  export_scope: string;
  non_goals: string[];
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
  issue_kinds: AnswerArtifactExportIssueKindCount[];
  sources: AnswerArtifactExportSummarySource[];
};

type AnswerArtifactExportBundleInspectionIssueKind =
  | "missing_manifest"
  | "manifest_read_failed"
  | "missing_issues"
  | "issues_read_failed"
  | "missing_summary"
  | "summary_read_failed"
  | "missing_integrity"
  | "integrity_read_failed"
  | "integrity_schema_version_missing"
  | "integrity_schema_version_unsupported"
  | "integrity_algorithm_missing"
  | "integrity_algorithm_unsupported"
  | "integrity_duplicate_path"
  | "integrity_path_invalid"
  | "integrity_missing_file"
  | "integrity_byte_count_mismatch"
  | "integrity_digest_mismatch"
  | "schema_version_missing"
  | "schema_version_unsupported"
  | "schema_version_mismatch"
  | "summary_counts_mismatch"
  | "summary_issue_count_mismatch"
  | "summary_issue_kind_counts_mismatch"
  | "summary_export_id_mismatch"
  | "summary_metadata_mismatch";

type AnswerArtifactExportBundleInspectionIssue = {
  kind: AnswerArtifactExportBundleInspectionIssueKind;
  message: string;
  relative_path?: string | null;
};

type AnswerArtifactExportBundleInspection = {
  schema_version?: string | null;
  manifest_schema_version?: string | null;
  issues_schema_version?: string | null;
  summary_schema_version?: string | null;
  integrity_schema_version?: string | null;
  integrity_algorithm?: string | null;
  inspection_status: AnswerArtifactExportBundleInspectionStatus;
  inspection_summary: AnswerArtifactExportBundleInspectionSummary;
  report_preview: AnswerArtifactExportBundleInspectionReportPreview;
  issue_groups: AnswerArtifactExportBundleInspectionIssueGroup[];
  file_statuses: AnswerArtifactExportBundleFileStatus[];
  has_manifest: boolean;
  has_issues: boolean;
  has_summary: boolean;
  has_integrity: boolean;
  is_consistent: boolean;
  issue_count: number;
  warning_count: number;
  errors: AnswerArtifactExportBundleInspectionIssue[];
  warnings: AnswerArtifactExportBundleInspectionIssue[];
  manifest_counts?: AnswerArtifactExportManifest | null;
  summary_counts?: AnswerArtifactExportSummary | null;
  integrity_counts?: AnswerArtifactExportIntegrity | null;
  issue_kind_counts?: AnswerArtifactExportIssueKindCount[] | null;
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

function renderMetricGrid(entries: { label: string; value: string | number }[]) {
  return (
    <div class="contract-meta">
      {entries.map((entry) => (
        <div>
          <span>{entry.label}</span>
          <strong>{entry.value}</strong>
        </div>
      ))}
    </div>
  );
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
  const [retrievalIndex, setRetrievalIndex] = createSignal<RetrievalIndex | null>(null);
  const [retrievalIndexError, setRetrievalIndexError] = createSignal<string | null>(null);
  const [retrievalIndexLoading, setRetrievalIndexLoading] = createSignal(false);
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
  const [exportBundleRoot, setExportBundleRoot] = createSignal("");
  const [artifactBundleInspection, setArtifactBundleInspection] = createSignal<AnswerArtifactExportBundleInspection | null>(null);
  const [artifactBundleInspectionError, setArtifactBundleInspectionError] = createSignal<string | null>(null);
  const [artifactBundleInspectionLoading, setArtifactBundleInspectionLoading] = createSignal(false);

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

  async function loadRetrievalIndex() {
    const trimmedSourceId = sourceId().trim();
    if (!trimmedSourceId) {
      setRetrievalIndexError("Source ID is required to load the retrieval index.");
      return;
    }
    if (retrievalIndexLoading()) {
      return;
    }
    setRetrievalIndexLoading(true);
    setRetrievalIndexError(null);
    try {
      const result = await invoke<RetrievalIndex>("get_retrieval_index", {
        root: ".",
        source_id: trimmedSourceId,
      });
      setRetrievalIndex(result);
    } catch (err) {
      setRetrievalIndex(null);
      setRetrievalIndexError(sanitizeBackendError(err));
    } finally {
      setRetrievalIndexLoading(false);
    }
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

  async function inspectExportBundle() {
    const trimmedExportBundleRoot = exportBundleRoot().trim();
    if (!trimmedExportBundleRoot) {
      setArtifactBundleInspectionError("Export bundle root is required.");
      return;
    }
    if (artifactBundleInspectionLoading()) {
      return;
    }
    setArtifactBundleInspectionLoading(true);
    setArtifactBundleInspectionError(null);
    try {
      const result = await invoke<AnswerArtifactExportBundleInspection>("inspect_answer_artifact_export_bundle", {
        export_root: trimmedExportBundleRoot,
      });
      setArtifactBundleInspection(result);
    } catch (err) {
      setArtifactBundleInspection(null);
      setArtifactBundleInspectionError(sanitizeBackendError(err));
    } finally {
      setArtifactBundleInspectionLoading(false);
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
        {retrievalIndexError() && <p class="error">{retrievalIndexError()}</p>}
        {artifactSourcesError() && <p class="error">{artifactSourcesError()}</p>}
        {artifactHealthError() && <p class="error">{artifactHealthError()}</p>}
        {artifactIssuesError() && <p class="error">{artifactIssuesError()}</p>}
        {artifactManifestError() && <p class="error">{artifactManifestError()}</p>}
        {artifactExportError() && <p class="error">{artifactExportError()}</p>}
        {artifactBundleInspectionError() && <p class="error">{artifactBundleInspectionError()}</p>}
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
                <div><span>Schema</span><strong>{artifactManifest()!.schema_version || "missing"}</strong></div>
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
                <div><span>Schema</span><strong>{artifactExportResult()!.schema_version || "missing"}</strong></div>
                <div><span>Export ID</span><strong>{artifactExportResult()!.export_id}</strong></div>
                <div><span>Sources</span><strong>{artifactExportResult()!.exported_source_count}</strong></div>
                <div><span>Drafts</span><strong>{artifactExportResult()!.exported_draft_count}</strong></div>
                <div><span>Grounded answers</span><strong>{artifactExportResult()!.exported_grounded_answer_count}</strong></div>
                <div><span>Final answers</span><strong>{artifactExportResult()!.exported_final_answer_count}</strong></div>
                <div><span>Issues</span><strong>{artifactExportResult()!.exported_issue_count}</strong></div>
                <div><span>Integrity</span><strong>{artifactExportResult()!.integrity.schema_version ? `${artifactExportResult()!.integrity.algorithm} | ${artifactExportResult()!.integrity.files.length} files` : "missing"}</strong></div>
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
          <h3>Export bundle inspector</h3>
          <p class="muted">
            Read-only validation for an existing manual export bundle.
          </p>
          <div class="form-row">
            <label class="inline-field">
              Export bundle root
              <input
                type="text"
                value={exportBundleRoot()}
                onInput={(event) => setExportBundleRoot(event.currentTarget.value)}
                placeholder="E:\\path\\to\\export-bundle"
              />
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={inspectExportBundle} disabled={artifactBundleInspectionLoading()}>
              {artifactBundleInspectionLoading() ? "Loading..." : "Inspect export bundle"}
            </button>
          </div>
          {artifactBundleInspection() ? (
            <>
              <div class="artifact-overview">
                <h4>Inspection status</h4>
                {renderMetricGrid([
                  { label: "Status code", value: artifactBundleInspection()!.inspection_status.code },
                  { label: "Status label", value: artifactBundleInspection()!.inspection_status.label },
                  { label: "Severity", value: artifactBundleInspection()!.inspection_status.severity },
                ])}
                <p class="muted">{artifactBundleInspection()!.inspection_status.reason}</p>
              </div>
              <div class="artifact-overview">
                <h4>Inspection summary</h4>
                {renderMetricGrid([
                  { label: "Consistent", value: artifactBundleInspection()!.inspection_summary.is_consistent ? "yes" : "no" },
                  { label: "Schema supported", value: artifactBundleInspection()!.inspection_summary.schema_supported ? "yes" : "no" },
                  { label: "Integrity verified", value: artifactBundleInspection()!.inspection_summary.integrity_verified ? "yes" : "no" },
                  { label: "Issues", value: artifactBundleInspection()!.inspection_summary.issue_count },
                  { label: "Warnings", value: artifactBundleInspection()!.inspection_summary.warning_count },
                  { label: "Checked files", value: artifactBundleInspection()!.inspection_summary.checked_file_count },
                  { label: "Integrity files", value: artifactBundleInspection()!.inspection_summary.integrity_file_count },
                ])}
                {artifactBundleInspection()!.inspection_summary.issue_counts_by_kind.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {artifactBundleInspection()!.inspection_summary.issue_counts_by_kind.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.kind}</span>
                          <small>count={item.count}</small>
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : null}
              </div>
              <div class="artifact-overview">
                <h4>File statuses</h4>
                {artifactBundleInspection()!.file_statuses.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {artifactBundleInspection()!.file_statuses.map((fileStatus) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{fileStatus.file_label}</span>
                          <small>
                            status={fileStatus.status} | present={fileStatus.present ? "yes" : "no"} | parsed={fileStatus.parsed ? "yes" : "no"} | malformed={fileStatus.malformed ? "yes" : "no"} | schema={fileStatus.schema_status} | integrity={fileStatus.integrity_status} | issues={fileStatus.issue_count}
                          </small>
                          {fileStatus.schema_version ? <small>schema_version={fileStatus.schema_version}</small> : null}
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No file statuses available.</p>
                )}
              </div>
              <div class="artifact-overview">
                <h4>Issue detail groups</h4>
                {artifactBundleInspection()!.issue_groups.length > 0 ? (
                  artifactBundleInspection()!.issue_groups.map((group) => (
                    <div class="artifact-overview">
                      <h5>{group.kind}</h5>
                      {renderMetricGrid([{ label: "Count", value: group.count }])}
                      {group.lines.length > 0 ? (
                        <ul class="final-answer-list-items">
                          {group.lines.map((line) => (
                            <li>
                              <div class="final-answer-list-item">
                                <span>{line}</span>
                              </div>
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p>No lines available.</p>
                      )}
                    </div>
                  ))
                ) : (
                  <p>No issue groups available.</p>
                )}
              </div>
              <div class="artifact-overview">
                <h4>{artifactBundleInspection()!.report_preview.title}</h4>
                {renderMetricGrid([
                  { label: "Schema", value: artifactBundleInspection()!.report_preview.schema_version },
                  { label: "Consistent", value: artifactBundleInspection()!.report_preview.is_consistent ? "yes" : "no" },
                  { label: "Integrity verified", value: artifactBundleInspection()!.report_preview.integrity_verified ? "yes" : "no" },
                  { label: "Issues", value: artifactBundleInspection()!.report_preview.issue_count },
                  { label: "Warnings", value: artifactBundleInspection()!.report_preview.warning_count },
                ])}
                {artifactBundleInspection()!.report_preview.sections.length > 0 ? (
                  artifactBundleInspection()!.report_preview.sections.map((section) => (
                    <div class="artifact-overview">
                      <h5>{section.heading}</h5>
                      {section.lines.length > 0 ? (
                        <ul class="final-answer-list-items">
                          {section.lines.map((line) => (
                            <li>
                              <div class="final-answer-list-item">
                                <span>{line}</span>
                              </div>
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p>No lines available.</p>
                      )}
                    </div>
                  ))
                ) : (
                  <p>No preview sections available.</p>
                )}
              </div>
              <div class="contract-meta">
                <div><span>Schema</span><strong>{artifactBundleInspection()!.schema_version || "mixed / missing"}</strong></div>
                <div><span>Has manifest</span><strong>{artifactBundleInspection()!.has_manifest ? "yes" : "no"}</strong></div>
                <div><span>Has issues</span><strong>{artifactBundleInspection()!.has_issues ? "yes" : "no"}</strong></div>
                <div><span>Has summary</span><strong>{artifactBundleInspection()!.has_summary ? "yes" : "no"}</strong></div>
                <div><span>Has integrity</span><strong>{artifactBundleInspection()!.has_integrity ? "yes" : "no"}</strong></div>
                <div><span>Consistent</span><strong>{artifactBundleInspection()!.is_consistent ? "yes" : "no"}</strong></div>
                <div><span>Issues</span><strong>{artifactBundleInspection()!.issue_count}</strong></div>
                <div><span>Warnings</span><strong>{artifactBundleInspection()!.warning_count}</strong></div>
              </div>
              {artifactBundleInspection()!.manifest_counts ? (
                <div class="artifact-overview">
                  <h4>Manifest counts</h4>
                  {renderMetricGrid([
                    { label: "Schema", value: artifactBundleInspection()!.manifest_counts!.schema_version || "missing" },
                    { label: "Sources", value: artifactBundleInspection()!.manifest_counts!.source_count },
                    { label: "Drafts", value: artifactBundleInspection()!.manifest_counts!.draft_count },
                    { label: "Grounded answers", value: artifactBundleInspection()!.manifest_counts!.grounded_answer_count },
                    { label: "Final answers", value: artifactBundleInspection()!.manifest_counts!.final_answer_count },
                    { label: "Issues", value: artifactBundleInspection()!.manifest_counts!.issue_count },
                  ])}
                  {artifactBundleInspection()!.manifest_counts!.sources.length > 0 ? (
                    <ul class="final-answer-list-items">
                      {artifactBundleInspection()!.manifest_counts!.sources.map((item) => (
                        <li>
                          <div class="final-answer-list-item">
                            <span>{item.source_id}</span>
                            <small>
                              drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count} | issues={item.issue_count}
                            </small>
                            {item.final_answers.length > 0 && (
                              <small>
                                {item.final_answers.map((answer) => answer.final_answer_id).join(", ")}
                              </small>
                            )}
                          </div>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p>No manifest sources listed yet.</p>
                  )}
                </div>
              ) : null}
              {artifactBundleInspection()!.summary_counts ? (
                <div class="artifact-overview">
                  <h4>Summary counts</h4>
                  {renderMetricGrid([
                    { label: "Schema", value: artifactBundleInspection()!.summary_counts!.schema_version || "missing" },
                    { label: "Export ID", value: artifactBundleInspection()!.summary_counts!.export_id },
                    { label: "Generated from", value: artifactBundleInspection()!.summary_counts!.generated_from },
                    { label: "Scope", value: artifactBundleInspection()!.summary_counts!.export_scope },
                    { label: "Sources", value: artifactBundleInspection()!.summary_counts!.source_count },
                    { label: "Drafts", value: artifactBundleInspection()!.summary_counts!.draft_count },
                    { label: "Grounded answers", value: artifactBundleInspection()!.summary_counts!.grounded_answer_count },
                    { label: "Final answers", value: artifactBundleInspection()!.summary_counts!.final_answer_count },
                    { label: "Issues", value: artifactBundleInspection()!.summary_counts!.issue_count },
                  ])}
                  {artifactBundleInspection()!.summary_counts!.issue_kinds.length > 0 && (
                    <ul class="final-answer-list-items">
                      {artifactBundleInspection()!.summary_counts!.issue_kinds.map((item) => (
                        <li>
                          <div class="final-answer-list-item">
                            <span>{item.issue_kind}</span>
                            <small>count={item.count}</small>
                          </div>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              ) : null}
              {artifactBundleInspection()!.integrity_counts ? (
                <div class="artifact-overview">
                  <h4>Integrity metadata</h4>
                  {renderMetricGrid([
                    { label: "Schema", value: artifactBundleInspection()!.integrity_counts!.schema_version || "missing" },
                    { label: "Algorithm", value: artifactBundleInspection()!.integrity_counts!.algorithm },
                    { label: "Files", value: artifactBundleInspection()!.integrity_counts!.files.length },
                  ])}
                </div>
              ) : null}
              {artifactBundleInspection()!.issue_kind_counts && artifactBundleInspection()!.issue_kind_counts!.length > 0 ? (
                <div class="artifact-overview">
                  <h4>Issue kind counts</h4>
                  <ul class="final-answer-list-items">
                    {artifactBundleInspection()!.issue_kind_counts!.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.issue_kind}</span>
                          <small>count={item.count}</small>
                        </div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
              {artifactBundleInspection()!.errors.length > 0 ? (
                <div class="warning-box">
                  <h4>Inspection errors</h4>
                  <ul>
                    {artifactBundleInspection()!.errors.map((item) => (
                      <li>
                        <strong>{item.kind}</strong>
                        <div>{item.message}</div>
                        {item.relative_path && <small>{item.relative_path}</small>}
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
              {artifactBundleInspection()!.warnings.length > 0 ? (
                <div class="warning-box">
                  <h4>Inspection warnings</h4>
                  <ul>
                    {artifactBundleInspection()!.warnings.map((item) => (
                      <li>
                        <strong>{item.kind}</strong>
                        <div>{item.message}</div>
                        {item.relative_path && <small>{item.relative_path}</small>}
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
            </>
          ) : (
            <p>No export bundle inspection loaded yet.</p>
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
        <div class="artifact-overview">
          <h3>Retrieval index</h3>
          <p class="muted">Read-only retrieval metadata for the source ID above.</p>
          <div class="hero-actions">
            <button onClick={loadRetrievalIndex} disabled={retrievalIndexLoading()}>
              {retrievalIndexLoading() ? "Loading..." : "Load retrieval index"}
            </button>
          </div>
          {retrievalIndex() ? (
            <>
              <div class="contract-meta">
                <div><span>Source ID</span><strong>{retrievalIndex()!.source_id}</strong></div>
                <div><span>Version ID</span><strong>{retrievalIndex()!.version_id}</strong></div>
                <div><span>Indexed at</span><strong>{retrievalIndex()!.indexed_at}</strong></div>
                <div><span>Chunk count</span><strong>{retrievalIndex()!.chunk_count}</strong></div>
                <div><span>Index version</span><strong>{retrievalIndex()!.index_version}</strong></div>
                <div><span>Chunk report hash</span><strong>{retrievalIndex()!.chunk_report_hash}</strong></div>
                <div><span>Entries</span><strong>{retrievalIndex()!.entries.length}</strong></div>
                <div><span>Warnings</span><strong>{retrievalIndex()!.warnings.length}</strong></div>
              </div>
              {retrievalIndex()!.warnings.length > 0 ? (
                <ul class="final-answer-list-items">
                  {retrievalIndex()!.warnings.map((warning) => (
                    <li>
                      <div class="final-answer-list-item">
                        <span>{warning}</span>
                      </div>
                    </li>
                  ))}
                </ul>
              ) : null}
            </>
          ) : (
            <p>No retrieval index loaded yet.</p>
          )}
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
