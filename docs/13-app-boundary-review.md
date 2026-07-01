# 13 - App.tsx Size and Boundary Review

## Purpose

This document records safe future extraction boundaries for `src/App.tsx` without starting a refactor now.

Phase 152.0 is review-only and docs-only:

- no code movement
- no product logic change
- no UI change
- no backend change
- no dependency change

The build warning about `src/App.tsx` exceeding the Babel 500KB heuristic is real, but it is not a reason for a large immediate rewrite. The correct response is a small-phase extraction plan with explicit guardrails.

## Big-bang warning

Do not attempt a broad "split App.tsx" pass in one phase.

Reasons:

- `App.tsx` still coordinates cross-workspace state
- many loaders and diagnostics share sanitization and selection state
- artifact, runtime, export, and retrieval panels still depend on the same top-level signals
- large diffs would make regressions and stale state bugs hard to detect

The next refactor phases should stay mechanical, additive, and verifiable.

## Current structure of `src/App.tsx`

The file currently combines several distinct concerns.

### 1. Type and DTO layer

The top of the file is dominated by frontend DTO and preview-contract types.

Observed groups include:

- Scholar Chat request, transcript, workflow-preview, execution-gate, retrieval, evidence-plan, and prompt-pack types
- scientific metadata preview and provider-request types
- runtime validation, runtime probe, managed llama-server, and smoke-diagnostic types
- retrieval index and retrieval search types
- answer artifact, export manifest, bundle inspection, and final-answer types

This area is structurally large but mechanically extractable because it is mostly declarative.

### 2. Shared helper layer

`App.tsx` includes several general helpers before `App()` starts:

- `sanitizeBackendError`
- `locatorSummary`
- `compactTextPreview`
- `renderMetricGrid`

These are reused across diagnostics and artifact views and are strong low-risk candidates.

### 3. Top-level orchestration state

`App()` begins around line 2273 and immediately declares a very large number of Solid signals.

The signal groups include:

- Scholar Chat prompt, transcript, preview, retrieval, evidence, prompt-pack, readiness, draft-inspection, and grounded-draft diagnostics
- local runtime configuration, validation, invocation, smoke, capability, and readiness diagnostics
- managed llama-server configuration and lifecycle diagnostics
- source/artifact/final-answer/retrieval/export/bundle-inspection selection and loading state

This is the main reason a big-bang refactor would be risky. The state is not merely large; it is the current coordination surface.

### 4. Loader and invoke-handler layer

After signal declaration, `App.tsx` contains many invoke-driven loaders and preview handlers.

Representative families:

- corpus status and source context loading
- final answer and artifact overview loading
- retrieval index and retrieval search loading
- artifact sources, artifact health, issues, manifest, export, and bundle inspection loading
- Scholar Chat preview and runtime diagnostic preview loading
- managed llama-server preview and diagnostic loading

Many of these functions are read-only, but they still close over top-level selection state and error state.

### 5. Read-only diagnostics and artifact render layer

A large part of the file is read-only diagnostics rendering:

- metric-grid-driven preview cards
- retrieval result cards
- managed runtime diagnostic panels
- artifact overview, manifest, issues, and inspection panels
- final answer preview and statement detail panels

This is the safest presentation-heavy area for future extraction, as long as handler ownership stays in `App.tsx` at first.

### 6. Final composition layer

The render tail mounts:

- `WorkspaceShell`
- `SourcesWorkspace`
- `ScholarChatWorkspace`
- `EvidencePacksWorkspace`

Those workspace components already reduce the product-workflow surface inside `App.tsx`, but the read-only diagnostic and artifact areas remain centralized.

## Existing workspace boundaries

### `SourcesWorkspace.tsx`

Already owns:

- source import types local to the Sources flow
- local wizard signals
- explicit register / extract / chunk / index actions
- source status summary and source list presentation

Boundary quality:

- good for the import workflow itself
- still depends on top-level callbacks and formatting helpers passed from `App.tsx`

### `EvidencePacksWorkspace.tsx`

Already owns:

- Evidence Pack build flow
- Answer Draft build flow
- Grounded Answer build flow
- Final Answer build flow
- local status handling and flow-lock behavior

Boundary quality:

- good for workflow mutation UI
- still depends on `App.tsx` for shared source context, evidence-pack list data, sanitizer, and refresh callbacks

