# Codex Prompt — Phase 1 Corpus Authority + Source Registry

Branch:
`feature/aegis-corpus-authority`

Goal:
Implement the local Corpus Authority foundation for AEGIS Scholar.

AEGIS Scholar is not a coding app. The product core is local scientific literature memory, source-grounded retrieval, evidence packs and discipline-aware academic workflows.

## Must not implement

- llama.cpp integration
- embeddings
- vector search
- answer synthesis
- Skill Router runtime
- Agent Runner
- MCP
- Pi
- general coding-agent behavior
- UI redesign

## Required modules

If scaffold exists, add:

```text
src-tauri/src/corpus_authority.rs
src-tauri/src/source_registry.rs
src-tauri/src/source_metadata.rs
src-tauri/src/corpus_paths.rs
src-tauri/src/audit.rs
src-tauri/src/errors.rs
```

If scaffold does not exist, create a minimal Tauri v2 + Rust scaffold needed to support these modules. Keep it minimal.

## Required contracts

Implement according to:

- `schemas/source.schema.json`
- `schemas/audit-event.schema.json`
- `docs/03-corpus-authority.md`
- `docs/07-ingestion-locators.md`

## Required commands

Expose Tauri commands if the architecture uses commands:

```text
register_source(path, metadata)
get_source(source_id)
list_sources(filter)
update_source_metadata(source_id, metadata_patch)
remove_source(source_id)
get_corpus_status()
```

## Required behavior

- empty corpus is valid
- source registration creates stable source ID and version ID
- local path is canonicalized
- content hash is recorded
- required metadata is validated
- duplicate content hashes are detected
- registry persists across restart
- corpus status reports source count and ingestion states
- mutating source operations emit audit events
- user-facing errors are safe

## Tests

Add Rust tests for:

- empty corpus valid
- register source with valid metadata
- missing title denied
- missing source type denied
- duplicate content hash detected
- invalid source type denied
- list sources returns registered source
- update metadata preserves source ID and hash
- remove source marks removed or deletes according to policy
- audit event emitted for registration

## Verification

Run and report:

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

If a command cannot run because the repo does not yet contain the required package/project files, report that clearly.

## Commit

`feat: add aegis corpus authority`
