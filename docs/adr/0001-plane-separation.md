# ADR 0001 — Plane Separation

## Status

Accepted.

## Context

A scientific LLM OS can become unmaintainable if source state, retrieval indexes, model prompts, tools and task skills are mixed together.

## Decision

AEGIS separates the system into four planes:

1. Control Plane
   - source identity
   - metadata
   - registry state
   - audit
   - permissions
   - migration state

2. Retrieval Plane
   - lexical search
   - vector search later
   - metadata filters
   - reranking later
   - retrieval profiles

3. Model Plane
   - model runtime later
   - synthesis later
   - embeddings later
   - model profiles

4. Skill Plane
   - academic task contracts
   - input requirements
   - output contracts
   - evidence policies

## Consequences

- Skills do not own the database.
- Retrieval backends are replaceable.
- Corpus Authority remains stable when retrieval implementation changes.
- Evidence Pack schemas remain stable across model changes.
- Codex implementation tasks must not collapse the planes for convenience.
