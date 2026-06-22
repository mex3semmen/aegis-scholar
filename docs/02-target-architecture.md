# 02 — Target Architecture

## System diagram

```text
Windows Host
  ├─ AEGIS Desktop App
  │   ├─ Solid 1.x + Vite UI
  │   └─ Tauri v2 + Rust Core
  │       ├─ Corpus Authority
  │       ├─ Source Registry
  │       ├─ Ingestion Pipeline
  │       ├─ Metadata Store
  │       ├─ Retrieval Engine
  │       ├─ Evidence Pack Builder
  │       ├─ Skill System
  │       ├─ Discipline System
  │       ├─ Audit Store
  │       ├─ Model Manager later
  │       └─ Workspace Authority later
  │
  ├─ Local Corpus
  │   └─ .aegis/corpus/
  │
  ├─ RAG Store
  │   └─ .aegis/rag/
  │
  ├─ Skill Contracts
  │   └─ .aegis/skills/
  │
  └─ Model/cache directories later
```

## Core flow

```text
source -> registry -> extraction -> chunks -> retrieval -> evidence pack -> skill output
```

## Plane separation

### Control Plane

Owns stable identity and state:

- sources
- source versions
- metadata
- ingestion state
- audit events
- skill registry
- settings

### Retrieval Plane

Owns search and ranking:

- lexical search
- metadata filters
- vector search later
- reranking later
- retrieval profiles

### Model Plane

Owns model execution later:

- synthesis
- embedding generation later
- local model profiles
- runtime supervision

### Skill Plane

Owns academic workflows:

- write scientific paper
- study course
- literature review
- statistics tutor
- future discipline workflows

## Frontend role

The frontend handles views and user interaction. It does not own source authority, filesystem writes, direct process execution or hidden database mutation.
