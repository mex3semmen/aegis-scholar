# 06 — Retrieval Architecture

AEGIS must not use one generic RAG mode for all scientific work.

## Retrieval pipeline

```text
query -> skill intent -> retrieval profile -> metadata filters -> lexical candidates -> vector candidates later -> merge -> rerank later -> evidence pack
```

## Retrieval modes

1. `fact_lookup`
   - answer what a registered source says
   - strict locator requirement

2. `concept_synthesis`
   - combine multiple local sources
   - show agreement, disagreement and uncertainty

3. `method_check`
   - evaluate statistics or method fit
   - use deterministic tools where possible

4. `citation_grounded_answer`
   - answer only from retrieved evidence
   - mark uncertainty when evidence is insufficient

5. `study_summary`
   - prioritize course sources
   - generate structured notes and questions

6. `scientific_writing`
   - build evidence packs before drafting claims
   - unsupported claims must be flagged

## Retrieval adapters

Retrieval must be behind an adapter interface.

Initial adapters:

- metadata filter adapter
- lexical adapter

Later adapters:

- vector adapter
- reranker adapter
- formula/definition adapter for mathematics

## Scaling rule

SQLite may index metadata, source identity and audit state. Vector storage may later move behind an adapter without changing Corpus Authority, Skill contracts or Evidence Pack schemas.

## Failure rule

If retrieval cannot produce sufficient evidence, the skill output must say so instead of fabricating source-grounded claims.
