# 10 — Literature Memory, RAG and Evidence Packs

## Purpose

Literature Memory makes scientific work source-grounded.

It handles:

- registered sources
- PDFs
- lecture slides
- notes
- papers
- extracted text
- chunks
- retrieval indexes
- evidence packs
- citations
- contradiction warnings

## Evidence Pack

An Evidence Pack is not just search results. It is a grounded context object used before synthesis.

Minimum fields:

- evidence pack ID
- query
- retrieval mode
- created time
- source references
- claim units
- warnings
- insufficiency notes

Phase 5.0 stores a local Evidence Pack JSON file at:

`./.aegis/corpus/sources/{source_id}/versions/{version_id}/evidence/{evidence_pack_id}.json`

The pack is built from deterministic lexical retrieval only.
It preserves source, version, chunk, locator, score, text hash, matched terms, and a short preview.
This phase does not use embeddings, vector search, or answer synthesis.

Phase 6.0 adds a mechanical Answer Draft scaffold over an Evidence Pack.
It converts each evidence item into one conservative supported claim and does not synthesize final prose.

Phase 7.0 adds a Grounded Answer contract over Answer Drafts.
It stores mechanical statement projections under the managed corpus tree and preserves unsupported / needs-evidence status without final prose generation.

Phase 8.0 adds a Final Answer contract over Grounded Answers.
It stores a deterministic contract projection under the managed corpus tree, preserves grounded statement identity and status, and does not generate final prose, rankings, or LLM output.

Phase 9.0 adds a narrow backend smoke test over the persisted answer contract chain.
It exercises answer draft, grounded answer, and final answer persistence and read-back without adding UI, synthesis, or ranking.

Current regression checks:

- `cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture`
- `cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture`
- `cargo check --manifest-path .\src-tauri\Cargo.toml`
- `git diff --check`

Current non-goals remain:

- no final prose synthesis
- no LLM generation
- no semantic ranking
- no UI answer display
- no claim invention
- no evidence rewriting

Phase 10.0 adds a read-only Final Answer inspector in the Solid frontend.
It loads an existing FinalAnswer by `source_id` plus `final_answer_id` through `get_final_answer`, shows the persisted contract read-only, and does not build, edit, or synthesize answers.
It keeps unsupported and `needs_evidence` statements visible and preserves statement order.
It shows claim, evidence, chunk, and locator references where present.

Phase 10.1 hardens that inspector boundary.
It trims IDs before invoking the backend, rejects empty inputs client-side, disables loading during fetch, keeps the last successful FinalAnswer visible until a new load succeeds, masks obvious filesystem-looking substrings in frontend error display, and renders locator summaries safely.

Phase 11.0 adds read-only discovery for existing Final Answers.
The inspector can list already persisted FinalAnswer contracts for a source and then select one for display.
Discovery only reads metadata from persisted contracts; it does not build, synthesize, or edit answers.
It does not hide unsupported or `needs_evidence` statements.

Phase 11.1 hardens that discovery boundary.
Listing remains read-only and returns metadata only, not filesystem paths.
Ordering is deterministic, metadata counts are derived from the persisted FinalAnswer statements, malformed files are treated as typed read failures, traversal-like `source_id` inputs stay away from arbitrary path access, and listing does not create missing directories as a side effect.

Phase 12.0 adds a read-only answer-artifact overview for a source.
It reports persisted answer-draft, grounded-answer, and final-answer counts, and reuses FinalAnswer metadata for inspection only.
It is read-only, exposes metadata only, does not surface filesystem paths, does not create directories, and does not create draft, grounded, or final artifacts.
It does not build, generate, edit, synthesize, or rank answers.
Phase 12.1 keeps the overview aligned with list_final_answers and hardens the boundary with deterministic ordering, multiple-artifact count coverage, and conservative typed malformed-final-answer handling.

Phase 13.0 adds a read-only source artifact index.
It lists only source_ids that already have persisted answer artifacts and shows draft, grounded-answer, and final-answer counts.
It is discovery only, does not create directories or artifacts, does not build or generate answers, and does not expose filesystem paths.
It orders results deterministically by `source_id`, returns an empty index when no relevant artifacts exist, ignores unrelated files safely, and keeps malformed final answers as conservative typed read failures.
Phase 13.1 hardens that boundary with deterministic ordering, empty-storage behavior, unrelated-file safety, and conservative malformed-final-answer handling.