### `ScholarChatWorkspace.tsx`

Already exists as a workspace boundary, but a large amount of Scholar Chat and runtime-related state and diagnostics still live in `App.tsx`.

Boundary quality:

- current separation is only partial
- future work should prefer read-only diagnostics extraction before touching prompt/runtime state ownership

### `WorkspaceShell.tsx`

Already a clean UI shell.

Boundary quality:

- low risk
- does not need further review in this phase

## Guardrails for future extraction

Any later extraction phase must preserve these boundaries:

- Rust remains the authority for filesystem access, source mutation, export, and local-path validation
- no new automatic mutation chains
- no change to explicit user-click gates
- normal UI cards remain path-free
- locator contents must not be broadened accidentally during extraction
- citation and locator rendering boundaries must remain unchanged
- export and filesystem gates must remain explicit and easy to audit
- status and readiness semantics must remain stable
- large multi-area diffs are to be avoided

## Extraction candidates by risk

## Low-risk candidates

These are strong candidates for small mechanical follow-up phases.

| Candidate | Current area | Suggested target file | Dependencies | Risk | Recommended phase | Mechanical Codex refactor later |
| --- | --- | --- | --- | --- | --- | --- |
| Shared backend error sanitizer | `sanitizeBackendError` in `App.tsx` helper block | `src/lib/ui/sanitizeBackendError.ts` | pure string/error normalization; used by top-level loaders and workspaces | low | 153.x | yes |
| Shared locator formatter | `locatorSummary` in `App.tsx` helper block | `src/lib/ui/locatorSummary.ts` | depends only on locator shape | low | 153.x | yes |
| Shared text preview helper | `compactTextPreview` in `App.tsx` helper block | `src/lib/ui/textPreview.ts` | pure string formatting | low | 153.x | yes |
| Metric grid renderer | `renderMetricGrid` helper | `src/components/diagnostics/MetricGrid.tsx` | read-only JSX helper; many callers | low | 153.x | yes |
| Read-only constant lists | option sets such as workspace sections and prompt suggestions | `src/lib/constants/*` later | compile-time data only | low | 153.x | yes |
| Artifact export preview gate presentation | current read-only artifact readiness card in `App.tsx` | `src/components/artifacts/ArtifactExportReadinessCard.tsx` | metric grid, booleans, formatted props | low | 154.x | yes |
| Artifact overview presentation | read-only overview and selected-final-answer preview blocks | `src/components/artifacts/ArtifactOverviewPanel.tsx` | receives already-loaded data and callbacks | low | 154.x | yes |
| Artifact health / issues / manifest presentation | read-only diagnostics cards | `src/components/artifacts/ArtifactHealthPanel.tsx`, `ArtifactIssuesPanel.tsx`, `ArtifactManifestPanel.tsx` | metric grid, sanitized strings, lists | low | 154.x | yes |
| Retrieval result cards | read-only retrieval index/search display blocks | `src/components/retrieval/RetrievalInspectorPanel.tsx` | locator formatter, loaded DTOs, button callbacks | low | 154.x | yes |
| Final answer read-only statement cards | selected-final-answer and final-answer preview JSX | `src/components/artifacts/FinalAnswerPreviewPanel.tsx` | locator formatter, loaded DTOs | low | 154.x | yes |

## Medium-risk candidates

These should follow only after low-risk helpers and pure UI extraction are stable.

| Candidate | Current area | Suggested target file | Dependencies | Risk | Recommended phase | Mechanical Codex refactor later |
| --- | --- | --- | --- | --- | --- | --- |
| Readiness derivation helpers | export/artifact/runtime readiness decisions derived inside `App.tsx` | `src/lib/readiness/*` | depends on many DTOs and booleans but can stay pure if inputs are explicit | medium | 155.x | yes, if kept pure |
| Scholar Chat diagnostic formatting helpers | transcript/preview display helpers and summary formatting | `src/lib/scholarChat/formatters.ts` | coupled to many preview DTOs | medium | 155.x | yes |
| Artifact selection view-model helpers | overview/detail selection and display normalization | `src/lib/artifacts/viewModels.ts` | depends on top-level DTO shapes and current selection conventions | medium | 155.x | yes |
| Developer diagnostics subsection components | large read-only diagnostic subsections still rendered in `App.tsx` | `src/components/diagnostics/*Panel.tsx` | many props, but no mutation ownership if kept presentational | medium | 154.x or 155.x | yes |
| Shared type modules for large DTO families | preview-contract types near the top of `App.tsx` | `src/types/*` later | many imports across workspaces and app diagnostics | medium | 153.x or 155.x | yes, but sequence carefully |

