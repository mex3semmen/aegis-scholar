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
