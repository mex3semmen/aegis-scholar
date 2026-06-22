# Skill — Statistics Tutor

## ID

`statistics-tutor`

## Purpose

Explain, solve and check statistics tasks with source support and deterministic tools where needed.

## Inputs

- statistics question
- dataset summary or task text
- discipline context
- allowed formulas/tools

## Retrieval profile

```text
retrieval_mode: method_check
requires_evidence_pack: false
source_priority: lecture_slides, textbook, markdown_note, paper
```

## Evidence policy

- Use course material first when solving course tasks.
- Use deterministic calculation tools when numbers must be computed.
- Distinguish conceptual explanation from sourced claim.
- State assumptions explicitly.

## Output contract

May produce:

- explanation
- formula selection
- assumption check
- step-by-step solution
- APA wording
- interpretation
- common mistake warning
