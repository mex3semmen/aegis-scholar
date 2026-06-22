# AGENTS.md

## Project

AEGIS Scholar is a Windows-first local AI knowledge OS for scientific work, literature memory, RAG, evidence packs, academic writing, course study, psychology/statistics support, mathematics/MINT expansion and disciplined long-term project memory.

## Product scope

AEGIS Scholar is not a coding app and does not replace Pi, Codex, Aider or OpenCode.

AEGIS may touch code only when code is part of scientific work, such as statistics scripts, reproducible analysis, teaching examples, notebooks or documentation.

Product core:

- local literature memory
- Corpus Authority
- Source Registry
- discipline-grounded retrieval
- Evidence Packs
- declarative academic skills
- psychology/statistics/APA MVP
- mathematics and broader MINT expansion later
- local model runtime later
- audit and provenance

## Core stack

AEGIS Scholar v1 targets:

- Tauri v2
- Rust backend authority
- Solid 1.x + Vite SPA frontend
- SQLite via `rusqlite`
- SQLite migrations via `rusqlite_migration`
- llama.cpp later through Rust Runtime Supervisor as a localhost HTTP server
- local files + SQLite for `.aegis/` project state
- no Pi runtime dependency
- no MCP core in v1
- no Docker main runtime

## Architecture authority

Rust owns sensitive local authority:

- source registration
- source metadata
- corpus path validation
- filesystem access
- permission decisions
- audit events
- runtime supervision later
- model server lifecycle later

Frontend must call Tauri commands only. No general-purpose shell access may be exposed to the UI.

## Phase discipline

If a prompt says Phase 1, implement only Phase 1.

For Phase 1, do not implement:

- Agent Runner
- Model Manager
- embeddings
- vector search
- answer synthesis
- Skill Router runtime
- MCP
- Pi
- UI redesign
- Docker runtime behavior
- llama.cpp integration
- general coding-agent behavior

## Skill discipline

Skills are declarative contracts. They define intent, allowed sources, retrieval profile, evidence policy, output contract and allowed tools.

Skills must not own database state or bypass Corpus Authority.

## Corpus Authority rules

Guarantees:

- empty corpus is valid
- every registered source gets a stable source ID
- source metadata is validated
- local source paths are canonicalized
- source content hash is recorded
- duplicate content hashes are detected
- source registry persists across restart
- source registration emits audit events
- source locators are preserved by extraction/chunking phases
- RAG must not use anonymous context blobs without source identity

## Error strategy

Use typed Rust errors in core modules.

Preferred strategy:

- `thiserror` for domain/core errors
- `anyhow` only at outer command/application boundaries if needed
- no stringly-typed errors in Corpus Authority core
- user-facing errors must be safe

## Verification

Before final response, run when applicable:

```text
git status --short --branch
git diff --name-status
git ls-files --others --exclude-standard
git stash list
npx tsc --noEmit
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

If a command cannot run because the repo does not yet contain package/project files, report that clearly.

## Git rules

- Use explicit staging only.
- Do not use `git add -A`.
- Do not touch stash.
- Do not rewrite history unless explicitly instructed.
- Do not delete unrelated files.
- Do not commit generated large artifacts unless explicitly required.
- Keep commits phase-scoped.
