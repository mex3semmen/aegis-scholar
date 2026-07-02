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

