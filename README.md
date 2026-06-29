# AEGIS Scholar

**AEGIS = Agentic Evidence & Grounded Intelligence System**

AEGIS Scholar is a Windows-first, local-first academic Scholar Chat workspace for source-grounded retrieval, evidence planning, provenance, and long-term project memory.

Project knowledge base: `docs/00-project-overview.md`, `docs/01-architecture-overview.md`, `docs/02-phase-index.md`, and `docs/03-github-wiki-outline.md`.

## Current implementation status

AEGIS Scholar currently emphasizes preview-first Scholar Chat and Scientific Retrieval surfaces. The app has read-only diagnostic panels, preview contracts, and a narrow set of consent-gated execution slices, but it is not yet a finished product chat app.

Implemented or visible today:

- local corpus, Source Registry, retrieval, evidence, and answer-artifact contracts
- preview-first Scholar Chat and Scientific Retrieval planning surfaces
- OpenAlex-only metadata execution as a guarded execution slice
- Phase 116 backend-only Evidence Pack Assembly Plan Preview
- local PDF text-layer extraction MVP with page-level locators
- first-run source import readiness guidance for empty local corpora
- source workflow action hints for the manual register -> extract -> chunk -> retrieval -> Evidence Pack path
- Scholar Chat agentic workflow planning preview
- Scholar Chat execution-gate preview for later workflow steps
- runtime diagnostic previews and guarded execution candidates for developer use

## What AEGIS Scholar is

- a local academic Scholar Chat workspace
- a literature memory and source-grounded retrieval system
- a prompt interface over registered local sources, retrieval planning, evidence planning, and provenance
- a modular skill system for academic workflows
- a discipline system for psychology, statistics, mathematics, and later broader MINT domains
- a Tauri v2 + Rust authority desktop app
- a Solid 1.x + Vite desktop UI
- a local model runtime path for later phases, behind explicit boundary gates

## What it is not yet

AEGIS Scholar is not yet a finished product chat app.

It does not yet provide:

- a finished answer-generation workflow
- semantic/vector retrieval as the main product workflow
- scanned PDF OCR or broad PDF ingestion beyond text-layer extraction
- a finished source import wizard or drag-and-drop import flow
- finished Evidence Pack creation
- citation emission
- local LLM inference as a product workflow
- share/export workflows unless explicitly documented as implemented elsewhere

## Product stance

AEGIS Scholar is:

- local-first and source-grounded
- designed around Corpus Authority, source identity, locators, and provenance
- preview-first today, with read-only diagnostics and narrow guarded execution slices
- intended to support Scholar Chat, Scientific Paper Mode, Course Mode, and later MINT expansion

AEGIS Scholar is not:

- a coding app
- a Pi replacement
- an OpenCode/Aider/Codex-style coding harness
- an ungrounded generic chatbot dashboard
- a Docker-first product
- a browser-only app
- a hidden autonomous execution layer
- a self-trained universal science LLM

AEGIS may support code only where code is part of scientific work: statistics scripts, reproducible analysis, teaching examples, notebooks, or documentation.

## Core architecture rule

```text
selected context -> prompt -> source registry -> extraction -> chunks -> retrieval -> evidence pack -> grounded answer or skill output
```

The model must not answer from anonymous context blobs when external or uploaded material is involved. Scientific output must keep source identity, locators, and evidence provenance.

Local memory is not model training. It is a curated local corpus and project memory used for retrieval, evidence packs, and source grounding. The LLM is a reasoning and formulation engine, not the source of truth for scientific claims.

Default answer policy:

- selected course or project context first
- registered local sources second
- previously created local artifacts third
- external scholarly adapters later, after results become Source Registry entries
- general model knowledge only when explicitly allowed or clearly marked as not locally grounded
- if no local evidence is found, say so instead of bluffing

## Non-negotiables

1. Scientific claims based on sources must be source-grounded.
2. Corpus Authority owns source identity, metadata, versions, locators, and ingestion state.
3. Skills are declarative contracts, not hidden prompt dumps.
4. Retrieval is a separate plane behind adapters.
5. Evidence Packs are created before synthesis for source-grounded answers.
6. Rust owns local authority later: filesystem, source registry, audit, process supervision, and safe mutation.
7. Frontend never executes shell commands directly.
8. Workspace automation is a support layer, not the product core.
9. Pi remains an external developer tool.
10. MCP is not part of v1 core.

## Docs index

Start here:

1. `docs/00-executive-summary.md`
2. `docs/00.5-stack-decisions.md`
3. `docs/00.7-research-foundation.md`
4. `docs/01-product-blueprint.md`
5. `docs/02-target-architecture.md`
6. `docs/03-corpus-authority.md`
7. `docs/04-skill-system.md`
8. `docs/06-retrieval-architecture.md`
9. `docs/07-ingestion-locators.md`
10. `docs/10-literature-rag-evidence.md`
11. `docs/11-evaluation-harness.md`
12. `docs/12-roadmap.md`
13. `docs/00-project-overview.md`
14. `docs/01-architecture-overview.md`
15. `docs/02-phase-index.md`
16. `docs/03-github-wiki-outline.md`
17. `docs/04-wiki-export-prep.md`

## Developer onboarding

If you are trying to orient yourself quickly:

- architecture and boundaries: `docs/00-executive-summary.md`, `docs/02-target-architecture.md`, `docs/03-corpus-authority.md`
- retrieval and evidence packs: `docs/06-retrieval-architecture.md`, `docs/10-literature-rag-evidence.md`
- current roadmap and closed review blocks: `docs/12-roadmap.md`
- project overview and external orientation: `docs/00-project-overview.md`, `docs/01-architecture-overview.md`, `docs/02-phase-index.md`, `docs/03-github-wiki-outline.md`

## Developer verification

Use the script when you want the standard local checks:

```text
scripts/verify.ps1
scripts/verify.ps1 -Fast
scripts/verify.ps1 -BackendOnly
```

The script runs the same verification chain used by CI.

Standard verification:

```text
npm run build
cargo test --manifest-path .\src-tauri\Cargo.toml final_answer -- --nocapture
cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture
cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture
cargo check --manifest-path .\src-tauri\Cargo.toml
git diff --check
```

## Closed contract areas

- export bundle inspector stack
- answer/evidence contract hardening
- retrieval/evidence-pack review block
- source/chunk metadata review slice

## Current direction

Current product work emphasizes Scholar Chat request contracts, preview-first scientific retrieval surfaces, and narrow guarded execution slices, while keeping local-first retrieval, evidence planning, and path-free diagnostics intact.

## Initial skills

The first skill contracts live under `.aegis/skills/`:

- `write-scientific-paper`
- `study-course`
- `literature-review`
- `statistics-tutor`

## Current code scaffold

The current app foundation lives in:

```text
package.json
src/
src-tauri/
```

Implemented Rust commands:

```text
register_source
get_source
list_sources
update_source_metadata
remove_source
get_corpus_status
```

## Next implementation prompt

Use:

```text
prompts/codex-phase-1-corpus-authority.md
```

Near-term product work should move toward a Scholar Chat request contract and shell while preserving source identity, local-first retrieval, evidence packs, and path-free diagnostics. It must not implement model runtime, embeddings, answer synthesis, browser automation, Pi/MCP integration, or coding-agent behavior until those phases are explicitly selected.
