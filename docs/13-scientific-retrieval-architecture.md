# 13 — Scientific Retrieval Architecture

## Purpose

AEGIS Scientific Retrieval is not generic web search.
It is a controlled scientific retrieval pipeline for disciplined local-first work:

- discipline understanding
- source selection
- query planning
- local and external evidence planning
- evidence pack preparation
- Scholar Chat support
- Scientific Paper Mode support
- Course Mode support

The architecture is designed to support preview-first rollout phases before any execution or answer-writing behavior is enabled.

## Current State

The currently implemented system has preview gates for local runtime and Scholar Chat answer-related planning.
It does not yet implement the scientific retrieval stack itself.
Local scientific models and corpus data must remain outside Git in user-controlled storage until explicit app-managed storage is implemented.

Not yet implemented:

- discipline registry
- source registry
- scientific query planner
- external metadata connectors
- local literature index
- BM25 retrieval
- vector retrieval
- evidence pack generation from scientific search
- automatic literature search

## Phase 98.1

Phase 98.1 adds the first backend preview of the scientific discipline registry.
It maps early example concepts such as Signalentdeckung, ANOVA, and Hypothesentests.
It remains preview-only and does not add web requests, scraping, connectors, local indexing, model loading, runtime inference, answer generation, Evidence Pack creation, or artifact writes.

## Phase 98.2

Phase 98.2 adds the first backend preview of the scientific source registry.
It maps source-family plans for Signalentdeckung, ANOVA, and Hypothesentests.
It remains preview-only and does not add web requests, scraping, connectors, source import, local indexing, BM25/vector index, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.

## Target Architecture

Target pipeline:

User query
→ Scientific query understanding
→ Discipline graph lookup
→ Source registry lookup
→ Query expansion
→ Local data search plan
→ External metadata search plan
→ Ranking / deduplication plan
→ Evidence Pack plan
→ Scholar Chat / Scientific Paper Mode / Course Mode response

Early phases remain preview-only until explicitly changed.

## Discipline Graph

The discipline layer is a graph, not a rigid tree.

Required relationship types:

- `belongs_to`
- `parent_path`
- `uses_methods_from`
- `appears_in`
- `preferred_sources`
- `curriculum_sources`
- `canonical_mappings`

A graph is better than a simple tree because scientific concepts can participate in multiple useful relationships at once.
For example, Signal Detection Theory belongs to psychology / psychophysics, but it also uses statistics, probability theory, decision theory, and ROC analysis.

## Core Science Classification

The internal science classes are:

- `core_science`
- `method_science`
- `applied_domain`
- `curriculum_layer`

Working taxonomy:

### core_science

- mathematics
- statistics
- physics
- chemistry
- biology
- psychology
- neuroscience
- medicine_biomedical_science
- computer_science

### method_science

- logic
- measurement_theory
- research_methods
- experimental_design
- information_retrieval
- machine_learning
- data_analysis

### applied_domain

- engineering
- education
- economics
- social_sciences
- law
- business

Social sciences are not treated as a core science in this AEGIS taxonomy.
They may be included as `applied_domain` or neighboring-domain metadata when needed.

## Canonical Classification Systems

These are target mapping candidates, not implemented claims:

- MSC2020 for mathematics / statistics
- MeSH for medicine / biomedical science
- ACM CCS for computer science
- OpenAlex Topics for broad multidisciplinary mapping
- APA / PsycInfo / PubPsych-style categories for psychology
- arXiv categories for preprints
- zbMATH classification for mathematics
- custom AEGIS method tags for cross-disciplinary methods

## Curriculum Layer

Module handbooks can be useful curriculum metadata.

Examples of useful curriculum signals:

- module names
- course hierarchy
- prerequisites
- ECTS
- learning goals
- topic lists

Curriculum metadata is not the same as scientific truth.
It must be mapped to canonical classifications and literature sources.

Do not scrape protected or access-restricted full texts.
User-authenticated or local user-provided documents may be indexed later only within user-controlled / local boundaries.

## Source Registry

Candidate source families:

### Multidisciplinary

