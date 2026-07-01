# 02 - Phase Index

This is a compressed status index, not a full changelog.

| Phase or phase group | Status | Product / technical outcome | Main files or component family | Remaining gap |
| --- | --- | --- | --- | --- |
| Docs foundation and stack decisions | complete | baseline product framing, architecture intent, and stack constraints | `docs/00-executive-summary.md`, `docs/00.5-stack-decisions.md`, `docs/00.7-research-foundation.md` | keep aligned with later phases |
| Corpus authority and source registry | implemented foundation | stable source identity, validation, hashing, registry persistence, audit direction | `src-tauri/src/corpus_authority.rs`, `src-tauri/src/source_registry.rs`, `src-tauri/src/source_metadata.rs` | broader workflow polish and UX integration |
| Extraction and locator preservation | implemented foundation | extraction reports and locator continuity, including PDF text-layer extraction | `src-tauri/src/extraction.rs`, `src-tauri/src/locators.rs` | OCR and broader ingestion are not in scope |
| Chunking and retrieval | implemented foundation | deterministic chunking and retrieval contracts | `src-tauri/src/chunking.rs`, `src-tauri/src/retrieval.rs` | semantic/vector retrieval is not the main workflow |
| Evidence Pack groundwork | implemented foundation | evidence data structures and local Evidence Pack support | `src-tauri/src/evidence.rs` | synthesis and broader answer-generation workflow remain later |
| Phase 123.0 | complete | project knowledge base for external orientation and GitHub Wiki readiness | `docs/00-project-overview.md`, `docs/01-architecture-overview.md`, `docs/03-github-wiki-outline.md` | wiki mirror can follow later |
| Phase 124.0 | complete | documentation QA, terminology normalization, and wiki-ready cleanup | `README.md`, `docs/00-project-overview.md`, `docs/01-architecture-overview.md`, `docs/02-phase-index.md`, `docs/03-github-wiki-outline.md`, `docs/12-roadmap.md`, `docs/13-scientific-retrieval-architecture.md` | repo docs remain the source of truth; wiki sync can follow later |
| Phase 125.0 | complete | wiki export prep guide for copy-ready GitHub Wiki source material | `docs/04-wiki-export-prep.md`, `docs/03-github-wiki-outline.md` | repo docs remain authoritative; wiki material is a mirror only |
| Phase 126.0 | complete | wiki copy QA and publication checklist pass | `docs/04-wiki-export-prep.md`, `docs/03-github-wiki-outline.md` | repo docs remain authoritative; wiki publication is manual and mirror-only |
| Phase 127.0 | complete | docs-only product UX reorientation plan | `docs/05-product-ux-reorientation.md`, `README.md`, `docs/00-project-overview.md`, `docs/01-architecture-overview.md`, `docs/12-roadmap.md` | repo docs remain authoritative; chat-first UX work is planned, not implemented |
| Phase 128.0 | in progress | app shell / navigation skeleton | `src/App.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | diagnostics stay available but secondary; backend behavior is unchanged |
| Phase 129.0 | in progress | chat-first UX polish | `src/App.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | composer-first layout is polish only; preview/gate behavior stays intact |
| Phase 129.1 | in progress | focused workspace rendering refinement | `src/App.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | active workspace rendering replaces long default scrolling; diagnostics stay reachable |
| Phase 130.0 | in progress | chat product surface refinement | `src/App.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | assistant-like default chat surface is polish only; preview/gate behavior and diagnostics remain available |
| Phase 131.0 | in progress | chat transcript interaction model | `src/App.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | transcript turns are in-memory only; preview/gate behavior remains preview-only |
| Phase 132.0 | complete | frontend surface extraction | `src/App.tsx`, `src/workspaces/WorkspaceShell.tsx`, `src/workspaces/ScholarChatWorkspace.tsx`, `src/workspaces/SourcesWorkspace.tsx`, `src/workspaces/EvidencePacksWorkspace.tsx`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | behavior remains unchanged; Developer Diagnostics stays in `src/App.tsx` until a later safe extraction |
| Phase 133.0 | complete | chat transcript interaction model | `src/App.tsx`, `src/workspaces/ScholarChatWorkspace.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | transcript turns are in-memory only; preview/gate behavior remains preview-only and assistant-style responses stay secondary |
| Phase 134.0 | in progress | chat UX interaction polish | `src/workspaces/ScholarChatWorkspace.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | assistant-style chat ergonomics are being refined; backend behavior remains unchanged |
| Phase 135.0 | complete | local model runtime setup UX | `src/App.tsx`, `src/workspaces/ScholarChatWorkspace.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | exact GGUF and llama.cpp setup are surfaced; diagnostics remain secondary and answer generation stays absent |
| Phase 136.0 | complete | local runtime probe validation | `src/App.tsx`, `src/workspaces/ScholarChatWorkspace.tsx`, `src/styles.css`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | exact `.gguf` and llama.cpp probe flow is clarified; diagnostics remain secondary and answer generation stays absent |
| Phase 137.0 | complete | managed llama-server lifecycle | `src-tauri/src/local_server.rs`, `src-tauri/src/lib.rs`, `src/App.tsx`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | preview/start/health/stop stay consent-gated and localhost-only; managed output still does not feed Scholar Chat answers |
| Phase 138.0 | complete | managed server lifecycle hardening | `src-tauri/src/local_server.rs`, `src-tauri/src/lib.rs`, `src/App.tsx`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | port preflight, ownership clarity, and shutdown cleanup stay consent-gated and localhost-only; external servers are never stopped |
| Phase 139.0 | complete | managed server chat diagnostic | `src-tauri/src/local_server.rs`, `src-tauri/src/lib.rs`, `src/App.tsx`, `docs/05-product-ux-reorientation.md`, `docs/12-roadmap.md` | diagnostic-only local chat requests stay consent-gated, localhost-only, and answer-generation-free; external servers remain unmanaged |
| Phase 151.0 | complete | chat-first shell, sidebar, and composer redesign | `src/workspaces/WorkspaceShell.tsx`, `src/workspaces/ScholarChatWorkspace.tsx`, `src/styles.css`, `docs/02-phase-index.md`, `docs/05-product-ux-reorientation.md` | area-switcher navigation stays in place; shell chrome is calmer and preview/gate behavior is unchanged |
| Phase 154.0 | complete | Scholar Chat session store hardening | `src-tauri/src/chat_sessions.rs`, `src-tauri/src/corpus_paths.rs`, `src-tauri/src/lib.rs`, `src/App.tsx`, `src/appTypes.ts`, `docs/02-phase-index.md`, `docs/05-product-ux-reorientation.md` | per-project file-backed session history lives under `.aegis/chat/`; SQLite is not introduced yet and no session rail UI exists |
| Phase 117.0 | implemented | local Evidence Pack creation MVP | Evidence Pack backend path and managed storage | integrate with broader product surfaces |
| Phase 118.0 | implemented | PDF text-layer extraction MVP with page-level locators | extraction and locator handling | OCR and broad PDF ingestion remain out of scope |
| Phase 119.0 | implemented | first-run source import readiness UI | frontend guidance surfaces | actual import wizard remains missing |
| Phase 120.0 | implemented | manual source workflow action hints | frontend workflow copy and guidance | no automatic orchestration yet |
| Phase 121.0 | implemented preview-only | Scholar Chat agentic workflow planner preview | Scholar Chat planning surfaces | still non-executing |
| Phase 122.0 | implemented preview-only | Scholar Chat execution-gate preview | Scholar Chat gate surfaces | execution remains future work |
| Runtime diagnostics and metadata preview stack | implemented preview / guarded execution | local runtime diagnostics, metadata connector previews, OpenAlex-only execution slice | `src-tauri/src/local_runtime.rs`, scientific metadata / Scholar Chat preview commands | broader provider support and runtime productization remain later |
| Current product gap | open | finished Scholar Chat product workflow | frontend and backend workflow surfaces | execution, synthesis, broader workflow integration, and polished UX remain incomplete |

## Notes on grouping

Earlier phases are grouped by capability rather than listed one by one.

The most relevant recent product milestones are:

- Phase 123.0 docs knowledge base creation
- Phase 124.0 docs QA / GitHub Wiki readiness review
- Phase 125.0 wiki export prep guide
- Phase 126.0 wiki copy QA / publication checklist
- Phase 133.0 chat transcript interaction model
- Phase 134.0 chat UX interaction polish
- Phase 135.0 local model runtime setup UX
- Phase 136.0 local runtime probe validation
- Phase 137.0 managed llama-server lifecycle
- Phase 138.0 managed server lifecycle hardening
- Phase 139.0 managed server chat diagnostic
- Evidence Pack planning and creation
- PDF text-layer extraction
- first-run and source workflow guidance
- Scholar Chat planner preview
- Scholar Chat execution-gate preview

## Practical reading order

1. `docs/00-project-overview.md`
2. `docs/01-architecture-overview.md`
3. `docs/12-roadmap.md`
4. `docs/13-scientific-retrieval-architecture.md`
