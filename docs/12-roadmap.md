# 12 — Roadmap

## Phase 0 — Documentation baseline

Status: done.

## Phase 0.5 — Stack Decisions Freeze

Locks stack decisions and excludes Pi/MCP/coding-agent behavior from product core.

## Phase 0.7 — Research Foundation

Adds contracts before implementation:

- source schemas
- chunk schemas
- retrieval profile schemas
- evidence pack schema
- skill schema
- eval case schema
- initial skill files

## Phase 1 — Corpus Authority + Source Registry

Implement:

- source registration
- stable source IDs
- source version IDs
- metadata validation
- content hashing
- duplicate-source detection
- corpus status
- source audit events

Do not implement:

- model runtime
- embeddings
- RAG answer synthesis
- coding agent behavior
- UI redesign
- Pi/MCP

## Phase 2 — Text Extraction + Locator Preservation

Implement extraction for local text sources while preserving paragraph and section locators.

Phase 2.0 stores extraction reports under the managed corpus tree, supports `markdown_note`, `dataset_note` and `web_snapshot`, and returns a typed unsupported error for PDFs and slides until narrow locator-safe extraction is added later.

## Phase 3 — Chunking + Metadata Index

Implement deterministic chunk generation from extraction reports, preserving source/version/locator continuity.

Phase 3 chunk reports live under the managed corpus tree and do not add embeddings or retrieval indexes.

## Phase 4 — Hybrid Retrieval MVP

Implement a deterministic local retrieval index and lexical search over chunk reports.

Phase 4.0 stores retrieval indexes under the managed corpus tree and does not add embeddings, answer synthesis or Evidence Packs.
Phase 4.1 hardens read-back, query normalization, on-demand index build, and deterministic score/tie-break behavior without adding semantic retrieval.

## Phase 5 — Evidence Pack Builder

Build source-grounded evidence packs before answer synthesis.

Phase 5.0 stores deterministic evidence packs as JSON under the managed corpus tree and does not add embeddings, vector search, or answer synthesis.

## Phase 6 — Skill Runtime MVP

Load declarative skills and route tasks to retrieval profiles and output contracts.

Phase 6.0 introduces a mechanical Answer Draft scaffold built only from local Evidence Packs; it does not generate final answers or citations.

## Phase 7 — Psychology/Statistics MVP

Add APA output, statistics explanations, method checks and course study workflows.

Phase 7.0 introduces a mechanical Grounded Answer contract built only from Answer Drafts; it does not generate final prose or new claims.

## Phase 8 — Local Model Runtime

Implement llama.cpp process lifecycle, health, logs, port/PID ownership and model profiles.

Phase 8.0 introduces a deterministic Final Answer contract built only from Grounded Answers; it does not generate final prose, add semantic ranking, or add LLM inference.

Phase 9.0 documents a stable regression target for the answer contract pipeline:
`cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture`
and
`cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture`.
These checks cover mechanical answer draft, grounded answer, and final answer persistence and read-back only.

Phase 9.0 does not add final prose synthesis, semantic ranking, or UI answer presentation.

## Phase 9 — Composer UI

Show sources, evidence packs, retrieval mode, diagnostics and output controls.

## Phase 10 — Mathematics / MINT Expansion

Add mathematics, computer science, biology and related discipline profiles gradually.

Phase 10.0 and Phase 10.1 implement only a read-only Final Answer inspector for debugging and contract inspection.
This is not a product answer experience, and it does not add generation, synthesis, or editing workflows.

Phase 11.0 adds read-only discovery for already persisted Final Answer contracts so the inspector can list and select them.
Phase 11.1 hardens that discovery boundary with deterministic ordering, metadata-only output, typed malformed-file handling, and no directory-creation side effects.
This remains inspection-only and is still not a product answer experience; it does not introduce generation, synthesis, ranking, or editing.

Phase 12.0 adds a read-only answer-artifact overview that shows existing draft, grounded answer, and final answer counts for a source.
It remains inspection-only, does not create directories or artifacts, and is still not a product answer experience.
Phase 12.1 hardens that boundary with deterministic overview/list alignment, multiple-artifact count coverage, and conservative malformed-final-answer handling.

Phase 13.0 adds a read-only source artifact index that lists sources with persisted answer artifacts and their counts.
It remains inspection-only, does not create directories or artifacts, and is still not a product answer experience.
Phase 13.1 hardens that boundary with deterministic ordering, empty-storage behavior, unrelated-file safety, and conservative malformed-final-answer handling.

Phase 14.0 adds a read-only answer artifact health summary for persisted diagnostics.
It remains inspection-only, reports persisted metadata counts only, is deterministic by `source_id`, and is still not a product answer experience.
Phase 14.1 hardens that boundary with zero-count empty-storage behavior, per-source/global count consistency, path-free DTO/debug output, and conservative malformed-final-answer handling.

Phase 15.0 adds a read-only answer artifact issue list for persisted diagnostics.
It remains inspection-only, reports persisted issue metadata only, and is still not a product answer experience.
There are no repair/fix actions yet.
Phase 15.1 hardens that boundary with deterministic ordering, supported-statement exclusion, and path-free diagnostics.

Phase 16.0 adds a read-only answer artifact export manifest for preview-only inspection.
It remains inspection-only, reports persisted metadata only, does not write export or manifest files, does not add download buttons, and is still not a product answer experience.
There are no repair/fix actions yet.
Phase 16.1 hardens that boundary with deterministic ordering, issue-count rollup, and tolerant preview handling for malformed finals.
This remains preview-only; there is no actual export workflow yet.

