# 11 — Evaluation Harness

AEGIS needs evaluation before it claims scientific reliability.

## Eval goals

The evaluation harness checks:

- retrieval recall
- citation coverage
- locator correctness
- answer faithfulness
- unsupported claims
- contradiction handling
- uncertainty behavior
- skill output format compliance

## Eval case structure

Each eval case defines:

- task ID
- skill ID
- corpus fixture
- question or task
- expected sources
- expected locators
- acceptable answer properties
- forbidden claims
- scoring notes

## Minimum eval suites

### Course study

- summarize one lecture
- extract definitions
- create exam questions
- distinguish lecture content from outside knowledge

### Scientific writing

- draft an introduction from evidence
- flag unsupported claims
- preserve citations

### Literature review

- compare two sources
- identify disagreement
- avoid overclaiming

### Statistics tutor

- identify test type
- explain assumptions
- check result wording

## Acceptance gate

A feature is not done until it has at least one regression eval for retrieval, grounding and output format.
