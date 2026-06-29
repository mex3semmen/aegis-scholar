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
5. Phase 132: Developer diagnostics mode

Phase 128 now exists as the app shell/navigation skeleton. Scholar Chat is the default workspace, while Sources, Evidence Packs, and Developer Diagnostics remain available as secondary areas.

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