Phase 17.0 adds an explicit manual export step for persisted answer artifacts.
It remains artifact-only and a manual user-triggered export step, writes only under the chosen export destination, returns relative exported file paths, and is still not a product answer or share workflow.
There is no automatic export and no repair/fix action yet.
The export destination must be explicit and non-empty.
Phase 17.1 hardens that boundary with explicit destination handling, deterministic export output, empty-destination rejection before filesystem access, and path-safe export manifests.

Phase 18.0 adds a read-only `summary.json` audit file inside the manual export bundle.
It remains artifact-only, is derived from persisted manifest and issues data only, is deterministic and path-free, and is still not a product answer or share workflow.
There is no automatic export and no repair/fix action yet.
Phase 18.1 hardens that boundary with deterministic hash-derived summary identity and manifest/issues alignment.

Phase 19.0 adds a read-only export bundle inspector for existing manual export bundles.
It validates persisted `export_manifest.json`, `export_issues.json`, and `summary.json` metadata only, compares the parsed summary against the derived manifest/issues summary, and reports typed inspection issues without mutating the bundle.
It remains artifact-only and is still not a product answer or share workflow.
There is no import workflow, no automatic export, and no repair/fix action yet.
Phase 19.1 hardens that boundary with deterministic missing-file ordering, safe ignoring of unrelated files, and explicit empty-input rejection before filesystem access.

Phase 20.0 adds explicit schema-version metadata to manual export bundles.
The current schema version is `answer_artifact_export.v1`, written into export manifest, issue, summary, result, and inspection metadata and validated by the read-only inspector.
`export_issues.json` is a versioned `{ schema_version, issues }` object.
The inspector reports typed issues for missing, unsupported, and mismatched schema versions; it also accepts legacy raw issue arrays as a missing-schema-version compatibility case.
Malformed object-shaped issue files are still typed read failures.
The top-level inspection `schema_version` is only present for fully supported, fully consistent bundles; invalid bundles keep it absent.
This remains artifact-audit compatibility metadata only and is still not a product answer or share workflow.
There is no import workflow, no automatic export, and no repair/fix action yet.

Phase 20.1 hardens that boundary with strict aggregate schema-version handling, compatibility parsing for legacy raw issue arrays, malformed-object safety, and the rule that invalid bundles keep aggregate `schema_version` absent instead of echoing unsupported values.
Phase 20.2 is this docs-sync pass.

Phase 21.0 adds deterministic export integrity metadata to the manual export bundle.
It remains artifact-audit metadata only, is read-only, and validates exported bundle files without enabling import, migration, repair, share, or upload workflows.
The integrity bundle uses `answer_artifact_export.v1`, SHA-256 digests, and relative paths only.
Phase 21.1 hardens integrity path validation by rejecting dot-segment and traversal-style paths while keeping the inspector read-only.
Phase 21.2 is this docs-sync pass.
This is still not a product answer workflow.

Phase 22.0 adds a read-only inspection summary rollup for export bundle inspection results.
It is derived from parsed bundle metadata and typed inspection issues only, and it keeps the bundle inspector path-free and read-only.
The rollup reports deterministic consistency, schema support, integrity verification, and issue counts by kind.
Phase 22.0 is an audit/diagnostic pass, not a repair/import/share/download workflow.
Phase 22.1 was reviewed as a no-op boundary review.
Phase 22.2 is this docs-sync/finalization pass.

Phase 23.0 adds a read-only inspection report preview as part of the existing export bundle inspector DTO.
It is derived from the existing inspection summary and typed inspection issues only, is deterministic, path-free, non-persistent, and does not write report files or expose raw internal filesystem paths.
Phase 23.0 is implemented as an audit/diagnostic preview, not a product answer workflow.
Phase 23.1 is a tests-only boundary hardening pass.

Phase 24.0 adds a read-only inspection issue detail view as part of the existing export bundle inspector DTO.
It groups existing typed inspection issues by issue kind into deterministic, path-free compact detail lines and does not read additional files, write report artifacts, or mutate inspected bundles.
Each group count matches its line count, and the total grouped line count mirrors the typed inspection issue count.
Phase 24.0 is an audit/diagnostic UI aid, not a product answer workflow.
Phase 24.1 is tests-only boundary hardening.
Phase 24.2 is this docs-sync/finalization pass.

Phase 25.0 adds a read-only file status view to the existing export bundle inspector DTO and UI.
It reports deterministic rows for `export_manifest.json`, `export_issues.json`, `summary.json`, and `export_integrity.json`, with path-free present, parsed, malformed, schema, integrity, issue-count, and compact status metadata derived from existing inspection state only.
Phase 25.0 does not change validation semantics, read extra files beyond the inspector’s existing bundle reads, write report artifacts, mutate inspected bundles, or add import/migration/repair/share/download/generation/editing workflows.
Phase 25.1 is a no-op boundary review with verification only.

Phase 26.0 adds a compact read-only `inspection_status` field to the existing export bundle inspector DTO and UI.
It surfaces a deterministic code/label/severity/reason summary derived from the existing inspection summary, typed issues, file statuses, and schema/integrity state only.
Phase 26.0 does not change validation semantics, read extra files beyond the inspector’s existing bundle reads, write report artifacts, mutate inspected bundles, or add import/migration/repair/share/download/generation/editing workflows.
Phase 26.1 was a no-op boundary review with verification only.
Phase 26.2 is this docs-sync/finalization pass.

