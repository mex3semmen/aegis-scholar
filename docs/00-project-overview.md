# 00 - Project Overview

AEGIS Scholar is a Windows-first, local-first academic knowledge workspace for source-grounded retrieval, evidence planning, provenance, and long-term project memory.

The repository already provides source registration and Source Registry management, extraction and chunking, retrieval and Evidence Pack contracts, PDF text-layer extraction, and preview-first Scholar Chat and scientific retrieval surfaces with read-only diagnostics and guarded execution slices.

It is not yet a finished chat product or a broad ingestion / answer-generation system. OCR, polished import UX, and production automation remain future work.

For the next product-direction step, see `docs/05-product-ux-reorientation.md`.

## Current implementation status

The repository is past the foundational phase and has working backend contracts for source handling, extraction, chunking, retrieval, evidence artifacts, answer-artifact contracts, and several preview surfaces.

## Current user-facing reality

Today the app is still not a finished chat product.

What users can realistically expect is:

- local source registration and Source Registry workflows
- extraction and chunking support for local corpus material
- retrieval, evidence, and answer-artifact backend contracts
- preview-first Scholar Chat and scientific retrieval surfaces
- first-run guidance for empty or not-yet-populated local corpora
- PDF text-layer extraction support when a text layer exists
- manual, guided workflow hints for the register -> extract -> chunk -> retrieval -> Evidence Pack path

## Current backend capabilities

Backend capabilities currently include:

- source registration, source metadata, and stable source identity
- source path validation and audit-oriented authority boundaries
- extraction report handling and locator preservation
- chunk generation and retrieval artifact contracts
- Evidence Pack storage and preview/planning support
- answer-artifact contracts and inspectors
- preview and gated execution surfaces for scientific metadata workflows
- guarded OpenAlex-only metadata execution as a narrow slice
- local runtime / LLM boundary scaffolding for later phases

## What is deliberately preview-only

Several areas are intentionally not full product workflows yet:

- Scholar Chat planning and execution-gate surfaces
- scientific retrieval planning
- metadata connector planning
- Evidence Pack assembly planning
- runtime diagnostics
- GUI integration readiness surfaces

These are designed to show intent, contract shape, and future flow without claiming automatic execution.

## What is not yet implemented

AEGIS Scholar does not yet provide:

- automatic source import as a finished product workflow
- OCR
- broad PDF ingestion beyond text-layer extraction
- semantic/vector retrieval as the main product workflow
- a finished answer-generation workflow
- automatic agent execution
- general-purpose coding-agent behavior
- broad model-runtime integration
- finished share/export workflows

## How to read the repository

Start with:

- `docs/01-architecture-overview.md`
- `docs/02-phase-index.md`
- `docs/03-github-wiki-outline.md`
- `docs/12-roadmap.md`

Then use the older architecture and contract docs for detail:

- `docs/02-target-architecture.md`
- `docs/03-corpus-authority.md`
- `docs/06-retrieval-architecture.md`
- `docs/10-literature-rag-evidence.md`
- `docs/13-scientific-retrieval-architecture.md`

## Recommended next engineering priorities

1. complete the remaining local source and extraction workflow polish
2. keep Evidence Pack creation and retrieval boundaries explicit
3. continue tightening Scholar Chat planner/gate behavior as preview-only until execution is ready
4. improve diagnostics and source-oriented orientation for new contributors
5. keep the docs synchronized with phase and capability changes
