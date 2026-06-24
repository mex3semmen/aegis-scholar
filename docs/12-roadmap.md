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

## Phase 11 — Obsidian Export

Generate notes, backlinks and source cards.

## Phase 12 — Workspace Authority + Safe Artifact Writes

Add safe local artifact writes and workspace boundaries as a support layer.