Phase 27.0 consolidates the existing export bundle inspector UI into a clearer read-only diagnostic layout.
It reorders the existing inspection_status, inspection_summary, file_statuses, issue_groups, report_preview, and typed issue sections without changing backend inspection semantics.
It does not add import/migration/repair/share/download/export-writing behavior or any new controls.
Phase 27.1 is a no-op UX boundary review with verification only.

Phase 28.0 is a frontend-only empty-state and label consistency pass for the existing export bundle inspector.
It keeps the inspector read-only, keeps the existing diagnostic order, and only clarifies empty-state copy and section labels for existing DTO fields.
It does not change backend inspection, export, validation, schema, integrity, or status semantics, and it does not add import/repair/share/download/report-writing behavior.

Phase 29.0 is a no-op copy/accessibility review.
It confirmed the inspector remained read-only and path-free, with no backend, export, validation, or DTO semantics changes.
Phase 30.0 is this milestone-closure review for the export bundle inspector stack.
It closes out the read-only inspector phases as implemented and keeps future import/repair/share/download/export-writing work out of scope.
The export bundle inspector milestone from Phase 19 through Phase 30 is closed.
No further inspector workflow expansion is planned in the current milestone.

## Phase 31 — Post-Inspector Answer/Evidence Contract Hardening

Planning boundary only.

Focus areas:

- existing answer artifacts
- grounded answers
- evidence status handling
- pipeline contract preservation

Do not implement:

- import
- repair
- migration
- share/upload/download
- report-writing/export-writing
- generation
- ranking
- editing
- evidence rewriting

## Phase 32 — Post-Inspector Answer/Evidence Contract Boundary Inventory

Planning boundary only.

Phase 32.0 is this inventory/boundary pass.
It documents the existing answer draft, grounded answer, final answer, evidence status, typed error, and pipeline smoke guarantees without adding new behavior.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 33 — Answer/Evidence Status Preservation Review

Planning boundary only.

Phase 33.0 was a no-op review.
Existing tests already cover supported, `needs_evidence`, and unsupported status preservation across answer drafts, grounded answers, final answers, and the pipeline smoke contract.
No code, DTO, or validation changes were needed.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 34 — Answer/Evidence Typed Error Boundary Review

Planning boundary only.

Phase 34.0 is a no-op review.
Existing tests already cover invalid ID rejection, empty ID rejection, traversal-like ID rejection, missing-file typed errors, malformed-file typed errors, failure-side-effect checks, and path-free metadata/error output for answer drafts, grounded answers, final answers, and the pipeline smoke contract.
No code, DTO, or validation changes were needed.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 35 — Answer/Evidence Deterministic Metadata Review

Planning boundary only.

Phase 35.0 is a no-op review.
Existing tests already cover deterministic IDs, deterministic ordering, deterministic counts, deterministic summaries/overviews, path-free metadata, path-free typed errors, and read/list/overview no-side-effect behavior across answer drafts, grounded answers, final answers, and the pipeline smoke contract.
No code, DTO, or validation changes were needed.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 36 — Answer/Evidence Contract Closure Review

Planning boundary only.

Phase 36.0 is a bundled closure review for the remaining answer/evidence contract boundaries.
Existing tests already cover no-generation, no claim inference, no evidence rewriting, no semantic ranking, no editing workflow, pipeline contract preservation, and unchanged DTO/validation semantics.
No code, DTO, or validation changes were needed.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 37 — Next Implementation Milestone Selection

Planning boundary only.

Phase 37.0 is next-milestone selection only.
Recommended next implementation milestone: retrieval / evidence pack hardening.
Rationale: it is the next upstream boundary feeding answer drafts, grounded answers, and final answers, so strengthening retrieval inputs and evidence-pack metadata gives the most useful leverage without changing the inspector or answer contracts.
No production behavior changed in this pass.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 38 — Retrieval / Evidence Pack Contract Baseline

Planning boundary only.

Phase 38.0 is a contract baseline for retrieval and evidence packs.
It records the current mechanical, deterministic, path-free retrieval/evidence-pack boundaries without adding new behavior.
No generation, LLM calls, claim inference, evidence rewriting, ranking, editing, import/repair/share/download/report-writing/export-writing behavior was added.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 39 — Retrieval / Evidence Pack Fixture Contract Review

Planning boundary only.

Phase 39.0 is a no-op review.
Existing tests already cover deterministic retrieval results, stable source/chunk metadata, explicit evidence-pack input boundaries, and mechanical answer-draft construction from evidence packs.
No code, DTO, or validation changes were needed.

## Phase 40 — Retrieval / Evidence Pack Closure and Next Slice

Planning boundary only.

Phase 40.0 closes the retrieval/evidence-pack hardening review block.
Phases 38-39 did not change production behavior, and the existing coverage was sufficient for the retrieval/evidence-pack contract baseline and fixture contract review.
Recommended next implementation slice: source/chunk metadata guarantee tightening.
Rationale: it is upstream of retrieval, improves evidence-pack reliability, can be kept small and test-focused, and does not require generation, ranking, import/share/download, or UI expansion.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 41 — Source/Chunk Metadata Guarantee Baseline

Planning boundary only.

