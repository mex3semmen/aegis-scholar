# 13 - Architecture Decisions

## ADR 001 - AEGIS remains independent of Pi/OMP

Status: Accepted

### Context
OMP/Pi are strong coding-agent harnesses. AEGIS is a scientific provenance product. The repo already treats AEGIS as local-first, source-grounded, and not a Pi replacement.

### Decision
AEGIS will not be rebuilt on Pi/OMP. AEGIS will not gain a Pi/OMP runtime dependency. The product core remains Corpus Authority + Source Registry + Extraction + Retrieval + Evidence Pack + Grounded Answer.

### Allowed reuse
- UX ideas
- session patterns
- tool-card patterns
- permission-gate patterns
- an optional future sidecar/reference role for agent-harness patterns

### Rejected reuse
- core dependency
- free shell/browser/VPN agent
- credential automation
- automated institutional-login behavior
- publisher scraping

### ULB/VPN stance
Use OS-level VPN only. The app may guide, detect, or import user-provided VPN state/configuration, but it may not authenticate, scrape, or broker institutional access.

### V1 focus
Import -> extract -> chunk/index -> retrieve -> evidence pack -> grounded answer -> citations/export.

### Consequences
Stop adding new diagnostic surfaces unless they directly support the V1 path. Keep the existing closed boundaries in `docs/00.5-stack-decisions.md`, `docs/02-target-architecture.md`, `docs/03-corpus-authority.md`, and `docs/13-scientific-retrieval-architecture.md` intact.

## ADR 002 - Headless-first AEGIS Core with optional OMP adapter

Status: Accepted

### Context
AEGIS is a scientific provenance product. Pi/OMP are external harnesses. This pass finalizes a headless-first Core direction rather than rebuilding AEGIS on Pi/OMP.

### Decision
AEGIS Core is the product authority. The Tauri/Solid UI and future CLI are clients. Any OMP/Pi integration must happen only through explicit AEGIS JSON/CLI/RPC boundaries.

### Allowed adapter role
OMP/Pi may orchestrate AEGIS workflows later, but only by invoking explicit AEGIS commands and only after Core has established source identity, provenance, retrieval results, and Evidence Pack state.

### Rejected adapter role
OMP/Pi must not own source identity, provenance, retrieval truth, Evidence Packs, credentials, VPN state, institutional login, publisher scraping, browser automation, or scientific caching policy.

### ULB/VPN/access stance
OS-level VPN only, user-controlled; the app may guide, detect, or import user-provided VPN state/configuration, but it may not authenticate, scrape, broker institutional access, or handle credentials.

### V1 architecture target
`aegis-core` as authority and scientific workflow contracts; `aegis-cli` as the future JSON/headless interface; the Tauri/Solid UI as the desktop client; and an optional future OMP/Pi adapter that calls explicit AEGIS commands.

### Consequences
Future OMP/Pi integration is adapter work only; the closed boundaries in the rest of the docs stay intact; this docs pass introduces no runtime dependency, no adapter code path, and no product-behavior change.
