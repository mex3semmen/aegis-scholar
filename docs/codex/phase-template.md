# Codex Phase Prompt Template

Use this template to describe only the delta for a new repo-local phase.

## Template

```text
Goal mode: enable
Plan mode: enable

Repo:
<absolute repo path>

Branch:
<branch name>

Version:
V1 | V2 | V3

Phase:
<phase number and title>

Goal:
<one-paragraph outcome>

Context:
<short project and current-state context>

Scope:
- <requested implementation slice>
- <requested implementation slice>

Non-goals:
- <explicitly out of scope item>
- <explicitly out of scope item>

Acceptance criteria:
- <verifiable outcome>
- <verifiable outcome>

Verification:
- git status --short --branch
- git diff --name-status
- git ls-files --others --exclude-standard
- git stash list
- npx tsc --noEmit
- npm run build
- cargo check --manifest-path .\src-tauri\Cargo.toml
- git diff --check
- <phase-specific tests or scripts>

Scope check:
- <exact static search or manual review step>
- <exact static search or manual review step>

Final report:
- Files changed
- Implemented
- Intentionally not implemented
- Verification
- Scope check
- Risks / follow-up
```

## Usage notes

- Keep prompts phase-scoped and additive.
- Describe product deltas, not the whole architecture, unless the phase changes the architecture.
- Prefer exact commands and exact forbidden areas over broad wording.
- Call out path, locator, export, network, and auto-mutation boundaries when relevant.
