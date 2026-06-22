# 04 — Skill System

Skills are declarative academic workflow contracts.

They are not hardcoded app features and not hidden prompt dumps.

## Skill layers

```text
Core system
  Corpus Authority, Retrieval, Evidence Packs, Audit

Discipline modules
  Psychology, Statistics, Mathematics, Computer Science, Biology, etc.

Task skills
  Write scientific paper, Study course, Literature review, Statistics tutor
```

## Skill contract

A skill defines:

- id
- title
- purpose
- inputs
- allowed source types
- required retrieval mode
- evidence policy
- output contract
- allowed tools
- refusal or uncertainty behavior

## Initial skills

### write-scientific-paper

Drafts scientific paper sections from registered sources and user notes.

Required pipeline:

```text
topic -> retrieve evidence -> build evidence pack -> outline -> section draft -> claim check
```

### study-course

Turns lecture slides, PDFs and notes into summaries, definitions, examples, likely exam questions and later flashcards.

Retrieval priority:

```text
course material first -> textbooks second -> papers only if enabled
```

### literature-review

Compares sources, extracts themes, identifies agreement/disagreement and prepares literature review structures.

### statistics-tutor

Explains and checks statistics tasks with deterministic tools where needed.

## Rule

Skills may request retrieval and tools, but they do not own database state and must not bypass Corpus Authority.
