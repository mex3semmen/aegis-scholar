# 10 — Literature Memory, RAG and Evidence Packs

## Purpose

Literature Memory makes scientific work source-grounded.

It handles:

- registered sources
- PDFs
- lecture slides
- notes
- papers
- extracted text
- chunks
- retrieval indexes
- evidence packs
- citations
- contradiction warnings

## Evidence Pack

An Evidence Pack is not just search results. It is a grounded context object used before synthesis.

Minimum fields:

- evidence pack ID
- query
- retrieval mode
- created time
- source references
- claim units
- warnings
- insufficiency notes

Phase 5.0 stores a local Evidence Pack JSON file at:

`./.aegis/corpus/sources/{source_id}/versions/{version_id}/evidence/{evidence_pack_id}.json`

The pack is built from deterministic lexical retrieval only.
It preserves source, version, chunk, locator, score, text hash, matched terms, and a short preview.
This phase does not use embeddings, vector search, or answer synthesis.

Phase 6.0 adds a mechanical Answer Draft scaffold over an Evidence Pack.
It converts each evidence item into one conservative supported claim and does not synthesize final prose.

Phase 7.0 adds a Grounded Answer contract over Answer Drafts.
It stores mechanical statement projections under the managed corpus tree and preserves unsupported / needs-evidence status without final prose generation.

## Evidence unit

Each evidence unit must carry:

- source ID
- source version ID
- chunk ID
- title
- locator
- excerpt
- claim
- confidence

## RAG v1

Start simple:

- registered local source files
- SQLite source index
- lexical search baseline
- chunk retrieval
- metadata filters
- evidence pack creation before answer synthesis

No external vector server is required in v1.

## RAG later

Add behind adapters:

- embeddings
- vector index
- reranking
- formula/definition retrieval for mathematics

## Grounding rule

If the system cannot build a sufficient Evidence Pack, it must not present the answer as source-grounded.
