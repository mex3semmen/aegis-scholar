# 07 — Ingestion and Locators

Scientific RAG fails when source location is lost. AEGIS treats locators as mandatory.

## Ingestion stages

```text
registered -> copied/indexed -> extracted -> chunked -> indexed -> retrievable
```

## Extraction report

Every extraction produces a report:

- extractor name and version
- source version ID
- pages/slides detected
- extraction warnings
- failed pages/slides
- table/figure handling notes
- text quality estimate

## Locator types

Allowed locators:

- page
- slide
- section
- paragraph
- theorem
- definition
- equation
- table
- figure

## Chunking rule

Each chunk must include:

- chunk ID
- source ID
- source version ID
- locator
- text
- hash
- discipline metadata
- extraction confidence

## Course material rule

Course summaries must keep lecture order where available.

For lecture slides, slide number is the primary locator. For PDFs, page number is the primary locator. For Markdown notes, heading path is the primary locator.

## Scientific writing rule

A generated paragraph that uses external or uploaded material must be traceable to evidence units.

Unsupported claims must be flagged before final output.

## Phase 2.0 note

Phase 2.0 writes deterministic extraction reports under:

```text
.aegis/corpus/sources/{source_id}/versions/{version_id}/extraction/report.json
```

Supported Phase 2.0 source types for text extraction are:

- `markdown_note`
- `dataset_note`
- `web_snapshot`

Unsupported Phase 2.0 behavior:

- `pdf`, `lecture_slides`, `paper`, and `textbook` return a typed unsupported extraction error for now
- paragraph locators are emitted for text units
- section locators are derived from Markdown headings and section paths

## Phase 3 note

Chunking consumes extraction reports and writes chunk reports under:

```text
.aegis/corpus/sources/{source_id}/versions/{version_id}/chunks/chunks.json
```

Phase 3 preserves source identity, version identity and locator identity without adding embeddings or retrieval indexes.
