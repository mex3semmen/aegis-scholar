mod audit;
mod corpus_authority;
mod corpus_paths;
mod chunking;
mod extraction;
mod evidence;
mod answer_draft;
mod final_answer;
mod grounded_answer;
mod errors;
mod local_runtime;
mod locators;
mod retrieval;
mod scholar_chat;
mod source_metadata;
mod source_registry;

use corpus_authority::CorpusAuthority;
use chunking::{ChunkingReport, ChunkingService};
use extraction::{ExtractionReport, ExtractionService};
use errors::to_user_error;
use answer_draft::{AnswerDraft, AnswerDraftService};
use evidence::{EvidencePack, EvidencePackMetadata, EvidenceService};
use final_answer::{build_final_answer as build_final_answer_impl, export_answer_artifacts as export_answer_artifacts_impl, get_answer_artifact_export_manifest as get_answer_artifact_export_manifest_impl, get_answer_artifact_health as get_answer_artifact_health_impl, inspect_answer_artifact_export_bundle as inspect_answer_artifact_export_bundle_impl, list_answer_artifact_issues as list_answer_artifact_issues_impl, get_answer_artifact_overview as get_answer_artifact_overview_impl, list_answer_artifact_sources as list_answer_artifact_sources_impl, list_final_answers as list_final_answers_impl, read_final_answer as read_final_answer_impl, AnswerArtifactExportBundleInspection, AnswerArtifactExportManifest, AnswerArtifactExportResult, AnswerArtifactHealth, AnswerArtifactIssue, AnswerArtifactOverview, AnswerArtifactSourceMetadata, FinalAnswer, FinalAnswerMetadata};
use grounded_answer::{build_grounded_answer as build_grounded_answer_impl, read_grounded_answer as read_grounded_answer_impl, GroundedAnswer};
use local_runtime::{
    preview_local_model_runtime_health as preview_local_model_runtime_health_impl,
    preview_local_runtime_invocation_plan as preview_local_runtime_invocation_plan_impl,
    preview_llama_runtime_adapter_contract as preview_llama_runtime_adapter_contract_impl,
    preview_llama_runtime_validation as preview_llama_runtime_validation_impl,
    preview_llama_runtime_probe_readiness as preview_llama_runtime_probe_readiness_impl,
    run_llama_runtime_version_probe as run_llama_runtime_version_probe_impl,
    preview_llama_runtime_capability as preview_llama_runtime_capability_impl,
    preview_llama_runtime_smoke_readiness as preview_llama_runtime_smoke_readiness_impl,
    preview_llama_runtime_smoke_execution_plan as preview_llama_runtime_smoke_execution_plan_impl,
    run_llama_runtime_smoke_diagnostic as run_llama_runtime_smoke_diagnostic_impl,
    probe_local_runtime_version as probe_local_runtime_version_impl,
    LocalModelRuntimeConfig,
    LocalModelRuntimeHealthPreview,
    LocalRuntimeAdapterContractPreview,
    LocalRuntimeAdapterContractPreviewRequest,
    LocalRuntimeInvocationPlanPreview,
    LocalRuntimeInvocationPlanRequest,
    LocalRuntimeProbeRequest,
    LocalRuntimeProbeResult,
    LocalRuntimeProbeReadinessPreview,
    LocalRuntimeProbeReadinessPreviewRequest,
    LocalRuntimeVersionProbePreview,
    LocalRuntimeVersionProbePreviewRequest,
    LocalRuntimeCapabilityPreview,
    LocalRuntimeCapabilityPreviewRequest,
    LocalRuntimeSmokeDiagnosticPreview,
    LocalRuntimeSmokeDiagnosticRequest,
    LocalRuntimeSmokeExecutionPlanPreview,
    LocalRuntimeSmokeExecutionPlanPreviewRequest,
    LocalRuntimeSmokeReadinessPreview,
    LocalRuntimeSmokeReadinessPreviewRequest,
    LocalRuntimeValidationPreview,
    LocalRuntimeValidationPreviewRequest,
};
use retrieval::{RetrievalIndex, RetrievalResponse, RetrievalService};
use scholar_chat::{
    preview_scholar_chat_answer_readiness as preview_scholar_chat_answer_readiness_impl,
    preview_scholar_chat_draft_inference as preview_scholar_chat_draft_inference_impl,
    preview_scholar_chat_draft_grounding_inspection as preview_scholar_chat_draft_grounding_inspection_impl,
    preview_scholar_chat_grounded_answer_build_preflight as preview_scholar_chat_grounded_answer_build_preflight_impl,
    preview_scholar_chat_grounded_answer_build_plan as preview_scholar_chat_grounded_answer_build_plan_impl,
    preview_scholar_chat_grounded_answer_candidate as preview_scholar_chat_grounded_answer_candidate_impl,
    preview_scholar_chat_grounded_answer_build_intent as preview_scholar_chat_grounded_answer_build_intent_impl,
    preview_scholar_chat_grounded_answer_build_request as preview_scholar_chat_grounded_answer_build_request_impl,
    preview_scholar_chat_grounded_answer_execution_readiness as preview_scholar_chat_grounded_answer_execution_readiness_impl,
    preview_scholar_chat_grounded_answer_execution_plan as preview_scholar_chat_grounded_answer_execution_plan_impl,
    preview_scholar_chat_runtime_diagnostic_bridge as preview_scholar_chat_runtime_diagnostic_bridge_impl,
    preview_scholar_chat_runtime_diagnostic_result as preview_scholar_chat_runtime_diagnostic_result_impl,
    preview_scholar_chat_runtime_answer_pipeline_gate as preview_scholar_chat_runtime_answer_pipeline_gate_impl,
    preview_scholar_chat_scientific_discipline_registry as preview_scholar_chat_scientific_discipline_registry_impl,
    preview_scholar_chat_scientific_source_registry as preview_scholar_chat_scientific_source_registry_impl,
    preview_scholar_chat_scientific_query_understanding as preview_scholar_chat_scientific_query_understanding_impl,
    preview_scholar_chat_scientific_search_plan as preview_scholar_chat_scientific_search_plan_impl,
    preview_scholar_chat_local_literature_index as preview_scholar_chat_local_literature_index_impl,
    preview_scholar_chat_course_literature_registry as preview_scholar_chat_course_literature_registry_impl,
    preview_scholar_chat_metadata_connector_plan as preview_scholar_chat_metadata_connector_plan_impl,
    preview_scholar_chat_psychology_source_connector_plan as preview_scholar_chat_psychology_source_connector_plan_impl,
    preview_scholar_chat_scientific_evidence_pack_plan as preview_scholar_chat_scientific_evidence_pack_plan_impl,
    preview_scholar_chat_scientific_paper_literature_review_plan as preview_scholar_chat_scientific_paper_literature_review_plan_impl,
    preview_scholar_chat_scientific_metadata_execution_boundary as preview_scholar_chat_scientific_metadata_execution_boundary_impl,
    preview_scholar_chat_scientific_metadata_provider_config as preview_scholar_chat_scientific_metadata_provider_config_impl,
    preview_scholar_chat_scientific_metadata_query_plan as preview_scholar_chat_scientific_metadata_query_plan_impl,
    preview_scholar_chat_scientific_metadata_provider_request as preview_scholar_chat_scientific_metadata_provider_request_impl,
    preview_scholar_chat_openalex_metadata_cache_write_gate as preview_scholar_chat_openalex_metadata_cache_write_gate_impl,
    preview_scholar_chat_openalex_evidence_candidate_conversion as preview_scholar_chat_openalex_evidence_candidate_conversion_impl,
    ScholarChatOpenAlexEvidenceCandidateConversionPreview,
    ScholarChatOpenAlexEvidenceCandidateConversionPreviewRequest,
    preview_scholar_chat_evidence_pack_assembly_plan as preview_scholar_chat_evidence_pack_assembly_plan_impl,
    ScholarChatEvidencePackAssemblyPlanPreview,
    ScholarChatEvidencePackAssemblyPlanPreviewRequest,
    run_scholar_chat_openalex_metadata_execution_slice as run_scholar_chat_openalex_metadata_execution_slice_impl,
    preview_scholar_chat_grounded_answer_write_eligibility as preview_scholar_chat_grounded_answer_write_eligibility_impl,
    preview_scholar_chat_request as preview_scholar_chat_request_impl,
    preview_scholar_chat_evidence_plan as preview_scholar_chat_evidence_plan_impl,
    preview_scholar_chat_prompt_pack as preview_scholar_chat_prompt_pack_impl,
    preview_scholar_chat_retrieval as preview_scholar_chat_retrieval_impl,
    preview_scholar_chat_grounded_draft_readiness as preview_scholar_chat_grounded_draft_readiness_impl,
    ScholarChatAnswerReadinessPreview,
    ScholarChatAnswerReadinessRequest,
    ScholarChatDraftInferencePreview,
    ScholarChatDraftInferenceRequest,
    ScholarChatDraftGroundingInspectionPreview,
    ScholarChatDraftGroundingInspectionRequest,
    ScholarChatGroundedAnswerBuildPlanPreview,
    ScholarChatGroundedAnswerCandidatePreview,
    ScholarChatGroundedAnswerBuildIntentPreview,
    ScholarChatGroundedAnswerBuildIntentRequest,
    ScholarChatGroundedAnswerBuildRequestPreview,
    ScholarChatGroundedAnswerBuildRequestPreviewRequest,
    ScholarChatGroundedAnswerWriteEligibilityPreview,
    ScholarChatGroundedDraftReadinessPreview,
    ScholarChatRuntimeDiagnosticBridgePreview,
    ScholarChatRuntimeDiagnosticBridgePreviewRequest,
    ScholarChatRuntimeDiagnosticResultPreview,
    ScholarChatRuntimeDiagnosticResultPreviewRequest,
    ScholarChatRuntimeAnswerPipelineGatePreview,
    ScholarChatRuntimeAnswerPipelineGatePreviewRequest,
    ScholarChatScientificDisciplineRegistryPreview,
    ScholarChatScientificDisciplineRegistryPreviewRequest,
    ScholarChatScientificSourceRegistryPreview,
    ScholarChatScientificSourceRegistryPreviewRequest,
    ScholarChatScientificQueryUnderstandingPreview,
    ScholarChatScientificQueryUnderstandingPreviewRequest,
    ScholarChatScientificSearchPlanPreview,
    ScholarChatScientificSearchPlanRequest,
    ScholarChatLocalLiteratureIndexPreview,
    ScholarChatLocalLiteratureIndexRequest,
    ScholarChatCourseLiteratureRegistryPreview,
    ScholarChatCourseLiteratureRegistryPreviewRequest,
    ScholarChatMetadataConnectorPlanPreview,
    ScholarChatMetadataConnectorPlanRequest,
    ScholarChatPsychologySourceConnectorPlanPreview,
    ScholarChatPsychologySourceConnectorPlanRequest,
    ScholarChatScientificEvidencePackPlanPreview,
    ScholarChatScientificEvidencePackPlanRequest,
    ScholarChatScientificPaperLiteratureReviewPlanPreview,
    ScholarChatScientificPaperLiteratureReviewPlanRequest,
    ScholarChatScientificMetadataExecutionBoundaryPreview,
    ScholarChatScientificMetadataExecutionBoundaryPreviewRequest,
    ScholarChatScientificMetadataProviderConfigPreview,
    ScholarChatScientificMetadataProviderConfigPreviewRequest,
    ScholarChatScientificMetadataQueryPlanPreview,
    ScholarChatScientificMetadataQueryPlanPreviewRequest,
    ScholarChatScientificMetadataProviderRequestPreview,
    ScholarChatScientificMetadataProviderRequestPreviewRequest,
    ScholarChatRequest,
    ScholarChatEvidencePlanResponse,
    ScholarChatPromptPackPreviewResponse,
    ScholarChatResponse,
    ScholarChatRetrievalPreviewResponse,
};
use source_metadata::{CorpusStatus, SourceMetadataInput, SourceMetadataPatch, SourceRecord};

