# Skill — Study Course

## ID

`study-course`

## Purpose

Turn lecture slides, course PDFs and notes into structured study material.

## Inputs

- course or topic
- source set
- desired depth
- output type

## Retrieval profile

```text
retrieval_mode: study_summary
requires_evidence_pack: true
source_priority: lecture_slides, markdown_note, textbook, paper
```

## Evidence policy

- Course material is preferred over external sources.
- Textbooks are allowed as secondary support.
- Papers are used only when explicitly enabled.
- Lecture order must be preserved where possible.

## Output contract

May produce:

- structured course summary
- key concepts
- definitions
- examples
- likely exam questions
- confusion points
- short quiz
- later flashcards

## Required behavior

If the course sources are insufficient, the skill must say which source material is missing.
