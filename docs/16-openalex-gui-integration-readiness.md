# 16 — OpenAlex GUI Integration Readiness

## Purpose

This is the GUI integration contract for the OpenAlex metadata lane only. It explains how future GUI panels may safely present the existing OpenAlex metadata commands without accidentally triggering provider execution, writes, retrieval, citations, Evidence Pack creation, Literature Review creation, answer generation, runtime behavior, model behavior, or provider expansion.

It is a docs-only readiness contract. It does not add UI, commands, DTOs, provider execution, writes, or any product behavior.

## Command sequence and panel mapping

Safe command flow:

- `preview_scholar_chat_scientific_metadata_provider_request`
  - GUI panel: Provider Request Preview
  - Button behavior: safe preview only
  - No provider call
- `run_scholar_chat_openalex_metadata_execution_slice`
  - GUI panel: OpenAlex Execution
  - Button behavior: advanced / developer explicit action only
  - Requires all existing execution gates and consent flags
  - Provider execution may happen only after explicit user action and explicit network / provider consent
  - No writes
- `preview_scholar_chat_openalex_metadata_cache_write_gate`
  - GUI panel: Cache / Write Gate Diagnostics
  - Button behavior: safe preview only
  - Must not expose an actual write button yet
  - No writes

## Safe GUI states

- `empty`
  - May show the selected mode or a prompt to choose a query
  - Preview buttons disabled until required inputs exist
  - Must not imply permission to execute
- `request_preview_ready`
  - May show normalized request details and provider-selection guidance
  - Preview button enabled
  - Must not imply provider execution
- `execution_blocked`
  - May show blockers and next required actions
  - Execution button disabled or hidden
  - Must not allow provider execution
- `execution_ready_but_not_requested`
  - May show readiness diagnostics only
  - Execution button may remain hidden behind advanced / developer mode
  - Must not auto-run execution
- `execution_running`
  - May show progress or busy state after explicit action only
  - Preview outputs remain read-only
  - Must not imply writes or downstream artifacts
- `execution_result_ready`
  - May show normalized records and result summaries
  - Must not auto-create Evidence Packs or citations
- `cache_write_gate_preview_ready`
  - May show future cache/write diagnostics
  - Must remain preview-only
  - Must not show an actual write button
- `cache_write_requested_but_blocked`
  - May show that write eligibility is blocked or deferred
  - Write action remains unavailable
  - Must not write anything
- `provider_error`
  - May show category, status, and next step only
  - Must remain redacted
  - Must not expose raw provider output or secrets
- `no_records`
  - May show empty-state messaging
  - Must not imply failure
  - Must not create fallback records
- `redaction_warning`
  - May show that details were intentionally redacted
  - Must not reveal raw URLs, hostnames, keys, or local paths
- `offline_only_test_state`
  - May show fake-transport / offline test diagnostics only
  - Must not imply live provider execution

## Button and action rules

- Preview buttons are safe by default.
- The OpenAlex execution button must be hidden behind advanced / developer mode.
- The execution button requires explicit consent flags.
- The cache / write panel is diagnostics-only.
- No write button should appear yet.
- No automatic execution after preview.
- No automatic cache/write after execution.
- No automatic Evidence Pack or citation generation after result display.
- No automatic answer generation.

## Data display rules

- Display normalized query and provider status labels.
- Display result summaries and normalized records.
- Display evidence-candidate hints as hints only.
- Display cache/write diagnostics as diagnostics only.
- Display blockers and warnings prominently.
- Never display secrets, provider hostnames, raw URLs, DOI URLs, landing-page URLs, raw provider JSON, local paths, `.aegis`, model paths, or target / dist / models paths.
- Do not show raw request internals as copyable network instructions.

## Consent and gating rules

- Network execution consent must be explicit.
- Provider terms and config acknowledgement must be explicit.
- OpenAlex-only provider selection must stay explicit.
- No-key usage consent is only relevant when no API key is supplied.
- No metadata write consent is actionable yet.
- Write flags may be shown only as blocked diagnostics.
- The UI must not infer permission from preview outputs.

## Error and redaction handling

- Provider error display must remain redacted.
- Show category, status, and user-actionable next step only.
- No raw network details.
- No raw provider response.
- No API key.
- No local paths.

## Evidence Pack boundary

- Normalized metadata records are not Evidence Pack entries yet.
- Evidence-candidate hints do not authorize Evidence Pack creation.
- Later phases must introduce an explicit Evidence Pack conversion contract.
- No automatic citation emission.

## Cache/write boundary

- Cache/write gate preview plans future write eligibility only.
- No cache files.
- No metadata records.
- No registry write.
- No audit write.
- No persistence.
- No `.aegis`.
- Actual cache/write execution requires a later explicit phase.

## Future GUI integration checklist

- Add read-only panels first.
- Wire provider request preview.
- Add execution button only in advanced / developer section.
- Display normalized records.
- Display cache/write diagnostics.
- Keep writes disabled.
- Keep Evidence Pack / citation buttons disabled.
- Add telemetry / audit only in a later explicit no-secret contract.
- Add user-facing copy only after redaction review.

## Explicit non-goals

- No frontend implementation.
- No CSS / layout work.
- No new Rust command.
- No DTO changes.
- No provider expansion.
- No cache/write execution.
- No Evidence Pack.
- No citation generation.
- No answer generation.
- No model/runtime behavior.
- No scraping, paywall bypass, or authenticated automation.
