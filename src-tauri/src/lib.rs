mod audit;
mod corpus_authority;
mod corpus_paths;
mod chunking;
mod extraction;
mod evidence;
mod answer_draft;
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
use grounded_answer::{GroundedAnswer, GroundedAnswerService};
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
    GroundedAnswerService::new(root)
        .build_grounded_answer(&source_id, &answer_draft_id)
        .map_err(to_user_error)
}

#[tauri::command]
fn get_grounded_answer(root: String, source_id: String, grounded_answer_id: String) -> Result<GroundedAnswer, String> {
    GroundedAnswerService::new(root)
        .read_grounded_answer(&source_id, &grounded_answer_id)
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
            get_grounded_answer
        ])
        .run(tauri::generate_context!())
        .expect("error while running AEGIS Scholar");
}
