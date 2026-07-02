# 02 — Target Architecture

## System diagram

```text
Windows Host
  ├─ AEGIS Desktop App
  │   ├─ Solid 1.x + Vite UI
  │   └─ Tauri v2 + Rust Core
  │       ├─ Scholar Chat Request Contract later
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
selected context -> prompt -> source registry -> extraction -> chunks -> retrieval -> evidence pack -> grounded answer or skill output
```
## Target boundary split

```text
aegis-core = authority and scientific workflow contracts
aegis-cli = future JSON/headless interface
Tauri/Solid UI = desktop client
optional OMP/Pi adapter = future boundary that calls explicit AEGIS commands
```

`aegis-cli` and the adapter are target/future boundaries only; they do not exist yet.

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

The model plane is not the source of truth for scientific claims. It formats and reasons over explicit local context after retrieval and evidence-pack assembly. General model knowledge must be explicitly allowed or clearly marked as not locally grounded.

### Skill Plane

Owns academic workflows:

- write scientific paper
- study course
- literature review
- statistics tutor
- future discipline workflows

## Frontend role

The frontend handles views and user interaction. It does not own source authority, filesystem writes, direct process execution or hidden database mutation.

The v1 frontend target is a Scholar Chat shell: mode/context selection plus a natural prompt surface backed by local corpus retrieval and evidence packs. Current diagnostic surfaces are implementation aids, not the final product workflow.

## External scholarly search

External scientific search should come later through explicit adapters. External results must become Source Registry entries before they are used as evidence. The system must not answer scientific claims from anonymous external context blobs.