Phase 41.0 is the source/chunk metadata guarantee baseline.
It is a focused backend test/docs pass for deterministic source and chunk identifiers, stable ordering, path-free metadata, and retrieval previews derived only from chunk text.
No production behavior changed in this pass.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 42 — Developer-Facing Diagnostics and Onboarding Cleanup

Planning boundary only.

Phase 42.0 closes the source/chunk metadata review slice.
Phase 41 did not change production behavior, and the existing coverage was sufficient for the source/chunk metadata guarantee baseline.
Recommended next implementation slice: developer-facing diagnostics and onboarding cleanup.
Rationale: it is a low-risk, docs/test-command focused follow-up that can improve README/docs navigation, verification command consolidation, and the architecture map without changing product workflows.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 43 — Developer-Facing Diagnostics and Onboarding Baseline

Planning boundary only.

Phase 43.0 is a docs-only developer-facing diagnostics and onboarding baseline.
It adds concise navigation and verification pointers for the current architecture, current closed contract areas, and standard checks without changing production behavior or adding new workflows.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 48 — Backend Command Inventory and Next Product Slice

Planning boundary only.

Phase 48.0 is a backend/frontend command inventory and next-slice selection note.
The backend already exposes retrieval, evidence-pack, answer-draft, grounded-answer, final-answer, artifact overview, source index, health, issues, export manifest, and export bundle inspection commands; the frontend already surfaces the answer-artifact and export-bundle diagnostics, but not retrieval search results or retrieval index metadata.
Recommended next product-facing slice: surface existing read-only retrieval search/index results in the UI.
Rationale: it adds visible value from already-tested backend behavior without changing retrieval semantics or answer/export contracts.
No production behavior changed in this pass.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 49 — Read-Only Metadata UI Surface

Planning boundary only.

Phase 49.0 adds a small read-only retrieval index metadata surface in the UI using the already-exposed `get_retrieval_index` command.
It shows compact deterministic metadata with loading, empty, and error states, and it does not change backend semantics.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 50 — Read-Only Retrieval Search UI Surface

Implementation boundary.

Phase 50.0 adds a small read-only retrieval search surface in the UI using the already-exposed `search_source` command.
It shows compact deterministic search results with loading, empty, and error states, and it does not change retrieval scoring or evidence semantics.
No backend command exposure was needed in this pass.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 51 — Retrieval UI Guardrails

Frontend-only hardening.

Phase 51.0 tightens the read-only retrieval search surface with query validation, capped visible results, and clearer empty/error states.
No backend retrieval behavior changed.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 52 — Retrieval UI Source Selection

Frontend-only refinement.

Phase 52.0 adds a compact read-only source selector for retrieval search using the loaded retrieval index data.
It defaults deterministically to the first available source ID, keeps query guardrails, and does not change backend retrieval behavior.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 53 — Answer Artifact Health UI Surface

Frontend-only diagnostic surface.

Phase 53.0 adds a compact read-only answer artifact health card using the already-exposed `get_answer_artifact_health` command.
It shows deterministic summary counts and keeps the UI path-free.
No backend answer-artifact semantics changed.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 54 — Answer Artifact Source Index UI Surface

Frontend-only diagnostic surface.

Phase 54.0 adds a compact read-only answer artifact source index card using the already-exposed `list_answer_artifact_sources` command.
It shows deterministic per-source counts and simple aggregates, keeps the UI path-free, and does not change backend answer-artifact semantics.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 55 — Answer Artifact Issues UI Surface

Frontend-only diagnostic surface.

Phase 55.0 adds a compact read-only answer artifact issues card using the already-exposed `list_answer_artifact_issues` command.
It shows deterministic issue summaries and keeps the UI path-free, and it does not change backend answer-artifact semantics.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 56 — Diagnostic UI Refresh Controls

Frontend-only refinement.

Phase 56.0 adds a small read-only refresh control for the existing diagnostic cards using already-exposed read-only commands.
It refreshes displayed diagnostics without changing backend semantics.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 57 — Final Answer List UI Surface

Frontend-only diagnostic surface.

Phase 57.0 adds a compact read-only final answer list card using the already-exposed `get_answer_artifact_overview` command and the existing selected answer artifact source flow.
It shows deterministic final-answer metadata and keeps the UI path-free.
No backend answer-artifact semantics changed.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 58 — Final Answer Detail Preview UI Surface

Frontend-only diagnostic surface.

Phase 58.0 extends the existing read-only final answer list card with a compact detail preview for the selected final answer, using the already-exposed `get_final_answer` command.
It keeps the UI path-free and does not change backend answer-artifact semantics.
It adds no new backend command exposure and no new answer-artifact workflow.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 59 — Evidence Pack UI Surface

Planning boundary only.

The backend has evidence-pack build/read logic, but no exposed read-only list surface yet, so Phase 59.0 defers the UI card until a narrow evidence-pack listing/read adapter exists.
The next backend slice is read-only evidence-pack listing exposure for an existing source; it should stay deterministic, path-free, and non-mutating.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 60 — Evidence Pack Listing Command

Backend read-only listing surface.

Phase 60.0 adds `list_evidence_packs` for an existing source and returns compact metadata only.
It is deterministic, path-free, non-mutating, and does not add generation, editing, import, repair, share/upload/download, or report-writing/export-writing workflows.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 61 — Evidence Pack UI Surface

Frontend-only diagnostic surface.