## High-risk candidates

These should not be first extraction targets.

| Candidate | Current area | Suggested target file | Dependencies | Risk | Recommended phase | Mechanical Codex refactor later |
| --- | --- | --- | --- | --- | --- | --- |
| Top-level mutation handlers | invoke-based functions that write, export, start runtime flows, or coordinate refresh order | none yet; keep in `App.tsx` first | tightly coupled to multiple signals, error setters, and refresh chains | high | later than 155.x | only after preceding simplification |
| Cross-workspace selection state | source IDs, final-answer selection, evidence-pack source selection, runtime consent toggles | none yet | shared across diagnostics, artifact views, and workspaces | high | later than 155.x | not as a first mechanical pass |
| Loader families that close over many setters | artifact loaders, runtime loaders, chat preview loaders | none yet or future controller modules | high coupling to signal ownership and error semantics | high | later than 155.x | only after props/state contracts are explicit |
| Export / bundle inspection orchestration | export root, inspection root, export result, inspection result, related errors | none yet | filesystem gate semantics, path-related diagnostics, artifact consistency | high | later than 155.x | only after presentation is extracted |
| Runtime and managed-server orchestration | runtime config, probe, smoke, invocation, managed llama-server status/start/stop | none yet | consent gates, lifecycle ordering, multiple preview modes | high | later than 155.x | not recommended early |
| Large Scholar Chat orchestration block | prompt state, transcript state, preview loaders, grounding inspection, readiness flows | none yet | many preview DTOs and cross-panel dependencies | high | later than 155.x | only after helper and panel extraction |

## Recommended extraction order

The safest order is:

1. Extract pure helpers and constants.
2. Extract read-only UI panels that can receive data and callbacks as props.
3. Extract pure derived selectors and readiness helpers with explicit input arguments.
4. Reassess actual coupling after those smaller phases land.
5. Touch mutation handlers and cross-workspace orchestration only in later, separately reviewed slices.

This order keeps behavior stable while reducing file size and render noise first.

## Suggested follow-up phases

### Phase 153.x - Pure helper and type boundary pass

Scope:

- move shared UI helpers out of `App.tsx`
- move the safest DTO/type families out of `App.tsx`
- keep all top-level signal ownership and all invoke handlers in place

Verification focus:

- no behavior change
- import-only diff outside helpers/types
- build and cargo check remain green

### Phase 154.x - Read-only diagnostics and artifact UI extraction

Scope:

- extract read-only panels from `App.tsx` into presentational components
- pass loaded DTOs and callbacks in via props
- keep loaders and mutations in `App.tsx`

Verification focus:

- rendered output unchanged
- no new invoke sites
- no export or filesystem semantic change

### Phase 155.x - Pure derived selector and readiness extraction

Scope:

- extract pure readiness and view-model helpers
- keep signal ownership in `App.tsx`
- keep mutation ordering and refresh sequencing unchanged

Verification focus:

- no state machine change
- no status/readiness regression
- no stale-selection regressions

## What should not happen early

The following should explicitly stay out of the first extraction phases:

- moving invoke-heavy mutation handlers into new modules first
- changing top-level signal ownership before helper/UI extraction reduces pressure
- mixing artifact export refactors with runtime diagnostics refactors
- moving multiple high-risk domains in one phase
- rewriting `App.tsx` structure for aesthetics alone

## Recommendation

Treat `App.tsx` reduction as a sequence of small mechanical cleanup phases, not as a one-time architecture rewrite.

The best next move is to reduce presentation and helper bulk first, then reevaluate the remaining orchestration complexity with smaller diffs and clearer boundaries.


## Phase 153.0 result

Phase 153.0 completes the first low-risk extraction promised by this review:

- pure frontend types moved to `src/appTypes.ts`
- stateless helpers moved to `src/appHelpers.ts`
- `renderMetricGrid`, constants, signals, loaders, invoke handlers, mutation flows, and JSX stayed in `src/App.tsx`

This confirms the recommended order: types and state-free helpers can move first without touching orchestration or render boundaries.
