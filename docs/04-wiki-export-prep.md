# 04 - Wiki Export Prep

## Purpose

This is the operational guide for manually preparing GitHub Wiki pages from the repository docs.

It is copy-prep material only. It does not publish a wiki, create a wiki clone, or replace the repo docs.

## Source Of Truth

Repo docs remain authoritative.

The GitHub Wiki is a downstream mirror of selected repo content. If the wiki and repo docs diverge, the repo docs win.

## Proposed Wiki Pages

Use the page titles below as the manual GitHub Wiki page names.

| GitHub Wiki page title / filename | Source repo docs to copy from | Intended audience | Recommended summary | Status caveat |
| --- | --- | --- | --- | --- |
| Wiki Home | `docs/00-project-overview.md` | user-facing | Short entry page with current status and navigation links. | Do not imply the wiki is the authoritative spec. |
| Project Vision | `docs/00-project-overview.md`, `docs/12-roadmap.md` | user-facing | High-level product direction and long-term scope. | Keep future-facing language separate from implemented behavior. |
| Current Capabilities | `docs/00-project-overview.md`, `docs/02-phase-index.md` | user-facing | What works today, what is preview-only, and what is still gated. | Call out preview-first, diagnostics-heavy, and partial behavior explicitly. |
| Architecture | `docs/01-architecture-overview.md`, `docs/02-target-architecture.md` | developer-facing | Compact boundary model for Tauri, Rust authority, retrieval, evidence, and runtime. | Do not blur preview surfaces with execution. |
| Agentic Workflow Model | `docs/01-architecture-overview.md`, `docs/13-scientific-retrieval-architecture.md`, `docs/12-roadmap.md` | developer-facing | Planner, gate, preview, and execution distinctions for Scholar Chat. | Planner and gate are not autonomous execution. |
| Local Evidence Pipeline | `docs/01-architecture-overview.md`, `docs/03-corpus-authority.md`, `docs/06-retrieval-architecture.md`, `docs/10-literature-rag-evidence.md` | maintainer-facing | Source registration, extraction, chunking, retrieval, and Evidence Packs. | Keep the pipeline source-grounded and do not overstate completion. |
| Scholar Chat | `docs/12-roadmap.md`, `docs/13-scientific-retrieval-architecture.md` | user-facing | Chat-first workflow, planner preview, execution gate, and current limits. | Do not present the product as a finished ChatGPT or Claude replacement. |
| Source Handling | `docs/03-corpus-authority.md`, `docs/07-ingestion-locators.md` | maintainer-facing | Source Registry identity, metadata validation, hashing, and registry rules. | Source identity and provenance rules stay strict. |
| PDF Support | `docs/00-project-overview.md`, `docs/13-scientific-retrieval-architecture.md`, `docs/07-ingestion-locators.md` | user-facing | Supported PDF path and OCR boundary. | Text-layer extraction only; no OCR or broad ingestion claim. |
| Evidence Packs | `docs/10-literature-rag-evidence.md`, `docs/01-architecture-overview.md` | developer-facing | Evidence Pack contract and how it relates to answers. | Evidence Packs are preparatory, not the final answer layer. |
| Local Runtime / LLM Boundary | `docs/01-architecture-overview.md`, `docs/02-target-architecture.md` | developer-facing | Separation between local authority, runtime supervision, and model execution. | Do not claim a finished local model manager or autonomous runtime stack. |
| Developer Setup | `README.md`, `docs/00-project-overview.md` | maintainer-facing | Repo orientation, local build expectations, and doc entry points. | Keep build instructions tied to repo state, not wiki promises. |
| Roadmap | `docs/12-roadmap.md`, `docs/02-phase-index.md` | maintainer-facing | Current phase direction and remaining gaps. | Phases are status markers, not shipping claims. |
| Glossary | `docs/00-project-overview.md`, `docs/01-architecture-overview.md`, `docs/03-corpus-authority.md` | user-facing | Stable project terms and boundary language. | Use the repo definitions exactly. |

## Manual Copy Checklist

1. Open the source repo docs for the page you are copying.
2. Copy only the sections that match the proposed wiki page purpose.
3. Keep the repo doc wording when the page is describing status, boundaries, or caveats.
4. Preserve term casing for `Source Registry`, `Evidence Pack`, `Scholar Chat`, `preview`, `gate`, and `execution`.
5. Keep preview-only, gated, and diagnostic language intact.
6. Keep PDF support phrased as text-layer extraction only unless the repo docs say otherwise.
7. Do not add claims about OCR, polished import UX, fully autonomous agents, or finished answer generation.
8. Verify the wiki page still points back to the repo docs when a local note is useful.
9. Stop and refresh the source repo doc if the wiki copy would need a new claim.

## Maintenance Checklist

1. When a repo doc changes, update the mirrored wiki page before calling the wiki current.
2. Keep this guide and `docs/03-github-wiki-outline.md` aligned.
3. Recheck phase numbers, status words, and terminology whenever Phase 123+ docs change.
4. Refresh the wiki copy if preview, gate, or execution boundaries change in the repo docs.
5. Keep the wiki subordinate to the repo docs if the two drift.
6. Review the caveats before publishing any page that mentions current capability.

## Do Not Overclaim

Do not publish any of the following as completed or fully automated features:

- Scholar Chat planner preview
- Scholar Chat execution-gate preview
- runtime diagnostics
- OpenAlex-only metadata execution slice
- Evidence Pack assembly plan preview
- local runtime / LLM boundary scaffolding
- PDF text-layer extraction only
- first-run source import readiness guidance
- manual source workflow hints

Also do not claim:

- OCR is implemented
- broad PDF ingestion is implemented
- polished production UX is complete
- fully autonomous agentic execution exists
- answer generation is production-ready

## Recommended Copy Order

1. `docs/00-project-overview.md`
2. `docs/01-architecture-overview.md`
3. `docs/02-phase-index.md`
4. `docs/03-github-wiki-outline.md`
5. `README.md`
6. `docs/12-roadmap.md`
7. `docs/13-scientific-retrieval-architecture.md`

