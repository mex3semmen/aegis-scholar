# 03 - GitHub Wiki Outline

This is a proposed GitHub Wiki structure for external reviewers, collaborators, and future wiki editors.

Repo docs remain the source of truth; the wiki is a later mirror, not a separate spec.

For the operational copy guide and manual export checklist, see `docs/04-wiki-export-prep.md`.
For the product UX direction plan that should inform future wiki wording, see `docs/05-product-ux-reorientation.md`.

This page stays the conceptual outline for the wiki page set.

## Wiki Home

- Purpose: entry page with short project summary, navigation, and current status
- Mirror from: `docs/00-project-overview.md`
- Audience: user-facing

## Project Vision

- Purpose: describe the target product and what AEGIS Scholar is trying to become
- Mirror from: `docs/00-project-overview.md`, `docs/12-roadmap.md`
- Audience: user-facing

## Current Capabilities

- Purpose: describe what works today, what is preview-only, and what is still gated
- Mirror from: `docs/00-project-overview.md`, `docs/02-phase-index.md`
- Audience: user-facing

## Architecture

- Purpose: compact architecture map and boundary model
- Mirror from: `docs/01-architecture-overview.md`, `docs/02-target-architecture.md`
- Audience: developer-facing

## Agentic Workflow Model

- Purpose: explain planner, gate, preview, and execution distinctions for Scholar Chat
- Mirror from: `docs/01-architecture-overview.md`, `docs/13-scientific-retrieval-architecture.md`, `docs/12-roadmap.md`
- Audience: developer-facing

## Local Evidence Pipeline

- Purpose: explain source registration, extraction, chunking, retrieval, and Evidence Packs
- Mirror from: `docs/01-architecture-overview.md`, `docs/03-corpus-authority.md`, `docs/06-retrieval-architecture.md`, `docs/10-literature-rag-evidence.md`
- Audience: maintainer-facing

## Scholar Chat

- Purpose: summarize the chat-first workflow, planner preview, execution gate, and current limitations
- Mirror from: `docs/12-roadmap.md`, `docs/13-scientific-retrieval-architecture.md`
- Audience: user-facing

## Source Handling

- Purpose: explain Source Registry identity, metadata validation, hashing, and registry rules
- Mirror from: `docs/03-corpus-authority.md`, `docs/07-ingestion-locators.md`
- Audience: maintainer-facing

## PDF Support

- Purpose: document the supported PDF path and the OCR boundary
- Mirror from: `docs/00-project-overview.md`, `docs/13-scientific-retrieval-architecture.md`, `docs/07-ingestion-locators.md`
- Audience: user-facing

## Evidence Packs

- Purpose: explain the Evidence Pack contract and how it relates to answers
- Mirror from: `docs/10-literature-rag-evidence.md`, `docs/01-architecture-overview.md`
- Audience: developer-facing

## Local Runtime / LLM Boundary

- Purpose: document the separation between local authority, runtime supervision, and model execution
- Mirror from: `docs/01-architecture-overview.md`, `docs/02-target-architecture.md`
- Audience: developer-facing

## Developer Setup

- Purpose: quick orientation for contributors, local build expectations, and repo layout
- Mirror from: `README.md`, `docs/00-project-overview.md`
- Audience: maintainer-facing

## Roadmap

- Purpose: concise forward-looking phase summary and current work boundaries
- Mirror from: `docs/12-roadmap.md`, `docs/02-phase-index.md`
- Audience: maintainer-facing

## Glossary

- Purpose: define project terms such as Corpus Authority, Source Registry, Evidence Pack, planner, and gate
- Mirror from: `docs/00-project-overview.md`, `docs/01-architecture-overview.md`, `docs/03-corpus-authority.md`
- Audience: user-facing

## Recommendation

Repo docs remain the source of truth.

Use `docs/04-wiki-export-prep.md` as the operational copy guide when manually populating the wiki.

The GitHub Wiki can be manually populated from these docs now, or synced later if the documentation process is formalized.
