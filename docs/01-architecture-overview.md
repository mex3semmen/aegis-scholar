# 01 - Architecture Overview

## App shell

AEGIS Scholar uses a Windows desktop shell built with:

- Solid 1.x + Vite for the UI
- Tauri v2 as the desktop boundary
- Rust as the authority layer for local state and sensitive operations

The UI is not a general shell. It should only call Tauri commands.

## Tauri command boundary

All authority-bearing actions flow through Tauri commands. This boundary is where the frontend requests:

- source registration
- metadata updates
- extraction and retrieval-related operations
- preview contracts
- gated execution slices

The frontend must not bypass Rust for filesystem authority, source identity, or sensitive local writes.

## Rust backend modules

The Rust backend is the authority and workflow layer. The main module families currently include:

- corpus authority
- source registry
- source metadata
- path validation
- extraction
- chunking
- retrieval
- evidence
- locators
- audit
- local runtime supervision scaffolding
- scholar chat planner/gate previews

## Source registry

The Source Registry is the persistent authority for local sources.

It owns:

- stable source IDs
- source metadata validation
- path canonicalization
- content hash tracking
- duplicate content detection
- registry persistence
- audit event emission

## Extraction

Extraction turns registered local source material into structured extraction reports.

It preserves source identity and locators so later stages can keep provenance intact.

Current emphasis:

- local source handling
- PDF text-layer extraction
- page-level locators for PDFs where a text layer exists
- no OCR in this phase

## Chunking

Chunking produces deterministic downstream units from extraction output.

The purpose is to preserve source/version/locator continuity, not to synthesize answers.

## Retrieval

Retrieval is a separate plane from extraction and evidence construction.

It exists to support disciplined search and ranking over registered local material and related metadata, but it is not the same thing as answer synthesis.

## Evidence Packs

Evidence Packs are source-grounded bundles prepared before synthesis.

They are part of the evidence pipeline, not the final answer layer.

## Scholar Chat planner and gate previews

Scholar Chat currently includes preview-only planner and execution-gate surfaces.

These components:

- classify workflow intent
- decide whether a future execution step would need context, consent, or clarification
- remain non-executing
- do not claim automatic orchestration

## Runtime / LLM boundary

Model runtime and local LLM execution are later concerns.

The architecture keeps the runtime boundary separate from authority, retrieval, and evidence preparation. The model may reason over curated local context, but it is not the source of truth.

## Diagnostics vs product workflow

The repository contains both diagnostic surfaces and product-flow surfaces.

Diagnostics are useful for developers and reviewers, but they are not the final user experience.

## Data and write boundaries

Key boundaries:

- the frontend does not perform direct shell access
- Rust owns source identity and filesystem authority
- preview surfaces must not be described as execution
- writes must remain explicit and auditable
- anonymous context blobs must not replace source identity

## Preview vs execution distinction

This repository uses preview, gate, and execution language deliberately.

- preview means the system is planning or describing a future action
- gate means the system is deciding whether a future action is allowed or needs consent
- execution means the action actually happens

That distinction should stay visible in documentation and in the product surface.

