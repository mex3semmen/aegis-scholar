# 15 — Scientific Retrieval Contract Map

## Purpose

This document is the central contract map for Scientific Retrieval. It tracks preview-only contracts, guarded execution slices, GUI readiness, backend refactor targets, and later execution cutover.

## Current pipeline map

```text
User Query
-> Scientific Query Understanding
-> Scientific Source / Discipline Context
-> Scientific Search Plan
-> Local Literature Index / Course Literature Registry
-> Metadata Connector / Psychology Source Connector planning
-> Evidence Pack planning
-> Paper Literature Review planning
-> Metadata Execution Boundary
-> Provider Config Preview
-> Metadata Query Plan Preview
-> Provider Request Preview
-> OpenAlex-only execution slice
-> normalized OpenAlex metadata result contract
-> OpenAlex cache/write gate preview
-> OpenAlex metadata to Evidence Candidate conversion preview
-> OpenAlex GUI integration readiness contract
-> Evidence Pack assembly plan preview
-> Local Evidence Pack creation MVP
-> future Evidence Pack conversion contract
-> future Literature Review / Final Answer
```

Preview-only contracts that already exist:

- Scientific Query Understanding Preview
- Scientific Source Registry Preview
- Scientific Discipline Registry Preview
- Scientific Search Plan Preview
- Local Literature Index Preview
- Course Literature Registry Preview
- OpenAlex / Crossref Metadata Connector Preview
- Psychology Source Connector Preview
- Scientific Evidence Pack Preview
- Scientific Paper Mode Literature Review Preview
- Scientific Metadata Execution Boundary Preview
- Scientific Metadata Provider Config Preview
- Scientific Metadata Query Plan Preview
- Scientific Metadata Provider Request Preview

Current execution and preview stages:

- OpenAlex-only execution slice
- normalized OpenAlex metadata result contract
- OpenAlex cache/write gate preview
- OpenAlex metadata to Evidence Candidate conversion preview
- Evidence Pack assembly plan preview
- Local Evidence Pack creation MVP

### Managed Evidence Pack creation

| Command | Current role | Intended GUI usage | Status | No-op boundary summary |
| --- | --- | --- | --- | --- |
| `build_evidence_pack` | Creates a deterministic local Evidence Pack from local source/retrieval/evidence data. | Local Evidence Pack creation MVP. | Managed local creation | No network, no model/runtime/LLM, no OpenAlex. |
| `get_evidence_pack` | Reads an existing managed Evidence Pack. | Evidence Pack inspection. | Managed read-back | No writes. |
| `list_evidence_packs` | Lists managed Evidence Pack metadata. | Evidence Pack inspection. | Managed read-back | No writes. |

Later execution stages:

- future Literature Review / Final Answer

## Command catalog

### General Scholar Chat preview / answer pipeline

