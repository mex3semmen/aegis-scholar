# Codex Prompt — Phase 0.7 Research Foundation

Branch:
`feature/aegis-research-foundation`

Goal:
Validate and refine the research foundation docs and schemas before implementation.

AEGIS Scholar is not a coding app. The product core is local scientific literature memory, source-grounded retrieval, evidence packs and discipline-aware academic workflows.

## Tasks

Review and improve:

- `docs/00.7-research-foundation.md`
- `docs/02-target-architecture.md`
- `docs/03-corpus-authority.md`
- `docs/04-skill-system.md`
- `docs/06-retrieval-architecture.md`
- `docs/07-ingestion-locators.md`
- `docs/10-literature-rag-evidence.md`
- `docs/11-evaluation-harness.md`
- `schemas/*.schema.json`
- `.aegis/skills/*/skill.md`

## Constraints

Do not implement app code.
Do not add model runtime.
Do not add embeddings.
Do not add answer synthesis.
Do not add Pi/MCP integration.

## Verification

Report:

```text
git status --short --branch
git diff --name-status
git diff --check
```

## Commit

`docs: strengthen aegis research foundation contracts`
