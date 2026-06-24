# 01 — Product Blueprint

## Product definition

AEGIS Scholar is a local-first academic Scholar Chat workspace for scientific learning, literature work, evidence-grounded reasoning and academic production.

The v1 product target is a ChatGPT/Claude/Codex-like prompt interface for academic work, but grounded in local registered sources, local corpus memory, retrieval, evidence packs and source provenance. The local LLM runtime comes later; the product contract is local-first and source-grounded before synthesis.

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
  ├─ Scholar Chat Shell
  ├─ Mode / Context Selector
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
- ask natural questions in a selected local course/project context
- summarize courses
- draft scientific sections from evidence packs
- solve/check statistics exercises
- create evidence packs
- manage project memory

## Primary Scholar Chat modes

- Lecture learning / course assistant: answer from lecture and course material first, summarize lectures, explain concepts and later generate study questions.
- Thesis / scientific writing: support research questions, outlines, literature synthesis and chapter drafting from local corpus evidence first.
- Literature review: compare papers, summarize findings and maintain provenance.
- Flashcards: later generate source-linked study cards from course or literature material.
- Statistics / methods help: explain methods and support reproducible academic work.

## Local-first answer policy

AEGIS local memory is not model training. It is a curated corpus and project store used for retrieval, evidence packs, provenance and source grounding.

Answer context priority:

- selected course or project context
- registered local sources
- previously created local artifacts
- external scholarly adapters later, after results become Source Registry entries
- general model knowledge only when explicitly allowed or clearly marked as not locally grounded

Scientific claims must not be answered from anonymous context blobs. If local evidence is missing, the app should say so rather than invent or imply support.

## Long-term vision

AEGIS grows by adding skills, discipline profiles and corpus profiles, not by training a new monolithic science model first.