Phase 61.0 implements a compact read-only Evidence Packs card using the already-exposed `list_evidence_packs` command and the existing source-selection flow.
It keeps the UI deterministic and path-free, adds no backend exposure, and does not change backend Evidence Pack semantics.

Do not implement:

- generation
- claim inference
- evidence rewriting
- semantic ranking
- editing
- import
- repair
- migration
- share/upload/download
- report-writing/export-writing

## Phase 62 — Product Target / Scholar Chat UX Contract

Product and architecture alignment only.

Phase 62.0 defines the v1 product target as a local-first academic Scholar Chat workspace: mode/context selection, natural prompting, local corpus retrieval, evidence-pack assembly, source provenance, and later local llama.cpp/GGUF model runtime.
This pass changes docs only and does not add chat execution, model runtime, retrieval semantics, evidence generation, or answer synthesis.

Default answer policy:

- selected course or project context first
- registered local sources second
- existing local artifacts third
- external scholarly adapters later, after results become Source Registry entries
- general model knowledge only when explicitly allowed or clearly marked as not locally grounded

Recommended next implementation slice: Scholar Chat Request Contract or Chat Shell UI.
The next slice should start the product workflow boundary rather than add another diagnostic card.

Do not implement:

- local model runtime
- LLM calls
- chat execution
- web search
- scholarly database integration
- source import
- evidence generation
- answer generation
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 63 — Scholar Chat Request Contract

Backend contract boundary only.

Phase 63.0 adds a serializable Scholar Chat request/response contract and the read-only `preview_scholar_chat_request` command.
The preview validates and normalizes prompts and selected source IDs, returns a deterministic grounding plan, and marks the response as `preview_only`.
It does not run retrieval, build Evidence Packs, call an LLM, require a local model, generate answers, write files, or create directories.

Do not implement:

- chat UI
- local model runtime
- LLM calls
- retrieval execution
- Evidence Pack generation
- answer generation
- web search
- scholarly database integration
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 64 — Scholar Chat Shell UI

Frontend-only product shell.

Phase 64.0 adds a compact Scholar Chat card that uses the existing `preview_scholar_chat_request` command.
The shell lets the user choose mode and grounding policy, enter a prompt, pass existing selected source context when available, and preview the deterministic grounding plan and warnings.
It does not run retrieval, build Evidence Packs, call an LLM, require a local model, generate answers, write files, or create directories.

Do not implement:

- local model runtime
- LLM calls
- retrieval execution
- Evidence Pack generation
- answer generation
- web search
- scholarly database integration
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 65 — Scholar Chat Retrieval Preview

Product-facing preview slice.

Phase 65.0 adds a compact Scholar Chat retrieval preview card using the existing read-only `preview_scholar_chat_retrieval` command.
It searches selected source context in preview-only mode, shows compact candidate metadata and warnings, and does not build retrieval indexes, build Evidence Packs, call an LLM, or generate answers.

Do not implement:

- local model runtime
- LLM calls
- answer generation
- Evidence Pack generation
- retrieval index building
- web search
- scholarly database integration
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 66 — Scholar Chat Source Context Selector

Frontend-facing context selector slice.

Phase 66.0 adds a compact Scholar Chat source context selector using the existing `list_sources` command.
It lets the user choose one or more source IDs for Scholar Chat previews, keeps the UI read-only and deterministic, and falls back to the existing diagnostic source context only when no Scholar Chat selection has been made.
This phase does not add source import, retrieval execution changes, Evidence Pack building, LLM calls, or answer generation.

Do not implement:

- local model runtime
- LLM calls
- retrieval execution changes
- Evidence Pack generation/building
- answer generation
- web search
- scholarly database integration
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 67 — Scholar Chat Evidence Plan Preview

Read-only preview bridge toward Evidence Pack assembly.

Phase 67.0 adds a compact Scholar Chat evidence plan preview using the existing preview-only `preview_scholar_chat_evidence_plan` command.
It reuses the Scholar Chat prompt, mode, grounding policy, and source context, describes which retrieval candidates would be eligible for later Evidence Pack assembly, and remains preview-only and read-only.
It does not build Evidence Packs, call an LLM, generate answers, or write files.

Do not implement:

- local model runtime
- LLM calls
- answer generation

## Phase 68 — Scholar Chat Prompt Pack Preview

Preview-bound Scholar Chat formatting slice.

Phase 68.0 adds the preview-only `preview_scholar_chat_prompt_pack` command and a compact Scholar Chat prompt pack preview card.
It reuses the Scholar Chat prompt, mode, grounding policy, and selected source context, shows the planned prompt sections and compact context items, and remains preview-only and read-only.
It does not call an LLM, run retrieval, build Evidence Packs, generate answers, or write files.

Do not implement:

- local model runtime
- LLM calls
- retrieval execution changes
- Evidence Pack building
- answer generation
- source import
- editing workflows
- export/report/share workflows
- web search or scholarly database integration
- routing/charts/frontend test framework

## Phase 69 — Local Model Runtime Config / Health Preview

Preview-bound local runtime readiness slice.

Phase 69.0 adds the preview-only `preview_local_model_runtime_health` command and a compact local model runtime preview card.
It accepts a read-only runtime config, checks model/executable file readiness when paths are provided, and stays path-free, deterministic, and non-persistent.
It does not start a process, call an LLM, stream tokens, generate answers, persist config, or download/install models.

Do not implement:

