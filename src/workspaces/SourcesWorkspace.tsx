import { createSignal, JSX, Setter } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

type CorpusStatus = {
  source_count: number;
  registered_count: number;
  extracted_count: number;
  failed_count: number;
};

type RegisteredSource = {
  source_id: string;
  version_id: string;
  title: string;
  source_type: string;
  ingestion_status: string;
};

type SourceImportSourceType = "pdf" | "markdown_note" | "dataset_note" | "web_snapshot";
type SourceImportStep = "register" | "extract" | "chunk" | "index";
type SourceImportStepStatus = "not_started" | "running" | "succeeded" | "failed";

type SourceMetadataInput = {
  title: string;
  source_type: SourceImportSourceType;
  discipline: string;
  subdiscipline: string | null;
  language: string;
  tags: string[];
  reliability_notes: string | null;
};

const SOURCE_TYPE_OPTIONS: { value: SourceImportSourceType; label: string; description: string }[] = [
  { value: "pdf", label: "PDF (text layer only)", description: "Requires an extractable text layer. OCR is not part of this phase." },
  { value: "markdown_note", label: "Markdown note", description: "Best for local notes and hand-curated source summaries." },
  { value: "dataset_note", label: "Dataset note", description: "Use for structured local dataset notes or annotated records." },
  { value: "web_snapshot", label: "Web snapshot (local UTF-8)", description: "Use a local UTF-8 snapshot file that was captured earlier. No live web, scraping, download, or browser access." },
];

const STEP_LABELS: Record<SourceImportStep, string> = {
  register: "Register source",
  extract: "Extract source",
  chunk: "Chunk source",
  index: "Build retrieval index",
};

const STEP_DESCRIPTIONS: Record<SourceImportStep, string> = {
  register: "Registers the local file into Corpus Authority and creates a stable source ID.",
  extract: "Extracts text into a managed extraction report for the registered source.",
  chunk: "Splits extracted text into source-linked chunks with preserved locators.",
  index: "Builds the local retrieval index from the chunk report.",
};

