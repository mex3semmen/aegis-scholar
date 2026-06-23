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
The current schema version is `answer_artifact_export.v1`, written into export manifest, issue, and summary metadata and validated by the read-only inspector.
The inspector reports typed issues for missing, unsupported, and mismatched schema versions without mutating, migrating, importing, repairing, sharing, or regenerating the bundle.
This remains artifact-audit compatibility metadata only and is still not a product answer or share workflow.

## Phase 11 — Obsidian Export

Generate notes, backlinks and source cards.

## Phase 12 — Workspace Authority + Safe Artifact Writes

Add safe local artifact writes and workspace boundaries as a support layer.
