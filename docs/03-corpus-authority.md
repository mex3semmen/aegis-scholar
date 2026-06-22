# 03 — Corpus Authority + Source Registry

Corpus Authority is the foundation of AEGIS Scholar.

## Core rule

```text
Every source-grounded answer must be traceable to registered sources.
```

A source is not just a file. A source has identity, version, metadata, locators, ingestion state and provenance.

## Responsibilities

Corpus Authority owns:

- source registration
- source IDs
- source versions
- source metadata
- source assets
- local source paths
- content hashes
- ingestion state
- extraction state
- locator policy
- evidence provenance
- source deletion and replacement policy

## Minimum source record

```json
{
  "source_id": "src_...",
  "version_id": "srcv_...",
  "title": "Introductory Statistics Lecture 01",
  "source_type": "lecture_slides",
  "discipline": "psychology",
  "subdiscipline": "statistics",
  "language": "de",
  "content_hash": "sha256:...",
  "created_at": "2026-06-22T00:00:00Z",
  "ingestion_status": "registered"
}
```

## Source types

Initial source types:

- `pdf`
- `lecture_slides`
- `paper`
- `textbook`
- `markdown_note`
- `web_snapshot`
- `dataset_note`

## Locator requirement

Chunks must preserve source locators. Allowed locator types:

- page
- slide
- section
- paragraph
- theorem
- definition
- table
- figure

RAG output without locators is not acceptable for source-grounded answers.

## Storage layout

```text
.aegis/
  corpus/
    sources/
      src_.../
        versions/
          srcv_.../
            source.pdf
            metadata.json
            extraction-report.json
            extracted-text.md
    registry.json

  rag/
    chunks/
    indexes/
    evidence-packs/

  audit/
    aegis.db
```

Human-readable source metadata may live as JSON files. SQLite indexes registry state, chunks, locators and audit events.

## Phase 1 commands

```text
register_source(path, metadata)
get_source(source_id)
list_sources(filter)
update_source_metadata(source_id, metadata_patch)
remove_source(source_id)
get_corpus_status()
```

## Non-goals for Phase 1

Do not implement embeddings, vector search, reranking, answer synthesis, model runtime or coding-agent behavior.