Phase 14.0 adds a read-only answer artifact health summary for persisted diagnostics.
It reports persisted metadata only, including global and per-source counts for `source_count`, `draft_count`, `grounded_answer_count`, `final_answer_count`, `malformed_final_answer_count`, `unsupported_statement_count`, and `needs_evidence_statement_count`.
The output is deterministic by `source_id`, global counts equal the sum of the per-source counts where applicable, empty storage returns a zero-count summary, and malformed final answers are counted conservatively without exposing filesystem paths.
It does not create directories or artifacts, does not build or generate answers, and does not synthesize, rank, infer, rewrite, or edit anything.
Phase 15.0 adds a read-only answer artifact issue list for persisted diagnostics.
It reports persisted artifact issues only, with issue kinds `malformed_final_answer`, `unsupported_statement`, and `needs_evidence_statement`.
Empty storage returns an empty list, supported statements do not produce issues, ordering is deterministic by `source_id`, issue kind, `final_answer_id`, and `statement_index`, and malformed finals are reported conservatively without filesystem paths.
Unsupported and `needs_evidence` issues include path-free source/final/statement metadata.
It does not create directories or artifacts, and it does not build, generate, synthesize, rank, infer, rewrite, repair, or edit anything.
Phase 15.1 hardens that boundary with supported-statement exclusion, deterministic ordering, and path-free diagnostics.

Phase 16.0 adds a read-only answer artifact export manifest for preview-only inspection.
It reports persisted metadata only, does not write export or manifest files, does not add download buttons, and does not create directories or artifacts.
It reports global and per-source counts, derives `issue_count` from the issue-list diagnostics, keeps per-source `issue_count` aligned with source issues, and preserves deterministic ordering by `source_id` and `final_answer_id`.
Malformed final answers are excluded from valid final-answer metadata and reflected through issue counts; DTO/debug output remains path-free.
It does not build, generate, synthesize, rank, infer, rewrite, repair, or edit anything.
Phase 16.1 hardens that boundary with deterministic ordering, issue-count rollup, and tolerant handling of malformed final answers in the preview.
This remains preview-only and still does not provide actual export.

Phase 17.0 adds an explicit manual export step for persisted answer artifacts.
It is explicit user-triggered export only, uses persisted artifact data only, writes manifest JSON and issues JSON under the chosen export destination, and keeps returned file paths relative.
It exports valid persisted draft, grounded-answer, and final-answer artifacts, while malformed final answers remain visible through issue information and counts rather than being exported as valid final-answer files.
It does not build missing artifacts and does not generate, build, infer, rank, rewrite, repair, or edit answers.
The export destination must be explicit and non-empty, and repeated export to the same non-empty destination fails safely.
Phase 17.1 hardens that boundary with deterministic export order, empty-destination rejection before filesystem access, and path-safe export output.
Phase 18.0 adds a read-only `summary.json` audit file inside the manual export bundle.
The summary is derived only from the persisted manifest and issues data, stays deterministic and path-free, includes compact per-source and issue-kind counts, and is for audit/review only rather than import, share, or product answer flow.
Phase 18.1 hardens that summary boundary with deterministic hash-derived summary identity, manifest/issues alignment, and path-free export bundle output.

Phase 19.0 adds a read-only export bundle inspector for an existing manual export bundle.
It validates the persisted `export_manifest.json`, `export_issues.json`, and `summary.json` metadata only, compares the parsed summary against the derived manifest/issues summary, and reports typed inspection issues for missing or malformed bundle files.
It remains path-free, does not mutate the bundle, does not import, repair, rewrite, or regenerate anything, and does not add a share/download workflow.
Phase 19.1 hardens that inspector boundary with deterministic missing-file ordering and safe ignoring of unrelated files and nested noise inside the export bundle.
Empty bundle input is rejected with `ExportBundleInputMissing` before filesystem access, and valid exported bundles inspect as consistent.

Phase 20.0 adds explicit schema-version metadata to manual export bundles.
The current bundle schema version is `answer_artifact_export.v1`, written to `export_manifest.json`, `export_issues.json`, and `summary.json`, and returned through export/inspection metadata.
This is compatibility and audit metadata only.
`export_issues.json` is now a versioned object of the form `{ schema_version, issues }`.
The inspector validates missing, unsupported, and mismatched schema versions with typed deterministic inspection issues, but it does not migrate, repair, import, rewrite, or regenerate bundles.
Older `export_issues.json` arrays are still accepted by the inspector and reported as missing schema version instead of malformed JSON.
Malformed object-shaped `export_issues.json` remains a typed read failure.
The top-level inspection `schema_version` is only present for a fully supported, fully consistent bundle; invalid bundles keep it absent.
Valid current-version bundles inspect as consistent, and the bundle remains path-free and read-only during inspection.

