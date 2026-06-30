# 05 - Product UX Reorientation Plan

## Purpose

This is a concise product-direction plan for the next UI phase.

It defines where the product should go before any UI refactor begins. It does not claim that the chat-first shell exists yet, and it does not change the implementation status of any current surface.

## Current Product Problem

AEGIS Scholar is still useful for development and review, but the current experience is diagnostics-heavy.

Current friction points:

- too many preview panels and inspector-style views
- too much developer-oriented copy in the primary user path
- the default experience still feels like a contract-review surface
- the app is not yet shaped like a polished research workflow

This is acceptable for the current phase of technical validation, but it is not the desired product default.

## Target Product Direction

The next product direction is a chat-first academic workflow with clear secondary diagnostics.

Target structure:

- Scholar Chat as the primary entry point
- sidebar-style navigation for major areas
- Sources area for source readiness and source-oriented work
- Evidence Packs area for review and inspection
- Settings and Developer Diagnostics area for advanced or inspectable behavior
- explicit local-first boundaries
- explicit consent-gated execution boundaries

This direction keeps the app local, source-grounded, and inspectable, while making the default surface easier to understand for non-developers.

## What Becomes Primary

The primary product path should center on:

- user research intent
- source import and readiness
- grounded answer workflow
- Evidence Pack creation and inspection

The default surface should make it obvious how a user moves from question to local source grounding to evidence review.

## What Becomes Secondary

Secondary surfaces should still exist, but they should not dominate the first impression:

- raw preview panels
- debug dumps
- contract inspection views
- developer-only diagnostics
- managed llama-server lifecycle controls for local runtime startup, health checks, and stop actions

These remain important for engineering and review, but they should be clearly separated from the main research workflow.

## Non-Goals

This phase does not introduce:

- autonomous execution without a gate
- OCR support
- production answer-generation claims
- a large UI rewrite in Phase 127

The plan is directional. It sets the target product shape before implementation work begins.

## Proposed Next Phases

The next implementation sequence should be:

1. Phase 128: App shell/navigation skeleton
2. Phase 129: Chat-first workflow surface consolidation
3. Phase 130: Sources workspace MVP
4. Phase 131: Evidence Pack workspace MVP
5. Phase 132.0: Frontend surface extraction

Phase 128 now exists as the app shell/navigation skeleton. Phase 129.1 is the focused workspace-rendering polish pass that turns the shell from dashboard-like navigation into workspace-specific rendering. Phase 130.0 is the chat product surface refinement pass that makes Scholar Chat feel more assistant-like while keeping preview and gate behavior intact. Phase 131.0 is the chat transcript interaction model pass that turns previews into transcript-style assistant turns while keeping the UI preview-only. Phase 132.0 is the frontend surface extraction pass that splits major workspace rendering into smaller frontend components without changing behavior. Scholar Chat is the primary conversational workspace, and Sources, Evidence Packs, and Developer Diagnostics remain secondary areas. The frontend extraction work follows the chat transcript interaction model and keeps diagnostics available but secondary.

Phase 133.0 is the chat transcript interaction model implementation pass that keeps the composer dominant, shows user prompts as messages, and renders preview and execution-gate results as assistant-style responses without changing backend behavior.
Phase 134.0 is the chat UX interaction polish pass that makes the first viewport calmer, the composer more prominent, and the sidebar lighter while preserving the same preview and transcript behavior.
Phase 135.0 is the local model runtime setup UX pass that keeps the model path and llama.cpp executable setup secondary, consent-gated, and diagnostics-only until future answer generation work exists.
Phase 136.0 is the local runtime probe validation pass that sharpens the readiness, validation, version-probe, and smoke-diagnostic flow for an exact `.gguf` model file and llama.cpp executable without changing answer generation behavior.
Phase 137.0 is the managed llama-server lifecycle pass that keeps backend-owned localhost startup, health, and stop controls visible in Developer Diagnostics while remaining preview-first and answer-generation-free.
Phase 138.0 is the managed server lifecycle hardening pass that adds port preflight, ownership clarity, and shutdown cleanup for the already-managed llama-server without turning the output into a Scholar Chat answer.
Phase 139.0 is the managed server chat diagnostic pass that adds a consent-gated, diagnostic-only local chat completion request for the already-managed localhost server while keeping Scholar Chat answer generation out of scope.
Phase 140.0 is the Sources workspace import-wizard pass that turns local source onboarding into a guided register -> extract -> chunk -> index flow while keeping Scholar Chat preview-only and diagnostics secondary.
Phase 141.0 adds an explicit, user-triggered Evidence Pack build action for indexed local sources while keeping Scholar Chat preview-only and answer generation out of scope.
Phase 142.0 adds an explicit Answer Draft build action for existing Evidence Packs. The result remains a mechanical evidence-only claim scaffold; GroundedAnswer, FinalAnswer, citations, and LLM prose remain out of scope.
Phase 143.0 adds an explicit Grounded Answer build action for the currently displayed Answer Draft. The result is a mechanical contract-only statement scaffold; FinalAnswer, citations, and LLM prose remain out of scope.

## Acceptance Criteria For Future UX Work

Future UX work should satisfy these checks:

- the default screen is understandable to a non-developer
- diagnostics are accessible but not dominant
- preview and execute distinction is visible
- consent gate remains explicit
- local/private boundaries remain visible

## Source Of Truth

Repo docs remain authoritative.

Use this plan as a product-direction guide, not as a statement that the chat-first shell or the later workspace redesign has already been implemented.