#[tauri::command]
fn register_source(
    root: String,
    path: String,
    metadata: SourceMetadataInput,
) -> Result<SourceRecord, String> {
    CorpusAuthority::new(root)
        .register_source(path, metadata)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_source(root: String, source_id: String) -> Result<SourceRecord, String> {
    CorpusAuthority::new(root)
        .get_source(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn list_sources(root: String) -> Result<Vec<SourceRecord>, String> {
    CorpusAuthority::new(root)
        .list_sources()
        .map_err(to_user_error)
}

#[tauri::command]
fn update_source_metadata(
    root: String,
    source_id: String,
    metadata_patch: SourceMetadataPatch,
) -> Result<SourceRecord, String> {
    CorpusAuthority::new(root)
        .update_source_metadata(&source_id, metadata_patch)
        .map_err(to_user_error)
}

#[tauri::command]
fn remove_source(root: String, source_id: String) -> Result<SourceRecord, String> {
    CorpusAuthority::new(root)
        .remove_source(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_corpus_status(root: String) -> Result<CorpusStatus, String> {
    CorpusAuthority::new(root)
        .get_corpus_status()
        .map_err(to_user_error)
}

#[tauri::command]
fn extract_source(root: String, source_id: String) -> Result<ExtractionReport, String> {
    ExtractionService::new(root)
        .extract_source(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_extraction_report(root: String, source_id: String) -> Result<ExtractionReport, String> {
    ExtractionService::new(root)
        .read_extraction_report(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn chunk_source(root: String, source_id: String) -> Result<ChunkingReport, String> {
    ChunkingService::new(root)
        .chunk_source(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_chunking_report(root: String, source_id: String) -> Result<ChunkingReport, String> {
    ChunkingService::new(root)
        .read_chunking_report(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn build_retrieval_index(root: String, source_id: String) -> Result<RetrievalIndex, String> {
    RetrievalService::new(root)
        .build_index(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_retrieval_index(root: String, source_id: String) -> Result<RetrievalIndex, String> {
    RetrievalService::new(root)
        .read_index(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn search_source(root: String, source_id: String, query: String, max_results: usize) -> Result<RetrievalResponse, String> {
    RetrievalService::new(root)
        .search_source(&source_id, &query, max_results)
        .map_err(to_user_error)
}

#[tauri::command]
fn build_evidence_pack(root: String, source_id: String, query: String, max_results: usize) -> Result<EvidencePack, String> {
    EvidenceService::new(root)
        .build_evidence_pack(&source_id, &query, max_results)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_evidence_pack(root: String, source_id: String, evidence_pack_id: String) -> Result<EvidencePack, String> {
    EvidenceService::new(root)
        .read_evidence_pack(&source_id, &evidence_pack_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn list_evidence_packs(root: String, source_id: String) -> Result<Vec<EvidencePackMetadata>, String> {
    EvidenceService::new(root)
        .list_evidence_packs(&source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn build_answer_draft(root: String, source_id: String, evidence_pack_id: String) -> Result<AnswerDraft, String> {
    AnswerDraftService::new(root)
        .build_answer_draft(&source_id, &evidence_pack_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_answer_draft(root: String, source_id: String, answer_draft_id: String) -> Result<AnswerDraft, String> {
    AnswerDraftService::new(root)
        .read_answer_draft(&source_id, &answer_draft_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn build_grounded_answer(root: String, source_id: String, answer_draft_id: String) -> Result<GroundedAnswer, String> {
    build_grounded_answer_impl(root, &source_id, &answer_draft_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_grounded_answer(root: String, source_id: String, grounded_answer_id: String) -> Result<GroundedAnswer, String> {
    read_grounded_answer_impl(root, &source_id, &grounded_answer_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn build_final_answer(root: String, source_id: String, grounded_answer_id: String) -> Result<FinalAnswer, String> {
    build_final_answer_impl(root, &source_id, &grounded_answer_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_final_answer(root: String, source_id: String, final_answer_id: String) -> Result<FinalAnswer, String> {
    read_final_answer_impl(root, &source_id, &final_answer_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn list_final_answers(root: String, source_id: String) -> Result<Vec<FinalAnswerMetadata>, String> {
    list_final_answers_impl(root, &source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_answer_artifact_overview(root: String, source_id: String) -> Result<AnswerArtifactOverview, String> {
    get_answer_artifact_overview_impl(root, &source_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn list_answer_artifact_sources(root: String) -> Result<Vec<AnswerArtifactSourceMetadata>, String> {
    list_answer_artifact_sources_impl(root)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_answer_artifact_health(root: String) -> Result<AnswerArtifactHealth, String> {
    get_answer_artifact_health_impl(root)
        .map_err(to_user_error)
}

#[tauri::command]
fn list_answer_artifact_issues(root: String) -> Result<Vec<AnswerArtifactIssue>, String> {
    list_answer_artifact_issues_impl(root)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_answer_artifact_export_manifest(root: String) -> Result<AnswerArtifactExportManifest, String> {
    get_answer_artifact_export_manifest_impl(root)
        .map_err(to_user_error)
}

#[tauri::command]
fn inspect_answer_artifact_export_bundle(export_root: String) -> Result<AnswerArtifactExportBundleInspection, String> {
    inspect_answer_artifact_export_bundle_impl(export_root)
        .map_err(to_user_error)
}

#[tauri::command]
fn export_answer_artifacts(root: String, export_root: String) -> Result<AnswerArtifactExportResult, String> {
    export_answer_artifacts_impl(root, export_root)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_request(root: String, request: ScholarChatRequest) -> Result<ScholarChatResponse, String> {
    preview_scholar_chat_request_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_answer_readiness(
    root: String,
    request: ScholarChatAnswerReadinessRequest,
) -> Result<ScholarChatAnswerReadinessPreview, String> {
    preview_scholar_chat_answer_readiness_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_draft_inference(
    root: String,
    request: ScholarChatDraftInferenceRequest,
) -> Result<ScholarChatDraftInferencePreview, String> {
    preview_scholar_chat_draft_inference_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_draft_grounding_inspection(
    root: String,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> Result<ScholarChatDraftGroundingInspectionPreview, String> {
    preview_scholar_chat_draft_grounding_inspection_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_draft_readiness(
    root: String,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> Result<ScholarChatGroundedDraftReadinessPreview, String> {
    preview_scholar_chat_grounded_draft_readiness_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn run_llama_runtime_version_probe(
    root: String,
    request: LocalRuntimeVersionProbePreviewRequest,
) -> Result<LocalRuntimeVersionProbePreview, String> {
    run_llama_runtime_version_probe_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_llama_runtime_capability(
    root: String,
    request: LocalRuntimeCapabilityPreviewRequest,
) -> Result<LocalRuntimeCapabilityPreview, String> {
    preview_llama_runtime_capability_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_llama_runtime_smoke_readiness(
    root: String,
    request: LocalRuntimeSmokeReadinessPreviewRequest,
) -> Result<LocalRuntimeSmokeReadinessPreview, String> {
    preview_llama_runtime_smoke_readiness_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_llama_runtime_smoke_execution_plan(
    root: String,
    request: LocalRuntimeSmokeExecutionPlanPreviewRequest,
) -> Result<LocalRuntimeSmokeExecutionPlanPreview, String> {
    preview_llama_runtime_smoke_execution_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_build_preflight(
    root: String,
    request: scholar_chat::ScholarChatGroundedAnswerBuildPreflightPreviewRequest,
) -> Result<scholar_chat::ScholarChatGroundedAnswerBuildPreflightPreview, String> {
    preview_scholar_chat_grounded_answer_build_preflight_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_execution_readiness(
    root: String,
    request: scholar_chat::ScholarChatGroundedAnswerExecutionReadinessPreviewRequest,
) -> Result<scholar_chat::ScholarChatGroundedAnswerExecutionReadinessPreview, String> {
    preview_scholar_chat_grounded_answer_execution_readiness_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_execution_plan(
    root: String,
    request: scholar_chat::ScholarChatGroundedAnswerExecutionPlanPreviewRequest,
) -> Result<scholar_chat::ScholarChatGroundedAnswerExecutionPlanPreview, String> {
    preview_scholar_chat_grounded_answer_execution_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_build_plan(
    root: String,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> Result<ScholarChatGroundedAnswerBuildPlanPreview, String> {
    preview_scholar_chat_grounded_answer_build_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_candidate(
    root: String,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> Result<ScholarChatGroundedAnswerCandidatePreview, String> {
    preview_scholar_chat_grounded_answer_candidate_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_write_eligibility(
    root: String,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> Result<ScholarChatGroundedAnswerWriteEligibilityPreview, String> {
    preview_scholar_chat_grounded_answer_write_eligibility_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_build_intent(
    root: String,
    request: ScholarChatGroundedAnswerBuildIntentRequest,
) -> Result<ScholarChatGroundedAnswerBuildIntentPreview, String> {
    preview_scholar_chat_grounded_answer_build_intent_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_grounded_answer_build_request(
    root: String,
    request: ScholarChatGroundedAnswerBuildRequestPreviewRequest,
) -> Result<ScholarChatGroundedAnswerBuildRequestPreview, String> {
    preview_scholar_chat_grounded_answer_build_request_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_retrieval(root: String, request: ScholarChatRequest) -> Result<ScholarChatRetrievalPreviewResponse, String> {
    preview_scholar_chat_retrieval_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_evidence_plan(root: String, request: ScholarChatRequest) -> Result<ScholarChatEvidencePlanResponse, String> {
    preview_scholar_chat_evidence_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_prompt_pack(root: String, request: ScholarChatRequest) -> Result<ScholarChatPromptPackPreviewResponse, String> {
    preview_scholar_chat_prompt_pack_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_local_model_runtime_health(root: String, config: LocalModelRuntimeConfig) -> Result<LocalModelRuntimeHealthPreview, String> {
    preview_local_model_runtime_health_impl(root, config)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_local_runtime_invocation_plan(
    root: String,
    request: LocalRuntimeInvocationPlanRequest,
) -> Result<LocalRuntimeInvocationPlanPreview, String> {
    preview_local_runtime_invocation_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_llama_runtime_adapter_contract(
    root: String,
    request: LocalRuntimeAdapterContractPreviewRequest,
) -> Result<LocalRuntimeAdapterContractPreview, String> {
    preview_llama_runtime_adapter_contract_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_llama_runtime_validation(
    root: String,
    request: LocalRuntimeValidationPreviewRequest,
) -> Result<LocalRuntimeValidationPreview, String> {
    preview_llama_runtime_validation_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_llama_runtime_probe_readiness(
    root: String,
    request: LocalRuntimeProbeReadinessPreviewRequest,
) -> Result<LocalRuntimeProbeReadinessPreview, String> {
    preview_llama_runtime_probe_readiness_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn probe_local_runtime_version(root: String, request: LocalRuntimeProbeRequest) -> Result<LocalRuntimeProbeResult, String> {
    probe_local_runtime_version_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn run_llama_runtime_smoke_diagnostic(
    root: String,
    request: LocalRuntimeSmokeDiagnosticRequest,
) -> Result<LocalRuntimeSmokeDiagnosticPreview, String> {
    run_llama_runtime_smoke_diagnostic_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_runtime_diagnostic_bridge(
    root: String,
    request: ScholarChatRuntimeDiagnosticBridgePreviewRequest,
) -> Result<ScholarChatRuntimeDiagnosticBridgePreview, String> {
    preview_scholar_chat_runtime_diagnostic_bridge_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_runtime_diagnostic_result(
    root: String,
    request: ScholarChatRuntimeDiagnosticResultPreviewRequest,
) -> Result<ScholarChatRuntimeDiagnosticResultPreview, String> {
    preview_scholar_chat_runtime_diagnostic_result_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_runtime_answer_pipeline_gate(
    root: String,
    request: ScholarChatRuntimeAnswerPipelineGatePreviewRequest,
) -> Result<ScholarChatRuntimeAnswerPipelineGatePreview, String> {
    preview_scholar_chat_runtime_answer_pipeline_gate_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_discipline_registry(
    root: String,
    request: ScholarChatScientificDisciplineRegistryPreviewRequest,
) -> Result<ScholarChatScientificDisciplineRegistryPreview, String> {
    preview_scholar_chat_scientific_discipline_registry_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_source_registry(
    root: String,
    request: ScholarChatScientificSourceRegistryPreviewRequest,
) -> Result<ScholarChatScientificSourceRegistryPreview, String> {
    preview_scholar_chat_scientific_source_registry_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_query_understanding(
    root: String,
    request: ScholarChatScientificQueryUnderstandingPreviewRequest,
) -> Result<ScholarChatScientificQueryUnderstandingPreview, String> {
    preview_scholar_chat_scientific_query_understanding_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_search_plan(
    root: String,
    request: ScholarChatScientificSearchPlanRequest,
) -> Result<ScholarChatScientificSearchPlanPreview, String> {
    preview_scholar_chat_scientific_search_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_local_literature_index(
    root: String,
    request: ScholarChatLocalLiteratureIndexRequest,
) -> Result<ScholarChatLocalLiteratureIndexPreview, String> {
    preview_scholar_chat_local_literature_index_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_course_literature_registry(
    root: String,
    request: ScholarChatCourseLiteratureRegistryPreviewRequest,
) -> Result<ScholarChatCourseLiteratureRegistryPreview, String> {
    preview_scholar_chat_course_literature_registry_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_metadata_connector_plan(
    root: String,
    request: ScholarChatMetadataConnectorPlanRequest,
) -> Result<ScholarChatMetadataConnectorPlanPreview, String> {
    preview_scholar_chat_metadata_connector_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_psychology_source_connector_plan(
    root: String,
    request: ScholarChatPsychologySourceConnectorPlanRequest,
) -> Result<ScholarChatPsychologySourceConnectorPlanPreview, String> {
    preview_scholar_chat_psychology_source_connector_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_evidence_pack_plan(
    root: String,
    request: ScholarChatScientificEvidencePackPlanRequest,
) -> Result<ScholarChatScientificEvidencePackPlanPreview, String> {
    preview_scholar_chat_scientific_evidence_pack_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_paper_literature_review_plan(
    root: String,
    request: ScholarChatScientificPaperLiteratureReviewPlanRequest,
) -> Result<ScholarChatScientificPaperLiteratureReviewPlanPreview, String> {
    preview_scholar_chat_scientific_paper_literature_review_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_metadata_execution_boundary(
    root: String,
    request: ScholarChatScientificMetadataExecutionBoundaryPreviewRequest,
) -> Result<ScholarChatScientificMetadataExecutionBoundaryPreview, String> {
    preview_scholar_chat_scientific_metadata_execution_boundary_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_metadata_provider_config(
    root: String,
    request: ScholarChatScientificMetadataProviderConfigPreviewRequest,
) -> Result<ScholarChatScientificMetadataProviderConfigPreview, String> {
    preview_scholar_chat_scientific_metadata_provider_config_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_metadata_query_plan(
    root: String,
    request: ScholarChatScientificMetadataQueryPlanPreviewRequest,
) -> Result<ScholarChatScientificMetadataQueryPlanPreview, String> {
    preview_scholar_chat_scientific_metadata_query_plan_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_scientific_metadata_provider_request(
    root: String,
    request: ScholarChatScientificMetadataProviderRequestPreviewRequest,
) -> Result<ScholarChatScientificMetadataProviderRequestPreview, String> {
    preview_scholar_chat_scientific_metadata_provider_request_impl(root, request)
        .map_err(to_user_error)
}

#[tauri::command]
fn run_scholar_chat_openalex_metadata_execution_slice(
    root: String,
    request: scholar_chat::ScholarChatOpenAlexMetadataExecutionSliceRequest,
) -> Result<scholar_chat::ScholarChatOpenAlexMetadataExecutionSliceResult, String> {
    run_scholar_chat_openalex_metadata_execution_slice_impl(root, request).map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_openalex_metadata_cache_write_gate(
    root: String,
    request: scholar_chat::ScholarChatOpenAlexMetadataCacheWriteGatePreviewRequest,
) -> Result<scholar_chat::ScholarChatOpenAlexMetadataCacheWriteGatePreview, String> {
    preview_scholar_chat_openalex_metadata_cache_write_gate_impl(root, request).map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_openalex_evidence_candidate_conversion(
    root: String,
    request: ScholarChatOpenAlexEvidenceCandidateConversionPreviewRequest,
) -> Result<ScholarChatOpenAlexEvidenceCandidateConversionPreview, String> {
    preview_scholar_chat_openalex_evidence_candidate_conversion_impl(root.into(), request)
        .map_err(to_user_error)
}

#[tauri::command]
fn preview_scholar_chat_evidence_pack_assembly_plan(
    root: String,
    request: ScholarChatEvidencePackAssemblyPlanPreviewRequest,
) -> Result<ScholarChatEvidencePackAssemblyPlanPreview, String> {
    preview_scholar_chat_evidence_pack_assembly_plan_impl(root, request)
        .map_err(to_user_error)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            register_source,
            get_source,
            list_sources,
            update_source_metadata,
            remove_source,
            get_corpus_status,
            extract_source,
            get_extraction_report,
            chunk_source,
            get_chunking_report,
            build_retrieval_index,
            get_retrieval_index,
            search_source,
            build_evidence_pack,
            get_evidence_pack,
            list_evidence_packs,
            build_answer_draft,
            get_answer_draft,
            build_grounded_answer,
            get_grounded_answer,
            build_final_answer,
            get_final_answer,
            list_final_answers,
            get_answer_artifact_overview,
            list_answer_artifact_sources,
            get_answer_artifact_health,
            list_answer_artifact_issues,
            get_answer_artifact_export_manifest,
            inspect_answer_artifact_export_bundle,
            export_answer_artifacts,
            preview_scholar_chat_request,
            preview_scholar_chat_answer_readiness,
            preview_scholar_chat_draft_inference,
            preview_scholar_chat_draft_grounding_inspection,
            preview_scholar_chat_grounded_draft_readiness,
            preview_scholar_chat_grounded_answer_build_preflight,
            preview_scholar_chat_grounded_answer_execution_readiness,
            preview_scholar_chat_grounded_answer_execution_plan,
            preview_scholar_chat_grounded_answer_build_plan,
            preview_scholar_chat_grounded_answer_candidate,
            preview_scholar_chat_grounded_answer_build_intent,
            preview_scholar_chat_grounded_answer_build_request,
            preview_scholar_chat_grounded_answer_write_eligibility,
            preview_scholar_chat_retrieval,
            preview_scholar_chat_evidence_plan,
            preview_scholar_chat_prompt_pack,
            preview_scholar_chat_scientific_discipline_registry,
            preview_scholar_chat_scientific_source_registry,
            preview_scholar_chat_scientific_query_understanding,
            preview_scholar_chat_scientific_search_plan,
            preview_scholar_chat_local_literature_index,
            preview_scholar_chat_course_literature_registry,
            preview_scholar_chat_metadata_connector_plan,
            preview_scholar_chat_psychology_source_connector_plan,
            preview_scholar_chat_scientific_evidence_pack_plan,
            preview_scholar_chat_scientific_paper_literature_review_plan,
            preview_scholar_chat_scientific_metadata_execution_boundary,
            preview_scholar_chat_scientific_metadata_provider_config,
            preview_scholar_chat_scientific_metadata_query_plan,
            preview_scholar_chat_scientific_metadata_provider_request,
            run_scholar_chat_openalex_metadata_execution_slice,
            preview_scholar_chat_openalex_metadata_cache_write_gate,
            preview_scholar_chat_openalex_evidence_candidate_conversion,
            preview_scholar_chat_evidence_pack_assembly_plan,
            preview_scholar_chat_runtime_diagnostic_bridge,
            preview_scholar_chat_runtime_diagnostic_result,
            preview_scholar_chat_runtime_answer_pipeline_gate,
            preview_local_model_runtime_health,
            preview_local_runtime_invocation_plan,
            preview_llama_runtime_adapter_contract,
            preview_llama_runtime_validation,
            preview_llama_runtime_probe_readiness,
            probe_local_runtime_version,
            preview_llama_runtime_capability,
            preview_llama_runtime_smoke_readiness,
            preview_llama_runtime_smoke_execution_plan,
            run_llama_runtime_version_probe,
            run_llama_runtime_smoke_diagnostic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AEGIS Scholar");
}