function sanitizeBackendError(error: unknown) {
  return String(error)
    .replace(/[A-Za-z]:\\[^"'\n]+/g, "[path hidden]")
    .replace(/\.aegis[\\/][^"'\n]+/g, "[path hidden]");
}

function explainImportError(error: unknown) {
  const message = sanitizeBackendError(error);
  if (message.includes("PDF text layer missing")) {
    return "PDF has no extractable text layer. OCR is not supported in this phase.";
  }
  if (message.includes("Unsupported extraction type")) {
    return "This source type is not supported by the current extraction pipeline.";
  }
  if (message.includes("Source path does not exist")) {
    return "The local path does not exist.";
  }
  if (message.includes("Source path must point to a file")) {
    return "The local path must point to a file.";
  }
  if (message.includes("Source path cannot be inside the corpus workspace")) {
    return "The selected path is inside the managed corpus workspace and cannot be imported.";
  }
  if (message.includes("Extraction input is not valid UTF-8")) {
    return "This snapshot is not valid UTF-8 and cannot be imported as a local web snapshot.";
  }
  return message;
}

function normalizeCommaSeparatedList(raw: string) {
  return raw
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}

function emptyStepStatus(): Record<SourceImportStep, SourceImportStepStatus> {
  return {
    register: "not_started",
    extract: "not_started",
    chunk: "not_started",
    index: "not_started",
  };
}

function statusLabel(status: SourceImportStepStatus) {
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

function wizardStepClass(status: SourceImportStepStatus) {
  return `status-pill status-${status}`;
}

export default function SourcesWorkspace(props: any): JSX.Element {
  const [sourceImportPath, setSourceImportPath] = createSignal("");
  const [sourceImportTitle, setSourceImportTitle] = createSignal("");
  const [sourceImportType, setSourceImportType] = createSignal<SourceImportSourceType>("markdown_note");
  const [sourceImportDiscipline, setSourceImportDiscipline] = createSignal("psychology");
  const [sourceImportSubdiscipline, setSourceImportSubdiscipline] = createSignal("statistics");
  const [sourceImportLanguage, setSourceImportLanguage] = createSignal("en");
  const [sourceImportTags, setSourceImportTags] = createSignal("");
  const [sourceImportReliabilityNotes, setSourceImportReliabilityNotes] = createSignal("");
  const [sourceImportCurrentSource, setSourceImportCurrentSource] = createSignal<RegisteredSource | null>(null);
  const [sourceImportStepStatus, setSourceImportStepStatus] = createSignal<Record<SourceImportStep, SourceImportStepStatus>>(emptyStepStatus());
  const [sourceImportBusyStep, setSourceImportBusyStep] = createSignal<SourceImportStep | null>(null);
  const [sourceImportError, setSourceImportError] = createSignal<string | null>(null);
  const [sourceImportValidationError, setSourceImportValidationError] = createSignal<string | null>(null);

  function resetImportProgress() {
    setSourceImportStepStatus(emptyStepStatus());
    setSourceImportBusyStep(null);
    setSourceImportError(null);
    setSourceImportValidationError(null);
    setSourceImportCurrentSource(null);
  }

  function setField<T>(setter: Setter<T>, value: T) {
    resetImportProgress();
    setter(() => value);
  }

  function buildMetadataInput(): SourceMetadataInput | null {
    const trimmedPath = sourceImportPath().trim();
    const trimmedTitle = sourceImportTitle().trim();
    const trimmedDiscipline = sourceImportDiscipline().trim();
    const trimmedLanguage = sourceImportLanguage().trim();
    const trimmedSubdiscipline = sourceImportSubdiscipline().trim();
    const tags = normalizeCommaSeparatedList(sourceImportTags());
    const reliabilityNotes = sourceImportReliabilityNotes().trim();

    if (!trimmedPath) {
      setSourceImportValidationError("Local path is required.");
      return null;
    }
    if (!trimmedTitle) {
      setSourceImportValidationError("Title is required.");
      return null;
    }
    if (!trimmedDiscipline) {
      setSourceImportValidationError("Discipline is required.");
      return null;
    }
    if (!trimmedLanguage) {
      setSourceImportValidationError("Language is required.");
      return null;
    }

    return {
      title: trimmedTitle,
      source_type: sourceImportType(),
      discipline: trimmedDiscipline,
      subdiscipline: trimmedSubdiscipline ? trimmedSubdiscipline : null,
      language: trimmedLanguage,
      tags,
      reliability_notes: reliabilityNotes ? reliabilityNotes : null,
    };
  }

  async function refreshCorpusViews() {
    await Promise.all([props.refreshCorpusStatus(), props.refreshSourceContext(true)]);
  }

  function canRunStep(step: SourceImportStep) {
    if (sourceImportBusyStep()) {
      return false;
    }
    const statuses = sourceImportStepStatus();
    switch (step) {
      case "register":
        return true;
      case "extract":
        return !!sourceImportCurrentSource() && statuses.register === "succeeded";
      case "chunk":
        return !!sourceImportCurrentSource() && statuses.extract === "succeeded";
      case "index":
        return !!sourceImportCurrentSource() && statuses.chunk === "succeeded";
    }
  }

  function markStepRunning(step: SourceImportStep) {
    setSourceImportBusyStep(step);
    setSourceImportError(null);
    setSourceImportValidationError(null);
    setSourceImportStepStatus((current) => ({
      ...current,
      [step]: "running",
    }));
  }

  function markStepResult(step: SourceImportStep, status: SourceImportStepStatus, error?: string) {
    setSourceImportStepStatus((current) => ({
      ...current,
      [step]: status,
    }));
    setSourceImportError(error ?? null);
  }

  async function runRegisterSource() {
    if (sourceImportBusyStep()) {
      return;
    }

    const metadata = buildMetadataInput();
    const trimmedPath = sourceImportPath().trim();
    if (!metadata) {
      return;
    }

    markStepRunning("register");
    try {
      const result = await invoke<RegisteredSource>("register_source", {
        root: ".",
        path: trimmedPath,
        metadata,
      });
      setSourceImportCurrentSource(result);
      markStepResult("register", "succeeded");
      try {
        await refreshCorpusViews();
      } catch (refreshError) {
        setSourceImportError(explainImportError(refreshError));
      }
    } catch (error) {
      markStepResult("register", "failed", explainImportError(error));
    } finally {
      setSourceImportBusyStep(null);
    }
  }

  async function runExtractSource() {
    const source = sourceImportCurrentSource();
    if (sourceImportBusyStep() || !source) {
      return;
    }

    markStepRunning("extract");
    try {
      await invoke("extract_source", {
        root: ".",
        source_id: source.source_id,
      });
      markStepResult("extract", "succeeded");
      try {
        await refreshCorpusViews();
      } catch (refreshError) {
        setSourceImportError(explainImportError(refreshError));
      }
    } catch (error) {
      markStepResult("extract", "failed", explainImportError(error));
    } finally {
      setSourceImportBusyStep(null);
    }
  }

  async function runChunkSource() {
    const source = sourceImportCurrentSource();
    if (sourceImportBusyStep() || !source) {
      return;
    }

    markStepRunning("chunk");
    try {
      await invoke("chunk_source", {
        root: ".",
        source_id: source.source_id,
      });
      markStepResult("chunk", "succeeded");
      try {
        await refreshCorpusViews();
      } catch (refreshError) {
        setSourceImportError(explainImportError(refreshError));
      }
    } catch (error) {
      markStepResult("chunk", "failed", explainImportError(error));
    } finally {
      setSourceImportBusyStep(null);
    }
  }

  async function runBuildRetrievalIndex() {
    const source = sourceImportCurrentSource();
    if (sourceImportBusyStep() || !source) {
      return;
    }

    markStepRunning("index");
    try {
      await invoke("build_retrieval_index", {
        root: ".",
        source_id: source.source_id,
      });
      markStepResult("index", "succeeded");
      try {
        await refreshCorpusViews();
      } catch (refreshError) {
        setSourceImportError(explainImportError(refreshError));
      }
    } catch (error) {
      markStepResult("index", "failed", explainImportError(error));
    } finally {
      setSourceImportBusyStep(null);
    }
  }

  const currentSource = () => {
    const imported = sourceImportCurrentSource();
    if (!imported) {
      return null;
    }
    return props.sourceContext.find((item: RegisteredSource) => item.source_id === imported.source_id) ?? imported;
  };

  const sourceImportSummary = () => {
    const statuses = sourceImportStepStatus();
    const current = currentSource();
    if (!current) {
      return "Register a local file to start the import pipeline.";
    }
    if (statuses.index === "failed" || statuses.chunk === "failed" || statuses.extract === "failed" || statuses.register === "failed") {
      return "One step failed. Check the error message above, then retry the failed step.";
    }
    if (statuses.index === "succeeded") {
      return `Source ${current.title || current.source_id} is registered, extracted, chunked, and indexed.`;
    }
    if (statuses.chunk === "succeeded") {
      return `Source ${current.title || current.source_id} is ready for retrieval indexing.`;
    }
    if (statuses.extract === "succeeded") {
      return `Source ${current.title || current.source_id} is ready for chunking.`;
    }
    return `Source ${current.title || current.source_id} is registered and ready for extraction.`;
  };

  const sourceImportCurrentStatus = () => {
    const statuses = sourceImportStepStatus();
    if (statuses.index === "succeeded") {
      return "indexed";
    }
    if (statuses.chunk === "succeeded") {
      return "chunked";
    }
    if (statuses.extract === "succeeded") {
      return "extracted";
    }
    if (statuses.register === "succeeded") {
      return "registered";
    }
    return currentSource()?.ingestion_status ?? "not_started";
  };

  return (
    <section class="card workspace-panel" id="sources" data-workspace="sources">
      <div class="sources-workspace-header">
        <div>
          <h2>Sources</h2>
          <p class="muted">
            Register a local file, then run extraction, chunking, and retrieval indexing step by step.
          </p>
        </div>
        <div class="sources-workspace-summary">
          <div class="summary-chip">
            <span>Corpus sources</span>
            <strong>{props.status ? props.status.source_count : 0}</strong>
          </div>
          <div class="summary-chip">
            <span>Registered</span>
            <strong>{props.status ? props.status.registered_count : 0}</strong>
          </div>
          <div class="summary-chip">
            <span>Extracted</span>
            <strong>{props.status ? props.status.extracted_count : 0}</strong>
          </div>
          <div class="summary-chip">
            <span>Failed</span>
            <strong>{props.status ? props.status.failed_count : 0}</strong>
          </div>
        </div>
      </div>

      {props.statusError ? <p class="error">{props.statusError}</p> : null}

      <div class="sources-workspace-grid">
        <section class="warning-box source-import-panel">
          <div class="source-import-panel-header">
            <div>
              <h3>Source Import Wizard MVP</h3>
              <p class="muted">
                Local-first onboarding for existing Corpus Authority, extraction, chunking, and retrieval commands.
              </p>
            </div>
            <span class={`status-pill ${wizardStepClass(sourceImportBusyStep() ? "running" : "not_started")}`}>
              {sourceImportBusyStep() ? "Running" : "Ready"}
            </span>
          </div>

          <div class="form-row">
            <label>
              Local path
              <input
                type="text"
                value={sourceImportPath()}
                onInput={(event) => setField(setSourceImportPath, event.currentTarget.value)}
                placeholder="C:\\Users\\you\\Documents\\source.pdf"
                disabled={!!sourceImportBusyStep()}
              />
            </label>
            <label>
              Title
              <input
                type="text"
                value={sourceImportTitle()}
                onInput={(event) => setField(setSourceImportTitle, event.currentTarget.value)}
                placeholder="Lecture 01"
                disabled={!!sourceImportBusyStep()}
              />
            </label>
            <label>
              Source type
              <select
                value={sourceImportType()}
                onChange={(event) => setField(setSourceImportType, event.currentTarget.value as SourceImportSourceType)}
                disabled={!!sourceImportBusyStep()}
              >
                {SOURCE_TYPE_OPTIONS.map((item) => (
                  <option value={item.value}>{item.label}</option>
                ))}
              </select>
              <small class="muted">
                {SOURCE_TYPE_OPTIONS.find((item) => item.value === sourceImportType())?.description}
              </small>
            </label>
          </div>

          <details class="advanced-panels source-import-advanced">
            <summary>Optional metadata</summary>
            <div class="form-row">
              <label>
                Discipline
                <input
                  type="text"
                  value={sourceImportDiscipline()}
                  onInput={(event) => setField(setSourceImportDiscipline, event.currentTarget.value)}
                  placeholder="psychology"
                  disabled={!!sourceImportBusyStep()}
                />
              </label>
              <label>
                Language
                <input
                  type="text"
                  value={sourceImportLanguage()}
                  onInput={(event) => setField(setSourceImportLanguage, event.currentTarget.value)}
                  placeholder="en"
                  disabled={!!sourceImportBusyStep()}
                />
              </label>
              <label>
                Subdiscipline
                <input
                  type="text"
                  value={sourceImportSubdiscipline()}
                  onInput={(event) => setField(setSourceImportSubdiscipline, event.currentTarget.value)}
                  placeholder="statistics"
                  disabled={!!sourceImportBusyStep()}
                />
              </label>
              <label>
                Tags
                <input
                  type="text"
                  value={sourceImportTags()}
                  onInput={(event) => setField(setSourceImportTags, event.currentTarget.value)}
                  placeholder="lecture, statistics, methods"
                  disabled={!!sourceImportBusyStep()}
                />
              </label>
              <label class="source-import-full-span">
                Reliability notes
                <input
                  type="text"
                  value={sourceImportReliabilityNotes()}
                  onInput={(event) => setField(setSourceImportReliabilityNotes, event.currentTarget.value)}
                  placeholder="Local note, verified text layer"
                  disabled={!!sourceImportBusyStep()}
                />
              </label>
            </div>
          </details>

          {sourceImportValidationError() ? <p class="error">{sourceImportValidationError()}</p> : null}
          {sourceImportError() ? <p class="error">{sourceImportError()}</p> : null}

          <div class="source-import-steps">
            {(
              [
                { step: "register", onClick: runRegisterSource },
                { step: "extract", onClick: runExtractSource },
                { step: "chunk", onClick: runChunkSource },
                { step: "index", onClick: runBuildRetrievalIndex },
              ] as const
            ).map((item) => {
              const stepStatus = sourceImportStepStatus()[item.step];
              return (
                <article class="source-import-step">
                  <div class="source-import-step-heading">
                    <div>
                      <h4>{STEP_LABELS[item.step]}</h4>
                      <p class="muted">{STEP_DESCRIPTIONS[item.step]}</p>
                    </div>
                    <span class={wizardStepClass(stepStatus)}>{statusLabel(stepStatus)}</span>
                  </div>
                  <div class="hero-actions">
                    <button onClick={item.onClick} disabled={!canRunStep(item.step)}>
                      {sourceImportBusyStep() === item.step ? "Running..." : STEP_LABELS[item.step]}
                    </button>
                  </div>
                </article>
              );
            })}
          </div>

          <div class="contract-meta source-import-state">
            <div>
              <span>Current source</span>
              <strong>{currentSource()?.title || currentSource()?.source_id || "No source yet"}</strong>
            </div>
            <div>
              <span>Source ID</span>
              <strong>{currentSource()?.source_id || "n/a"}</strong>
            </div>
            <div>
              <span>Version ID</span>
              <strong>{currentSource()?.version_id || "n/a"}</strong>
            </div>
            <div>
              <span>Status</span>
              <strong>{props.formatSnakeCaseLabel(sourceImportCurrentStatus())}</strong>
            </div>
          </div>
          <p class="muted">{sourceImportSummary()}</p>
          <p class="muted">
            Supported types: PDF with text layer, markdown notes, dataset notes, and web snapshots already supported by the backend extraction contract.
          </p>
        </section>

        <section class="compact-note source-context-panel">
          <h3>Source context</h3>
          <p class="muted">
            Source selection stays available for Scholar Chat. The wizard updates this list after each successful step.
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
              <ul class="final-answer-list-items source-context-list">
                {props.sourceContext.map((item: RegisteredSource) => {
                  const selected = props.sourceContextSelectedIds.includes(item.source_id);
                  const current = currentSource()?.source_id === item.source_id;
                  return (
                    <li>
                      <label classList={{ "final-answer-list-item": true, "source-context-item": true, selected, current }}>
                        <span class="source-context-main">
                          <input
                            type="checkbox"
                            checked={selected}
                            onChange={() => {
                              props.toggleSourceContext(item.source_id);
                              props.setScholarChatPreview(null);
                              props.setScholarChatExecutionGatePreview(null);
                            }}
                          />
                          <strong>{item.title || item.source_id}</strong>
                        </span>
                        <small>
                          source_id={item.source_id} | type={props.formatSnakeCaseLabel(item.source_type)} | version={item.version_id} | status={props.formatSnakeCaseLabel(item.ingestion_status)}
                        </small>
                      </label>
                    </li>
                  );
                })}
              </ul>
            </>
          )}
          {props.sourceContext.length > 0 ? props.renderSourceWorkflowActionHints() : null}
          <p class="muted">{props.selectedSourceSummary}</p>
        </section>
      </div>

      <details class="advanced-panels source-diagnostics-panel">
        <summary>Diagnostics and raw corpus state</summary>
        <div class="compact-note">
          <h4>Corpus status</h4>
          {props.status ? <pre>{JSON.stringify(props.status, null, 2)}</pre> : <p>No status loaded yet.</p>}
        </div>
      </details>
    </section>
  );
}