- OpenAlex
- Crossref
- DOAJ

### Psychology

- PubPsych
- PsychArchives
- PsyArXiv
- PubMed when biomedical / neuro / clinical context applies

### Mathematics / Statistics

- zbMATH Open
- arXiv
- Crossref
- OpenAlex

### Medicine / Neuroscience

- PubMed
- Europe PMC
- MeSH-linked sources
- OpenAlex
- Crossref

### Computer Science

- arXiv
- ACM / DBLP-style metadata where legally / API appropriate
- OpenAlex
- Crossref

### Library / Curriculum

- TU Darmstadt / ULB Darmstadt catalog or discovery metadata where legally / API appropriate
- local user-provided course PDFs and module materials later

Document access classes:

- `open_metadata`
- `open_full_text`
- `user_provided_local_file`
- `user_authenticated_access`
- `restricted_no_ingest`
- `catalog_only`

Official APIs and metadata endpoints are preferred over scraping.

## Mode Behavior

### Scholar Chat

- interprets topic
- searches local evidence first
- plans external scientific search if needed
- answers only from verified evidence later

### Scientific Paper Mode

- research question decomposition
- literature search plan
- source ranking
- deduplication
- review / meta-analysis prioritization where appropriate
- citation / evidence pack planning
- no fabricated citations

### Course Mode

- course / module context first
- local course literature first
- module handbook / curriculum alignment
- prerequisite and learning path support
- external literature only as supplement

### Kursmodus

Course Mode may also be referenced as Kursmodus in UI or documentation that targets German-speaking academic users.

## Examples

### Example A: Signalentdeckung

Input: `Signalentdeckung`

Expected preview:

- `recognized_concept`: `signal_detection_theory`
- German label: `Signalentdeckungstheorie`
- path: `psychology > general_psychology > perception > psychophysics > signal_detection_theory`
- related methods: `statistics`, `probability_theory`, `decision_theory`, `roc_analysis`
- preferred sources: `pubpsych`, `psycharchives`, `openalex`, `crossref`, `pubmed` if biomedical context
- `no_web_request`: `true`
- `no_scraping`: `true`
- `preview_only`: `true`

### Example B: ANOVA

Input: `ANOVA`

Expected preview:

- `recognized_concept`: `analysis_of_variance`
- path: `statistics > inferential_statistics > hypothesis_testing > analysis_of_variance`
- psychology mapping: `psychology > methods > experimental_design > group_comparisons`
- related methods: `linear_models`, `f_test`, `effect_size`, `post_hoc_tests`, `repeated_measures`
- preferred sources: `openalex`, `crossref`, `psycharchives` / `pubpsych` for psychology context, `zbmath` / `arxiv` for mathematical / statistical theory
- local course materials prioritized in Course Mode
- `preview_only`: `true`

### Example C: Hypothesentests

Input: `Hypothesentests`

Expected preview:

- `recognized_concept`: `hypothesis_testing`
- path: `statistics > inferential_statistics > hypothesis_testing`
- related concepts: `null_hypothesis`, `alternative_hypothesis`, `p_value`, `type_i_error`, `type_ii_error`, `power`, `confidence_intervals`
- preferred sources: `openalex`, `crossref`, `zbmath`, `arxiv`, psychology-specific sources if applied psychology context
- `preview_only`: `true`

## Legal and Boundary Rules

- no scraping of protected full text
- no bypassing paywalls
- no automated scraping of authenticated library sessions
- prefer APIs and metadata
- user-provided local documents may be indexed later under local / user control
- store license / access metadata for every source
- full text ingestion requires explicit legal / access status
- preview phases must not perform network access

## Preview-First Implementation Roadmap

Proposed phases:

- Phase 98.0 - Scientific Retrieval Architecture Docs
- Phase 98.1 - Scientific Discipline Registry Preview
- Phase 98.2 - Scientific Source Registry Preview
- Phase 99.0 - Scientific Query Understanding Preview
- Phase 99.1 - Scholar Chat Scientific Search Plan Preview
- Phase 100.0 - Local Literature Index Preview
- Phase 100.1 - Course Literature Registry Preview
- Phase 101.0 - OpenAlex / Crossref Metadata Connector Preview
- Phase 102.0 - Psychology Source Connector Preview
- Phase 103.0 - Scientific Evidence Pack Preview
- Phase 103.1 - Scientific Evidence Pack DTO Review / Guard Hardening
- Phase 104.0 - Scientific Paper Mode Literature Review Preview
- Phase 105.0 - Scientific Metadata Connector Execution Boundary

Each phase remains preview-first until explicitly changed.
Phase 99.0 composes the discipline and source registry previews only; it stays preview-only and does not add web requests, scraping, connector implementation, source import, local file indexing, BM25/vector index, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 99.1 composes Scientific Query Understanding only and plans local-first search, metadata search, query expansion, source-family routing, ranking, deduplication, and evidence requirements. It stays preview-only and does not add retrieval execution, web requests, scraping, connectors, source import, local file indexing, BM25/vector index, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 99.2 is a review and hardening guard for the scientific preview DTO stack. It keeps the preview-first boundary intact before Local Literature Index work and does not change retrieval execution or runtime behavior.
Phase 100.0 adds a backend-only Local Literature Index Preview. It composes the Scholar Chat Scientific Search Plan Preview and only plans later local corpus manifest, metadata requirements, extraction/chunking/index artifacts, and retrieval readiness. It stays preview-only and does not add file reading, PDF extraction, OCR, chunking, embedding generation, index creation, retrieval execution, web requests, scraping, connectors, source import, model loading, inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 100.1 adds a backend-only Course Literature Registry Preview. It composes the Local Literature Index Preview with mode forced to course and only plans course identity, course-material kinds, local course-source alignment, curriculum metadata requirements, and learning-path alignment. It stays preview-only and does not add file reading, PDF extraction, OCR, chunking, embedding generation, index creation, retrieval execution, web requests, scraping, connectors, source import, model loading, inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 100.2 is a backend-only scientific preview command-surface review. It hardens the existing Tauri wiring and source-string tests for the scientific preview DTO stack without changing behavior, and it keeps the preview-only boundary intact without adding file reading, PDF extraction, OCR, chunking, embedding generation, index creation, retrieval execution, web requests, scraping, connectors, source import, model loading, inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 101.0 adds a backend-only OpenAlex / Crossref Metadata Connector Preview. It composes the Scientific Search Plan Preview and only plans later metadata-source selection, query shaping, DOI and access filters, result-shape mapping, deduplication, attribution, compliance, and downstream Evidence Pack alignment. It stays preview-only and does not add web requests, HTTP client use, API key or environment reads, scraping, connector calls, source import, metadata writes, retrieval execution, local file indexing, model loading, inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 101.1 is a backend-only Metadata Connector DTO Review / Guard Hardening pass. It is test/docs hardening only, verifies DTO declarations, serde enum values, command-surface wiring, query-plan invariants, compliance/attribution invariants, boundary flags, deterministic/path-free output, and forbidden-call guards, and it does not add new commands, behavior changes, frontend changes, web requests, HTTP client use, API key or environment reads, scraping, connector calls, source import, metadata writes, retrieval execution, local file indexing, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 102.0 is a backend-only Psychology Source Connector Preview. It composes the metadata connector preview and only plans later psychology source-family selection, methodology routing, population routing, topic-area routing, evidence requirements, compliance, and downstream Evidence Pack alignment. It stays preview-only and does not add connector calls, web requests, source import, file reading, PDF extraction, OCR, chunking, embedding generation, index creation, retrieval execution, model loading, runtime inference, LLM calls, answer generation, Evidence Pack creation, artifact writes, persistence, registry status changes, or audit writes.
Phase 102.1 is a backend-only Psychology Source Connector Review / Guard Hardening pass. It keeps the Phase 102.0 psychology source-family routing boundary stable before Scientific Evidence Pack Preview and does not change retrieval execution or runtime behavior.
Phase 103.0 adds a backend-only Scientific Evidence Pack Preview. It composes the Psychology Source Connector Preview and only plans local evidence context, metadata evidence, psychology source-family evidence, claim coverage, citation attribution, quality signals, compliance, and downstream answer-readiness alignment. It stays preview-only and does not add Evidence Pack creation, retrieval execution, web requests, HTTP client use, API key or environment reads, scraping, authenticated library access, paywall bypass, connector calls, source import, metadata writes, file reads, PDF extraction, OCR, chunking, embedding generation, index creation, model loading, runtime inference, LLM calls, answer generation, artifact writes, persistence, registry status changes, or audit writes.
Phase 103.1 is a backend-only Scientific Evidence Pack DTO Review / Guard Hardening pass. It is test/docs hardening only, verifies DTO declarations, serde enum values, command-surface wiring, composition boundary, status/strategy matrix, per-item no-op flags, planned step order, evidence/citation/compliance invariants, deterministic/path-free output, and forbidden-call guards, and it does not add new commands, behavior changes, frontend changes, Evidence Pack creation, retrieval execution, web requests, HTTP client use, API key or environment reads, scraping, authenticated library access, paywall bypass, connector calls, source import, metadata writes, file reads, model loading, runtime inference, LLM calls, answer generation, artifact writes, persistence, registry status changes, or audit writes.
Phase 104.0 adds a backend-only Scientific Paper Mode Literature Review Preview. It composes the Scientific Evidence Pack Preview and only plans later literature-review structure, research-question planning, search strategy planning, evidence-map planning, review sections, claim synthesis, citation planning, quality review, compliance, and downstream paper-generation alignment. It stays preview-only and does not add Literature Review creation, Evidence Pack creation, retrieval execution, web requests, HTTP client use, API key or environment reads, scraping, authenticated library access, paywall bypass, connector implementation or connector calls, source import, metadata writes, file reading, PDF extraction, OCR, chunking, embedding generation, index creation, model loading, runtime inference, LLM calls, answer generation, citation emission, artifact writes, persistence, registry status changes, or audit writes.
Phase 104.1 is a backend-only Scientific Paper Mode Literature Review Preview Guard Hardening pass. It is test/docs hardening only, verifies DTO declarations, serde enum values, command-surface wiring, composition boundary, status/strategy matrix, planned step order, review-section and claim/citation/compliance invariants, deterministic/path-free output, and forbidden-call guards, and it does not add new commands, behavior changes, frontend changes, Literature Review creation, Evidence Pack creation, retrieval execution, web requests, HTTP client use, API key or environment reads, scraping, authenticated library access, paywall bypass, connector implementation or connector calls, source import, metadata writes, file reading, PDF extraction, OCR, chunking, embedding generation, index creation, model loading, runtime inference, LLM calls, answer generation, citation emission, artifact writes, persistence, registry status changes, or audit writes.
Phase 105.0 is a backend-only Scientific Metadata Connector Execution Boundary Preview. It composes the Scientific Evidence Pack Preview and only plans later provider selection, provider request planning, metadata connector alignment, network and terms gates, metadata write gating, safety boundary, and downstream Evidence Pack / Literature Review alignment. It is dry-run and disabled by default, and it does not add real OpenAlex, Crossref, PubMed, ERIC, APA PsycNet, or PsycINFO calls, web requests, HTTP client use, API key or environment reads, scraping, authenticated library access, paywall bypass, connector implementation or connector calls, source import, metadata writes, file reads, PDF extraction, OCR, chunking, embedding generation, index creation, retrieval execution, Evidence Pack creation, Literature Review creation, model loading, runtime inference, LLM calls, answer generation, citation emission, artifact writes, persistence, registry status changes, or audit writes.
Phase 105.1 is a backend-only Scientific Metadata Execution Boundary Guard Hardening pass. It is test/docs hardening only, verifies DTO declarations, serde values, provider override normalization, command-surface wiring, deterministic/path-free output, and forbidden-call guards, and it does not add new commands, behavior changes, frontend changes, web requests, HTTP client use, API key or environment reads, scraping, authenticated library access, paywall bypass, connector implementation or connector calls, source import, metadata writes, file reads, PDF extraction, OCR, chunking, embedding generation, index creation, retrieval execution, Evidence Pack creation, Literature Review creation, model loading, runtime inference, LLM calls, answer generation, citation emission, artifact writes, persistence, registry status changes, or audit writes.
Phase 106.0 is a backend-only Scientific Metadata Provider Config Preview. It composes the Scientific Metadata Execution Boundary Preview and only plans later provider config, access, terms, rate-limit, attribution, safety, and downstream alignment boundaries. It remains preview-only and dry-run / disabled-by-default. It does not add real OpenAlex, Crossref, PubMed, ERIC, APA PsycNet, or PsycINFO calls; web requests; HTTP client use; API key or environment reads; scraping; authenticated library access; paywall bypass; connector implementation or connector calls; source import; metadata writes; file reads; PDF extraction; OCR; chunking; embedding generation; index creation; retrieval execution; Evidence Pack creation; Literature Review creation; model loading; runtime inference; LLM calls; answer generation; citation emission; artifact writes; persistence; registry status changes; or audit writes.
Phase 106.1 is a backend-only guard-hardening pass for the Scientific Metadata Provider Config Preview. It stays test/docs hardening only, keeps provider config dry-run and disabled by default, and does not change runtime or execution behavior.
Phase 107.0 is a backend-only Scientific Metadata Query Plan Preview. It composes the Scientific Metadata Provider Config Preview and only plans later query templates, filters, result fields, provider-family partitioning, rate-limit notes, attribution notes, safety boundaries, and downstream metadata connector / Evidence Pack / Literature Review alignment. It separates public metadata providers from APA PsycNet / PsycINFO institutional boundary. It remains preview-only and dry-run / disabled-by-default. It does not add real OpenAlex, Crossref, PubMed, ERIC, APA PsycNet, or PsycINFO calls; URL building; web requests; HTTP client use; API key or environment reads; scraping; authenticated library access; paywall bypass; connector implementation or connector calls; source import; metadata writes; file reads; PDF extraction; OCR; chunking; embedding generation; index creation; retrieval execution; Evidence Pack creation; Literature Review creation; model loading; runtime inference; LLM calls; answer generation; citation emission; artifact writes; persistence; registry status changes; or audit writes.
Phase 107.1 is a backend-only Scientific Metadata Query Plan Guard Hardening pass. It is test/docs hardening only, verifies DTO declarations, serde enum values, command-surface wiring, composition boundary, request defaults, provider override normalization, provider-family partitioning invariants, query template / filter / result-field invariants, safety boundary, planned step order, deterministic/path-free output, and forbidden-call guards, and it does not add new commands, behavior changes, frontend changes, or real OpenAlex, Crossref, PubMed, ERIC, APA PsycNet, or PsycINFO calls; URL building; web requests; HTTP client use; API key or environment reads; scraping; authenticated library access; paywall bypass; connector implementation or connector calls; source import; metadata writes; file reads; PDF extraction; OCR; chunking; embedding generation; index creation; retrieval execution; Evidence Pack creation; Literature Review creation; model loading; runtime inference; LLM calls; answer generation; citation emission; artifact writes; persistence; registry status changes; or audit writes.
Phase 108.0 is a backend-only Scientific Metadata Provider Request Preview. It composes the Scientific Metadata Query Plan Preview and only plans later provider request templates, methods, parameters, headers, bodies, provider-family boundaries, rate-limit notes, attribution notes, safety boundaries, and downstream connector / Evidence Pack / Literature Review alignment. It remains preview-only and dry-run / disabled-by-default. It does not add real OpenAlex, Crossref, PubMed, ERIC, APA PsycNet, or PsycINFO calls; web requests; HTTP client use; API key or environment reads; scraping; authenticated library access; paywall bypass; connector implementation or connector calls; source import; metadata writes; file reads; PDF extraction; OCR; chunking; embedding generation; index creation; retrieval execution; Evidence Pack creation; Literature Review creation; model loading; runtime inference; LLM calls; answer generation; citation emission; artifact writes; persistence; registry status changes; or audit writes.