| Command | Current role | Intended GUI usage | Status | No-op boundary summary |
| --- | --- | --- | --- | --- |
| `preview_scholar_chat_request` | Normalizes prompt, mode, grounding policy, and selected source context. | Main Scholar Chat entry preview. | GUI-ready preview | No generation, retrieval execution, or writes. |
| `preview_scholar_chat_agentic_workflow_plan` | Classifies chat prompt intent into a local workflow plan and next steps. | Chat-first workflow planner. | GUI-ready preview | No execution or writes. |
| `preview_scholar_chat_agentic_workflow_execution_gate` | Evaluates whether a planned chat workflow could be executed later with the right consent and local context. | Chat-first execution gate preview. | GUI-ready preview | No execution now; consent/readiness only. |
| `preview_scholar_chat_answer_readiness` | Explains whether a future answer path could proceed. | Readiness panel. | GUI-ready preview | No answer generation; diagnostic only. |
| `preview_scholar_chat_draft_inference` | Describes the draft-inference boundary. | Advanced diagnostic panel. | GUI-ready preview | No runtime execution or model calls. |
| `preview_scholar_chat_draft_grounding_inspection` | Inspects draft text against local evidence. | Draft grounding inspection panel. | GUI-ready preview | No grounded answer, Evidence Pack, or persistence. |
| `preview_scholar_chat_grounded_draft_readiness` | Summarizes whether a draft looks ready for future grounding. | Readiness summary panel. | GUI-ready preview | No grounded answer creation. |
| `preview_scholar_chat_grounded_answer_build_preflight` | Checks whether an existing draft is readable and preflight-ready. | Advanced readiness panel. | GUI-ready preview | No artifact writes or grounded answer creation. |
| `preview_scholar_chat_grounded_answer_execution_readiness` | Explains whether an explicit future build execution could be attempted. | Execution-readiness panel. | GUI-ready preview | No execution, no service calls. |
| `preview_scholar_chat_grounded_answer_execution_plan` | Describes a future grounded-answer build plan. | Execution-plan panel. | GUI-ready preview | No execution; plan only. |
| `preview_scholar_chat_grounded_answer_build_plan` | Plans a future grounded-answer build. | Build-plan panel. | GUI-ready preview | No answer artifact creation. |
| `preview_scholar_chat_grounded_answer_candidate` | Summarizes a non-persisted grounded-answer candidate. | Candidate review panel. | GUI-ready preview | No artifact writes or writes to registry/audit. |
| `preview_scholar_chat_grounded_answer_build_intent` | Describes an explicit future build intent. | Intent panel. | GUI-ready preview | No grounded answer creation. |
| `preview_scholar_chat_grounded_answer_build_request` | Describes a future build request. | Request panel. | GUI-ready preview | No write or execution behavior. |
| `preview_scholar_chat_grounded_answer_write_eligibility` | Explains whether future writing would be allowed. | Write-eligibility panel. | GUI-ready preview | No write, persistence, or registry mutation. |
| `preview_scholar_chat_retrieval` | Preview-only retrieval contract. | Retrieval diagnostics. | GUI-ready preview | No retrieval execution. |
| `preview_scholar_chat_evidence_plan` | Preview-only evidence-plan contract. | Evidence-plan diagnostics. | GUI-ready preview | No Evidence Pack build. |
| `preview_scholar_chat_prompt_pack` | Preview-only prompt-pack contract. | Prompt-pack diagnostics. | GUI-ready preview | No generation or persistence. |

### Scientific retrieval foundation

| Command | Current role | Intended GUI usage | Status | No-op boundary summary |
| --- | --- | --- | --- | --- |
| `preview_scholar_chat_scientific_discipline_registry` | Maps concepts to discipline context. | Scientific discipline panel. | GUI-ready preview | Preview only; no writes. |
| `preview_scholar_chat_scientific_source_registry` | Maps concepts to source-family context. | Scientific source panel. | GUI-ready preview | Preview only; no source import. |
| `preview_scholar_chat_scientific_query_understanding` | Normalizes the scientific query and concept path. | Query-understanding panel. | GUI-ready preview | Preview only; no search execution. |
| `preview_scholar_chat_scientific_search_plan` | Plans local/course search routing. | Search-plan panel. | GUI-ready preview | No retrieval execution. |
| `preview_scholar_chat_local_literature_index` | Plans local literature-index readiness. | Local literature panel. | GUI-ready preview | No file reads or indexing. |
| `preview_scholar_chat_course_literature_registry` | Plans course-literature registry readiness. | Course literature panel. | GUI-ready preview | No file reads or registry writes. |

### Metadata / provider planning