- local model inference/runtime execution
- LLM calls
- streaming tokens
- answer generation
- config persistence
- model download/install behavior
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 70 — Local Runtime Invocation Plan Preview

Preview-bound local runtime invocation slice.

Phase 70.0 adds the preview-only `preview_local_runtime_invocation_plan` command and a compact runtime invocation plan card.
It uses the existing local runtime config and optional Scholar Chat prompt/text estimates to describe a future invocation without executing a process, calling an LLM, streaming tokens, generating answers, persisting config, or downloading/installing models.

Do not implement:

- local model inference/runtime execution
- LLM calls
- streaming tokens
- answer generation
- config persistence
- model download/install behavior
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 71 — Local Runtime Version Probe

Guarded process-probe slice.

Phase 71.0 adds the preview-only `probe_local_runtime_version` command and a compact runtime version probe card.
It performs an explicit allow-execution version probe only, stays read-only and path-free, and does not load a model, run inference, stream tokens, generate answers, persist config, or download/install models.

Do not implement:

- local model inference/runtime execution
- LLM calls
- streaming tokens
- answer generation
- config persistence
- model download/install behavior
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 72 — Local Runtime Smoke Inference Probe

Guarded smoke-inference slice.

Phase 72.0 adds the preview-only `smoke_test_local_runtime_inference` command and a compact local runtime smoke test card. It validates configured runtime paths, clamps timeout and output-token limits, and can launch a tiny direct executable probe only when execution is allowed. It does not call an LLM, run retrieval, build Evidence Packs, generate answers, or persist results.

Do not implement:

- LLM calls
- retrieval execution
- Evidence Pack building
- answer generation
- config persistence
- model download/install behavior
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 73 — Runtime Smoke Output Boundary

Phase 73.0 adds diagnostic-only boundary metadata and UI copy for local runtime smoke output. Smoke results now carry explicit runtime-diagnostic classification flags so the preview cannot be confused with Scholar Chat answers. This remains preview-only and does not generate Scholar Chat answers, grounded answers, Evidence Packs, or final answers.

Do not implement:

- LLM calls
- retrieval execution
- Evidence Pack building
- answer generation
- config persistence
- model download/install behavior
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 74 — Scholar Chat Answer Readiness Gate

Phase 74.0 adds a read-only Scholar Chat answer-readiness gate using the existing `preview_scholar_chat_answer_readiness` command and compact UI copy.
It previews whether the current Scholar Chat request, source context, and local runtime configuration could proceed toward a future local draft inference, but it remains preview-only and does not execute the runtime, call an LLM, generate answers, build Evidence Packs, or persist anything.

Do not implement:

- LLM calls
- runtime execution
- retrieval execution changes
- Evidence Pack building
- answer generation
- config persistence
- model download/install behavior
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 75 — Scholar Chat Draft Inference Preview

Phase 75.0 adds a guarded Scholar Chat draft-inference preview using the existing `preview_scholar_chat_draft_inference` command and the Scholar Chat shell UI.
It may run the local model as a preview, but it remains preview-only and does not create Scholar Chat answers, grounded answers, Evidence Packs, final answers, or persisted artifacts.

Do not implement:

- real Scholar Chat answer generation
- grounded answer generation
- Evidence Pack building
- retrieval execution changes
- LLM calls outside preview
- config persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 75.1 — Draft Inference Boundary Review

Phase 75.1 is a boundary review for the Scholar Chat draft-inference preview.
It hardens the preview-only, draft-only, non-persistent boundary without broadening runtime execution or changing product scope.

Do not implement:

- Scholar Chat answer generation
- grounded answer generation
- Evidence Pack building
- chat history persistence
- export/report/share workflows
- source import
- web search or scholarly database integration
- routing/charts/frontend test framework

## Phase 76 — Draft Grounding Inspection Preview

Phase 76.0 adds a read-only draft grounding inspection preview using `preview_scholar_chat_draft_grounding_inspection` and a compact Scholar Chat UI card.
It inspects draft text against local evidence candidates only and remains diagnostic-only: no grounded answer, no final answer, no Evidence Pack, no runtime execution, no LLM call, and no persistence.

Do not implement:

- Scholar Chat answer generation
- grounded answer generation
- Evidence Pack building
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 76.1 — Draft Grounding Inspection Boundary Review

Phase 76.1 is a review-only hardening pass for the draft grounding inspection preview.
It keeps support classification conservative and diagnostic-only, and does not broaden product behavior.

Do not implement:

- grounded answer generation
- final answer generation
- Evidence Pack building
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 76.2 — Draft Grounding Inspection Term Filter

Phase 76.2 tightens the lexical term filter used only by draft grounding inspection so short noise terms and common stopwords do not overstate support.
This is inspection-only lexical hardening and does not add truth verification, grounded-answer generation, final-answer generation, or any other broader product behavior.

Do not implement:

- grounded answer generation
- final answer generation
- Evidence Pack building
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 77 — Draft Output to Grounding Inspection Convenience

Phase 77.0 adds a small UI convenience action that copies the draft inference stdout preview into the draft grounding inspection textarea.
It remains frontend-only, keeps the stdout preview diagnostic, and does not run grounding inspection automatically or broaden backend behavior.

Do not implement:

- grounded answer generation
- final answer generation
- Evidence Pack building
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 78 — Grounded Draft Readiness Preview

