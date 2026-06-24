# AEGIS Scholar

**AEGIS = Agentic Evidence & Grounded Intelligence System**

AEGIS Scholar is a Windows-first, local-first academic Scholar Chat workspace for literature memory, discipline-grounded RAG, evidence packs, study workflows, scientific writing support, statistics/method support and long-term academic project memory.

This repository contains the **v0.7 Research Foundation Edition** plus an evolving Tauri v2, Solid/Vite and Rust foundation. The current app still exposes many read-only diagnostic surfaces, but the v1 product target is a ChatGPT/Claude/Codex-like academic prompt workspace grounded in local sources.

## Product stance

AEGIS Scholar is:

- a local academic Scholar Chat workspace
- a literature memory and source-grounded RAG system
- a prompt interface over registered local sources, retrieval, evidence packs and provenance
- a modular skill system for academic workflows
- a discipline system for psychology, statistics, mathematics and later broader MINT domains
- a Tauri v2 + Rust authority desktop app
- a Solid 1.x + Vite desktop UI
- a llama.cpp-controlled local model runtime later
- a corpus, skill, retrieval, evidence and audit system under `.aegis/`

AEGIS Scholar is not:

- a coding app
- a Pi replacement
- an OpenCode/Aider/Codex-style coding harness
- an ungrounded generic chatbot dashboard
- a Docker-first product
- a browser-only app
- a hidden autonomous execution layer
- a self-trained universal science LLM

AEGIS may support code only where code is part of scientific work: statistics scripts, reproducible analysis, teaching examples, notebooks or documentation.

## Core architecture rule

```text
selected context -> prompt -> source registry -> extraction -> chunks -> retrieval -> evidence pack -> grounded answer or skill output
```

The model must not answer from anonymous context blobs when external or uploaded material is involved. Scientific output must keep source identity, locators and evidence provenance.

Local memory is not model training. It is a curated local corpus and project memory used for retrieval, evidence packs and source grounding. The LLM is a reasoning and formulation engine, not the source of truth for scientific claims.

Default answer policy:

- selected course or project context first
- registered local sources second
- previously created local artifacts third
- external scholarly adapters later, after results become Source Registry entries
- general model knowledge only when explicitly allowed or clearly marked as not locally grounded
- if no local evidence is found, say so instead of bluffing

## Non-negotiables

1. Scientific claims based on sources must be source-grounded.
2. Corpus Authority owns source identity, metadata, versions, locators and ingestion state.
3. Skills are declarative contracts, not hidden prompt dumps.
4. Retrieval is a separate plane behind adapters.
5. Evidence Packs are created before synthesis for source-grounded answers.
6. Rust owns local authority later: filesystem, source registry, audit, process supervision and safe mutation.
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

## Developer onboarding

If you are trying to orient yourself quickly:

- architecture and boundaries: `docs/00-executive-summary.md`, `docs/02-target-architecture.md`, `docs/03-corpus-authority.md`
- retrieval and evidence packs: `docs/06-retrieval-architecture.md`, `docs/10-literature-rag-evidence.md`
- current roadmap and closed review blocks: `docs/12-roadmap.md`

Standard verification:

```text
npm run build
cargo test --manifest-path .\src-tauri\Cargo.toml final_answer -- --nocapture
cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture
cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture
cargo check --manifest-path .\src-tauri\Cargo.toml
git diff --check
```

You can run the same checks with `scripts/verify.ps1`.
Modes: `scripts/verify.ps1 -Fast` or `scripts/verify.ps1 -BackendOnly`.
GitHub Actions runs the same runner on push and pull_request.
You can launch it from the repo root or from another directory by using the script path.

Closed contract areas:

- export bundle inspector stack
- answer/evidence contract hardening
- retrieval/evidence-pack review block
- source/chunk metadata review slice

Current product direction:

- Scholar Chat request contract and chat shell work, using existing local-first retrieval and evidence boundaries

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

Near-term product work should move toward a Scholar Chat request contract and shell while preserving source identity, local-first retrieval, evidence packs and path-free diagnostics. It must not implement model runtime, embeddings, answer synthesis, browser automation, Pi/MCP integration or coding-agent behavior until those phases are explicitly selected.
