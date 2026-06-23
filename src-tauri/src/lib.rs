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
mod locators;
mod retrieval;
mod source_metadata;
mod source_registry;

use corpus_authority::CorpusAuthority;
use chunking::{ChunkingReport, ChunkingService};
use extraction::{ExtractionReport, ExtractionService};
use errors::to_user_error;
use answer_draft::{AnswerDraft, AnswerDraftService};
use evidence::{EvidencePack, EvidenceService};
use final_answer::{build_final_answer as build_final_answer_impl, export_answer_artifacts as export_answer_artifacts_impl, get_answer_artifact_export_manifest as get_answer_artifact_export_manifest_impl, get_answer_artifact_health as get_answer_artifact_health_impl, list_answer_artifact_issues as list_answer_artifact_issues_impl, get_answer_artifact_overview as get_answer_artifact_overview_impl, list_answer_artifact_sources as list_answer_artifact_sources_impl, list_final_answers as list_final_answers_impl, read_final_answer as read_final_answer_impl, AnswerArtifactExportManifest, AnswerArtifactExportResult, AnswerArtifactHealth, AnswerArtifactIssue, AnswerArtifactOverview, AnswerArtifactSourceMetadata, FinalAnswer, FinalAnswerMetadata};
use grounded_answer::{build_grounded_answer as build_grounded_answer_impl, read_grounded_answer as read_grounded_answer_impl, GroundedAnswer};
use retrieval::{RetrievalIndex, RetrievalResponse, RetrievalService};
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
fn export_answer_artifacts(root: String, export_root: String) -> Result<AnswerArtifactExportResult, String> {
    export_answer_artifacts_impl(root, export_root)
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
            export_answer_artifacts
        ])
        .run(tauri::generate_context!())
        .expect("error while running AEGIS Scholar");
}
