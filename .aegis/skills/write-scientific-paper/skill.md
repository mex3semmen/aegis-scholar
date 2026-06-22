# Skill — Write Scientific Paper

## ID

`write-scientific-paper`

## Purpose

Draft scientific paper sections from registered sources, user notes and explicit user instructions.

## Inputs

- topic
- discipline
- target section
- source set
- citation style
- evidence strictness

## Retrieval profile

```text
retrieval_mode: scientific_writing
requires_evidence_pack: true
source_priority: papers, textbooks, lecture_slides, markdown_note
```

## Evidence policy

- Evidence Pack required for source-grounded claims.
- Unsupported claims must be flagged before final output.
- Model memory may be used only for generic writing structure, not for sourced claims.

## Output contract

May produce:

- outline
- abstract draft
- introduction draft
- theory section draft
- method section wording
- discussion draft
- limitations section
- reference TODO list
- unsupported claim report

## Required pipeline

```text
topic -> retrieve sources -> build evidence pack -> outline -> draft -> claim check -> final section
```