Phase 78.0 adds a preview-only grounded-draft readiness diagnostic that composes the existing draft grounding inspection preview and summarizes whether the current draft looks ready for a future grounded-answer path.
It remains diagnostic-only and does not create grounded answers, final answers, Evidence Packs, runtime execution, or persistence.

Do not implement:

- grounded answer generation
- final answer generation
- Evidence Pack building
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 79 — Grounded Answer Build Plan Preview

Phase 79.0 adds a preview-only grounded-answer build-plan diagnostic that composes the grounded-draft readiness preview and explains what would still be needed before a future GroundedAnswer could be written.
It remains plan-only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, or runtime execution.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 80 — Grounded Answer Candidate Preview

Phase 80.0 adds a preview-only grounded-answer candidate diagnostic that composes the grounded-answer build-plan preview and surfaces deterministic candidate items for a future GroundedAnswer path.
It remains candidate-only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, runtime execution, or LLM call.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 81 — Grounded Answer Write Eligibility Preview

Phase 81.0 adds a preview-only grounded-answer write-eligibility diagnostic that composes the grounded-answer candidate preview and reports whether a future GroundedAnswer write would be eligible later.
It remains preview-only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, registry change, audit write, runtime execution, or LLM call.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- artifact writes
- registry status changes
- audit writes
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 82 — Grounded Answer Build Intent Preview

Phase 82.0 adds a preview-only grounded-answer build-intent diagnostic that composes the grounded-answer write-eligibility preview and explains what would still be required before a future GroundedAnswer implementation could be accepted.
It remains preview-only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, registry change, audit write, runtime execution, or LLM call.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- artifact writes
- registry status changes
- audit writes
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 83 — Grounded Answer Build Request Preview

Phase 83.0 adds a preview-only grounded-answer build-request diagnostic that composes the grounded-answer build-intent preview and explains what would still be required before a future GroundedAnswer request could proceed.
It remains request-preview only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, registry change, audit write, runtime execution, or LLM call.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- artifact writes
- registry status changes
- audit writes
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 84 - Grounded Answer Build Preflight Preview

Phase 84.0 adds a preview-only grounded-answer build preflight that composes the grounded-answer build-request preview and checks whether an existing AnswerDraft is readable without creating one.
It remains preview-only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, registry change, audit write, runtime execution, or LLM call, and it does not create directories or files.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- artifact writes
- registry status changes
- audit writes
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 85 - Grounded Answer Execution Readiness Preview

Phase 85.0 adds a preview-only grounded-answer execution-readiness diagnostic that composes the grounded-answer build-preflight preview and checks whether a future execution step would be allowed later.
It remains preview-only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, registry change, audit write, runtime execution, or LLM call, and it does not create directories or files.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- artifact writes
- registry status changes
- audit writes
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 86 - Grounded Answer Execution Plan Preview

Phase 86.0 adds a preview-only grounded-answer execution-plan diagnostic that composes the grounded-answer execution-readiness preview and describes the logical next execution shape later.
It remains preview-only and does not create an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persisted artifact, registry change, audit write, runtime execution, or LLM call, and it does not create directories or files.

Do not implement:

- grounded answer generation
- AnswerDraft creation
- GroundedAnswer creation
- FinalAnswer creation
- Evidence Pack building
- artifact writes
- registry status changes
- audit writes
- runtime execution changes
- LLM calls outside preview
- chat history persistence
- source import
- editing workflows
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 87 - Llama Runtime Adapter Contract Preview

Phase 87.0 adds a preview-only llama.cpp adapter contract diagnostic for future GGUF model support. It validates adapter metadata and safe configuration hints only; it does not run llama.cpp, load a model, start a process, call an LLM, or persist settings. Gemma, GGUF, and related model-family/config values remain future adapter metadata, not bundled models.

Do not implement:

- llama.cpp execution
- model loading
- process spawning
- binary probing
- LLM calls
- settings persistence
- artifact writes
- registry status changes
- audit writes

## Phase 88 - Llama Runtime Validation Preview

Phase 88.0 adds a preview-only llama.cpp runtime validation layer on top of the adapter contract preview. It checks executable/model path presence and lightweight metadata for future GGUF runtime use, but it does not execute llama.cpp, probe binaries, load models, call an LLM, persist settings, or write artifacts. GGUF, Gemma, and related adapter values remain future runtime compatibility context, not bundled model support.

## Phase 89 - Llama Runtime Probe Readiness Preview

Phase 89.0 adds a preview-only llama.cpp runtime probe-readiness layer on top of runtime validation. It only decides whether a future binary probe may be attempted and does not probe binaries, execute runtime code, load models, call an LLM, persist settings, or write artifacts.

Do not implement:

- llama.cpp execution
- binary probing
- model loading
- LLM calls
- settings persistence
- artifact writes
- registry status changes
- audit writes
- source import
- export/report/share workflows
- Gemma download or bundling
- model import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 90 - Llama Runtime Version Probe Execution

Phase 90.0 adds a preview-only, consent-gated llama.cpp version probe on top of probe readiness. It can run the configured binary with `--version` only, and it does not pass a model path, load a model, run inference, call an LLM, persist settings, or write artifacts.

Do not implement:

- llama.cpp execution beyond a version-only probe
- model loading
- inference
- model path arguments
- persistence
- artifact writes
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 91 - Llama Runtime Capability Preview

Phase 91.0 adds a preview-only llama.cpp capability summary on top of the version-probe result. It is diagnostic only, not model loading, not model compatibility validation, and not inference.