| Command | Current role | Intended GUI usage | Status | No-op boundary summary |
| --- | --- | --- | --- | --- |
| `preview_scholar_chat_metadata_connector_plan` | Plans provider-family connectors and alignment. | Provider-planning panel. | GUI-ready preview | No provider calls or networking. |
| `preview_scholar_chat_psychology_source_connector_plan` | Plans psychology provider routing and family boundaries. | Psychology provider panel. | GUI-ready preview | No provider calls or writes. |
| `preview_scholar_chat_scientific_evidence_pack_plan` | Plans evidence-pack shape and boundaries. | Evidence-pack planning panel. | GUI-ready preview | No Evidence Pack creation. |
| `preview_scholar_chat_scientific_paper_literature_review_plan` | Plans paper-mode review boundaries. | Literature-review planning panel. | GUI-ready preview | No Literature Review creation. |
| `preview_scholar_chat_scientific_metadata_execution_boundary` | Defines the dry-run metadata execution boundary. | Safety / boundary panel. | Execution boundary | No provider execution. |
| `preview_scholar_chat_scientific_metadata_provider_config` | Plans provider config and access boundaries. | Provider config panel. | GUI-ready preview | No provider execution or writes. |
| `preview_scholar_chat_scientific_metadata_query_plan` | Plans query templates, filters, and provider-family partitioning. | Query-plan panel. | GUI-ready preview | No provider execution or writes. |
| `preview_scholar_chat_scientific_metadata_provider_request` | Plans provider request templates, methods, parameters, headers, and bodies. | Provider request preview panel. | GUI-ready preview | No URL building, no request emission, no provider calls. |
| `run_scholar_chat_openalex_metadata_execution_slice` | Executes the consent-gated OpenAlex-only metadata execution slice. | OpenAlex execution panel. | Guarded execution slice | OpenAlex only; disabled by default; explicit developer/advanced action; no writes by default. |
| `preview_scholar_chat_openalex_metadata_cache_write_gate` | Plans cache scope, retention, deduplication, and future record/audit write boundaries from normalized OpenAlex execution output. | OpenAlex cache/write gate preview panel. | GUI-ready preview | No cache writes, no record writes, no audit writes. |
| `preview_scholar_chat_openalex_evidence_candidate_conversion` | Converts normalized OpenAlex execution records into deterministic evidence-candidate input previews. | OpenAlex evidence candidate conversion panel. | GUI-ready preview | No execution, no writes. |
| `preview_scholar_chat_evidence_pack_assembly_plan` | Plans future Evidence Pack assembly from deterministic evidence-candidate input previews. | Evidence Pack assembly plan panel. | GUI-ready preview | No Evidence Pack creation, no citations, no writes. |

### OpenAlex GUI integration readiness

Phase 113.0 adds a docs-only OpenAlex Metadata GUI Integration Readiness Contract. It defines how future GUI panels may safely integrate the existing OpenAlex metadata commands before any frontend wiring, and it keeps the command flow preview-only, consent-gated, redacted, and non-writing. GUI may use the contract before any frontend wiring, but it must not infer permission from preview output or expose an actual write button yet.
Phase 114.0 adds the first frontend read-only OpenAlex metadata panel scaffold. It uses the existing Scholar Chat prompt as query input, invokes only `preview_scholar_chat_scientific_metadata_provider_request`, and may display provider-request preview results. It does not wire OpenAlex execution or cache/write gate execution, exposes no write button, and does not add Evidence Pack, citation, Literature Review, answer, provider expansion, persistence, model/runtime/LLM, or retrieval execution behavior.
Phase 114.1 hardens the read-only OpenAlex panel boundaries. It keeps only provider request preview wired, keeps execution and cache/write unwired, adds a visible boundary checklist, and does not add any backend, dependency, or product behavior.
Phase 115.0 is a backend-only OpenAlex metadata to Evidence Candidate conversion preview. It composes the normalized OpenAlex metadata result contract, derives deterministic candidate-input previews, stays preview-only and in-memory, and does not add execution, cache/write, Evidence Pack, citation, Literature Review, answer, retrieval, runtime, or LLM behavior.
Phase 115.1 hardens the conversion preview boundary with guard tests for command surface, safety flags, deterministic caps/order, and no output leakage. It adds no new command and no behavior beyond guards.
Phase 116.0 is a backend-only Evidence Candidate to Evidence Pack Assembly Plan Preview. It composes the OpenAlex Evidence Candidate conversion preview and only plans later pack-item selection, ordering, grouping, caps, skips, and readiness boundaries. It stays preview-only and in-memory, and does not add Evidence Pack creation, citations, writes, retrieval execution, provider expansion, model/runtime/LLM behavior, or answer generation.

Reference:

- [docs/16-openalex-gui-integration-readiness.md](16-openalex-gui-integration-readiness.md)

### Runtime diagnostics