Phase 21.0 adds `export_integrity.json` to the manual export bundle as deterministic audit metadata.
It is audit/integrity metadata only, versioned with `answer_artifact_export.v1`, uses SHA-256 digests, and lists only relative bundle paths with byte counts and digests.
The integrity file excludes itself from hashing, remains path-free, and does not expose raw internal filesystem paths.
The inspector validates integrity metadata read-only for presence, schema version, algorithm, relative-path safety, missing files, byte-count mismatches, digest mismatches, and duplicate entries.
It also validates invalid/traversal-style integrity paths when they appear in a bundle.
It does not import, migrate, repair, rewrite, or share bundles, and it does not add any answer-generation, synthesis, ranking, or editing behavior.

Phase 22.0 adds a read-only inspection summary rollup for export bundle inspection results.
The summary is derived from the inspector’s parsed bundle state and typed issues only, and it surfaces consistent/unsupported/integrity status plus deterministic issue counts by kind.
It also exposes checked-file and integrity-file counts when available.
The rollup is path-free and does not repair, migrate, import, share, upload, download, or generate answers.
Phase 22.1 was reviewed and required no code, UI, or docs changes.
Phase 22.2 is this docs-sync/finalization pass.

Phase 23.0 adds a read-only inspection report preview as part of the existing export bundle inspector DTO.
The preview is derived from the existing inspection summary and typed inspection issues only, is deterministic, path-free, and non-persistent, and does not write report files or expose raw internal filesystem paths.
It surfaces a title, schema version, high-level status, issue and warning counts, issue counts by kind, and compact status/issue sections for inspection results.
Phase 23.1 hardens that preview boundary with tests only and does not change production behavior.
It does not import, migrate, repair, rewrite, share, download, generate, or edit bundles or answers.

Phase 24.0 adds a read-only inspection issue detail view as part of the existing export bundle inspector DTO.
The issue detail view is derived only from existing typed inspection issues, is deterministic and path-free, and does not read additional files, write report artifacts, or mutate inspected bundles.
It groups existing issues into compact detail lines so review flows can inspect malformed final answers, schema-version problems, integrity problems, and other typed diagnostics without exposing raw internal filesystem paths.
Phase 24.1, if added later, should remain a tests-only boundary hardening pass.
It does not import, migrate, repair, rewrite, share, download, generate, or edit bundles or answers.

Manual verification checklist:

- `npm run build`
- `cargo test --manifest-path .\src-tauri\Cargo.toml final_answer -- --nocapture`
- `cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture`
- `cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture`
- `cargo check --manifest-path .\src-tauri\Cargo.toml`
- `git diff --check`
- open the app and confirm:
  - empty inputs are rejected
  - loading is disabled during fetch
  - artifact overview shows persisted draft, grounded answer, and final answer counts
  - source artifact index shows only sources with persisted artifacts
  - artifact health shows persisted diagnostic counts only
  - artifact health output stays path-free and deterministic by source
  - artifact issues show malformed finals and unsupported / `needs_evidence` statements only
  - export manifest shows preview-only metadata and issue counts only
  - export bundle inspector validates manifest, issues, and summary consistency without mutation
  - export bundle inspector reports missing, unsupported, or mismatched schema versions without mutation
  - export bundle inspection report preview renders below the inspection summary and stays read-only
  - `export_issues.json` shows the versioned `{ schema_version, issues }` shape
  - legacy raw `export_issues.json` arrays still inspect as missing schema version
  - empty export bundles report missing-file inspection issues
  - malformed export bundle files report typed inspection issues
  - export bundle inspection stays path-free and read-only
  - unrelated files and nested noise inside the export bundle are ignored safely
  - selecting a listed final answer fills the ID and loads it read-only
  - the list shows existing final answers when a source ID is entered
  - supported / `needs_evidence` / unsupported statements remain visible
  - statement order is preserved
  - locator summaries render
  - backend errors do not show raw filesystem paths

## Evidence unit

Each evidence unit must carry:

- source ID
- source version ID
- chunk ID
- title
- locator
- excerpt
- claim
- confidence

## RAG v1

Start simple:

- registered local source files
- SQLite source index
- lexical search baseline
- chunk retrieval
- metadata filters
- evidence pack creation before answer synthesis

No external vector server is required in v1.

## RAG later

Add behind adapters:

- embeddings
- vector index
- reranking
- formula/definition retrieval for mathematics

## Grounding rule

If the system cannot build a sufficient Evidence Pack, it must not present the answer as source-grounded.