Do not implement:

- llama.cpp execution beyond the wrapped version probe
- model loading
- model compatibility claims
- inference
- LLM calls
- persistence
- artifact writes
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 92 - Llama Runtime Smoke Readiness Preview

Phase 92.0 adds a preview-only smoke-readiness layer on top of the capability preview. It only prepares a future diagnostic smoke inference preview and does not run inference, load a model, or call an LLM.

Do not implement:

- llama.cpp execution beyond the wrapped capability preview
- model loading
- inference
- smoke inference execution
- model path arguments
- persistence
- artifact writes
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 93 - Llama Runtime Smoke Execution Plan Preview

Phase 93.0 adds a preview-only smoke-execution plan on top of the smoke-readiness preview. It only describes a future diagnostic smoke inference plan and does not run smoke inference or begin Scholar Chat answering.

Do not implement:

- llama.cpp execution beyond the wrapped smoke-readiness preview
- smoke inference execution
- model loading
- inference
- model path arguments
- persistence
- artifact writes
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 94 - Llama Runtime Smoke Diagnostic Execution

Phase 94.0 adds a consent-gated diagnostic smoke execution path that may run the configured llama.cpp executable for smoke diagnostics only. It is not Scholar Chat answering and it does not create an answer, GroundedAnswer, FinalAnswer, Evidence Pack, artifact write, registry change, audit write, or persisted state.

Do not implement:

- Scholar Chat answer generation
- grounded answer generation
- final answer generation
- Evidence Pack building
- persistence
- artifact writes
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 95 - Scholar Chat Runtime Diagnostic Bridge

Phase 95.1 adds a UI/docs sync for the preview-only Scholar Chat runtime diagnostic bridge. It remains preview-only, does not run runtime execution, does not generate answers, and does not create artifacts, persistence, registry changes, or audit writes.

Do not implement:

- Scholar Chat answer generation
- runtime execution
- artifact writes
- persistence
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 96 - Scholar Chat Runtime Diagnostic Result Preview

Phase 96.0 adds a backend-only Scholar Chat runtime diagnostic result preview that classifies an already-provided smoke diagnostic preview/result for future Scholar Chat use. It does not run smoke diagnostics, does not run inference, and does not create artifacts, persistence, registry changes, or audit writes.

Phase 96.1 adds a UI/docs sync for the runtime diagnostic result preview. It surfaces the already-loaded smoke diagnostic preview/result, does not run smoke diagnostics or inference, and does not create artifacts, persistence, registry changes, or audit writes.

## Phase 97 — Scholar Chat Runtime Answer Pipeline Gate Preview

Phase 97.0 adds a backend-only Scholar Chat runtime answer pipeline gate preview that combines the grounded-answer execution plan preview and the runtime diagnostic result preview. It is preview/gate only and does not run smoke diagnostics, does not run inference, does not spawn a process, does not call an LLM, does not generate an answer, and does not build Evidence Packs or write artifacts.

Do not implement:

- runtime execution
- smoke diagnostic execution
- smoke inference execution
- Scholar Chat answer generation
- artifact writes
- persistence
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 98 — Scientific Retrieval Architecture Docs

Phase 98.0 documents the scientific retrieval architecture for the next product direction.
It is docs-only and introduces the planned discipline graph, source registry, curriculum layer, mode behavior, and preview-first roadmap.
It does not add production code, web requests, scraping, model loading, runtime inference, answer generation, or artifact writes.

Phase 98.1 adds a backend-only Scientific Discipline Registry Preview.
It maps early example concepts, stays preview-only, and does not add web requests, scraping, connector implementation, local file indexing, BM25/vector index, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.

Phase 98.1.1 adds local workspace hygiene guardrails for models, build output, and local research data.
It is docs/.gitignore only and keeps local GGUF/GGML/Safetensors files, generated artifacts, and local corpus data outside Git until explicit app-managed storage is implemented.

Phase 98.2 adds a backend-only Scientific Source Registry Preview.
It maps source-family plans for early scientific concepts, stays preview-only, and does not add web requests, scraping, connector implementation, source import, local file indexing, BM25/vector index, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.

Phase 99.0 adds a backend-only Scientific Query Understanding Preview.
It composes the discipline and source registry previews, stays preview-only, and does not add web requests, scraping, connector implementation, source import, local file indexing, BM25/vector index, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.

Phase 99.1 adds a backend-only Scholar Chat Scientific Search Plan Preview.
It composes scientific query understanding and plans local-first search, metadata search, query expansion, source-family routing, ranking, deduplication, and evidence requirements. It stays preview-only and does not add retrieval execution, web requests, scraping, connector implementation, source import, local file indexing, BM25/vector index, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.

Phase 99.2 adds a backend-only Scientific Preview DTO Review / Refactor Guard.
It hardens DTO naming, serde/status consistency, deterministic output, boundary flags, path-free tests, and forbidden-call guards across the scientific preview stack. It adds no new features and does not expand behavior beyond preview-only guards.

Do not implement:

- web requests
- scraping
- connector implementation
- model loading
- runtime inference
- answer generation
- artifact writes
- registry status changes
- audit writes
- source import
- export/report/share workflows
- routing/charts/frontend test framework

## Phase 11 — Obsidian Export

Generate notes, backlinks and source cards.

## Phase 12 — Workspace Authority + Safe Artifact Writes

Add safe local artifact writes and workspace boundaries as a support layer.