| Command | Current role | Intended GUI usage | Status | No-op boundary summary |
| --- | --- | --- | --- | --- |
| `preview_local_model_runtime_health` | Checks local runtime availability at a preview level. | Developer runtime panel. | Runtime diagnostic | No execution. |
| `preview_local_runtime_invocation_plan` | Plans local runtime invocation shape. | Developer runtime panel. | Runtime diagnostic | No execution. |
| `preview_llama_runtime_adapter_contract` | Describes the llama.cpp adapter contract. | Developer runtime panel. | Runtime diagnostic | No process spawn or model load. |
| `preview_llama_runtime_validation` | Validates adapter/runtime metadata boundaries. | Developer runtime panel. | Runtime diagnostic | No process spawn or model load. |
| `preview_llama_runtime_probe_readiness` | Explains whether a future version probe could be attempted. | Developer runtime panel. | Runtime diagnostic | No process spawn or binary probe. |
| `preview_llama_runtime_capability` | Summarizes runtime capability from probe readiness. | Developer runtime panel. | Runtime diagnostic | No new execution path. |
| `preview_llama_runtime_smoke_readiness` | Explains whether a future smoke diagnostic could be attempted. | Developer runtime panel. | Runtime diagnostic | No smoke execution. |
| `preview_llama_runtime_smoke_execution_plan` | Plans a future smoke diagnostic execution. | Developer runtime panel. | Runtime diagnostic | No smoke execution. |
| `run_llama_runtime_version_probe` | Executes the consent-gated version probe. | Explicit developer action only. | Future execution candidate | Version-only execution gate only. |
| `run_llama_runtime_smoke_diagnostic` | Executes the consent-gated smoke diagnostic. | Explicit developer action only. | Future execution candidate | Diagnostic-only execution gate only. |
| `preview_scholar_chat_runtime_diagnostic_bridge` | Bridges Scholar Chat to smoke-execution-plan status. | Developer diagnostic bridge panel. | Runtime diagnostic | No smoke execution or answer generation. |
| `preview_scholar_chat_runtime_diagnostic_result` | Summarizes smoke diagnostic results. | Developer diagnostic result panel. | Runtime diagnostic | No answer generation. |
| `preview_scholar_chat_runtime_answer_pipeline_gate` | Explains whether the runtime answer pipeline is blocked or ready later. | Developer gate panel. | Runtime diagnostic | No answer generation. |

### Future execution candidates

- `run_llama_runtime_version_probe`
- `run_llama_runtime_smoke_diagnostic`

These require explicit gates and remain outside the GUI-ready preview contract until a later execution phase.

## GUI readiness map

Safe to expose as read-only previews now:

- query understanding panel
- scientific source / discipline panel
- local / course literature panel
- metadata provider config panel
- metadata query plan panel
- provider request preview panel
- safety / boundary panel
- next required actions panel
- retrieval / evidence / prompt-pack preview panels
- evidence-pack assembly plan preview panel
- runtime diagnostic bridge / result / pipeline-gate panels for developer diagnostics

Useful for developer or advanced diagnostics only:

- local runtime health and invocation planning previews
- llama adapter contract, validation, probe readiness, capability, and smoke-readiness previews
- version probe and smoke diagnostic execution candidates as explicit developer actions
- boundary-heavy execution plan / readiness previews

Not suitable for a user-facing execution GUI yet:

- `run_llama_runtime_version_probe`
- `run_llama_runtime_smoke_diagnostic`
- any panel whose primary state is boundary booleans without actionable preview detail

Probable widgets for GUI-ready previews:

- query understanding panel
- scientific source / discipline panel
- local / course literature panel
- metadata provider config panel
- metadata query plan panel
- provider request preview panel
- safety / boundary panel
- next required actions panel

Boundary booleans should remain secondary technical details; `warnings`, `blockers`, and `next_required_actions` should drive the primary UI state.

## DTO and status contract map

- Treat preview DTOs as stable enough for read-only GUI inspection, not as permission to execute.
- Status values are explanatory, not execution results.
- Boundary booleans should be shown as safety diagnostics or collapsed technical details.
- `warnings`, `blockers`, and `next_required_actions` should drive the UI state.
- Provider IDs must stay stable canonical identifiers.
- Unknown provider IDs should be surfaced as warnings, not executed.

## Provider boundary model

