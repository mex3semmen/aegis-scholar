# 00 — Executive Summary

AEGIS Scholar is a local-first academic Scholar Chat workspace. It combines a natural prompt interface with literature memory, source-grounded retrieval, evidence packs, course study, scientific writing support, statistics/method tools, discipline profiles and long-term academic project memory.

The central principle:

```text
The model may be useful only where the system can ground, retrieve, constrain and audit what it uses.
```

## Why AEGIS exists

The target user does not need another generic chatbot or coding assistant. The target user needs a local academic assistant that can:

- register literature, slides, PDFs and notes
- preserve source identity, version, metadata and locators
- retrieve evidence from a local academic corpus
- answer natural prompts through selected course or project context
- summarize courses without losing lecture context
- support scientific writing with evidence-backed claims
- handle psychology, statistics and APA workflows first
- expand into mathematics and broader MINT disciplines through discipline modules
- preserve project memory without becoming a prompt dump

## V1 product target

AEGIS Scholar v1 should feel like a local Scholar Chat workspace: the user selects a course, project, literature context or method mode, asks naturally, and the app retrieves local material first before assembling evidence context. Local model runtime and answer generation come later; the current target is the product contract those features must obey.

Primary modes:

- lecture learning / course assistant
- thesis / scientific writing
- literature review
- flashcards
- statistics / methods help

Local memory is not model training. It is a curated local corpus and project store used for retrieval, evidence packs, provenance and project memory. The LLM is a reasoning and formulation engine, not the source of truth for scientific claims.

Default answer policy is local-first: selected context, registered local sources, existing local artifacts, explicit external scholarly adapters later, and only then clearly marked general model knowledge when allowed. If no local evidence is found, AEGIS should say so instead of presenting unsupported claims as grounded.

## MVP

The first meaningful MVP is:

```text
Psychology + Statistics + Literature Memory + Evidence Packs + Study/Course Summaries + APA
```

First user-visible workflows:

- ask questions over registered literature
- summarize course material
- create study notes and exam questions
- check statistics/method reasoning
- draft APA-compatible sections from evidence packs
- build source-linked evidence packs

## Non-goal

AEGIS Scholar is not a general coding app. Pi remains the external developer/coding tool.
