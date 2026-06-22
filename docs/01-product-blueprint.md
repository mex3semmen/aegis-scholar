# 01 — Product Blueprint

## Product definition

AEGIS Scholar is a local-first AI knowledge workspace for scientific learning, literature work, evidence-grounded reasoning and academic production.

It combines:

- local literature memory
- Corpus Authority
- Source Registry
- ingestion and locator preservation
- hybrid retrieval architecture
- Evidence Packs
- discipline-specific retrieval profiles
- declarative academic skills
- deterministic tools for statistics/math where needed
- local model runtime later
- audit and provenance tracking

## Product modules

```text
AEGIS Scholar
  ├─ App Shell
  ├─ Ask / Study Composer
  ├─ Corpus Authority
  ├─ Source Registry
  ├─ Ingestion Pipeline
  ├─ Metadata Store
  ├─ Retrieval Engine
  ├─ Evidence Pack Builder
  ├─ Skill System
  ├─ Scholar Discipline System
  ├─ Literature Memory / RAG
  ├─ Long-Term Memory
  ├─ Deterministic Statistics/Math Tools
  ├─ Model Manager later
  ├─ Runtime Supervisor later
  ├─ Audit / Diagnostics
  └─ Workspace Authority as safety layer later
```

## MVP workflows

```text
Psychology + Statistics + Course Study + Literature Memory + APA
```

First tasks:

- register local PDFs/slides/notes as sources
- extract source text with locators
- chunk source material with metadata
- retrieve relevant evidence from local material
- summarize courses
- draft scientific sections from evidence packs
- solve/check statistics exercises
- create evidence packs
- manage project memory

## Long-term vision

AEGIS grows by adding skills, discipline profiles and corpus profiles, not by training a new monolithic science model first.