- Public metadata providers: OpenAlex, Crossref, PubMed, ERIC.
- Institutional boundary providers: APA PsycNet, PsycINFO.
- Public providers may become execution candidates later.
- APA PsycNet / PsycINFO remain manual / institutional boundary until explicit legal and terms-compliant access is designed.
- No scraping.
- No paywall bypass.
- No automated authenticated library access.
- No connector calls until a later execution phase explicitly allows them.

## Backend refactor target map

Suggested future modules for reducing `src-tauri/src/scholar_chat.rs` size:

- `scientific_query.rs`
- `scientific_source_registry.rs`
- `local_literature_index.rs`
- `metadata_provider_config.rs`
- `metadata_query_plan.rs`
- `metadata_provider_request.rs`
- `metadata_execution.rs`
- `evidence_pack_plan.rs`
- `literature_review_plan.rs`
- `scholar_chat_contracts.rs` or `scholar_chat_dto.rs`
- `scholar_chat_commands.rs`

This is a future refactor target, not part of Phase 109.0.

## Execution cutover plan

The intended next execution path after this documentation phase is:

- Phase 110.0: OpenAlex Metadata Execution Slice
- implemented first OpenAlex-only execution slice
- disabled by default
- OpenAlex only
- requires `execution_requested == true`
- requires `allow_network == true`
- requires OpenAlex terms/config acceptance
- requires either a request-provided API key or explicit no-key usage consent
- writes remain blocked by default
- output is temporary in-memory normalized metadata records
- no raw provider URL, no provider hostname, no API key value, no raw provider response
- no citation emission
- no Evidence Pack creation
- no Literature Review creation
- no Crossref / PubMed / ERIC execution
- no APA PsycNet / PsycINFO execution
- GUI treats this as an explicit developer / advanced action until a later product UI phase
- Phase 110.1: OpenAlex Metadata Execution Slice Guard Hardening
- confirms the first execution slice remains explicit, OpenAlex-only, offline-tested, redacted, in-memory, and non-writing
- Phase 111.0: OpenAlex Metadata Result Normalization Contract
- stabilizes the normalized result summary, record fields, provider status labels, cursor contract, and evidence-candidate readiness hints
- remains in-memory, redacted, and non-writing
- GUI may use the summary and record hints for display, not for automatic Evidence Pack creation
- Crossref / PubMed / ERIC remain later providers
- APA PsycNet / PsycINFO remain institutional / manual boundary only
- Phase 112.0: OpenAlex Metadata Cache/Write Gate Preview
- previews future write eligibility, deduplication, retention, record-write, and audit boundaries only
- remains preview-only, in-memory, redacted, and non-writing
- GUI may show write-readiness diagnostics, but no actual write button is exposed yet
- no cache files, metadata writes, registry writes, audit writes, artifacts, Evidence Packs, Literature Reviews, citations, retrieval execution, provider expansion, persistence, model/runtime/LLM behavior, or answer generation
- Crossref / PubMed / ERIC remain later providers
- APA PsycNet / PsycINFO remain institutional / manual boundary only
- Phase 112.1: OpenAlex Metadata Cache/Write Gate Preview Guard Hardening
- confirms GUI diagnostics remain no-op and no actual write button should be exposed
- confirms policy labels, dedup/readiness planning, redaction, fake-transport-only tests, and no-write/no-registry/no-audit/no-artifact invariants
- keeps future explicit cache/write execution gate after preview hardening as future work
- Crossref / PubMed / ERIC remain later providers
- APA PsycNet / PsycINFO remain institutional / manual boundary only

Later phases should add:

- OpenAlex guard hardening
- future explicit cache/write execution gate after preview hardening
- Crossref / PubMed / ERIC as later providers
- GUI integration using preview contracts

## Refactor and GUI safety rules

- GUI must not infer execution permission from preview records.
- GUI must display blockers and warnings before any future execution button.
- Future execution buttons must require explicit gates.
- Preview DTOs may be verbose; GUI should group details into collapsible panels.
- No command should read files, call network, or write artifacts unless its phase explicitly allows that.

## Phase 109.0 boundary

- Docs-only.
- No code changes.
- No frontend changes.
- No command changes.
- No runtime behavior changes.
- No execution behavior.
- No provider calls.
- No URL building.
- No concrete URL / hostname / API-key / env / header / request-object emission.
- No writes or persistence.
