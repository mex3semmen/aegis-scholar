# 14 — Local Workspace Hygiene

AEGIS Scholar source control should stay limited to source code, docs, and small config files.
Local models, generated build output, caches, and local research data belong outside Git.

## Keep Local Assets Outside the Repo

Use user-controlled local storage for large or generated data, for example:

- `D:\AEGIS-Models\` for GGUF, GGML, and Safetensors models
- `D:\AEGIS-Data\` for local literature PDFs, course materials, generated corpus data, and local indexes

Do not place model files under `E:\AEGIS Scholar\models`.
Do not stage local data or generated build outputs.

## Git Hygiene

- Run `git status --short --branch` before every commit.
- Run `git diff --cached --name-only` before every commit.
- Prefer explicit `git add <file1> <file2>`.
- Do not use `git add .` in this repository while local data may exist.
- Do not use `git clean -fdx` unless local untracked data has already been backed up or intentionally removed.

## Safe Cleanup

- `Remove-Item -Recurse -Force ".\src-tauri\target" -ErrorAction SilentlyContinue`
- `Remove-Item -Recurse -Force ".\dist" -ErrorAction SilentlyContinue`

If a model file is locked, close AEGIS, local runtime processes, Explorer previews, and editor terminals before deleting it, or reboot if necessary.

## Boundary

This hygiene note does not add production code, model loading, runtime inference, indexing, scraping, or connector behavior.
