# AEGIS Scholar

**AEGIS = Agentic Evidence & Grounded Intelligence System**

AEGIS Scholar is a Windows-first, local-first scientific knowledge OS for literature memory, discipline-grounded RAG, evidence packs, study workflows, scientific writing support, statistics/method support and long-term academic project memory.

This repository currently contains the **v0.7 Research Foundation Edition**: architecture, contracts, schemas, skill definitions and Codex prompts. It is intentionally documentation-first before implementation.

## Product stance

AEGIS Scholar is:

- a local scientific knowledge workspace
- a literature memory and source-grounded RAG system
- a modular skill system for academic workflows
- a discipline system for psychology, statistics, mathematics and later broader MINT domains
- a Tauri v2 + Rust authority desktop app later
- a Solid 1.x + Vite desktop UI later
- a llama.cpp-controlled local model runtime later
- a corpus, skill, retrieval, evidence and audit system under `.aegis/`

AEGIS Scholar is not:

- a coding app
- a Pi replacement
- an OpenCode/Aider/Codex-style coding harness
- a generic chatbot dashboard
- a Docker-first product
- a browser-only app
- a hidden autonomous execution layer
- a self-trained universal science LLM

AEGIS may support code only where code is part of scientific work: statistics scripts, reproducible analysis, teaching examples, notebooks or documentation.

## Core architecture rule

```text
source -> registry -> extraction -> chunks -> retrieval -> evidence pack -> grounded answer or skill output
```

The model must not answer from anonymous context blobs when external or uploaded material is involved. Scientific output must keep source identity, locators and evidence provenance.

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

## Initial skills

The first skill contracts live under `.aegis/skills/`:

- `write-scientific-paper`
- `study-course`
- `literature-review`
- `statistics-tutor`

## Next implementation prompt

Use:

```text
prompts/codex-phase-1-corpus-authority.md
```

Phase 1 builds Corpus Authority and Source Registry. It must not implement model runtime, embeddings, answer synthesis, browser automation, Pi/MCP integration or coding-agent behavior.
