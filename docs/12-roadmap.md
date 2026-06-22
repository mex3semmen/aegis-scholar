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

Implement extraction for local PDFs, slides and notes while preserving page, slide, section and paragraph locators.

## Phase 3 — Chunking + Metadata Index

Implement chunk generation, discipline tags, method tags, locator indexes and ingestion reports.

## Phase 4 — Hybrid Retrieval MVP

Implement lexical search, metadata filters and retrieval profiles. Add embeddings only behind a retrieval adapter later.

## Phase 5 — Evidence Pack Builder

Build source-grounded evidence packs before answer synthesis.

## Phase 6 — Skill Runtime MVP

Load declarative skills and route tasks to retrieval profiles and output contracts.

## Phase 7 — Psychology/Statistics MVP

Add APA output, statistics explanations, method checks and course study workflows.

## Phase 8 — Local Model Runtime

Implement llama.cpp process lifecycle, health, logs, port/PID ownership and model profiles.

## Phase 9 — Composer UI

Show sources, evidence packs, retrieval mode, diagnostics and output controls.

## Phase 10 — Mathematics / MINT Expansion

Add mathematics, computer science, biology and related discipline profiles gradually.

## Phase 11 — Obsidian Export

Generate notes, backlinks and source cards.

## Phase 12 — Workspace Authority + Safe Artifact Writes

Add safe local artifact writes and workspace boundaries as a support layer.
