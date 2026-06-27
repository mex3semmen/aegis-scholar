use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::source_registry::SourceRegistry;
use crate::retrieval::{RetrievalResponse, RetrievalService};
use crate::local_runtime::{
    preview_local_model_runtime_health,
    preview_local_runtime_invocation_plan,
    smoke_test_local_runtime_inference,
    LocalModelRuntimeConfig,
    LocalRuntimeAdapterContractStatus,
    LocalRuntimeAdapterKind,
    LocalModelRuntimeHealthStatus,
    LocalRuntimeCapabilityStatus,
    LocalRuntimeInvocationPlanRequest,
    LocalRuntimeInvocationPlanStatus,
    LocalRuntimeProbeReadinessStatus,
    LocalRuntimeSmokeInferenceRequest,
    LocalRuntimeSmokeInferenceStatus,
    LocalRuntimeProbeWarning,
    LocalRuntimeSmokeExecutionPlanPreviewRequest,
    LocalRuntimeSmokeExecutionPlanStatus,
    LocalRuntimeSmokeDiagnosticPreview,
    LocalRuntimeSmokeDiagnosticStatus,
    LocalRuntimeSmokeReadinessStatus,
    LocalRuntimeValidationStatus,
    LocalRuntimeVersionProbeStatus,
    preview_llama_runtime_smoke_execution_plan,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const SCHOLAR_CHAT_RETRIEVAL_PREVIEW_LIMIT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatMode {
    LectureLearning,
    ThesisWriting,
    LiteratureReview,
    Flashcards,
    StatisticsMethods,
    GeneralScholar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroundingPolicy {
    LocalOnly,
    LocalFirst,
    AllowMarkedModelKnowledge,
    ExternalAdaptersLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatStatus {
    PreviewOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatRequest {
    pub prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundingPlan {
    pub selected_source_count: usize,
    pub local_corpus_required: bool,
    pub retrieval_would_run: bool,
    pub evidence_pack_would_be_required: bool,
    pub model_knowledge_allowed: bool,
    pub external_adapters_available: bool,
    pub summary: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatResponse {
    pub status: ScholarChatStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub grounding_plan: ScholarChatGroundingPlan,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificDisciplineRegistryStatus {
    Blocked,
    ConceptMapped,
    UnknownConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificDisciplineScienceClass {
    CoreScience,
    MethodScience,
    AppliedDomain,
    CurriculumLayer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificDisciplineRegistryPreviewRequest {
    pub topic: String,
    pub mode: Option<String>,
    pub course_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificDisciplineRegistryPreview {
    pub status: ScholarChatScientificDisciplineRegistryStatus,
    pub normalized_topic: String,
    pub normalized_mode: String,
    pub recognized_concept: Option<String>,
    pub label: Option<String>,
    pub discipline_path: Vec<String>,
    pub science_class: Option<ScholarChatScientificDisciplineScienceClass>,
    pub parent_path: Vec<String>,
    pub related_methods: Vec<String>,
    pub appears_in: Vec<String>,
    pub preferred_sources: Vec<String>,
    pub curriculum_sources: Vec<String>,
    pub canonical_mappings: Vec<String>,
    pub planned_queries: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub registry_preview_only: bool,
    pub no_web_request: bool,
    pub no_scraping: bool,
    pub no_connector_call: bool,
    pub no_source_import: bool,
    pub no_local_file_indexing: bool,
    pub no_bm25_index: bool,
    pub no_vector_index: bool,
    pub no_model_loading: bool,
    pub no_runtime_inference: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_evidence_pack_created: bool,
    pub no_artifact_write: bool,
    pub no_persistence: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificSourceRegistryStatus {
    Blocked,
    SourcePlanReady,
    UnknownConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificSourceAccessClass {
    OpenMetadata,
    OpenFullText,
    UserProvidedLocalFile,
    UserAuthenticatedAccess,
    RestrictedNoIngest,
    CatalogOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificSourcePriority {
    Primary,
    Supporting,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSourceRegistryPreviewRequest {
    pub topic: String,
    pub mode: Option<String>,
    pub course_context: Option<String>,
    pub context_tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSourceRegistrySourceFamily {
    pub id: String,
    pub label: String,
    pub domain: String,
    pub access_class: ScholarChatScientificSourceAccessClass,
    pub priority: ScholarChatScientificSourcePriority,
    pub applies_when: String,
    pub active_for_current_context: bool,
    pub planned_use: String,
    pub query_roles: Vec<String>,
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSourceRegistrySourcePlan {
    pub source_family_count: usize,
    pub active_source_family_count: usize,
    pub conditional_source_family_count: usize,
    pub planned_metadata_query_count: usize,
    pub summary: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSourceRegistryPreview {
    pub status: ScholarChatScientificSourceRegistryStatus,
    pub normalized_topic: String,
    pub normalized_mode: String,
    pub normalized_context_tags: Vec<String>,
    pub discipline_status: ScholarChatScientificDisciplineRegistryStatus,
    pub recognized_concept: Option<String>,
    pub label: Option<String>,
    pub source_plan: ScholarChatScientificSourceRegistrySourcePlan,
    pub source_families: Vec<ScholarChatScientificSourceRegistrySourceFamily>,
    pub preferred_source_ids: Vec<String>,
    pub conditional_source_ids: Vec<String>,
    pub excluded_source_ids: Vec<String>,
    pub access_classes: Vec<ScholarChatScientificSourceAccessClass>,
    pub planned_metadata_queries: Vec<String>,
    pub ranking_hints: Vec<String>,
    pub deduplication_hints: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub source_registry_preview_only: bool,
    pub no_web_request: bool,
    pub no_scraping: bool,
    pub no_connector_call: bool,
    pub no_source_import: bool,
    pub no_local_file_indexing: bool,
    pub no_bm25_index: bool,
    pub no_vector_index: bool,
    pub no_model_loading: bool,
    pub no_runtime_inference: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_evidence_pack_created: bool,
    pub no_artifact_write: bool,
    pub no_persistence: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificQueryUnderstandingStatus {
    Blocked,
    Understood,
    Ambiguous,
    UnknownConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificQueryIntent {
    ConceptExplanation,
    MethodApplication,
    LiteratureSearch,
    CourseLearning,
    Comparison,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificAmbiguityLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificQueryDetectedAlias {
    pub alias: String,
    pub concept: String,
    pub language: String,
    pub start_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificQueryUnderstandingPreviewRequest {
    pub query: String,
    pub mode: Option<String>,
    pub course_context: Option<String>,
    pub context_tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificQueryUnderstandingPreview {
    pub status: ScholarChatScientificQueryUnderstandingStatus,
    pub normalized_query: String,
    pub normalized_mode: String,
    pub normalized_context_tags: Vec<String>,
    pub inferred_topic: Option<String>,
    pub query_intent: ScholarChatScientificQueryIntent,
    pub ambiguity_level: ScholarChatScientificAmbiguityLevel,
    pub ambiguity_warnings: Vec<String>,
    pub language_hints: Vec<String>,
    pub detected_aliases: Vec<ScholarChatScientificQueryDetectedAlias>,
    pub discipline_status: ScholarChatScientificDisciplineRegistryStatus,
    pub recognized_concept: Option<String>,
    pub label: Option<String>,
    pub source_registry_status: ScholarChatScientificSourceRegistryStatus,
    pub preferred_source_ids: Vec<String>,
    pub conditional_source_ids: Vec<String>,
    pub excluded_source_ids: Vec<String>,
    pub planned_metadata_queries: Vec<String>,
    pub planned_local_search_queries: Vec<String>,
    pub planned_expanded_queries: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub ranking_hints: Vec<String>,
    pub deduplication_hints: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub query_understanding_preview_only: bool,
    pub no_web_request: bool,
    pub no_scraping: bool,
    pub no_connector_call: bool,
    pub no_source_import: bool,
    pub no_local_file_indexing: bool,
    pub no_bm25_index: bool,
    pub no_vector_index: bool,
    pub no_model_loading: bool,
    pub no_runtime_inference: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_evidence_pack_created: bool,
    pub no_artifact_write: bool,
    pub no_persistence: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificSearchPlanStatus {
    Blocked,
    SearchPlanReady,
    NeedsDisambiguation,
    UnknownConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificSearchStrategy {
    LocalFirst,
    MetadataFirst,
    CourseLocalFirst,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatScientificSearchPlanStepKind {
    LocalSourceSearch,
    LocalCourseMaterialSearch,
    MetadataSourceSearch,
    QueryExpansion,
    SourceFamilyRouting,
    RankingPlan,
    DeduplicationPlan,
    EvidenceRequirementCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSearchPlanStep {
    pub kind: ScholarChatScientificSearchPlanStepKind,
    pub id: String,
    pub label: String,
    pub description: String,
    pub planned_queries: Vec<String>,
    pub source_ids: Vec<String>,
    pub depends_on: Vec<String>,
    pub active: bool,
    pub preview_only: bool,
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificLocalSearchPlan {
    pub local_source_count: usize,
    pub selected_local_source_ids: Vec<String>,
    pub planned_queries: Vec<String>,
    pub local_first: bool,
    pub requires_local_evidence_before_answer: bool,
    pub will_read_files: bool,
    pub will_build_index: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificMetadataSearchPlan {
    pub source_family_count: usize,
    pub preferred_source_ids: Vec<String>,
    pub conditional_source_ids: Vec<String>,
    pub excluded_source_ids: Vec<String>,
    pub planned_queries: Vec<String>,
    pub will_call_connectors: bool,
    pub will_make_web_requests: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSourceRoutingPlan {
    pub route_count: usize,
    pub active_routes: Vec<String>,
    pub conditional_routes: Vec<String>,
    pub excluded_routes: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSearchPlanRequest {
    pub query: String,
    pub mode: Option<String>,
    pub course_context: Option<String>,
    pub context_tags: Option<Vec<String>>,
    pub selected_local_source_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatScientificSearchPlanPreview {
    pub status: ScholarChatScientificSearchPlanStatus,
    pub normalized_query: String,
    pub normalized_mode: String,
    pub normalized_context_tags: Vec<String>,
    pub selected_local_source_ids: Vec<String>,
    pub query_understanding_status: ScholarChatScientificQueryUnderstandingStatus,
    pub inferred_topic: Option<String>,
    pub query_intent: ScholarChatScientificQueryIntent,
    pub ambiguity_level: ScholarChatScientificAmbiguityLevel,
    pub recognized_concept: Option<String>,
    pub label: Option<String>,
    pub source_registry_status: ScholarChatScientificSourceRegistryStatus,
    pub search_strategy: ScholarChatScientificSearchStrategy,
    pub local_search_plan: ScholarChatScientificLocalSearchPlan,
    pub metadata_search_plan: ScholarChatScientificMetadataSearchPlan,
    pub source_routing_plan: ScholarChatScientificSourceRoutingPlan,
    pub planned_search_steps: Vec<ScholarChatScientificSearchPlanStep>,
    pub planned_local_queries: Vec<String>,
    pub planned_metadata_queries: Vec<String>,
    pub planned_expanded_queries: Vec<String>,
    pub preferred_source_ids: Vec<String>,
    pub conditional_source_ids: Vec<String>,
    pub excluded_source_ids: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub ranking_hints: Vec<String>,
    pub deduplication_hints: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub scientific_search_plan_preview_only: bool,
    pub no_web_request: bool,
    pub no_scraping: bool,
    pub no_connector_call: bool,
    pub no_source_import: bool,
    pub no_local_file_indexing: bool,
    pub no_bm25_index: bool,
    pub no_vector_index: bool,
    pub no_model_loading: bool,
    pub no_runtime_inference: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_evidence_pack_created: bool,
    pub no_artifact_write: bool,
    pub no_persistence: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatLocalLiteratureIndexStatus {
    Blocked,
    IndexPlanReady,
    NeedsLocalSources,
    NeedsDisambiguation,
    UnknownConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatLocalLiteratureIndexStrategy {
    ScholarChatLocalFirst,
    ScientificPaperCitationLocalFirst,
    CourseMaterialLocalFirst,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatLocalLiteratureIngestionReadiness {
    Blocked,
    PreviewReady,
    NeedsSources,
    NeedsDisambiguation,
    UnknownConceptMappingNeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatLocalLiteratureIndexStepKind {
    SourceSelectionReview,
    MetadataRequirementCheck,
    CorpusManifestPlan,
    ExtractionPlan,
    ChunkingPolicyPlan,
    LexicalIndexPlan,
    VectorIndexPlan,
    DeduplicationPlan,
    RetrievalReadinessCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatLocalLiteratureCorpusPlan {
    pub selected_source_count: usize,
    pub selected_local_source_ids: Vec<String>,
    pub expected_source_kinds: Vec<String>,
    pub corpus_manifest_would_be_required: bool,
    pub will_create_corpus: bool,
    pub will_read_files: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatLocalLiteratureIndexArtifactPlan {
    pub planned_artifact_ids: Vec<String>,
    pub planned_artifact_descriptions: Vec<String>,
    pub will_create_artifacts: bool,
    pub will_create_bm25_index: bool,
    pub will_create_vector_index: bool,
    pub will_generate_embeddings: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatLocalLiteratureIndexStep {
    pub kind: ScholarChatLocalLiteratureIndexStepKind,
    pub id: String,
    pub label: String,
    pub description: String,
    pub planned_inputs: Vec<String>,
    pub planned_outputs: Vec<String>,
    pub active: bool,
    pub preview_only: bool,
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatLocalLiteratureIndexRequest {
    pub query: String,
    pub mode: Option<String>,
    pub course_context: Option<String>,
    pub context_tags: Option<Vec<String>>,
    pub selected_local_source_ids: Option<Vec<String>>,
    pub expected_source_kinds: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatLocalLiteratureIndexPreview {
    pub status: ScholarChatLocalLiteratureIndexStatus,
    pub normalized_query: String,
    pub normalized_mode: String,
    pub normalized_context_tags: Vec<String>,
    pub selected_local_source_ids: Vec<String>,
    pub expected_source_kinds: Vec<String>,
    pub search_plan_status: ScholarChatScientificSearchPlanStatus,
    pub query_understanding_status: ScholarChatScientificQueryUnderstandingStatus,
    pub inferred_topic: Option<String>,
    pub query_intent: ScholarChatScientificQueryIntent,
    pub recognized_concept: Option<String>,
    pub label: Option<String>,
    pub local_index_strategy: ScholarChatLocalLiteratureIndexStrategy,
    pub local_corpus_plan: ScholarChatLocalLiteratureCorpusPlan,
    pub index_artifact_plan: ScholarChatLocalLiteratureIndexArtifactPlan,
    pub ingestion_readiness: ScholarChatLocalLiteratureIngestionReadiness,
    pub planned_index_fields: Vec<String>,
    pub planned_chunking_policy: Vec<String>,
    pub planned_metadata_requirements: Vec<String>,
    pub planned_local_queries: Vec<String>,
    pub planned_index_steps: Vec<ScholarChatLocalLiteratureIndexStep>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub local_literature_index_preview_only: bool,
    pub no_file_read: bool,
    pub no_pdf_extraction: bool,
    pub no_ocr: bool,
    pub no_chunking_run: bool,
    pub no_embedding_generation: bool,
    pub no_index_created: bool,
    pub no_bm25_index: bool,
    pub no_vector_index: bool,
    pub no_retrieval_execution: bool,
    pub no_web_request: bool,
    pub no_scraping: bool,
    pub no_connector_call: bool,
    pub no_source_import: bool,
    pub no_local_file_indexing: bool,
    pub no_model_loading: bool,
    pub no_runtime_inference: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_evidence_pack_created: bool,
    pub no_artifact_write: bool,
    pub no_persistence: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatCourseLiteratureRegistryStatus {
    Blocked,
    CourseRegistryPlanReady,
    NeedsCourseContext,
    NeedsLocalSources,
    NeedsDisambiguation,
    UnknownConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatCourseLiteratureRegistryStrategy {
    CourseMaterialAlignmentFirst,
    ModuleContextFirst,
    LocalSourceAlignmentFirst,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatCourseLiteratureRegistryStepKind {
    CourseIdentityReview,
    ModuleContextReview,
    CourseMaterialKindPlan,
    LocalSourceAlignmentPlan,
    CurriculumMetadataRequirementCheck,
    LocalLiteratureIndexAlignment,
    RetrievalReadinessCheck,
    LearningPathAlignmentPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatCourseIdentityPreview {
    pub course_context: Option<String>,
    pub module_code: Option<String>,
    pub course_title: Option<String>,
    pub instructor: Option<String>,
    pub semester: Option<String>,
    pub identity_key: Option<String>,
    pub has_course_context: bool,
    pub has_module_code: bool,
    pub has_course_title: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatCourseMaterialPlan {
    pub selected_source_count: usize,
    pub selected_local_source_ids: Vec<String>,
    pub expected_course_material_kinds: Vec<String>,
    pub known_material_kinds: Vec<String>,
    pub unknown_material_kinds: Vec<String>,
    pub will_read_files: bool,
    pub will_import_sources: bool,
    pub will_create_registry: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatCurriculumAlignmentPlan {
    pub requires_course_context: bool,
    pub requires_module_metadata: bool,
    pub requires_learning_objectives_later: bool,
    pub requires_prerequisites_later: bool,
    pub requires_session_or_week_mapping_later: bool,
    pub will_scrape_curriculum_sources: bool,
    pub will_call_connectors: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatCourseLiteratureRegistryStep {
    pub kind: ScholarChatCourseLiteratureRegistryStepKind,
    pub id: String,
    pub label: String,
    pub description: String,
    pub planned_inputs: Vec<String>,
    pub planned_outputs: Vec<String>,
    pub active: bool,
    pub preview_only: bool,
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatCourseLiteratureRegistryPreviewRequest {
    pub query: String,
    pub course_context: Option<String>,
    pub module_code: Option<String>,
    pub course_title: Option<String>,
    pub instructor: Option<String>,
    pub semester: Option<String>,
    pub context_tags: Option<Vec<String>>,
    pub selected_local_source_ids: Option<Vec<String>>,
    pub expected_course_material_kinds: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatCourseLiteratureRegistryPreview {
    pub status: ScholarChatCourseLiteratureRegistryStatus,
    pub normalized_query: String,
    pub normalized_course_context: Option<String>,
    pub normalized_module_code: Option<String>,
    pub normalized_course_title: Option<String>,
    pub normalized_instructor: Option<String>,
    pub normalized_semester: Option<String>,
    pub normalized_context_tags: Vec<String>,
    pub selected_local_source_ids: Vec<String>,
    pub expected_course_material_kinds: Vec<String>,
    pub local_literature_index_status: ScholarChatLocalLiteratureIndexStatus,
    pub search_plan_status: ScholarChatScientificSearchPlanStatus,
    pub query_understanding_status: ScholarChatScientificQueryUnderstandingStatus,
    pub inferred_topic: Option<String>,
    pub query_intent: ScholarChatScientificQueryIntent,
    pub recognized_concept: Option<String>,
    pub label: Option<String>,
    pub course_registry_strategy: ScholarChatCourseLiteratureRegistryStrategy,
    pub course_identity: ScholarChatCourseIdentityPreview,
    pub course_material_plan: ScholarChatCourseMaterialPlan,
    pub curriculum_alignment_plan: ScholarChatCurriculumAlignmentPlan,
    pub planned_course_metadata_requirements: Vec<String>,
    pub planned_course_material_queries: Vec<String>,
    pub planned_registry_steps: Vec<ScholarChatCourseLiteratureRegistryStep>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub course_literature_registry_preview_only: bool,
    pub no_file_read: bool,
    pub no_pdf_extraction: bool,
    pub no_ocr: bool,
    pub no_chunking_run: bool,
    pub no_embedding_generation: bool,
    pub no_index_created: bool,
    pub no_retrieval_execution: bool,
    pub no_web_request: bool,
    pub no_scraping: bool,
    pub no_connector_call: bool,
    pub no_source_import: bool,
    pub no_local_file_indexing: bool,
    pub no_model_loading: bool,
    pub no_runtime_inference: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_evidence_pack_created: bool,
    pub no_artifact_write: bool,
    pub no_persistence: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRetrievalCandidate {
    pub source_id: String,
    pub version_id: String,
    pub chunk_id: String,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub preview: String,
    pub locator: CitationLocator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRetrievalPreviewResponse {
    pub status: ScholarChatStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub candidate_count: usize,
    pub candidates: Vec<ScholarChatRetrievalCandidate>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatEvidencePlanStatus {
    EvidencePlanPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatEvidenceCandidate {
    pub source_id: String,
    pub version_id: String,
    pub chunk_id: String,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub preview: String,
    pub locator: CitationLocator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatEvidencePlan {
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub evidence_required: bool,
    pub evidence_pack_would_be_built_later: bool,
    pub summary: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatEvidencePlanResponse {
    pub status: ScholarChatEvidencePlanStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub evidence_plan: ScholarChatEvidencePlan,
    pub candidates: Vec<ScholarChatEvidenceCandidate>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatPromptPackStatus {
    PromptPackPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatPromptPackSectionKind {
    SystemOrPolicyInstructions,
    ModeInstructions,
    GroundingInstructions,
    SourceContext,
    UserPrompt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatPromptPackSection {
    pub kind: ScholarChatPromptPackSectionKind,
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatPromptContextItem {
    pub source_id: String,
    pub version_id: String,
    pub chunk_id: String,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub preview: String,
    pub locator: CitationLocator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatPromptPack {
    pub section_count: usize,
    pub context_item_count: usize,
    pub estimated_input_char_count: usize,
    pub sections: Vec<ScholarChatPromptPackSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatPromptPackPreviewResponse {
    pub status: ScholarChatPromptPackStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub prompt_pack: ScholarChatPromptPack,
    pub context_items: Vec<ScholarChatPromptContextItem>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatAnswerReadinessStatus {
    Blocked,
    NeedsSources,
    NeedsRetrievalIndex,
    NeedsEvidenceCandidates,
    NeedsRuntimeConfig,
    NeedsExecutionConsent,
    ReadyForDraftInferenceLater,
    ReadyForGroundedDraftLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatAnswerReadinessOutputClassification {
    Blocked,
    UngroundedDraft,
    SourceContextDraft,
    GroundedDraftCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatAnswerReadinessBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatAnswerReadinessWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatAnswerReadinessRequest {
    pub scholar_chat_request: ScholarChatRequest,
    pub runtime_config: LocalModelRuntimeConfig,
    pub allow_model_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatAnswerReadinessPreview {
    pub status: ScholarChatAnswerReadinessStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_count: usize,
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub prompt_pack_ready: bool,
    pub runtime_health_status: LocalModelRuntimeHealthStatus,
    pub invocation_plan_status: LocalRuntimeInvocationPlanStatus,
    pub allow_model_execution: bool,
    pub would_generate_answer_now: bool,
    pub would_build_evidence_pack_now: bool,
    pub would_create_final_answer_now: bool,
    pub future_output_classification: ScholarChatAnswerReadinessOutputClassification,
    pub blockers: Vec<ScholarChatAnswerReadinessBlocker>,
    pub warnings: Vec<ScholarChatAnswerReadinessWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftInferenceStatus {
    Blocked,
    NeedsSources,
    NeedsEvidence,
    NeedsRuntimeConfig,
    NeedsExecutionConsent,
    InferenceSucceeded,
    InferenceFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftOutputClassification {
    Blocked,
    UngroundedModelDraft,
    SourceContextDraft,
    GroundedDraftCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftInferenceBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftInferenceWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatDraftInferenceRequest {
    pub scholar_chat_request: ScholarChatRequest,
    pub runtime_config: LocalModelRuntimeConfig,
    pub allow_model_execution: bool,
    pub timeout_ms: Option<u64>,
    pub max_output_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatDraftInferencePreview {
    pub status: ScholarChatDraftInferenceStatus,
    pub output_classification: ScholarChatDraftOutputClassification,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_count: usize,
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub prompt_pack_section_count: usize,
    pub prompt_char_count: usize,
    pub runtime_health_status: LocalModelRuntimeHealthStatus,
    pub invocation_plan_status: LocalRuntimeInvocationPlanStatus,
    pub allow_model_execution: bool,
    pub execution_attempted: bool,
    pub safe_model_file_name: Option<String>,
    pub safe_executable_file_name: Option<String>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub draft_only: bool,
    pub preview_only: bool,
    pub not_final_answer: bool,
    pub not_grounded_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub blockers: Vec<ScholarChatDraftInferenceBlocker>,
    pub warnings: Vec<ScholarChatDraftInferenceWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftGroundingInspectionStatus {
    Blocked,
    NoDraftText,
    NoEvidenceCandidates,
    Inspected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftGroundingSupportStatus {
    NotInspected,
    Unsupported,
    WeaklySupported,
    SupportedByLocalEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionRequest {
    pub scholar_chat_request: ScholarChatRequest,
    pub draft_text: Option<String>,
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildIntentRequest {
    pub grounding_request: ScholarChatDraftGroundingInspectionRequest,
    pub answer_draft_id: Option<String>,
    pub explicit_user_intent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionItem {
    pub item_index: usize,
    pub text_preview: String,
    pub support_status: ScholarChatDraftGroundingSupportStatus,
    pub matched_evidence_count: usize,
    pub source_ids: Vec<String>,
    pub locator_previews: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionPreview {
    pub status: ScholarChatDraftGroundingInspectionStatus,
    pub normalized_prompt: String,
    pub draft_char_count: usize,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub unsupported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub supported_item_count: usize,
    pub items: Vec<ScholarChatDraftGroundingInspectionItem>,
    pub inspection_only: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_evidence_pack_built: bool,
    pub no_answer_artifact_created: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedDraftReadinessStatus {
    Blocked,
    NeedsReview,
    ReadyForGroundedDraftLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedDraftReadinessPreview {
    pub status: ScholarChatGroundedDraftReadinessStatus,
    pub inspection_status: ScholarChatDraftGroundingInspectionStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub summary: String,
    pub preview_only: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildPlanStatus {
    Blocked,
    NeedsReview,
    PlanReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildPlanPreview {
    pub status: ScholarChatGroundedAnswerBuildPlanStatus,
    pub readiness_status: ScholarChatGroundedDraftReadinessStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub summary: String,
    pub planned_steps: Vec<String>,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerCandidateStatus {
    Blocked,
    NeedsReview,
    CandidateReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerCandidateItem {
    pub item_index: usize,
    pub statement_preview: String,
    pub support_status: ScholarChatDraftGroundingSupportStatus,
    pub source_ids: Vec<String>,
    pub locator_previews: Vec<String>,
    pub matched_evidence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerCandidatePreview {
    pub status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub summary: String,
    pub candidate_items: Vec<ScholarChatGroundedAnswerCandidateItem>,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerWriteEligibilityStatus {
    Blocked,
    NeedsReview,
    WriteEligibleLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerWriteEligibilityPreview {
    pub status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub eligibility_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildIntentStatus {
    Blocked,
    NeedsReview,
    IntentReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildIntentPreview {
    pub status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub intent_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildRequestStatus {
    Blocked,
    NeedsReview,
    RequestReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildRequestPreviewRequest {
    pub build_intent_request: ScholarChatGroundedAnswerBuildIntentRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildRequestPreview {
    pub status: ScholarChatGroundedAnswerBuildRequestStatus,
    pub build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub answer_draft_id: Option<String>,
    pub selected_source_ids: Vec<String>,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub request_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildPreflightStatus {
    Blocked,
    NeedsReview,
    PreflightReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildPreflightPreviewRequest {
    pub build_request_preview_request: ScholarChatGroundedAnswerBuildRequestPreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildPreflightPreview {
    pub status: ScholarChatGroundedAnswerBuildPreflightStatus,
    pub build_request_status: ScholarChatGroundedAnswerBuildRequestStatus,
    pub build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub answer_draft_id: Option<String>,
    pub selected_source_ids: Vec<String>,
    pub answer_draft_present: bool,
    pub answer_draft_readable: bool,
    pub answer_draft_claim_count: usize,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub preflight_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
    pub no_grounded_answer_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerExecutionReadinessStatus {
    Blocked,
    NeedsReview,
    ExecutionReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerExecutionReadinessPreviewRequest {
    pub build_preflight_preview_request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest,
    pub execution_consent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerExecutionReadinessPreview {
    pub status: ScholarChatGroundedAnswerExecutionReadinessStatus,
    pub build_preflight_status: ScholarChatGroundedAnswerBuildPreflightStatus,
    pub build_request_status: ScholarChatGroundedAnswerBuildRequestStatus,
    pub build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub answer_draft_id: Option<String>,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub answer_draft_present: bool,
    pub answer_draft_readable: bool,
    pub answer_draft_claim_count: usize,
    pub execution_consent: bool,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub readiness_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
    pub no_grounded_answer_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerExecutionPlanStatus {
    Blocked,
    NeedsReview,
    PlanReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerExecutionPlanPreviewRequest {
    pub execution_readiness_preview_request: ScholarChatGroundedAnswerExecutionReadinessPreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerExecutionPlanPreview {
    pub status: ScholarChatGroundedAnswerExecutionPlanStatus,
    pub readiness_status: ScholarChatGroundedAnswerExecutionReadinessStatus,
    pub build_preflight_status: ScholarChatGroundedAnswerBuildPreflightStatus,
    pub build_request_status: ScholarChatGroundedAnswerBuildRequestStatus,
    pub build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub answer_draft_id: Option<String>,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub answer_draft_present: bool,
    pub answer_draft_readable: bool,
    pub answer_draft_claim_count: usize,
    pub execution_consent: bool,
    pub planned_operation: String,
    pub planned_inputs: Vec<String>,
    pub planned_outputs: Vec<String>,
    pub planned_write_targets: Vec<String>,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub plan_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
    pub no_grounded_answer_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatRuntimeDiagnosticBridgeStatus {
    Blocked,
    NeedsReview,
    RuntimeDiagnosticReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRuntimeDiagnosticBridgePreviewRequest {
    pub scholar_chat_request: ScholarChatRequest,
    pub smoke_execution_plan_preview_request: LocalRuntimeSmokeExecutionPlanPreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRuntimeDiagnosticBridgePreview {
    pub status: ScholarChatRuntimeDiagnosticBridgeStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus,
    pub smoke_readiness_status: LocalRuntimeSmokeReadinessStatus,
    pub capability_status: LocalRuntimeCapabilityStatus,
    pub version_probe_status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub diagnostic_prompt_char_count: usize,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub runtime_diagnostic_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeProbeWarning>,
    pub warnings: Vec<LocalRuntimeProbeWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_smoke_execution: bool,
    pub no_runtime_inference: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_answer_draft_created: bool,
    pub no_grounded_answer_created: bool,
    pub no_final_answer_created: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatRuntimeDiagnosticResultStatus {
    Blocked,
    NeedsReview,
    RuntimeDiagnosticFailed,
    RuntimeDiagnosticSucceededLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRuntimeDiagnosticResultPreviewRequest {
    pub bridge_preview_request: ScholarChatRuntimeDiagnosticBridgePreviewRequest,
    pub diagnostic_preview: LocalRuntimeSmokeDiagnosticPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRuntimeDiagnosticResultPreview {
    pub status: ScholarChatRuntimeDiagnosticResultStatus,
    pub bridge_status: ScholarChatRuntimeDiagnosticBridgeStatus,
    pub smoke_diagnostic_status: LocalRuntimeSmokeDiagnosticStatus,
    pub smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus,
    pub smoke_readiness_status: LocalRuntimeSmokeReadinessStatus,
    pub capability_status: LocalRuntimeCapabilityStatus,
    pub version_probe_status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub diagnostic_prompt_char_count: usize,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub exit_code: Option<i32>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub runtime_result_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeProbeWarning>,
    pub warnings: Vec<LocalRuntimeProbeWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub diagnostic_result_only: bool,
    pub no_smoke_execution: bool,
    pub no_runtime_inference: bool,
    pub no_new_process_spawn: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_answer_draft_created: bool,
    pub no_grounded_answer_created: bool,
    pub no_final_answer_created: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatRuntimeAnswerPipelineGateStatus {
    Blocked,
    NeedsReview,
    ReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRuntimeAnswerPipelineGatePreviewRequest {
    pub grounded_answer_execution_plan_preview_request: ScholarChatGroundedAnswerExecutionPlanPreviewRequest,
    pub runtime_diagnostic_result_preview_request: ScholarChatRuntimeDiagnosticResultPreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRuntimeAnswerPipelineGatePreview {
    pub status: ScholarChatRuntimeAnswerPipelineGateStatus,
    pub grounded_answer_execution_plan_status: ScholarChatGroundedAnswerExecutionPlanStatus,
    pub grounded_answer_execution_readiness_status: ScholarChatGroundedAnswerExecutionReadinessStatus,
    pub runtime_diagnostic_result_status: ScholarChatRuntimeDiagnosticResultStatus,
    pub runtime_diagnostic_bridge_status: ScholarChatRuntimeDiagnosticBridgeStatus,
    pub smoke_diagnostic_status: LocalRuntimeSmokeDiagnosticStatus,
    pub smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus,
    pub smoke_readiness_status: LocalRuntimeSmokeReadinessStatus,
    pub capability_status: LocalRuntimeCapabilityStatus,
    pub version_probe_status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub selected_source_count: usize,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub diagnostic_prompt_char_count: usize,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub pipeline_gate_reasons: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub gate_only: bool,
    pub no_smoke_execution: bool,
    pub no_runtime_inference: bool,
    pub no_new_process_spawn: bool,
    pub no_llm_call: bool,
    pub no_answer_generated: bool,
    pub no_answer_draft_created: bool,
    pub no_grounded_answer_created: bool,
    pub no_final_answer_created: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

enum ScholarChatPreviewKind {
    Request,
    Retrieval,
}

pub fn preview_scholar_chat_request(
    _root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatResponse> {
    let normalized_prompt = normalized_prompt_or_err(request.prompt)?;
    let (selected_source_ids, selected_source_count) = normalize_selected_source_ids(request.selected_source_ids)?;
    let warnings = preview_warnings(&request.grounding_policy, selected_source_count, ScholarChatPreviewKind::Request);
    let grounding_plan = grounding_plan(&request.mode, &request.grounding_policy, selected_source_count);

    Ok(ScholarChatResponse {
        status: ScholarChatStatus::PreviewOnly,
        normalized_prompt,
        mode: request.mode,
        grounding_policy: request.grounding_policy,
        selected_source_ids,
        selected_source_count,
        grounding_plan,
        warnings,
    })
}

pub fn preview_scholar_chat_retrieval(
    root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatRetrievalPreviewResponse> {
    let normalized_prompt = normalized_prompt_or_err(request.prompt)?;
    let (selected_source_ids, selected_source_count) = normalize_selected_source_ids(request.selected_source_ids)?;
    let mut warnings = preview_warnings(&request.grounding_policy, selected_source_count, ScholarChatPreviewKind::Retrieval);

    let retrieval_service = RetrievalService::new(root);
    let mut candidates = Vec::new();

    if selected_source_count > 0 {
        for source_id in &selected_source_ids {
            match retrieval_service.preview_search_source(source_id, &normalized_prompt, SCHOLAR_CHAT_RETRIEVAL_PREVIEW_LIMIT) {
                Ok(response) => {
                    if response.results.is_empty() {
                        warnings.push(format!("No retrieval candidates matched selected source {source_id}."));
                    }
                    candidates.extend(convert_retrieval_response(response));
                }
                Err(AegisError::RetrievalIndexMissing)
                | Err(AegisError::RetrievalIndexReadFailed)
                | Err(AegisError::ChunkingReportMissing)
                | Err(AegisError::ChunkingReportReadFailed) => {
                    warnings.push(format!("Retrieval data is not ready for selected source {source_id}; skipping this source."));
                }
                Err(error) => return Err(error),
            }
        }
    }

    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.source_id.cmp(&right.source_id))
            .then_with(|| left.chunk_id.cmp(&right.chunk_id))
    });

    if selected_source_count > 0 && candidates.is_empty() && !warnings.iter().any(|warning| warning.contains("Retrieval data is not ready")) {
        warnings.push("No retrieval candidates matched the selected sources.".to_string());
    }

    Ok(ScholarChatRetrievalPreviewResponse {
        status: ScholarChatStatus::PreviewOnly,
        normalized_prompt,
        mode: request.mode,
        grounding_policy: request.grounding_policy,
        selected_source_ids,
        selected_source_count,
        candidate_count: candidates.len(),
        candidates,
        warnings,
    })
}

pub fn preview_scholar_chat_evidence_plan(
    root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatEvidencePlanResponse> {
    let retrieval_preview = preview_scholar_chat_retrieval(root, request)?;
    let mut warnings = retrieval_preview.warnings.clone();
    if retrieval_preview.selected_source_count == 0 && !warnings.iter().any(|warning| warning.contains("No selected sources")) {
        warnings.push("No selected sources; evidence plan preview is unscoped.".to_string());
    }
    if retrieval_preview.candidate_count == 0 {
        warnings.push("No retrieval candidates were eligible for Evidence Pack assembly yet.".to_string());
    }
    warnings.push("This is a preview only; no Evidence Pack was built.".to_string());

    let evidence_candidates = convert_retrieval_candidates_from_preview(&retrieval_preview.candidates);
    let evidence_candidate_count = evidence_candidates.len();
    let evidence_plan = evidence_plan(
        &retrieval_preview.mode,
        &retrieval_preview.grounding_policy,
        retrieval_preview.selected_source_count,
        retrieval_preview.candidate_count,
        evidence_candidate_count,
    );

    Ok(ScholarChatEvidencePlanResponse {
        status: ScholarChatEvidencePlanStatus::EvidencePlanPreview,
        normalized_prompt: retrieval_preview.normalized_prompt,
        mode: retrieval_preview.mode,
        grounding_policy: retrieval_preview.grounding_policy,
        selected_source_ids: retrieval_preview.selected_source_ids,
        selected_source_count: retrieval_preview.selected_source_count,
        retrieval_candidate_count: retrieval_preview.candidate_count,
        evidence_candidate_count,
        evidence_plan,
        candidates: evidence_candidates,
        warnings,
    })
}

pub fn preview_scholar_chat_prompt_pack(
    root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatPromptPackPreviewResponse> {
    let evidence_plan = preview_scholar_chat_evidence_plan(root, request)?;
    let context_items = convert_evidence_candidates_to_prompt_context_items(&evidence_plan.candidates);
    let prompt_pack = build_prompt_pack(
        &evidence_plan.mode,
        &evidence_plan.grounding_policy,
        &evidence_plan.normalized_prompt,
        &evidence_plan.selected_source_ids,
        &context_items,
        evidence_plan.evidence_candidate_count,
    );
    let mut warnings = evidence_plan.warnings.clone();
    if evidence_plan.selected_source_count == 0 {
        push_warning(&mut warnings, "No selected sources; prompt pack preview is unscoped.");
    }
    if evidence_plan.evidence_candidate_count == 0 {
        push_warning(&mut warnings, "No evidence candidates were eligible for prompt-pack assembly yet.");
    }
    push_warning(&mut warnings, "This is a prompt pack preview only; no model inference was run.");
    if matches!(evidence_plan.grounding_policy, GroundingPolicy::LocalOnly) {
        push_warning(&mut warnings, "local_only requires local evidence before a prompt pack can be turned into an answer.");
    }
    if matches!(evidence_plan.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge) {
        push_warning(&mut warnings, "Model knowledge would need to be clearly marked later.");
    }

    Ok(ScholarChatPromptPackPreviewResponse {
        status: ScholarChatPromptPackStatus::PromptPackPreview,
        normalized_prompt: evidence_plan.normalized_prompt,
        mode: evidence_plan.mode,
        grounding_policy: evidence_plan.grounding_policy,
        selected_source_ids: evidence_plan.selected_source_ids,
        selected_source_count: evidence_plan.selected_source_count,
        evidence_candidate_count: evidence_plan.evidence_candidate_count,
        prompt_pack,
        context_items,
        warnings,
    })
}

pub fn preview_scholar_chat_scientific_discipline_registry(
    root: impl Into<PathBuf>,
    request: ScholarChatScientificDisciplineRegistryPreviewRequest,
) -> AegisResult<ScholarChatScientificDisciplineRegistryPreview> {
    let _root = root.into();
    let normalized_topic = normalize_scientific_topic_text(&request.topic);
    let normalized_mode = normalize_scientific_mode(request.mode);
    let topic_key = normalize_scientific_topic_key(&normalized_topic);
    let mapped_entry = scientific_discipline_registry_entry(&topic_key);

    let status = if normalized_topic.is_empty() {
        ScholarChatScientificDisciplineRegistryStatus::Blocked
    } else if mapped_entry.is_some() {
        ScholarChatScientificDisciplineRegistryStatus::ConceptMapped
    } else {
        ScholarChatScientificDisciplineRegistryStatus::UnknownConcept
    };

    let mut blockers = Vec::new();
    let mut warnings = vec![
        "This is a scientific discipline registry preview only; no web requests, scraping, connectors, or local indexing were run.".to_string(),
    ];
    let mut next_required_actions = Vec::new();

    match normalized_mode.as_str() {
        "scientific_paper" => {
            push_unique_text(
                &mut warnings,
                "Scientific Paper Mode will later prioritize research question decomposition, literature search, deduplication, and no fabricated citations.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Future Scientific Paper Mode phases should prioritize research question decomposition, literature search, deduplication, and citation-safe planning.",
            );
        }
        "course" => {
            push_unique_text(
                &mut warnings,
                "Course Mode will later prioritize local course materials, module context, prerequisites, and learning path support.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Future Course Mode phases should prioritize local course materials, module context, prerequisites, and learning path support.",
            );
        }
        _ => {
            push_unique_text(
                &mut next_required_actions,
                "Plan local evidence lookup first before later Scholar Chat answering.",
            );
        }
    }

    if normalized_topic.is_empty() {
        blockers.push("topic_missing: Provide a scientific topic to preview.".to_string());
        next_required_actions.push("Provide a scientific topic to preview the discipline registry.".to_string());
    }

    if normalized_topic.is_empty() {
        // handled below
    } else if mapped_entry.is_none() {
        push_unique_text(
            &mut warnings,
            "The topic is not yet in the local preview registry.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Add a discipline registry mapping in a later phase.",
        );
    }

    if let Some(entry) = &mapped_entry {
        if request.course_context.as_deref().map(str::trim).filter(|value| !value.is_empty()).is_some()
            && matches!(normalized_mode.as_str(), "course")
        {
            push_unique_text(
                &mut warnings,
                "Course context is preview-only and will later be mapped to curriculum metadata without local file indexing.",
            );
        }

        let summary = format!(
            "Scientific discipline preview mapped {} in {} mode.",
            entry.label, normalized_mode
        );

        return Ok(ScholarChatScientificDisciplineRegistryPreview {
            status,
            normalized_topic,
            normalized_mode,
            recognized_concept: Some(entry.recognized_concept.to_string()),
            label: Some(entry.label.to_string()),
            discipline_path: entry
                .discipline_path
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            science_class: Some(entry.science_class.clone()),
            parent_path: entry
                .parent_path
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            related_methods: entry
                .related_methods
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            appears_in: entry
                .appears_in
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            preferred_sources: entry
                .preferred_sources
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            curriculum_sources: entry
                .curriculum_sources
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            canonical_mappings: entry
                .canonical_mappings
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            planned_queries: entry
                .planned_queries
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            blockers,
            warnings,
            next_required_actions,
            summary,
            preview_only: true,
            registry_preview_only: true,
            no_web_request: true,
            no_scraping: true,
            no_connector_call: true,
            no_source_import: true,
            no_local_file_indexing: true,
            no_bm25_index: true,
            no_vector_index: true,
            no_model_loading: true,
            no_runtime_inference: true,
            no_llm_call: true,
            no_answer_generated: true,
            no_evidence_pack_created: true,
            no_artifact_write: true,
            no_persistence: true,
            no_registry_status_change: true,
            no_audit_write: true,
        });
    }

    if !normalized_topic.is_empty() {
        push_unique_text(
            &mut warnings,
            "The topic is not yet in the local preview registry.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Add a discipline registry mapping in a later phase.",
        );
    }

    let mut planned_queries = Vec::new();
    if !normalized_topic.is_empty() {
        planned_queries.push(normalized_topic.clone());
    }

    let summary = if normalized_topic.is_empty() {
        "Scientific discipline preview blocked because the topic is blank.".to_string()
    } else {
        format!(
            "Scientific discipline preview did not yet recognize the topic '{}'.",
            normalized_topic
        )
    };

    Ok(ScholarChatScientificDisciplineRegistryPreview {
        status,
        normalized_topic,
        normalized_mode,
        recognized_concept: None,
        label: None,
        discipline_path: Vec::new(),
        science_class: None,
        parent_path: Vec::new(),
        related_methods: Vec::new(),
        appears_in: Vec::new(),
        preferred_sources: Vec::new(),
        curriculum_sources: Vec::new(),
        canonical_mappings: Vec::new(),
        planned_queries,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        registry_preview_only: true,
        no_web_request: true,
        no_scraping: true,
        no_connector_call: true,
        no_source_import: true,
        no_local_file_indexing: true,
        no_bm25_index: true,
        no_vector_index: true,
        no_model_loading: true,
        no_runtime_inference: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_evidence_pack_created: true,
        no_artifact_write: true,
        no_persistence: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_scholar_chat_scientific_source_registry(
    root: impl Into<PathBuf>,
    request: ScholarChatScientificSourceRegistryPreviewRequest,
) -> AegisResult<ScholarChatScientificSourceRegistryPreview> {
    let root = root.into();
    let normalized_topic = normalize_scientific_topic_text(&request.topic);
    let normalized_mode = normalize_scientific_mode(request.mode);
    let normalized_context_tags = normalize_scientific_context_tags(request.context_tags);
    let discipline_preview = preview_scholar_chat_scientific_discipline_registry(
        &root,
        ScholarChatScientificDisciplineRegistryPreviewRequest {
            topic: normalized_topic.clone(),
            mode: Some(normalized_mode.clone()),
            course_context: request.course_context.clone(),
        },
    )?;

    let discipline_status = discipline_preview.status.clone();
    let recognized_concept = discipline_preview.recognized_concept.clone();
    let label = discipline_preview.label.clone();
    let status = match discipline_status {
        ScholarChatScientificDisciplineRegistryStatus::Blocked => ScholarChatScientificSourceRegistryStatus::Blocked,
        ScholarChatScientificDisciplineRegistryStatus::ConceptMapped => {
            ScholarChatScientificSourceRegistryStatus::SourcePlanReady
        }
        ScholarChatScientificDisciplineRegistryStatus::UnknownConcept => {
            ScholarChatScientificSourceRegistryStatus::UnknownConcept
        }
    };

    let mut source_families = Vec::new();
    let mut preferred_source_ids = Vec::new();
    let mut conditional_source_ids = Vec::new();
    let mut excluded_source_ids = Vec::new();
    let mut blockers = discipline_preview.blockers.clone();
    let mut warnings = discipline_preview.warnings.clone();
    let mut next_required_actions = discipline_preview.next_required_actions.clone();

    if normalized_topic.is_empty() {
        blockers.push("topic_missing: Provide a scientific topic to preview.".to_string());
        next_required_actions.push("Provide a scientific topic to preview the source registry.".to_string());
    } else if matches!(normalized_mode.as_str(), "scholar_chat") {
        push_unique_text(
            &mut next_required_actions,
            "Later Scholar Chat should plan local evidence first before answering.",
        );
    }

    if normalized_topic.is_empty() {
        let summary = scientific_source_registry_plan_summary(&status, label.as_deref(), &normalized_mode);
        let source_plan = ScholarChatScientificSourceRegistrySourcePlan {
            source_family_count: 0,
            active_source_family_count: 0,
            conditional_source_family_count: 0,
            planned_metadata_query_count: 0,
            summary: summary.clone(),
            steps: vec![
                "Normalize topic, mode, and context tags.".to_string(),
                "Ask the user to provide a scientific topic.".to_string(),
            ],
        };

        return Ok(ScholarChatScientificSourceRegistryPreview {
            status,
            normalized_topic,
            normalized_mode,
            normalized_context_tags,
            discipline_status,
            recognized_concept: None,
            label: None,
            source_plan,
            source_families,
            preferred_source_ids,
            conditional_source_ids,
            excluded_source_ids,
            access_classes: Vec::new(),
            planned_metadata_queries: discipline_preview.planned_queries,
            ranking_hints: Vec::new(),
            deduplication_hints: Vec::new(),
            blockers,
            warnings,
            next_required_actions,
            summary,
            preview_only: true,
            source_registry_preview_only: true,
            no_web_request: true,
            no_scraping: true,
            no_connector_call: true,
            no_source_import: true,
            no_local_file_indexing: true,
            no_bm25_index: true,
            no_vector_index: true,
            no_model_loading: true,
            no_runtime_inference: true,
            no_llm_call: true,
            no_answer_generated: true,
            no_evidence_pack_created: true,
            no_artifact_write: true,
            no_persistence: true,
            no_registry_status_change: true,
            no_audit_write: true,
        });
    }

    let course_context_mentions_psychology = scientific_context_mentions(&request.course_context, &["psychology", "psychologie"]);
    let biomedical_context = scientific_context_tags_contain(
        &normalized_context_tags,
        &["biomedical", "medical", "neuroscience", "clinical", "diagnostics", "medicine"],
    ) || scientific_context_mentions(&request.course_context, &["biomedical", "medical", "neuroscience", "clinical", "diagnostics", "medicine"]);
    let psychology_context = scientific_context_tags_contain(&normalized_context_tags, &["psychology", "psychologie"]) || course_context_mentions_psychology;
    let theory_context = scientific_context_tags_contain(
        &normalized_context_tags,
        &["theory", "mathematics", "math", "statistics", "statistical_theory"],
    ) || scientific_context_mentions(&request.course_context, &["theory", "mathematics", "math", "statistics", "statistical_theory"]);

    let (planned_metadata_queries, _ranking_hints, _deduplication_hints) = match recognized_concept.as_deref() {
        Some("signal_detection_theory") => {
            let pubmed_active = biomedical_context;
            source_families = vec![
                scientific_source_registry_family(
                    "pubpsych",
                    "PubPsych",
                    "psychology",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Primary,
                    "active for signal detection theory psychology metadata planning.",
                    true,
                    "psychology literature metadata planning",
                    &["psychology metadata", "literature planning"],
                    &["preview-only", "no scraping", "no connector call"],
                ),
                scientific_source_registry_family(
                    "psycharchives",
                    "PsychArchives",
                    "psychology",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Primary,
                    "active for signal detection theory psychology repository planning.",
                    true,
                    "psychology repository and source-family planning",
                    &["repository planning", "psychology metadata"],
                    &["preview-only", "no scraping", "no connector call"],
                ),
                scientific_source_registry_family(
                    "openalex",
                    "OpenAlex",
                    "multidisciplinary",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Supporting,
                    "active as multidisciplinary source-family metadata planning.",
                    true,
                    "broad multidisciplinary metadata planning",
                    &["multidisciplinary metadata", "cross-domain mapping"],
                    &["preview-only", "no web requests"],
                ),
                scientific_source_registry_family(
                    "crossref",
                    "Crossref",
                    "multidisciplinary",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Supporting,
                    "active as citation and DOI metadata planning.",
                    true,
                    "citation and DOI metadata planning",
                    &["doi metadata", "citation metadata"],
                    &["preview-only", "no web requests"],
                ),
                scientific_source_registry_family(
                    "pubmed_if_biomedical_context",
                    "PubMed if biomedical context",
                    "medicine_biomedical_science",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Conditional,
                    "active only when biomedical, medical, neuroscience, clinical, diagnostics, or medicine context is explicit.",
                    pubmed_active,
                    "biomedical metadata planning when context is explicit",
                    &["biomedical context", "clinical metadata"],
                    &["preview-only", "conditional on explicit biomedical context"],
                ),
            ];
            preferred_source_ids = vec![
                "pubpsych".to_string(),
                "psycharchives".to_string(),
                "openalex".to_string(),
                "crossref".to_string(),
            ];
            conditional_source_ids = vec!["pubmed_if_biomedical_context".to_string()];
            if !pubmed_active {
                excluded_source_ids.push("pubmed_if_biomedical_context".to_string());
            } else {
                push_unique_text(
                    &mut warnings,
                    "PubMed is active because biomedical, medical, neuroscience, clinical, diagnostics, or medicine context is explicit.",
                );
            }
            let planned_metadata_queries = vec![
                "Signalentdeckungstheorie".to_string(),
                "signal detection theory".to_string(),
                "psychophysics signal detection".to_string(),
                "d prime criterion ROC".to_string(),
            ];
            (
                planned_metadata_queries.clone(),
                vec![
                    "Prefer psychology and psychophysics sources first.".to_string(),
                    "Use biomedical sources only when biomedical context is explicit.".to_string(),
                    "Prefer method and review sources later where available.".to_string(),
                ],
                vec![
                    "Deduplicate later by DOI, title, and source identifiers.".to_string(),
                    "Keep psychophysics records distinct from broader psychology metadata until later consolidation.".to_string(),
                ],
            )
        }
        Some("analysis_of_variance") => {
            let psychology_context_active = psychology_context;
            let theory_context_active = theory_context;
            source_families = vec![
                scientific_source_registry_family(
                    "openalex",
                    "OpenAlex",
                    "multidisciplinary",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Primary,
                    "active as a broad statistics and methods metadata source.",
                    true,
                    "broad statistics and methods metadata planning",
                    &["methods metadata", "broad literature planning"],
                    &["preview-only", "no scraping", "no connector call"],
                ),
                scientific_source_registry_family(
                    "crossref",
                    "Crossref",
                    "multidisciplinary",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Primary,
                    "active as a citation and DOI metadata source.",
                    true,
                    "citation and DOI metadata planning",
                    &["doi metadata", "citation metadata"],
                    &["preview-only", "no web requests"],
                ),
                scientific_source_registry_family(
                    "pubpsych_if_psychology_context",
                    "PubPsych if psychology context",
                    "psychology",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Conditional,
                    "active when psychology or psychologies context is explicit.",
                    psychology_context_active,
                    "psychology methods planning when psychology context is explicit",
                    &["psychology context", "methods planning"],
                    &["preview-only", "conditional on explicit psychology context"],
                ),
                scientific_source_registry_family(
                    "psycharchives_if_psychology_context",
                    "PsychArchives if psychology context",
                    "psychology",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Conditional,
                    "active when psychology or psychologies context is explicit.",
                    psychology_context_active,
                    "psychology repository planning when psychology context is explicit",
                    &["psychology context", "repository planning"],
                    &["preview-only", "conditional on explicit psychology context"],
                ),
                scientific_source_registry_family(
                    "zbmath_if_theory_context",
                    "zbMATH if theory context",
                    "mathematics_statistics",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Conditional,
                    "active when theory, mathematics, math, statistics, or statistical_theory context is explicit.",
                    theory_context_active,
                    "mathematics and statistics theory planning when theory context is explicit",
                    &["theory context", "mathematics statistics"],
                    &["preview-only", "conditional on explicit theory or statistics context"],
                ),
                scientific_source_registry_family(
                    "arxiv_if_theory_context",
                    "arXiv if theory context",
                    "preprint",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Conditional,
                    "active when theory, mathematics, math, statistics, or statistical_theory context is explicit.",
                    theory_context_active,
                    "preprint planning when theory context is explicit",
                    &["theory context", "preprint planning"],
                    &["preview-only", "conditional on explicit theory or statistics context"],
                ),
            ];
            preferred_source_ids = vec!["openalex".to_string(), "crossref".to_string()];
            conditional_source_ids = vec![
                "pubpsych_if_psychology_context".to_string(),
                "psycharchives_if_psychology_context".to_string(),
                "zbmath_if_theory_context".to_string(),
                "arxiv_if_theory_context".to_string(),
            ];
            if !psychology_context_active {
                excluded_source_ids.extend([
                    "pubpsych_if_psychology_context".to_string(),
                    "psycharchives_if_psychology_context".to_string(),
                ]);
            }
            if !theory_context_active {
                excluded_source_ids.extend([
                    "zbmath_if_theory_context".to_string(),
                    "arxiv_if_theory_context".to_string(),
                ]);
            }
            let planned_metadata_queries = vec![
                "ANOVA".to_string(),
                "Varianzanalyse".to_string(),
                "analysis of variance".to_string(),
                "factorial ANOVA".to_string(),
                "repeated measures ANOVA".to_string(),
            ];
            (
                planned_metadata_queries.clone(),
                vec![
                    "Prefer statistics and methods sources first.".to_string(),
                    "Activate psychology-specific sources only when psychology context is explicit.".to_string(),
                    "Activate theory sources only when theory or statistics context is explicit.".to_string(),
                ],
                vec![
                    "Deduplicate later by DOI, title, and source identifiers.".to_string(),
                    "Keep method and psychology records distinct until later consolidation.".to_string(),
                ],
            )
        }
        Some("hypothesis_testing") => {
            let psychology_context_active = psychology_context;
            source_families = vec![
                scientific_source_registry_family(
                    "openalex",
                    "OpenAlex",
                    "multidisciplinary",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Primary,
                    "active as a broad methods and statistics metadata source.",
                    true,
                    "broad methods and statistics metadata planning",
                    &["methods metadata", "broad literature planning"],
                    &["preview-only", "no scraping", "no connector call"],
                ),
                scientific_source_registry_family(
                    "crossref",
                    "Crossref",
                    "multidisciplinary",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Primary,
                    "active as a citation and DOI metadata source.",
                    true,
                    "citation and DOI metadata planning",
                    &["doi metadata", "citation metadata"],
                    &["preview-only", "no web requests"],
                ),
                scientific_source_registry_family(
                    "zbmath",
                    "zbMATH Open",
                    "mathematics_statistics",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Supporting,
                    "active as a mathematics and statistics theory metadata source.",
                    true,
                    "math and statistics theory metadata planning",
                    &["statistics theory", "math metadata"],
                    &["preview-only", "no scraping"],
                ),
                scientific_source_registry_family(
                    "arxiv",
                    "arXiv",
                    "preprint",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Supporting,
                    "active as a preprint and methods metadata source.",
                    true,
                    "preprint methods metadata planning",
                    &["preprint metadata", "methods planning"],
                    &["preview-only", "no scraping"],
                ),
                scientific_source_registry_family(
                    "pubpsych_if_psychology_context",
                    "PubPsych if psychology context",
                    "psychology",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Conditional,
                    "active when psychology context is explicit.",
                    psychology_context_active,
                    "psychology methods planning when psychology context is explicit",
                    &["psychology context", "methods planning"],
                    &["preview-only", "conditional on explicit psychology context"],
                ),
                scientific_source_registry_family(
                    "psycharchives_if_psychology_context",
                    "PsychArchives if psychology context",
                    "psychology",
                    ScholarChatScientificSourceAccessClass::OpenMetadata,
                    ScholarChatScientificSourcePriority::Conditional,
                    "active when psychology context is explicit.",
                    psychology_context_active,
                    "psychology repository planning when psychology context is explicit",
                    &["psychology context", "repository planning"],
                    &["preview-only", "conditional on explicit psychology context"],
                ),
            ];
            preferred_source_ids = vec![
                "openalex".to_string(),
                "crossref".to_string(),
                "zbmath".to_string(),
                "arxiv".to_string(),
            ];
            conditional_source_ids = vec![
                "pubpsych_if_psychology_context".to_string(),
                "psycharchives_if_psychology_context".to_string(),
            ];
            if !psychology_context_active {
                excluded_source_ids.extend([
                    "pubpsych_if_psychology_context".to_string(),
                    "psycharchives_if_psychology_context".to_string(),
                ]);
            }
            let planned_metadata_queries = vec![
                "Hypothesentests".to_string(),
                "hypothesis testing".to_string(),
                "null hypothesis p value".to_string(),
                "statistical power type I error type II error".to_string(),
                "confidence intervals hypothesis testing".to_string(),
            ];
            (
                planned_metadata_queries.clone(),
                vec![
                    "Prefer methods and statistics sources first.".to_string(),
                    "Use psychology sources when applied psychology context is explicit.".to_string(),
                    "Later phases should deduplicate by DOI, title, and source identifiers.".to_string(),
                ],
                vec![
                    "Deduplicate later by DOI, title, and source identifiers.".to_string(),
                    "Keep methods and psychology records distinct until later consolidation.".to_string(),
                ],
            )
        }
        _ => (
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
    };

    let ranking_hints = scientific_source_registry_ranking_hints(
        recognized_concept.as_deref(),
        &normalized_mode,
        &normalized_context_tags,
        &request.course_context,
    );
    let deduplication_hints = scientific_source_registry_deduplication_hints(
        recognized_concept.as_deref(),
        &normalized_mode,
    );

    if matches!(normalized_mode.as_str(), "course") {
        warnings.push("Course Mode will later prioritize local course materials, module context, prerequisites, and learning path support.".to_string());
        next_required_actions.push("Future Course Mode phases should prioritize local course materials, module context, prerequisites, and learning path support.".to_string());
    } else if matches!(normalized_mode.as_str(), "scientific_paper") {
        warnings.push("Scientific Paper Mode will later prioritize literature search planning, deduplication, ranking, review/meta-analysis prioritization where appropriate, and citation-safe planning.".to_string());
        next_required_actions.push("Future Scientific Paper Mode phases should prioritize literature search planning, deduplication, ranking, review/meta-analysis prioritization where appropriate, and citation-safe planning.".to_string());
    }

    let planned_metadata_queries = if recognized_concept.is_none() && !normalized_topic.is_empty() {
        push_unique_text(
            &mut warnings,
            "The topic is not yet mapped through the discipline registry.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Consider adding discipline mapping before expanding the source registry preview.",
        );
        vec![normalized_topic.clone()]
    } else {
        planned_metadata_queries
    };
    let access_classes = scientific_source_registry_access_classes(&source_families);
    let source_family_count = source_families.len();
    let active_source_family_count = source_families.iter().filter(|family| family.active_for_current_context).count();
    let conditional_source_family_count = source_families
        .iter()
        .filter(|family| matches!(family.priority, ScholarChatScientificSourcePriority::Conditional))
        .count();
    let summary = scientific_source_registry_plan_summary(&status, label.as_deref(), &normalized_mode);
    let source_plan = ScholarChatScientificSourceRegistrySourcePlan {
        source_family_count,
        active_source_family_count,
        conditional_source_family_count,
        planned_metadata_query_count: planned_metadata_queries.len(),
        summary: summary.clone(),
        steps: scientific_source_registry_plan_steps(&normalized_mode),
    };

    Ok(ScholarChatScientificSourceRegistryPreview {
        status,
        normalized_topic,
        normalized_mode,
        normalized_context_tags,
        discipline_status,
        recognized_concept,
        label,
        source_plan,
        source_families,
        preferred_source_ids,
        conditional_source_ids,
        excluded_source_ids,
        access_classes,
        planned_metadata_queries,
        ranking_hints,
        deduplication_hints,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        source_registry_preview_only: true,
        no_web_request: true,
        no_scraping: true,
        no_connector_call: true,
        no_source_import: true,
        no_local_file_indexing: true,
        no_bm25_index: true,
        no_vector_index: true,
        no_model_loading: true,
        no_runtime_inference: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_evidence_pack_created: true,
        no_artifact_write: true,
        no_persistence: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_scholar_chat_scientific_query_understanding(
    root: impl Into<PathBuf>,
    request: ScholarChatScientificQueryUnderstandingPreviewRequest,
) -> AegisResult<ScholarChatScientificQueryUnderstandingPreview> {
    let root = root.into();
    let normalized_query = normalize_scientific_query_text(&request.query);
    let normalized_mode = normalize_scientific_mode(request.mode.clone());
    let normalized_context_tags = normalize_scientific_context_tags(request.context_tags.clone());
    let query_lower = normalized_query.to_lowercase();
    let detected_aliases = scientific_query_detected_aliases(&normalized_query);
    let detected_concepts = scientific_query_detected_concepts(&normalized_query);
    let ambiguity_warnings = scientific_query_ambiguity_warnings(&detected_concepts);
    let query_intent = scientific_query_intent(&normalized_mode, &query_lower);
    let language_hints = scientific_query_language_hints(&query_lower, &detected_aliases);

    let status = if normalized_query.is_empty() {
        ScholarChatScientificQueryUnderstandingStatus::Blocked
    } else if detected_concepts.len() > 1 {
        ScholarChatScientificQueryUnderstandingStatus::Ambiguous
    } else if detected_concepts.is_empty() {
        ScholarChatScientificQueryUnderstandingStatus::UnknownConcept
    } else {
        ScholarChatScientificQueryUnderstandingStatus::Understood
    };

    let ambiguity_level = if matches!(status, ScholarChatScientificQueryUnderstandingStatus::Ambiguous) {
        ScholarChatScientificAmbiguityLevel::Medium
    } else {
        ScholarChatScientificAmbiguityLevel::None
    };

    let inferred_topic = if normalized_query.is_empty() {
        None
    } else if let Some(first_concept) = detected_concepts.first() {
        Some(first_concept.topic_label.to_string())
    } else {
        Some(normalized_query.clone())
    };

    let preview_topic = inferred_topic.clone().unwrap_or_else(|| normalized_query.clone());
    let discipline_preview = preview_scholar_chat_scientific_discipline_registry(
        &root,
        ScholarChatScientificDisciplineRegistryPreviewRequest {
            topic: preview_topic.clone(),
            mode: Some(normalized_mode.clone()),
            course_context: request.course_context.clone(),
        },
    )?;
    let source_preview = preview_scholar_chat_scientific_source_registry(
        &root,
        ScholarChatScientificSourceRegistryPreviewRequest {
            topic: preview_topic.clone(),
            mode: Some(normalized_mode.clone()),
            course_context: request.course_context.clone(),
            context_tags: Some(normalized_context_tags.clone()),
        },
    )?;

    let recognized_concept = discipline_preview.recognized_concept.clone();
    let label = discipline_preview.label.clone();
    let discipline_status = discipline_preview.status.clone();
    let source_registry_status = source_preview.status.clone();

    let mut blockers = Vec::new();
    let mut warnings = vec![
        "This is a scientific query understanding preview only; no web requests, scraping, connectors, local indexing, model loading, runtime inference, or answer generation were run.".to_string(),
    ];
    let mut next_required_actions = Vec::new();

    if normalized_query.is_empty() {
        blockers.push("query_missing: Provide a scientific query to preview.".to_string());
        next_required_actions.push("Provide a scientific query to preview the understanding plan.".to_string());
    }

    if matches!(status, ScholarChatScientificQueryUnderstandingStatus::Ambiguous) {
        if !ambiguity_warnings.is_empty() {
            warnings.extend(ambiguity_warnings.iter().cloned());
        }
        next_required_actions.push(
            "Narrow the topic or accept the first inferred concept before expanding retrieval planning."
                .to_string(),
        );
    } else if matches!(status, ScholarChatScientificQueryUnderstandingStatus::UnknownConcept) && !normalized_query.is_empty() {
        warnings.push(
            "The query does not yet map to a known scientific concept.".to_string(),
        );
        next_required_actions.push(
            "Add discipline and source registry mappings before expanding retrieval planning."
                .to_string(),
        );
    }

    match normalized_mode.as_str() {
        "scientific_paper" => {
            warnings.push(
                "Scientific Paper Mode will later prioritize metadata search planning, deduplication, ranking, and citation-safe evidence preparation."
                    .to_string(),
            );
            next_required_actions.push(
                "Future Scientific Paper Mode phases should prioritize metadata search planning, deduplication, ranking, and citation-safe evidence preparation."
                    .to_string(),
            );
        }
        "course" => {
            warnings.push(
                "Course Mode will later prioritize local course materials, module context, prerequisites, and learning path support."
                    .to_string(),
            );
            next_required_actions.push(
                "Future Course Mode phases should prioritize local course materials, module context, prerequisites, and learning path support."
                    .to_string(),
            );
        }
        _ => {
            warnings.push(
                "Later Scholar Chat phases should retrieve local evidence before answering."
                    .to_string(),
            );
            next_required_actions.push(
                "Later Scholar Chat phases should retrieve local evidence before answering."
                    .to_string(),
            );
        }
    }

    if !matches!(source_registry_status, ScholarChatScientificSourceRegistryStatus::SourcePlanReady) {
        warnings.push(
            "The source registry preview is not yet ready later for this inferred topic."
                .to_string(),
        );
        next_required_actions.push(
            "Expand discipline and source registry coverage before retrieval planning."
                .to_string(),
        );
    }

    let planned_metadata_queries = if normalized_query.is_empty() {
        Vec::new()
    } else {
        source_preview.planned_metadata_queries.clone()
    };
    let planned_local_search_queries = scientific_query_planned_local_search_queries(
        recognized_concept.as_deref(),
        &normalized_query,
    );
    let planned_expanded_queries = scientific_query_planned_expanded_queries(recognized_concept.as_deref());
    let evidence_requirements = scientific_query_evidence_requirements(
        &normalized_mode,
        &query_intent,
        &source_registry_status,
    );

    let summary = match status {
        ScholarChatScientificQueryUnderstandingStatus::Blocked => {
            "Scientific query understanding preview blocked because the query is blank.".to_string()
        }
        ScholarChatScientificQueryUnderstandingStatus::Ambiguous => format!(
            "Scientific query understanding preview found multiple scientific concepts and prefers {} first.",
            inferred_topic.as_deref().unwrap_or("the first inferred concept")
        ),
        ScholarChatScientificQueryUnderstandingStatus::Understood => format!(
            "Scientific query understanding preview understood '{}' as {} in {} mode.",
            normalized_query,
            inferred_topic.as_deref().unwrap_or("the inferred topic"),
            normalized_mode
        ),
        ScholarChatScientificQueryUnderstandingStatus::UnknownConcept => format!(
            "Scientific query understanding preview could not yet map '{}' to a known scientific concept.",
            normalized_query
        ),
    };

    Ok(ScholarChatScientificQueryUnderstandingPreview {
        status,
        normalized_query,
        normalized_mode,
        normalized_context_tags,
        inferred_topic,
        query_intent,
        ambiguity_level,
        ambiguity_warnings,
        language_hints,
        detected_aliases,
        discipline_status,
        recognized_concept,
        label,
        source_registry_status,
        preferred_source_ids: source_preview.preferred_source_ids,
        conditional_source_ids: source_preview.conditional_source_ids,
        excluded_source_ids: source_preview.excluded_source_ids,
        planned_metadata_queries,
        planned_local_search_queries,
        planned_expanded_queries,
        evidence_requirements,
        ranking_hints: source_preview.ranking_hints,
        deduplication_hints: source_preview.deduplication_hints,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        query_understanding_preview_only: true,
        no_web_request: true,
        no_scraping: true,
        no_connector_call: true,
        no_source_import: true,
        no_local_file_indexing: true,
        no_bm25_index: true,
        no_vector_index: true,
        no_model_loading: true,
        no_runtime_inference: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_evidence_pack_created: true,
        no_artifact_write: true,
        no_persistence: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_scholar_chat_scientific_search_plan(
    root: impl Into<PathBuf>,
    request: ScholarChatScientificSearchPlanRequest,
) -> AegisResult<ScholarChatScientificSearchPlanPreview> {
    let root = root.into();
    let normalized_query = normalize_scientific_query_text(&request.query);
    let normalized_mode = normalize_scientific_mode(request.mode.clone());
    let normalized_context_tags = normalize_scientific_context_tags(request.context_tags.clone());
    let selected_local_source_ids = normalize_scientific_selected_local_source_ids(
        request.selected_local_source_ids.clone(),
    );
    let query_understanding_request = ScholarChatScientificQueryUnderstandingPreviewRequest {
        query: normalized_query.clone(),
        mode: Some(normalized_mode.clone()),
        course_context: request.course_context.clone(),
        context_tags: Some(normalized_context_tags.clone()),
    };

    let query_understanding_preview = preview_scholar_chat_scientific_query_understanding(
        &root,
        query_understanding_request,
    )?;
    let source_registry_status = query_understanding_preview.source_registry_status.clone();
    let query_understanding_status = query_understanding_preview.status.clone();
    let inferred_topic = query_understanding_preview.inferred_topic.clone();
    let recognized_concept = query_understanding_preview.recognized_concept.clone();
    let label = query_understanding_preview.label.clone();
    let query_intent = query_understanding_preview.query_intent.clone();
    let ambiguity_level = query_understanding_preview.ambiguity_level.clone();

    let status = if normalized_query.is_empty() {
        ScholarChatScientificSearchPlanStatus::Blocked
    } else {
        match query_understanding_status {
            ScholarChatScientificQueryUnderstandingStatus::Blocked => {
                ScholarChatScientificSearchPlanStatus::Blocked
            }
            ScholarChatScientificQueryUnderstandingStatus::Ambiguous => {
                ScholarChatScientificSearchPlanStatus::NeedsDisambiguation
            }
            ScholarChatScientificQueryUnderstandingStatus::UnknownConcept => {
                ScholarChatScientificSearchPlanStatus::UnknownConcept
            }
            ScholarChatScientificQueryUnderstandingStatus::Understood => {
                ScholarChatScientificSearchPlanStatus::SearchPlanReady
            }
        }
    };

    let search_strategy = scientific_search_plan_strategy(&status, &normalized_mode, &query_intent);
    let preferred_source_ids = query_understanding_preview.preferred_source_ids.clone();
    let conditional_source_ids = query_understanding_preview.conditional_source_ids.clone();
    let excluded_source_ids = query_understanding_preview.excluded_source_ids.clone();
    let planned_local_queries = query_understanding_preview.planned_local_search_queries.clone();
    let planned_metadata_queries = query_understanding_preview.planned_metadata_queries.clone();
    let planned_expanded_queries = query_understanding_preview.planned_expanded_queries.clone();
    let evidence_requirements = scientific_search_plan_evidence_requirements(
        &normalized_mode,
        &query_intent,
        &source_registry_status,
    );
    let source_routing_plan = scientific_search_plan_source_routing(
        &preferred_source_ids,
        &conditional_source_ids,
        &excluded_source_ids,
    );
    let local_search_plan = scientific_search_plan_local_search(
        &normalized_mode,
        if matches!(status, ScholarChatScientificSearchPlanStatus::Blocked) {
            Vec::new()
        } else {
            selected_local_source_ids.clone()
        },
        planned_local_queries.clone(),
    );
    let metadata_search_plan = scientific_search_plan_metadata(
        &source_registry_status,
        preferred_source_ids.clone(),
        conditional_source_ids.clone(),
        excluded_source_ids.clone(),
        planned_metadata_queries.clone(),
    );
    let planned_search_steps = scientific_search_plan_steps(
        &normalized_mode,
        &status,
        &local_search_plan.selected_local_source_ids,
        &preferred_source_ids,
        &conditional_source_ids,
        &excluded_source_ids,
        &planned_local_queries,
        &planned_metadata_queries,
        &planned_expanded_queries,
        &evidence_requirements,
    );
    let mut blockers = Vec::new();
    let mut warnings = query_understanding_preview.warnings.clone();
    let mut next_required_actions = query_understanding_preview.next_required_actions.clone();

    if normalized_query.is_empty() {
        blockers.push("query_missing: Provide a scientific query to preview the search plan.".to_string());
        next_required_actions.push("Provide a scientific query to preview the scientific search plan.".to_string());
    }

    if matches!(status, ScholarChatScientificSearchPlanStatus::NeedsDisambiguation) {
        warnings.push(
            "The search plan is still preview-only and prefers the first inferred concept until the query is narrowed."
                .to_string(),
        );
        next_required_actions.push(
            "Narrow the scientific concept before execution phases can plan retrieval in more detail."
                .to_string(),
        );
    }

    if matches!(status, ScholarChatScientificSearchPlanStatus::UnknownConcept) {
        warnings.push(
            "The query still does not map to a known scientific concept for full search planning."
                .to_string(),
        );
        next_required_actions.push(
            "Add discipline and source registry mappings before metadata routing can be refined."
                .to_string(),
        );
    }

    if !normalized_query.is_empty() && matches!(normalized_mode.as_str(), "scholar_chat" | "course") {
        if local_search_plan.selected_local_source_ids.is_empty() {
            warnings.push("No local sources selected.".to_string());
            next_required_actions.push(
                "Select or import local sources in a later phase before answering."
                    .to_string(),
            );
        } else {
            warnings.push(
                "Local source selection is preview-only; no files are read and no indexes are built."
                    .to_string(),
            );
        }
    }

    if !normalized_query.is_empty()
        && matches!(source_registry_status, ScholarChatScientificSourceRegistryStatus::Blocked)
    {
        blockers.push(
            "source_family_plan_required: The source registry preview must be ready before metadata routing can be executed later."
                .to_string(),
        );
    } else if matches!(source_registry_status, ScholarChatScientificSourceRegistryStatus::UnknownConcept) {
        warnings.push(
            "The source registry preview is not yet ready later for this inferred concept.".to_string(),
        );
        next_required_actions.push(
            "Add discipline and source registry mappings before metadata routing is planned in later phases."
                .to_string(),
        );
    }

    let summary = match status {
        ScholarChatScientificSearchPlanStatus::Blocked => {
            "Scientific search plan preview blocked because the query is blank or a required prerequisite is missing.".to_string()
        }
        ScholarChatScientificSearchPlanStatus::NeedsDisambiguation => {
            "Scientific search plan preview needs disambiguation before later retrieval planning can narrow the route.".to_string()
        }
        ScholarChatScientificSearchPlanStatus::UnknownConcept => {
            "Scientific search plan preview found an unknown concept and can only outline later planning steps.".to_string()
        }
        ScholarChatScientificSearchPlanStatus::SearchPlanReady => {
            "Scientific search plan preview is ready later and only describes future local and metadata search planning.".to_string()
        }
    };

    Ok(ScholarChatScientificSearchPlanPreview {
        status,
        normalized_query,
        normalized_mode,
        normalized_context_tags,
        selected_local_source_ids: local_search_plan.selected_local_source_ids.clone(),
        query_understanding_status,
        inferred_topic,
        query_intent,
        ambiguity_level,
        recognized_concept,
        label,
        source_registry_status,
        search_strategy,
        local_search_plan,
        metadata_search_plan,
        source_routing_plan,
        planned_search_steps,
        planned_local_queries,
        planned_metadata_queries,
        planned_expanded_queries,
        preferred_source_ids,
        conditional_source_ids,
        excluded_source_ids,
        evidence_requirements,
        ranking_hints: query_understanding_preview.ranking_hints.clone(),
        deduplication_hints: query_understanding_preview.deduplication_hints.clone(),
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        scientific_search_plan_preview_only: true,
        no_web_request: true,
        no_scraping: true,
        no_connector_call: true,
        no_source_import: true,
        no_local_file_indexing: true,
        no_bm25_index: true,
        no_vector_index: true,
        no_model_loading: true,
        no_runtime_inference: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_evidence_pack_created: true,
        no_artifact_write: true,
        no_persistence: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_scholar_chat_local_literature_index(
    root: impl Into<PathBuf>,
    request: ScholarChatLocalLiteratureIndexRequest,
) -> AegisResult<ScholarChatLocalLiteratureIndexPreview> {
    let root = root.into();
    let normalized_query = normalize_scientific_query_text(&request.query);
    let normalized_mode = normalize_scientific_mode(request.mode.clone());
    let normalized_context_tags = normalize_scientific_context_tags(request.context_tags.clone());
    let selected_local_source_ids = normalize_scientific_selected_local_source_ids(
        request.selected_local_source_ids.clone(),
    );
    let expected_source_kinds = normalize_scientific_expected_source_kinds(
        request.expected_source_kinds.clone(),
    );
    let search_plan_preview = preview_scholar_chat_scientific_search_plan(
        &root,
        ScholarChatScientificSearchPlanRequest {
            query: normalized_query.clone(),
            mode: Some(normalized_mode.clone()),
            course_context: request.course_context.clone(),
            context_tags: Some(normalized_context_tags.clone()),
            selected_local_source_ids: Some(selected_local_source_ids.clone()),
        },
    )?;
    let search_plan_status = search_plan_preview.status.clone();
    let query_understanding_status = search_plan_preview.query_understanding_status.clone();
    let inferred_topic = search_plan_preview.inferred_topic.clone();
    let query_intent = search_plan_preview.query_intent.clone();
    let recognized_concept = search_plan_preview.recognized_concept.clone();
    let label = search_plan_preview.label.clone();
    let planned_local_queries = scientific_local_literature_index_local_queries(
        &search_plan_preview.planned_local_queries,
        &normalized_query,
    );
    let planned_metadata_requirements = scientific_local_literature_index_metadata_requirements(
        &normalized_mode,
    );
    let status = if normalized_query.is_empty() {
        ScholarChatLocalLiteratureIndexStatus::Blocked
    } else {
        scientific_local_literature_index_status(&search_plan_status, &selected_local_source_ids)
    };
    let local_index_strategy =
        scientific_local_literature_index_strategy(&status, &normalized_mode);
    let local_corpus_plan = ScholarChatLocalLiteratureCorpusPlan {
        selected_source_count: selected_local_source_ids.len(),
        selected_local_source_ids: if matches!(status, ScholarChatLocalLiteratureIndexStatus::Blocked) {
            Vec::new()
        } else {
            selected_local_source_ids.clone()
        },
        expected_source_kinds: expected_source_kinds.clone(),
        corpus_manifest_would_be_required: !selected_local_source_ids.is_empty(),
        will_create_corpus: false,
        will_read_files: false,
        summary: if selected_local_source_ids.is_empty() {
            "Local corpus planning is preview-only; no local files are read and no corpus is created.".to_string()
        } else {
            "Local corpus planning is preview-only; selected local sources are noted without reading files or creating a corpus."
                .to_string()
        },
    };
    let index_artifact_plan = ScholarChatLocalLiteratureIndexArtifactPlan {
        planned_artifact_ids: scientific_local_literature_index_planned_artifact_ids(),
        planned_artifact_descriptions: scientific_local_literature_index_planned_artifact_descriptions(),
        will_create_artifacts: false,
        will_create_bm25_index: false,
        will_create_vector_index: false,
        will_generate_embeddings: false,
        summary: "Index artifacts are preview-only; no artifacts, BM25 index, vector index, or embeddings are created."
            .to_string(),
    };
    let ingestion_readiness = match status {
        ScholarChatLocalLiteratureIndexStatus::Blocked => {
            ScholarChatLocalLiteratureIngestionReadiness::Blocked
        }
        ScholarChatLocalLiteratureIndexStatus::NeedsDisambiguation => {
            ScholarChatLocalLiteratureIngestionReadiness::NeedsDisambiguation
        }
        ScholarChatLocalLiteratureIndexStatus::UnknownConcept => {
            ScholarChatLocalLiteratureIngestionReadiness::UnknownConceptMappingNeeded
        }
        ScholarChatLocalLiteratureIndexStatus::NeedsLocalSources => {
            ScholarChatLocalLiteratureIngestionReadiness::NeedsSources
        }
        ScholarChatLocalLiteratureIndexStatus::IndexPlanReady => {
            ScholarChatLocalLiteratureIngestionReadiness::PreviewReady
        }
    };
    let _source_registry_status = search_plan_preview.source_registry_status.clone();
    let mut blockers = Vec::new();
    let mut warnings = search_plan_preview.warnings.clone();
    let mut next_required_actions = search_plan_preview.next_required_actions.clone();

    if normalized_query.is_empty() {
        blockers.push("query_missing: Provide a scientific query to preview the local literature index.".to_string());
        next_required_actions.push("Provide a scientific query to preview the local literature index.".to_string());
    }

    if matches!(status, ScholarChatLocalLiteratureIndexStatus::NeedsLocalSources) {
        warnings.push("No local sources selected.".to_string());
        next_required_actions.push(
            "Select or import local sources in a later phase before indexing."
                .to_string(),
        );
    }

    if matches!(status, ScholarChatLocalLiteratureIndexStatus::NeedsDisambiguation) {
        warnings.push(
            "The local literature index preview still needs disambiguation before later corpus planning can narrow the route."
                .to_string(),
        );
        next_required_actions.push(
            "Narrow the scientific concept before local index planning can continue."
                .to_string(),
        );
    }

    if matches!(status, ScholarChatLocalLiteratureIndexStatus::UnknownConcept) {
        warnings.push(
            "The query still does not map to a known scientific concept for local literature indexing."
                .to_string(),
        );
        next_required_actions.push(
            "Add discipline and source registry mappings before local index planning can continue."
                .to_string(),
        );
    }

    let known_source_kinds = [
        "pdf",
        "markdown",
        "text",
        "course_material",
        "lecture_slide",
        "article",
        "book_chapter",
        "notes",
        "unknown",
    ]
    .iter()
    .copied()
    .collect::<BTreeSet<_>>();
    let unknown_expected_source_kinds = expected_source_kinds
        .iter()
        .filter(|kind| !known_source_kinds.contains(kind.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown_expected_source_kinds.is_empty() {
        warnings.push(format!(
            "Unknown expected source kinds are preserved as preview-only hints: {}.",
            unknown_expected_source_kinds.join(", ")
        ));
    }

    let planned_index_fields = scientific_local_literature_index_planned_index_fields();
    let planned_chunking_policy = scientific_local_literature_index_chunking_policy();
    let local_corpus_plan_inputs = local_corpus_plan
        .selected_local_source_ids
        .iter()
        .chain(local_corpus_plan.expected_source_kinds.iter())
        .cloned()
        .collect::<Vec<_>>();
    let local_corpus_plan_outputs = vec![
        "local_corpus_manifest_preview".to_string(),
        "local_literature_metadata_map_preview".to_string(),
    ];
    let step_active = !matches!(status, ScholarChatLocalLiteratureIndexStatus::Blocked);
    let planned_index_steps = vec![
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::SourceSelectionReview,
            "source_selection_review",
            "Source selection review",
            "Review local source selections only; no filesystem validation is performed.",
            vec![
                normalized_query.clone(),
                normalized_mode.clone(),
            ],
            local_corpus_plan.selected_local_source_ids.clone(),
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::MetadataRequirementCheck,
            "metadata_requirement_check",
            "Metadata requirement check",
            "Review metadata requirements only; no files are read and no corpus metadata is written.",
            vec![normalized_mode.clone()],
            planned_metadata_requirements.clone(),
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::CorpusManifestPlan,
            "corpus_manifest_plan",
            "Corpus manifest plan",
            "Plan a future corpus manifest only; no corpus is created and no files are read.",
            local_corpus_plan_inputs.clone(),
            local_corpus_plan_outputs.clone(),
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::ExtractionPlan,
            "extraction_plan",
            "Extraction plan",
            "Plan future extraction only; no PDF extraction or file reading occurs.",
            local_corpus_plan.selected_local_source_ids.clone(),
            vec!["extracted_text_preview".to_string()],
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::ChunkingPolicyPlan,
            "chunking_policy_plan",
            "Chunking policy plan",
            "Plan future chunking only; no chunking run, OCR, or extraction occurs.",
            planned_chunking_policy.clone(),
            vec!["chunk_policy_preview".to_string()],
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::LexicalIndexPlan,
            "lexical_index_plan",
            "Lexical index plan",
            "Plan future lexical indexing only; no BM25 index is created.",
            planned_local_queries.clone(),
            vec!["bm25_index_plan_preview".to_string()],
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::VectorIndexPlan,
            "vector_index_plan",
            "Vector index plan",
            "Plan future vector indexing only; no embeddings are generated.",
            planned_local_queries.clone(),
            vec!["vector_index_plan_preview".to_string()],
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::DeduplicationPlan,
            "deduplication_plan",
            "Deduplication plan",
            "Plan future deduplication only; no retrieval or evidence artifacts are created.",
            planned_metadata_requirements.clone(),
            vec!["deduplicated_literature_preview".to_string()],
            step_active,
        ),
        scientific_local_literature_index_step(
            ScholarChatLocalLiteratureIndexStepKind::RetrievalReadinessCheck,
            "retrieval_readiness_check",
            "Retrieval readiness check",
            "Plan later retrieval readiness only; no retrieval is executed.",
            vec![
                "search_plan_ready".to_string(),
                format!("search_plan_status={:?}", search_plan_status),
            ],
            vec!["retrieval_readiness_preview".to_string()],
            step_active,
        ),
    ];

    let summary = match status {
        ScholarChatLocalLiteratureIndexStatus::Blocked => {
            "Local literature index preview blocked because the query is blank or a prerequisite is missing."
                .to_string()
        }
        ScholarChatLocalLiteratureIndexStatus::NeedsDisambiguation => {
            "Local literature index preview needs disambiguation before later corpus planning can continue."
                .to_string()
        }
        ScholarChatLocalLiteratureIndexStatus::UnknownConcept => {
            "Local literature index preview found an unknown concept and can only outline later indexing steps."
                .to_string()
        }
        ScholarChatLocalLiteratureIndexStatus::NeedsLocalSources => {
            "Local literature index preview is ready later but still needs local sources before indexing can proceed."
                .to_string()
        }
        ScholarChatLocalLiteratureIndexStatus::IndexPlanReady => {
            "Local literature index preview is ready later and only describes future corpus, chunking, and indexing planning."
                .to_string()
        }
    };

    Ok(ScholarChatLocalLiteratureIndexPreview {
        status,
        normalized_query,
        normalized_mode,
        normalized_context_tags,
        selected_local_source_ids: local_corpus_plan.selected_local_source_ids.clone(),
        expected_source_kinds,
        search_plan_status,
        query_understanding_status,
        inferred_topic,
        query_intent,
        recognized_concept,
        label,
        local_index_strategy,
        local_corpus_plan,
        index_artifact_plan,
        ingestion_readiness,
        planned_index_fields,
        planned_chunking_policy,
        planned_metadata_requirements,
        planned_local_queries,
        planned_index_steps,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        local_literature_index_preview_only: true,
        no_file_read: true,
        no_pdf_extraction: true,
        no_ocr: true,
        no_chunking_run: true,
        no_embedding_generation: true,
        no_index_created: true,
        no_bm25_index: true,
        no_vector_index: true,
        no_retrieval_execution: true,
        no_web_request: true,
        no_scraping: true,
        no_connector_call: true,
        no_source_import: true,
        no_local_file_indexing: true,
        no_model_loading: true,
        no_runtime_inference: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_evidence_pack_created: true,
        no_artifact_write: true,
        no_persistence: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_scholar_chat_course_literature_registry(
    root: impl Into<PathBuf>,
    request: ScholarChatCourseLiteratureRegistryPreviewRequest,
) -> AegisResult<ScholarChatCourseLiteratureRegistryPreview> {
    let root = root.into();
    let normalized_query = normalize_scientific_query_text(&request.query);
    let normalized_course_context = normalize_course_optional_text(request.course_context);
    let normalized_module_code = normalize_course_optional_text(request.module_code);
    let normalized_course_title = normalize_course_optional_text(request.course_title);
    let normalized_instructor = normalize_course_optional_text(request.instructor);
    let normalized_semester = normalize_course_optional_text(request.semester);
    let normalized_context_tags = normalize_scientific_context_tags(request.context_tags.clone());
    let selected_local_source_ids = normalize_scientific_selected_local_source_ids(
        request.selected_local_source_ids.clone(),
    );
    let expected_course_material_kinds = normalize_course_material_kinds(
        request.expected_course_material_kinds.clone(),
    );
    let local_literature_index_preview = preview_scholar_chat_local_literature_index(
        &root,
        ScholarChatLocalLiteratureIndexRequest {
            query: normalized_query.clone(),
            mode: Some("course".to_string()),
            course_context: normalized_course_context.clone(),
            context_tags: Some(normalized_context_tags.clone()),
            selected_local_source_ids: Some(selected_local_source_ids.clone()),
            expected_source_kinds: Some(expected_course_material_kinds.clone()),
        },
    )?;

    let local_literature_index_status = local_literature_index_preview.status.clone();
    let search_plan_status = local_literature_index_preview.search_plan_status.clone();
    let query_understanding_status = local_literature_index_preview.query_understanding_status.clone();
    let inferred_topic = local_literature_index_preview.inferred_topic.clone();
    let query_intent = local_literature_index_preview.query_intent.clone();
    let recognized_concept = local_literature_index_preview.recognized_concept.clone();
    let label = local_literature_index_preview.label.clone();
    let course_identity = course_literature_registry_course_identity(
        normalized_course_context.clone(),
        normalized_module_code.clone(),
        normalized_course_title.clone(),
        normalized_instructor.clone(),
        normalized_semester.clone(),
    );
    let course_material_plan = course_literature_registry_course_material_plan(
        selected_local_source_ids.clone(),
        expected_course_material_kinds.clone(),
    );
    let curriculum_alignment_plan = course_literature_registry_curriculum_alignment_plan(
        &normalized_course_context,
        &normalized_module_code,
    );
    let planned_course_metadata_requirements =
        course_literature_registry_planned_course_metadata_requirements(
            &course_identity,
            &course_material_plan,
        );
    let planned_course_material_queries = course_literature_registry_planned_course_material_queries(
        &normalized_query,
        &normalized_module_code,
        &normalized_course_title,
        &local_literature_index_preview.planned_local_queries,
    );
    let status = course_literature_registry_status(
        &local_literature_index_status,
        &normalized_course_context,
        &normalized_module_code,
        &normalized_course_title,
        &selected_local_source_ids,
    );
    let course_registry_strategy = course_literature_registry_strategy(
        &status,
        &normalized_module_code,
        &selected_local_source_ids,
    );
    let step_active = !matches!(status, ScholarChatCourseLiteratureRegistryStatus::Blocked);
    let planned_registry_steps = vec![
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::CourseIdentityReview,
            "course_identity_review",
            "Course identity review",
            "Review course context, module code, title, instructor, and semester only; no course files are read.",
            vec![
                normalized_course_context.clone().unwrap_or_default(),
                normalized_module_code.clone().unwrap_or_default(),
                normalized_course_title.clone().unwrap_or_default(),
                normalized_instructor.clone().unwrap_or_default(),
                normalized_semester.clone().unwrap_or_default(),
            ]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect(),
            vec!["course_identity_preview".to_string()],
            step_active,
        ),
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::ModuleContextReview,
            "module_context_review",
            "Module context review",
            "Review module context only; no curriculum scraping or local file inspection occurs.",
            vec![
                normalized_module_code.clone().unwrap_or_default(),
                normalized_course_title.clone().unwrap_or_default(),
                normalized_semester.clone().unwrap_or_default(),
                normalized_instructor.clone().unwrap_or_default(),
            ]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect(),
            vec!["module_context_preview".to_string()],
            step_active,
        ),
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::CourseMaterialKindPlan,
            "course_material_kind_plan",
            "Course material kind plan",
            "Plan course material kinds only; no files are read, imported, or registered.",
            vec![
                course_material_plan.expected_course_material_kinds.join(", "),
                course_material_plan.known_material_kinds.join(", "),
                course_material_plan.unknown_material_kinds.join(", "),
            ]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect(),
            vec!["course_material_kind_plan_preview".to_string()],
            step_active,
        ),
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::LocalSourceAlignmentPlan,
            "local_source_alignment_plan",
            "Local source alignment plan",
            "Plan local course-source alignment only; no source validation or registry creation occurs.",
            vec![
                course_material_plan.selected_local_source_ids.join(", "),
                format!("selected_source_count={}", course_material_plan.selected_source_count),
            ],
            vec!["local_source_alignment_preview".to_string()],
            step_active,
        ),
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::CurriculumMetadataRequirementCheck,
            "curriculum_metadata_requirement_check",
            "Curriculum metadata requirement check",
            "Review curriculum metadata requirements only; no scraping or connector calls occur.",
            planned_course_metadata_requirements.clone(),
            vec!["curriculum_alignment_preview".to_string()],
            step_active,
        ),
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::LocalLiteratureIndexAlignment,
            "local_literature_index_alignment",
            "Local literature index alignment",
            "Align course planning with the local literature index preview only; no index creation or retrieval execution occurs.",
            vec![
                format!("local_literature_index_status={:?}", local_literature_index_status),
                format!("search_plan_status={:?}", search_plan_status),
            ],
            planned_course_material_queries.clone(),
            step_active,
        ),
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::RetrievalReadinessCheck,
            "retrieval_readiness_check",
            "Retrieval readiness check",
            "Check later retrieval readiness only; no retrieval is executed.",
            vec![
                format!("search_plan_status={:?}", search_plan_status),
                format!("selected_source_count={}", selected_local_source_ids.len()),
            ],
            vec!["retrieval_readiness_preview".to_string()],
            step_active,
        ),
        course_literature_registry_step(
            ScholarChatCourseLiteratureRegistryStepKind::LearningPathAlignmentPlan,
            "learning_path_alignment_plan",
            "Learning path alignment plan",
            "Plan learning-path alignment only; no answer generation or curriculum execution occurs.",
            vec![
                normalized_course_context.clone().unwrap_or_default(),
                normalized_course_title.clone().unwrap_or_default(),
                normalized_module_code.clone().unwrap_or_default(),
                normalized_instructor.clone().unwrap_or_default(),
                normalized_semester.clone().unwrap_or_default(),
                format!("{:?}", query_intent),
                recognized_concept.clone().unwrap_or_default(),
            ]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect(),
            vec!["learning_path_alignment_preview".to_string()],
            step_active,
        ),
    ];

    let mut blockers = local_literature_index_preview.blockers.clone();
    let mut warnings = local_literature_index_preview.warnings.clone();
    let mut next_required_actions = local_literature_index_preview.next_required_actions.clone();

    if normalized_query.is_empty() {
        push_unique_string(
            &mut blockers,
            "query_missing: Provide a course or scientific query to preview the course literature registry."
                .to_string(),
        );
        push_unique_string(
            &mut next_required_actions,
            "Provide a course or scientific query to preview the course literature registry."
                .to_string(),
        );
    }

    if matches!(status, ScholarChatCourseLiteratureRegistryStatus::NeedsCourseContext) {
        push_unique_string(
            &mut warnings,
            "No course context, module code, or course title was provided.".to_string(),
        );
        push_unique_string(
            &mut next_required_actions,
            "Provide course context, module code, or course title.".to_string(),
        );
    }

    if matches!(status, ScholarChatCourseLiteratureRegistryStatus::NeedsLocalSources) {
        push_unique_string(
            &mut warnings,
            "No local course sources selected.".to_string(),
        );
        push_unique_string(
            &mut next_required_actions,
            "Select or import local course material sources later before registry creation."
                .to_string(),
        );
    }

    if matches!(status, ScholarChatCourseLiteratureRegistryStatus::NeedsDisambiguation) {
        push_unique_string(
            &mut warnings,
            "The local literature index preview still needs disambiguation before course registry planning can continue."
                .to_string(),
        );
        push_unique_string(
            &mut next_required_actions,
            "Narrow the scientific concept before course registry planning can continue."
                .to_string(),
        );
    }

    if matches!(status, ScholarChatCourseLiteratureRegistryStatus::UnknownConcept) {
        push_unique_string(
            &mut warnings,
            "The query still does not map to a known scientific concept for course registry planning."
                .to_string(),
        );
        push_unique_string(
            &mut next_required_actions,
            "Add discipline and source registry mappings before course registry planning can continue."
                .to_string(),
        );
    }

    let unknown_expected_material_kinds = course_material_plan
        .unknown_material_kinds
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    if !unknown_expected_material_kinds.is_empty() {
        push_unique_string(
            &mut warnings,
            format!(
                "Unknown course material kinds are preserved as preview-only hints: {}.",
                unknown_expected_material_kinds.join(", ")
            ),
        );
    }

    let summary = match status {
        ScholarChatCourseLiteratureRegistryStatus::Blocked => {
            "Course literature registry preview blocked because the query is blank or the local literature index preview is blocked.".to_string()
        }
        ScholarChatCourseLiteratureRegistryStatus::NeedsCourseContext => {
            "Course literature registry preview needs course context, module code, or course title before later registry planning can continue.".to_string()
        }
        ScholarChatCourseLiteratureRegistryStatus::NeedsLocalSources => {
            "Course literature registry preview is ready later but still needs local course sources before registry alignment can proceed.".to_string()
        }
        ScholarChatCourseLiteratureRegistryStatus::NeedsDisambiguation => {
            "Course literature registry preview needs disambiguation before later course registry planning can continue.".to_string()
        }
        ScholarChatCourseLiteratureRegistryStatus::UnknownConcept => {
            "Course literature registry preview found an unknown concept and can only outline later course registry planning steps.".to_string()
        }
        ScholarChatCourseLiteratureRegistryStatus::CourseRegistryPlanReady => {
            "Course literature registry preview is ready later and only describes future course identity, course-material alignment, curriculum metadata, and learning-path planning.".to_string()
        }
    };

    Ok(ScholarChatCourseLiteratureRegistryPreview {
        status,
        normalized_query,
        normalized_course_context,
        normalized_module_code,
        normalized_course_title,
        normalized_instructor,
        normalized_semester,
        normalized_context_tags,
        selected_local_source_ids: selected_local_source_ids.clone(),
        expected_course_material_kinds: expected_course_material_kinds.clone(),
        local_literature_index_status,
        search_plan_status,
        query_understanding_status,
        inferred_topic,
        query_intent,
        recognized_concept,
        label,
        course_registry_strategy,
        course_identity,
        course_material_plan,
        curriculum_alignment_plan,
        planned_course_metadata_requirements,
        planned_course_material_queries,
        planned_registry_steps,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        course_literature_registry_preview_only: true,
        no_file_read: true,
        no_pdf_extraction: true,
        no_ocr: true,
        no_chunking_run: true,
        no_embedding_generation: true,
        no_index_created: true,
        no_retrieval_execution: true,
        no_web_request: true,
        no_scraping: true,
        no_connector_call: true,
        no_source_import: true,
        no_local_file_indexing: true,
        no_model_loading: true,
        no_runtime_inference: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_evidence_pack_created: true,
        no_artifact_write: true,
        no_persistence: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_scholar_chat_answer_readiness(
    root: impl Into<PathBuf>,
    request: ScholarChatAnswerReadinessRequest,
) -> AegisResult<ScholarChatAnswerReadinessPreview> {
    let root = root.into();
    let scholar_chat_request = request.scholar_chat_request;
    let request_preview = preview_scholar_chat_request(&root, scholar_chat_request.clone())?;
    let retrieval_preview = preview_scholar_chat_retrieval(&root, scholar_chat_request.clone())?;
    let evidence_plan_preview = preview_scholar_chat_evidence_plan(&root, scholar_chat_request.clone())?;
    let prompt_pack_preview = preview_scholar_chat_prompt_pack(&root, scholar_chat_request.clone())?;
    let runtime_health_preview = preview_local_model_runtime_health(&root, request.runtime_config.clone())?;
    let invocation_plan_preview = preview_local_runtime_invocation_plan(
        &root,
        LocalRuntimeInvocationPlanRequest {
            runtime_config: request.runtime_config,
            prompt_text: Some(request_preview.normalized_prompt.clone()),
            estimated_input_char_count: Some(request_preview.normalized_prompt.chars().count() as u32),
            max_output_tokens: Some(1),
            stop_sequences: None,
        },
    )?;

    let selected_source_count = request_preview.selected_source_count;
    let retrieval_candidate_count = retrieval_preview.candidate_count;
    let evidence_candidate_count = evidence_plan_preview.evidence_candidate_count;
    let prompt_pack_ready = prompt_pack_preview.prompt_pack.section_count > 0;
    let runtime_ready = matches!(runtime_health_preview.status, LocalModelRuntimeHealthStatus::ReadyToTestLater);
    let invocation_ready = matches!(invocation_plan_preview.status, LocalRuntimeInvocationPlanStatus::ReadyToInvokeLater);

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut next_required_actions = Vec::new();

    for warning in request_preview.warnings {
        push_readiness_warning(&mut warnings, "request_preview", &warning);
    }
    for warning in retrieval_preview.warnings {
        push_readiness_warning(&mut warnings, "retrieval_preview", &warning);
    }
    for warning in evidence_plan_preview.warnings {
        push_readiness_warning(&mut warnings, "evidence_plan_preview", &warning);
    }
    for warning in prompt_pack_preview.warnings {
        push_readiness_warning(&mut warnings, "prompt_pack_preview", &warning);
    }
    for warning in runtime_health_preview.warnings {
        push_readiness_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for warning in invocation_plan_preview.plan.warnings {
        push_readiness_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for blocker in invocation_plan_preview.plan.blockers {
        push_readiness_blocker(&mut blockers, &blocker.kind, &blocker.message);
    }

    if selected_source_count == 0 {
        push_readiness_action(
            &mut next_required_actions,
            "Select one or more Scholar Chat sources.",
        );
        if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
            push_readiness_blocker(
                &mut blockers,
                "needs_sources",
                "local_only requires selected sources before a grounded draft can be prepared.",
            );
        }
    }

    if selected_source_count > 0 && retrieval_candidate_count == 0 {
        push_readiness_action(
            &mut next_required_actions,
            "Build or refresh the retrieval index for the selected sources.",
        );
        if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
            push_readiness_blocker(
                &mut blockers,
                "blocked",
                "local_only requires local evidence before a grounded draft can be prepared.",
            );
        } else {
            push_readiness_blocker(
                &mut blockers,
                "needs_retrieval_index",
                "Retrieval data is not ready for the selected sources yet.",
            );
        }
    }

    if selected_source_count > 0 && evidence_candidate_count == 0 {
        push_readiness_action(
            &mut next_required_actions,
            "Assemble local evidence candidates for the selected sources.",
        );
        if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
            push_readiness_blocker(
                &mut blockers,
                "blocked",
                "local_only requires local evidence before a grounded draft can be prepared.",
            );
        } else {
            push_readiness_blocker(
                &mut blockers,
                "needs_evidence_candidates",
                "No evidence candidates are available yet for the selected sources.",
            );
        }
    }

    if !runtime_ready {
        push_readiness_action(
            &mut next_required_actions,
            "Configure a local runtime model and executable.",
        );
        push_readiness_blocker(
            &mut blockers,
            "needs_runtime_config",
            "The local runtime configuration is not ready yet.",
        );
    }

    if !request.allow_model_execution {
        push_readiness_action(
            &mut next_required_actions,
            "Allow future model execution when you are ready to proceed.",
        );
        push_readiness_blocker(
            &mut blockers,
            "needs_execution_consent",
            "Future model execution is not allowed yet.",
        );
    }

    if prompt_pack_ready {
        push_readiness_action(
            &mut next_required_actions,
            "The prompt pack can be assembled later from the current request preview.",
        );
    }

    let status = readiness_status(
        request_preview.grounding_policy.clone(),
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        runtime_ready,
        invocation_ready,
        request.allow_model_execution,
    );

    if matches!(request_preview.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge)
        && runtime_ready
        && invocation_ready
        && request.allow_model_execution
    {
        push_readiness_warning(
            &mut warnings,
            "future_draft_marking_required",
            "A future ungrounded draft would need explicit model-knowledge marking later.",
        );
    }

    Ok(ScholarChatAnswerReadinessPreview {
        status: status.clone(),
        normalized_prompt: request_preview.normalized_prompt,
        mode: request_preview.mode,
        grounding_policy: request_preview.grounding_policy,
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        prompt_pack_ready,
        runtime_health_status: runtime_health_preview.status,
        invocation_plan_status: invocation_plan_preview.status,
        allow_model_execution: request.allow_model_execution,
        would_generate_answer_now: false,
        would_build_evidence_pack_now: false,
        would_create_final_answer_now: false,
        future_output_classification: readiness_output_classification(status),
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_draft_inference(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftInferenceRequest,
) -> AegisResult<ScholarChatDraftInferencePreview> {
    let root = root.into();
    let scholar_chat_request = request.scholar_chat_request;
    let request_preview = preview_scholar_chat_request(&root, scholar_chat_request.clone())?;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    for warning in &request_preview.warnings {
        push_draft_warning(&mut warnings, "request_preview", warning);
    }

    if matches!(request_preview.grounding_policy, GroundingPolicy::ExternalAdaptersLater) {
        push_draft_warning(
            &mut warnings,
            "external_adapters_unavailable",
            "External adapters are not implemented yet and are unused in this preview.",
        );
    }

    if !request.allow_model_execution {
        push_draft_blocker(
            &mut blockers,
            "needs_execution_consent",
            "Future model execution is not allowed yet.",
        );
        push_draft_warning(
            &mut warnings,
            "execution_consent_required",
            "Draft inference preview will not consult the local runtime until execution consent is granted.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            0,
            0,
            0,
            String::new(),
            LocalModelRuntimeHealthStatus::NotConfigured,
            LocalRuntimeInvocationPlanStatus::PreviewOnly,
            ScholarChatDraftInferenceStatus::NeedsExecutionConsent,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            None,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let prompt_ready = request_preview.selected_source_count > 0
        || matches!(request_preview.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge);
    if !prompt_ready {
        push_draft_blocker(
            &mut blockers,
            "needs_sources",
            "Selected sources are required before a local draft can be prepared.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            0,
            0,
            0,
            String::new(),
            LocalModelRuntimeHealthStatus::NotConfigured,
            LocalRuntimeInvocationPlanStatus::PreviewOnly,
            ScholarChatDraftInferenceStatus::NeedsSources,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            None,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let runtime_health_preview = preview_local_model_runtime_health(&root, request.runtime_config.clone())?;
    let runtime_health_status = runtime_health_preview.status.clone();
    for warning in &runtime_health_preview.warnings {
        push_draft_warning(&mut warnings, &warning.kind, &warning.message);
    }
    if !matches!(runtime_health_status, LocalModelRuntimeHealthStatus::ReadyToTestLater) {
        push_draft_blocker(
            &mut blockers,
            "needs_runtime_config",
            "The local runtime configuration is not ready yet.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            0,
            0,
            0,
            String::new(),
            runtime_health_status,
            LocalRuntimeInvocationPlanStatus::NotConfigured,
            ScholarChatDraftInferenceStatus::NeedsRuntimeConfig,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            runtime_health_preview.model_file_name,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let evidence_plan_preview = if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
        Some(preview_scholar_chat_evidence_plan(&root, scholar_chat_request.clone())?)
    } else {
        None
    };
    let retrieval_candidate_count = evidence_plan_preview
        .as_ref()
        .map_or(0, |preview| preview.retrieval_candidate_count);
    let evidence_candidate_count = evidence_plan_preview
        .as_ref()
        .map_or(0, |preview| preview.evidence_candidate_count);

    if let Some(evidence_plan_preview) = &evidence_plan_preview {
        for warning in &evidence_plan_preview.warnings {
            push_draft_warning(&mut warnings, "evidence_plan_preview", warning);
        }
    }

    if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) && evidence_candidate_count == 0 {
        push_draft_blocker(
            &mut blockers,
            "needs_evidence",
            "local_only requires local evidence candidates before draft inference can proceed.",
        );
        push_draft_warning(
            &mut warnings,
            "evidence_required",
            "No local evidence candidates are available for a local_only draft preview.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            retrieval_candidate_count,
            evidence_candidate_count,
            0,
            String::new(),
            runtime_health_status,
            LocalRuntimeInvocationPlanStatus::PreviewOnly,
            ScholarChatDraftInferenceStatus::NeedsEvidence,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            runtime_health_preview.model_file_name,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let prompt_pack_preview = preview_scholar_chat_prompt_pack(&root, scholar_chat_request.clone())?;
    let prompt_pack_text = render_prompt_pack_for_runtime(&prompt_pack_preview.prompt_pack);
    let invocation_plan_preview = preview_local_runtime_invocation_plan(
        &root,
        LocalRuntimeInvocationPlanRequest {
            runtime_config: request.runtime_config.clone(),
            prompt_text: Some(prompt_pack_text.clone()),
            estimated_input_char_count: Some(prompt_pack_text.chars().count() as u32),
            max_output_tokens: request.max_output_tokens,
            stop_sequences: None,
        },
    )?;
    let invocation_plan_status = invocation_plan_preview.status.clone();
    for warning in &prompt_pack_preview.warnings {
        push_draft_warning(&mut warnings, "prompt_pack_preview", warning);
    }
    for warning in &invocation_plan_preview.plan.warnings {
        push_draft_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for blocker in &invocation_plan_preview.plan.blockers {
        push_draft_blocker(&mut blockers, &blocker.kind, &blocker.message);
    }

    let output_classification = draft_output_classification(&request_preview.grounding_policy, false);

    let smoke_result = smoke_test_local_runtime_inference(
        &root,
        LocalRuntimeSmokeInferenceRequest {
            runtime_config: request.runtime_config,
            allow_execution: true,
            prompt: Some(prompt_pack_text.clone()),
            timeout_ms: request.timeout_ms,
            max_output_tokens: request.max_output_tokens,
        },
    )?;
    for warning in smoke_result.warnings.iter() {
        push_draft_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for blocker in smoke_result.blockers.iter() {
        push_draft_blocker(&mut blockers, &blocker.kind, &blocker.message);
    }

    let status = match smoke_result.status {
        LocalRuntimeSmokeInferenceStatus::InferenceSucceeded => ScholarChatDraftInferenceStatus::InferenceSucceeded,
        LocalRuntimeSmokeInferenceStatus::InferenceFailed => ScholarChatDraftInferenceStatus::InferenceFailed,
        LocalRuntimeSmokeInferenceStatus::TimedOut => ScholarChatDraftInferenceStatus::TimedOut,
        LocalRuntimeSmokeInferenceStatus::Blocked => ScholarChatDraftInferenceStatus::Blocked,
        LocalRuntimeSmokeInferenceStatus::NotConfigured
        | LocalRuntimeSmokeInferenceStatus::ModelMissing
        | LocalRuntimeSmokeInferenceStatus::ExecutableMissing => ScholarChatDraftInferenceStatus::NeedsRuntimeConfig,
    };

    Ok(build_draft_inference_preview(
        request_preview.normalized_prompt,
        request_preview.mode,
        request_preview.grounding_policy,
        request_preview.selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        prompt_pack_preview.prompt_pack.section_count,
        prompt_pack_text,
        runtime_health_status,
        invocation_plan_status,
        status,
        output_classification,
        request.allow_model_execution,
        smoke_result.execution_attempted,
        smoke_result.safe_model_file_name,
        smoke_result.safe_executable_file_name,
        smoke_result.stdout_preview,
        smoke_result.stderr_preview,
        smoke_result.duration_ms,
        smoke_result.exit_code,
        blockers,
        warnings,
    ))
}

pub fn preview_scholar_chat_draft_grounding_inspection(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatDraftGroundingInspectionPreview> {
    let root = root.into();
    let scholar_chat_request = request.scholar_chat_request;
    let request_preview = preview_scholar_chat_request(&root, scholar_chat_request.clone())?;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let normalized_draft_text = normalize_optional_draft_text(request.draft_text);

    for warning in &request_preview.warnings {
        push_grounding_inspection_warning(&mut warnings, "request_preview", warning);
    }
    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a draft grounding inspection preview only; no grounded answer, final answer, Evidence Pack, or persistence was created.",
    );

    let Some(normalized_draft_text) = normalized_draft_text else {
        push_grounding_inspection_blocker(
            &mut blockers,
            "draft_text_missing",
            "No draft text was provided to inspect.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            0,
            request_preview.selected_source_count,
            0,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    };

    if request_preview.selected_source_count == 0 {
        push_grounding_inspection_blocker(
            &mut blockers,
            "needs_sources",
            "No Scholar Chat source context was selected for this inspection.",
        );
        push_grounding_inspection_warning(
            &mut warnings,
            "unscoped_inspection",
            "No Scholar Chat source context selected; inspection will be unscoped.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            normalized_draft_text.chars().count(),
            request_preview.selected_source_count,
            0,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    }

    let evidence_plan_preview = preview_scholar_chat_evidence_plan(&root, scholar_chat_request.clone())?;
    for warning in &evidence_plan_preview.warnings {
        push_grounding_inspection_warning(&mut warnings, "evidence_plan_preview", warning);
    }

    let evidence_candidate_count = evidence_plan_preview.evidence_candidate_count;
    if evidence_candidate_count == 0 {
        push_grounding_inspection_blocker(
            &mut blockers,
            "needs_evidence_candidates",
            "No local evidence candidates were available for the selected sources.",
        );
        push_grounding_inspection_warning(
            &mut warnings,
            "evidence_required",
            "No local evidence candidates are available yet for draft grounding inspection.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            normalized_draft_text.chars().count(),
            request_preview.selected_source_count,
            evidence_candidate_count,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    }

    let max_items = request
        .max_items
        .unwrap_or(SCHOLAR_CHAT_DRAFT_GROUNDING_INSPECTION_LIMIT)
        .clamp(1, SCHOLAR_CHAT_DRAFT_GROUNDING_INSPECTION_LIMIT);
    let inspected_items = inspect_draft_grounding_items(&normalized_draft_text, &evidence_plan_preview.candidates, max_items);
    if inspected_items.items.is_empty() {
        push_grounding_inspection_blocker(
            &mut blockers,
            "draft_text_missing",
            "Draft text did not contain inspectable content.",
        );
        push_grounding_inspection_warning(
            &mut warnings,
            "draft_text_empty",
            "No inspectable draft sentences were found in the provided draft text.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            normalized_draft_text.chars().count(),
            request_preview.selected_source_count,
            evidence_candidate_count,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    }
    if inspected_items.was_clamped {
        push_grounding_inspection_warning(
            &mut warnings,
            "inspection_clamped",
            &format!("Only the first {max_items} draft items were inspected."),
        );
    }
    warnings.extend(inspected_items.warnings);

    Ok(build_draft_grounding_inspection_preview(
        request_preview.normalized_prompt,
        normalized_draft_text.chars().count(),
        request_preview.selected_source_count,
        evidence_candidate_count,
        inspected_items.items,
        inspected_items.supported_item_count,
        inspected_items.weakly_supported_item_count,
        inspected_items.unsupported_item_count,
        blockers,
        warnings,
    ))
}

pub fn preview_scholar_chat_grounded_draft_readiness(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedDraftReadinessPreview> {
    let root = root.into();
    let inspection_preview = preview_scholar_chat_draft_grounding_inspection(&root, request)?;
    let status = grounded_draft_readiness_status(&inspection_preview);
    let mut blockers = inspection_preview.blockers.clone();
    let mut warnings = inspection_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-draft readiness preview only; it is not a grounded answer, final answer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "readiness_blocked",
                    "Grounded-draft readiness is blocked until draft text, source context, and local evidence are available.",
                );
            }
        }
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and need review before a grounded-answer path is added.",
            );
        }
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "ready_for_grounded_draft_later",
                "All inspected items were supported by local evidence. This is still only a readiness preview.",
            );
        }
    }

    let summary = grounded_draft_readiness_summary(&status, &inspection_preview);
    let next_required_actions = grounded_draft_readiness_next_required_actions(&status, &inspection_preview);

    Ok(ScholarChatGroundedDraftReadinessPreview {
        status,
        inspection_status: inspection_preview.status,
        normalized_prompt: inspection_preview.normalized_prompt,
        selected_source_count: inspection_preview.selected_source_count,
        evidence_candidate_count: inspection_preview.evidence_candidate_count,
        inspected_item_count: inspection_preview.inspected_item_count,
        supported_item_count: inspection_preview.supported_item_count,
        weakly_supported_item_count: inspection_preview.weakly_supported_item_count,
        unsupported_item_count: inspection_preview.unsupported_item_count,
        summary,
        preview_only: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_grounded_answer_build_plan(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildPlanPreview> {
    let root = root.into();
    let readiness_preview = preview_scholar_chat_grounded_draft_readiness(&root, request)?;
    let status = grounded_answer_build_plan_status(&readiness_preview);
    let mut blockers = readiness_preview.blockers.clone();
    let mut warnings = readiness_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build plan preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "build_plan_blocked",
                    "Grounded-answer build planning is blocked until draft grounding readiness is available.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and should be reviewed before planning a GroundedAnswer.",
            );
        }
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "plan_ready_later",
                "All inspected items were supported by local evidence. This is still only a plan preview.",
            );
        }
    }

    let summary = grounded_answer_build_plan_summary(&status, &readiness_preview);
    let planned_steps = grounded_answer_build_plan_planned_steps(&status);
    let next_required_actions =
        grounded_answer_build_plan_next_required_actions(&status, &readiness_preview);

    Ok(ScholarChatGroundedAnswerBuildPlanPreview {
        status,
        readiness_status: readiness_preview.status,
        normalized_prompt: readiness_preview.normalized_prompt,
        selected_source_count: readiness_preview.selected_source_count,
        evidence_candidate_count: readiness_preview.evidence_candidate_count,
        inspected_item_count: readiness_preview.inspected_item_count,
        supported_item_count: readiness_preview.supported_item_count,
        weakly_supported_item_count: readiness_preview.weakly_supported_item_count,
        unsupported_item_count: readiness_preview.unsupported_item_count,
        summary,
        planned_steps,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_grounded_answer_candidate(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedAnswerCandidatePreview> {
    let root = root.into();
    let build_plan_preview = preview_scholar_chat_grounded_answer_build_plan(&root, request.clone())?;
    let status = grounded_answer_candidate_status(&build_plan_preview);
    let mut blockers = build_plan_preview.blockers.clone();
    let mut warnings = build_plan_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer candidate preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    let candidate_items = if matches!(status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater) {
        let inspection_preview = preview_scholar_chat_draft_grounding_inspection(&root, request)?;
        grounded_answer_candidate_items_from_inspection(&inspection_preview)
    } else {
        Vec::new()
    };

    match status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "candidate_blocked",
                    "Grounded-answer candidate preview is blocked until grounded-draft readiness is available.",
                );
            }
        }
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and should be reviewed before a grounded-answer candidate is considered.",
            );
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "candidate_ready_later",
                "All inspected items were supported by local evidence. This is still only a grounded-answer candidate preview.",
            );
        }
    }

    let summary = grounded_answer_candidate_summary(&status, &build_plan_preview);
    let next_required_actions = grounded_answer_candidate_next_required_actions(&status, &build_plan_preview);

    Ok(ScholarChatGroundedAnswerCandidatePreview {
        status,
        normalized_prompt: build_plan_preview.normalized_prompt,
        selected_source_count: build_plan_preview.selected_source_count,
        evidence_candidate_count: build_plan_preview.evidence_candidate_count,
        inspected_item_count: build_plan_preview.inspected_item_count,
        supported_item_count: build_plan_preview.supported_item_count,
        weakly_supported_item_count: build_plan_preview.weakly_supported_item_count,
        unsupported_item_count: build_plan_preview.unsupported_item_count,
        candidate_statement_count: candidate_items.len(),
        summary,
        candidate_items,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_grounded_answer_write_eligibility(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedAnswerWriteEligibilityPreview> {
    let root = root.into();
    let candidate_preview = preview_scholar_chat_grounded_answer_candidate(&root, request)?;
    Ok(grounded_answer_write_eligibility_preview_from_candidate_preview(candidate_preview))
}

const SCHOLAR_CHAT_DRAFT_GROUNDING_INSPECTION_LIMIT: usize = 8;

struct DraftGroundingInspectionItems {
    items: Vec<ScholarChatDraftGroundingInspectionItem>,
    supported_item_count: usize,
    weakly_supported_item_count: usize,
    unsupported_item_count: usize,
    warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    was_clamped: bool,
}

fn build_draft_grounding_inspection_preview(
    normalized_prompt: String,
    draft_char_count: usize,
    selected_source_count: usize,
    evidence_candidate_count: usize,
    items: Vec<ScholarChatDraftGroundingInspectionItem>,
    supported_item_count: usize,
    weakly_supported_item_count: usize,
    unsupported_item_count: usize,
    blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
) -> ScholarChatDraftGroundingInspectionPreview {
    let inspected_item_count = items.len();
    ScholarChatDraftGroundingInspectionPreview {
        status: if draft_char_count == 0 {
            ScholarChatDraftGroundingInspectionStatus::NoDraftText
        } else if selected_source_count == 0 {
            ScholarChatDraftGroundingInspectionStatus::Blocked
        } else if evidence_candidate_count == 0 {
            ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates
        } else {
            ScholarChatDraftGroundingInspectionStatus::Inspected
        },
        normalized_prompt,
        draft_char_count,
        selected_source_count,
        evidence_candidate_count,
        inspected_item_count,
        unsupported_item_count,
        weakly_supported_item_count,
        supported_item_count,
        items,
        inspection_only: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_evidence_pack_built: true,
        no_answer_artifact_created: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
    }
}

fn grounded_draft_readiness_status(
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> ScholarChatGroundedDraftReadinessStatus {
    match inspection_preview.status {
        ScholarChatDraftGroundingInspectionStatus::Blocked
        | ScholarChatDraftGroundingInspectionStatus::NoDraftText
        | ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates => {
            ScholarChatGroundedDraftReadinessStatus::Blocked
        }
        ScholarChatDraftGroundingInspectionStatus::Inspected => {
            if inspection_preview.inspected_item_count == 0 {
                ScholarChatGroundedDraftReadinessStatus::Blocked
            } else if inspection_preview.unsupported_item_count > 0
                || inspection_preview.weakly_supported_item_count > 0
            {
                ScholarChatGroundedDraftReadinessStatus::NeedsReview
            } else {
                ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater
            }
        }
    }
}

fn grounded_draft_readiness_summary(
    status: &ScholarChatGroundedDraftReadinessStatus,
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> String {
    match status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => {
            match inspection_preview.status {
                ScholarChatDraftGroundingInspectionStatus::NoDraftText => {
                    "Grounded-draft readiness is blocked because no draft text was provided.".to_string()
                }
                ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates => {
                    "Grounded-draft readiness is blocked because no local evidence candidates were available.".to_string()
                }
                ScholarChatDraftGroundingInspectionStatus::Blocked => {
                    "Grounded-draft readiness is blocked because no Scholar Chat source context was selected.".to_string()
                }
                ScholarChatDraftGroundingInspectionStatus::Inspected => {
                    "Grounded-draft readiness is blocked because no inspectable draft items were found.".to_string()
                }
            }
        }
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => {
            "The draft is not ready yet: weakly supported or unsupported items remain.".to_string()
        }
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
            "All inspected items were supported by local evidence. This is still only a readiness preview.".to_string()
        }
    }
}

fn grounded_draft_readiness_next_required_actions(
    status: &ScholarChatGroundedDraftReadinessStatus,
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> Vec<String> {
    match status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => {
            match inspection_preview.status {
                ScholarChatDraftGroundingInspectionStatus::NoDraftText => {
                    vec!["Provide draft text before previewing grounded-draft readiness.".to_string()]
                }
                ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates => {
                    vec![
                        "Add local evidence candidates for the selected sources before retrying grounded-draft readiness."
                            .to_string(),
                    ]
                }
                ScholarChatDraftGroundingInspectionStatus::Blocked => {
                    vec!["Select Scholar Chat source context before previewing grounded-draft readiness.".to_string()]
                }
                ScholarChatDraftGroundingInspectionStatus::Inspected => {
                    vec!["Review the draft grounding inspection and try again.".to_string()]
                }
            }
        }
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => vec![
            "Review weakly supported and unsupported draft items before treating this draft as ready for a future grounded-answer path."
                .to_string(),
        ],
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => vec![
            "A grounded-answer implementation can be added later without changing this readiness preview.".to_string(),
        ],
    }
}

fn grounded_answer_build_plan_status(
    readiness_preview: &ScholarChatGroundedDraftReadinessPreview,
) -> ScholarChatGroundedAnswerBuildPlanStatus {
    match readiness_preview.status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => ScholarChatGroundedAnswerBuildPlanStatus::Blocked,
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview,
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
            ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater
        }
    }
}

fn grounded_answer_build_plan_summary(
    status: &ScholarChatGroundedAnswerBuildPlanStatus,
    readiness_preview: &ScholarChatGroundedDraftReadinessPreview,
) -> String {
    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => match readiness_preview.status {
            ScholarChatGroundedDraftReadinessStatus::Blocked => {
                "Grounded-answer build planning is blocked because grounded-draft readiness is blocked.".to_string()
            }
            ScholarChatGroundedDraftReadinessStatus::NeedsReview => {
                "Grounded-answer build planning is blocked because grounded-draft readiness still needs review.".to_string()
            }
            ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
                "Grounded-answer build planning is blocked until the readiness preview is available.".to_string()
            }
        },
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer build plan because weakly supported or unsupported items remain.".to_string()
        }
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            "All inspected items were supported by local evidence. This is still only a grounded-answer build plan preview.".to_string()
        }
    }
}

fn grounded_answer_build_plan_planned_steps(
    status: &ScholarChatGroundedAnswerBuildPlanStatus,
) -> Vec<String> {
    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => vec![
            "Resolve grounded-draft readiness blockers.".to_string(),
            "Re-run draft grounding inspection and readiness preview.".to_string(),
            "Only then add a future GroundedAnswer implementation.".to_string(),
        ],
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => vec![
            "Review supported draft items.".to_string(),
            "Resolve weakly supported and unsupported items.".to_string(),
            "Require an explicit implementation phase before writing GroundedAnswer.".to_string(),
        ],
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => vec![
            "Review supported draft items.".to_string(),
            "Map supported draft items to future grounded claims.".to_string(),
            "Require an explicit implementation phase before writing GroundedAnswer.".to_string(),
        ],
    }
}

fn grounded_answer_build_plan_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildPlanStatus,
    readiness_preview: &ScholarChatGroundedDraftReadinessPreview,
) -> Vec<String> {
    let mut next_required_actions = readiness_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Resolve grounded-draft readiness blockers before any GroundedAnswer implementation.",
            );
        }
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before any GroundedAnswer implementation.",
            );
        }
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A GroundedAnswer implementation can be added later without changing this plan preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_candidate_status(
    build_plan_preview: &ScholarChatGroundedAnswerBuildPlanPreview,
) -> ScholarChatGroundedAnswerCandidateStatus {
    match build_plan_preview.status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => ScholarChatGroundedAnswerCandidateStatus::Blocked,
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => ScholarChatGroundedAnswerCandidateStatus::NeedsReview,
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater
        }
    }
}

fn grounded_answer_candidate_summary(
    status: &ScholarChatGroundedAnswerCandidateStatus,
    build_plan_preview: &ScholarChatGroundedAnswerBuildPlanPreview,
) -> String {
    match status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => match build_plan_preview.status {
            ScholarChatGroundedAnswerBuildPlanStatus::Blocked => {
                "Grounded-answer candidate preview is blocked because grounded-answer build planning is blocked."
                    .to_string()
            }
            ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
                "Grounded-answer candidate preview is blocked because grounded-answer build planning still needs review."
                    .to_string()
            }
            ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
                "Grounded-answer candidate preview is blocked until the build-plan preview is available.".to_string()
            }
        },
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer candidate because weakly supported or unsupported items remain."
                .to_string()
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            "All inspected items were supported by local evidence. This is still only a grounded-answer candidate preview."
                .to_string()
        }
    }
}

fn grounded_answer_candidate_next_required_actions(
    status: &ScholarChatGroundedAnswerCandidateStatus,
    build_plan_preview: &ScholarChatGroundedAnswerBuildPlanPreview,
) -> Vec<String> {
    let mut next_required_actions = build_plan_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Resolve grounded-answer build-plan blockers before considering a grounded-answer candidate.",
            );
        }
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before considering a grounded-answer candidate.",
            );
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A GroundedAnswer implementation can be added later without changing this candidate preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_candidate_items_from_inspection(
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> Vec<ScholarChatGroundedAnswerCandidateItem> {
    inspection_preview
        .items
        .iter()
        .map(|item| ScholarChatGroundedAnswerCandidateItem {
            item_index: item.item_index,
            statement_preview: item.text_preview.clone(),
            support_status: item.support_status.clone(),
            source_ids: item.source_ids.clone(),
            locator_previews: item.locator_previews.clone(),
            matched_evidence_count: item.matched_evidence_count,
        })
        .collect()
}

fn grounded_answer_write_eligibility_status(
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> ScholarChatGroundedAnswerWriteEligibilityStatus {
    match candidate_preview.status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => {
            ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked
        }
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            if candidate_preview.candidate_statement_count > 0 {
                ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
            } else {
                ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked
            }
        }
    }
}

fn grounded_answer_write_eligibility_summary(
    status: &ScholarChatGroundedAnswerWriteEligibilityStatus,
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> String {
    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
            match candidate_preview.status {
                ScholarChatGroundedAnswerCandidateStatus::Blocked => {
                    "Grounded-answer write eligibility is blocked because grounded-answer candidate preview is blocked.".to_string()
                }
                ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
                    "Grounded-answer write eligibility is blocked because grounded-answer candidate preview still needs review.".to_string()
                }
                ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
                    if candidate_preview.candidate_statement_count == 0 {
                        "Grounded-answer write eligibility is blocked because no candidate statements were available.".to_string()
                    } else {
                        "Grounded-answer write eligibility is blocked until the candidate preview is available.".to_string()
                    }
                }
            }
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer write because weakly supported or unsupported items remain.".to_string()
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            "All inspected items were supported by local evidence. A future GroundedAnswer write can be added later.".to_string()
        }
    }
}

fn grounded_answer_write_eligibility_reasons(
    status: &ScholarChatGroundedAnswerWriteEligibilityStatus,
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Candidate status: {:?}", candidate_preview.status),
        format!("Candidate statements: {}", candidate_preview.candidate_statement_count),
    ];
    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => match candidate_preview.status {
            ScholarChatGroundedAnswerCandidateStatus::Blocked => {
                reasons.push("Grounded-answer candidate preview is blocked.".to_string());
            }
            ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
                reasons.push("Grounded-answer candidate preview still needs review.".to_string());
            }
            ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
                if candidate_preview.candidate_statement_count == 0 {
                    reasons.push("No candidate statements were available.".to_string());
                } else {
                    reasons.push("Grounded-answer write eligibility is still blocked.".to_string());
                }
            }
        },
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            reasons.push("Weakly supported or unsupported draft items remain.".to_string());
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            reasons.push("All inspected items were supported by local evidence.".to_string());
            reasons.push(
                "A future GroundedAnswer write can be added later after an explicit implementation phase."
                    .to_string(),
            );
        }
    }
    reasons
}

fn grounded_answer_write_eligibility_next_required_actions(
    status: &ScholarChatGroundedAnswerWriteEligibilityStatus,
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> Vec<String> {
    let mut next_required_actions = candidate_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Resolve grounded-answer candidate blockers before any GroundedAnswer write implementation.",
            );
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before any GroundedAnswer write implementation.",
            );
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            push_unique_text(
                &mut next_required_actions,
                "A GroundedAnswer write implementation can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_write_eligibility_preview_from_candidate_preview(
    candidate_preview: ScholarChatGroundedAnswerCandidatePreview,
) -> ScholarChatGroundedAnswerWriteEligibilityPreview {
    let status = grounded_answer_write_eligibility_status(&candidate_preview);
    let mut blockers = candidate_preview.blockers.clone();
    let mut warnings = candidate_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer write-eligibility preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "write_eligibility_blocked",
                    "Grounded-answer write eligibility is blocked until grounded-answer candidate preview is available.",
                );
            }
            if candidate_preview.candidate_statement_count == 0 {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "candidate_statements_missing",
                    "No candidate statements were available for write eligibility.",
                );
            }
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and need review before a future GroundedAnswer write is added.",
            );
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "write_eligible_later",
                "All inspected items were supported by local evidence. This is still only a write-eligibility preview.",
            );
        }
    }

    let summary = grounded_answer_write_eligibility_summary(&status, &candidate_preview);
    let eligibility_reasons = grounded_answer_write_eligibility_reasons(&status, &candidate_preview);
    let next_required_actions =
        grounded_answer_write_eligibility_next_required_actions(&status, &candidate_preview);

    ScholarChatGroundedAnswerWriteEligibilityPreview {
        status,
        candidate_status: candidate_preview.status,
        normalized_prompt: candidate_preview.normalized_prompt,
        selected_source_count: candidate_preview.selected_source_count,
        evidence_candidate_count: candidate_preview.evidence_candidate_count,
        inspected_item_count: candidate_preview.inspected_item_count,
        supported_item_count: candidate_preview.supported_item_count,
        weakly_supported_item_count: candidate_preview.weakly_supported_item_count,
        unsupported_item_count: candidate_preview.unsupported_item_count,
        candidate_statement_count: candidate_preview.candidate_statement_count,
        eligibility_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
    }
}

pub fn preview_scholar_chat_grounded_answer_build_intent(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerBuildIntentRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildIntentPreview> {
    let root = root.into();
    let normalized_prompt = normalized_prompt_or_err(request.grounding_request.scholar_chat_request.prompt.clone())?;
    let normalized_answer_draft_id = normalize_optional_answer_draft_id(request.answer_draft_id)?;
    let write_eligibility_preview =
        preview_scholar_chat_grounded_answer_write_eligibility(&root, request.grounding_request.clone())?;
    Ok(grounded_answer_build_intent_preview_from_write_eligibility_preview(
        write_eligibility_preview,
        normalized_prompt,
        normalized_answer_draft_id,
        request.explicit_user_intent,
    ))
}

pub fn preview_scholar_chat_grounded_answer_build_request(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerBuildRequestPreviewRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildRequestPreview> {
    let root = root.into();
    let normalized_prompt = normalized_prompt_or_err(
        request
            .build_intent_request
            .grounding_request
            .scholar_chat_request
            .prompt
            .clone(),
    )?;
    let normalized_answer_draft_id =
        normalize_optional_answer_draft_id(request.build_intent_request.answer_draft_id.clone())?;
    let (normalized_selected_source_ids, _selected_source_count) = normalize_selected_source_ids(
        request
            .build_intent_request
            .grounding_request
            .scholar_chat_request
            .selected_source_ids
            .clone(),
    )?;
    let normalized_build_intent_request = ScholarChatGroundedAnswerBuildIntentRequest {
        grounding_request: ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: normalized_prompt.clone(),
                mode: request
                    .build_intent_request
                    .grounding_request
                    .scholar_chat_request
                    .mode,
                grounding_policy: request
                    .build_intent_request
                    .grounding_request
                    .scholar_chat_request
                    .grounding_policy,
                selected_source_ids: normalized_selected_source_ids.clone(),
            },
            draft_text: request.build_intent_request.grounding_request.draft_text.clone(),
            max_items: request.build_intent_request.grounding_request.max_items,
        },
        answer_draft_id: normalized_answer_draft_id.clone(),
        explicit_user_intent: request.build_intent_request.explicit_user_intent,
    };
    let build_intent_preview =
        preview_scholar_chat_grounded_answer_build_intent(&root, normalized_build_intent_request)?;
    Ok(grounded_answer_build_request_preview_from_build_intent_preview(
        build_intent_preview,
        normalized_prompt,
        normalized_answer_draft_id,
        normalized_selected_source_ids,
    ))
}

pub fn preview_scholar_chat_grounded_answer_build_preflight(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildPreflightPreview> {
    let root = root.into();
    let build_request_preview =
        preview_scholar_chat_grounded_answer_build_request(&root, request.build_request_preview_request)?;
    let normalized_prompt = build_request_preview.normalized_prompt.clone();
    let selected_source_ids = build_request_preview.selected_source_ids.clone();
    let answer_draft_id = build_request_preview.answer_draft_id.clone();
    let answer_draft_id_present = answer_draft_id.is_some();
    let mut answer_draft_present = false;
    let mut answer_draft_readable = false;
    let mut answer_draft_claim_count = 0usize;
    let mut blockers = build_request_preview.blockers.clone();
    let mut warnings = build_request_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build preflight preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    let status = match build_request_preview.status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => ScholarChatGroundedAnswerBuildPreflightStatus::Blocked,
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview,
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            if let Some(answer_draft_id) = answer_draft_id.as_ref() {
                let corpus_paths = CorpusPaths::new(root.clone());
                let registry = SourceRegistry::load(&corpus_paths.registry_path());
                if let Ok(registry) = registry {
                    let mut answer_draft_missing = true;
                    for lookup_source_id in &selected_source_ids {
                        let record = match registry.get_source(lookup_source_id) {
                            Ok(record) => record,
                            Err(_) => continue,
                        };
                        let draft_path = corpus_paths
                            .source_version_dir(&record.source_id, &record.version_id)
                            .join("answer_drafts")
                            .join(format!("{answer_draft_id}.json"));
                        if !draft_path.exists() {
                            continue;
                        }
                        answer_draft_missing = false;
                        answer_draft_present = true;
                        match fs::read_to_string(&draft_path)
                            .ok()
                            .and_then(|content| serde_json::from_str::<crate::answer_draft::AnswerDraft>(&content).ok())
                        {
                            Some(answer_draft) => {
                                answer_draft_readable = true;
                                answer_draft_claim_count = answer_draft.claim_count;
                                break;
                            }
                            None => {
                                push_grounding_inspection_blocker(
                                    &mut blockers,
                                    "answer_draft_unreadable",
                                    "The referenced AnswerDraft is unreadable.",
                                );
                                break;
                            }
                        }
                    }
                    if answer_draft_missing {
                        push_grounding_inspection_blocker(
                            &mut blockers,
                            "answer_draft_missing",
                            "The referenced AnswerDraft is missing.",
                        );
                    }
                } else {
                    push_grounding_inspection_blocker(
                        &mut blockers,
                        "answer_draft_missing",
                        "The referenced AnswerDraft is missing.",
                    );
                }
                if answer_draft_readable {
                    ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
                } else {
                    if answer_draft_present && !blockers.iter().any(|blocker| blocker.kind == "answer_draft_unreadable") {
                        push_grounding_inspection_blocker(
                            &mut blockers,
                            "answer_draft_unreadable",
                            "The referenced AnswerDraft is unreadable.",
                        );
                    }
                    ScholarChatGroundedAnswerBuildPreflightStatus::Blocked
                }
            } else {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "answer_draft_id_missing",
                    "No answer draft ID was provided.",
                );
                ScholarChatGroundedAnswerBuildPreflightStatus::Blocked
            }
        }
    };

    if matches!(status, ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview) {
        push_grounding_inspection_warning(
            &mut warnings,
            "needs_review",
            "The build request still needs review before AnswerDraft preflight can be accepted.",
        );
    }
    if matches!(status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater) {
        push_grounding_inspection_warning(
            &mut warnings,
            "preflight_ready_later",
            "The referenced AnswerDraft is readable. This is still only a preflight preview.",
        );
    }

    let required_inputs = grounded_answer_build_preflight_required_inputs();
    let missing_inputs = grounded_answer_build_preflight_missing_inputs(
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_id.as_ref(),
        answer_draft_present,
        answer_draft_readable,
        &status,
    );
    let preflight_reasons = grounded_answer_build_preflight_reasons(
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
        &status,
    );
    let next_required_actions = grounded_answer_build_preflight_next_required_actions(
        &status,
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_present,
        answer_draft_readable,
    );
    let summary = grounded_answer_build_preflight_summary(
        &status,
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
    );

    Ok(ScholarChatGroundedAnswerBuildPreflightPreview {
        status,
        build_request_status: build_request_preview.status,
        build_intent_status: build_request_preview.build_intent_status,
        write_eligibility_status: build_request_preview.write_eligibility_status,
        candidate_status: build_request_preview.candidate_status,
        normalized_prompt,
        selected_source_count: build_request_preview.selected_source_count,
        evidence_candidate_count: build_request_preview.evidence_candidate_count,
        inspected_item_count: build_request_preview.inspected_item_count,
        supported_item_count: build_request_preview.supported_item_count,
        weakly_supported_item_count: build_request_preview.weakly_supported_item_count,
        unsupported_item_count: build_request_preview.unsupported_item_count,
        candidate_statement_count: build_request_preview.candidate_statement_count,
        answer_draft_id,
        selected_source_ids,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
        required_inputs,
        missing_inputs,
        preflight_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
        no_grounded_answer_write: true,
    })
}

pub fn preview_scholar_chat_grounded_answer_execution_readiness(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerExecutionReadinessPreviewRequest,
) -> AegisResult<ScholarChatGroundedAnswerExecutionReadinessPreview> {
    let root = root.into();
    let build_preflight_preview =
        preview_scholar_chat_grounded_answer_build_preflight(&root, request.build_preflight_preview_request)?;
    let normalized_prompt = build_preflight_preview.normalized_prompt.clone();
    let answer_draft_id = build_preflight_preview.answer_draft_id.clone();
    let selected_source_ids = build_preflight_preview.selected_source_ids.clone();
    let answer_draft_present = build_preflight_preview.answer_draft_present;
    let answer_draft_readable = build_preflight_preview.answer_draft_readable;
    let answer_draft_claim_count = build_preflight_preview.answer_draft_claim_count;
    let build_preflight_status = build_preflight_preview.status.clone();
    let mut blockers = build_preflight_preview.blockers.clone();
    let mut warnings = build_preflight_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is an execution-readiness preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );
    if !request.execution_consent {
        push_grounding_inspection_blocker(
            &mut blockers,
            "execution_consent_missing",
            "Execution consent was not given.",
        );
    }

    let status = match build_preflight_status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked,
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview,
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => {
            if request.execution_consent {
                ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater
            } else {
                ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked
            }
        }
    };

    if matches!(status, ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview) {
        push_grounding_inspection_warning(
            &mut warnings,
            "needs_review",
            "The grounded-answer build preflight still needs review before execution readiness can be accepted.",
        );
    }
    if matches!(status, ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater) {
        push_grounding_inspection_warning(
            &mut warnings,
            "execution_ready_later",
            "Execution consent is given and the build preflight is ready later. This is still only a preview.",
        );
    }

    let required_inputs = grounded_answer_execution_readiness_required_inputs();
    let missing_inputs = grounded_answer_execution_readiness_missing_inputs(
        &build_preflight_status,
        answer_draft_readable,
        request.execution_consent,
    );
    let readiness_reasons = grounded_answer_execution_readiness_reasons(
        &build_preflight_preview,
        request.execution_consent,
        &status,
    );
    let next_required_actions = grounded_answer_execution_readiness_next_required_actions(
        &status,
        &build_preflight_preview,
        request.execution_consent,
    );
    let summary = grounded_answer_execution_readiness_summary(
        &status,
        &build_preflight_preview,
        request.execution_consent,
    );

    Ok(ScholarChatGroundedAnswerExecutionReadinessPreview {
        status,
        build_preflight_status,
        build_request_status: build_preflight_preview.build_request_status,
        build_intent_status: build_preflight_preview.build_intent_status,
        write_eligibility_status: build_preflight_preview.write_eligibility_status,
        candidate_status: build_preflight_preview.candidate_status,
        normalized_prompt,
        answer_draft_id,
        selected_source_ids,
        selected_source_count: build_preflight_preview.selected_source_count,
        evidence_candidate_count: build_preflight_preview.evidence_candidate_count,
        inspected_item_count: build_preflight_preview.inspected_item_count,
        supported_item_count: build_preflight_preview.supported_item_count,
        weakly_supported_item_count: build_preflight_preview.weakly_supported_item_count,
        unsupported_item_count: build_preflight_preview.unsupported_item_count,
        candidate_statement_count: build_preflight_preview.candidate_statement_count,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
        execution_consent: request.execution_consent,
        required_inputs,
        missing_inputs,
        readiness_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
        no_grounded_answer_write: true,
    })
}

pub fn preview_scholar_chat_grounded_answer_execution_plan(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerExecutionPlanPreviewRequest,
) -> AegisResult<ScholarChatGroundedAnswerExecutionPlanPreview> {
    let root = root.into();
    let readiness_preview = preview_scholar_chat_grounded_answer_execution_readiness(
        &root,
        request.execution_readiness_preview_request,
    )?;
    let readiness_status = readiness_preview.status.clone();
    let planned_operation = "future_grounded_answer_build".to_string();
    let status = match readiness_status {
        ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked => {
            ScholarChatGroundedAnswerExecutionPlanStatus::Blocked
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview => {
            ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater => {
            ScholarChatGroundedAnswerExecutionPlanStatus::PlanReadyLater
        }
    };
    let planned_inputs = grounded_answer_execution_plan_planned_inputs();
    let planned_outputs = grounded_answer_execution_plan_planned_outputs();
    let planned_write_targets = grounded_answer_execution_plan_planned_write_targets();
    let required_inputs = grounded_answer_execution_plan_required_inputs();
    let missing_inputs = grounded_answer_execution_plan_missing_inputs(&readiness_preview);
    let mut plan_reasons = grounded_answer_execution_plan_reasons(
        &readiness_preview,
        &status,
        &planned_operation,
        &planned_inputs,
        &planned_outputs,
        &planned_write_targets,
    );
    let mut blockers = readiness_preview.blockers.clone();
    let mut warnings = readiness_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is an execution-plan preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );
    if !readiness_preview.execution_consent {
        push_grounding_inspection_blocker(
            &mut blockers,
            "execution_consent_missing",
            "Execution consent was not given.",
        );
    }

    let next_required_actions = grounded_answer_execution_plan_next_required_actions(
        &status,
        &readiness_preview,
    );
    let summary = grounded_answer_execution_plan_summary(&status, &readiness_preview);

    if matches!(status, ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview) {
        push_unique_text(
            &mut plan_reasons,
            "The grounded-answer execution readiness still needs review before planning a future build.",
        );
    }
    if matches!(status, ScholarChatGroundedAnswerExecutionPlanStatus::PlanReadyLater) {
        push_unique_text(
            &mut plan_reasons,
            "The grounded-answer execution readiness is ready later and execution consent is true.",
        );
    }

    Ok(ScholarChatGroundedAnswerExecutionPlanPreview {
        status,
        readiness_status,
        build_preflight_status: readiness_preview.build_preflight_status,
        build_request_status: readiness_preview.build_request_status,
        build_intent_status: readiness_preview.build_intent_status,
        write_eligibility_status: readiness_preview.write_eligibility_status,
        candidate_status: readiness_preview.candidate_status,
        normalized_prompt: readiness_preview.normalized_prompt,
        answer_draft_id: readiness_preview.answer_draft_id,
        selected_source_ids: readiness_preview.selected_source_ids,
        selected_source_count: readiness_preview.selected_source_count,
        evidence_candidate_count: readiness_preview.evidence_candidate_count,
        inspected_item_count: readiness_preview.inspected_item_count,
        supported_item_count: readiness_preview.supported_item_count,
        weakly_supported_item_count: readiness_preview.weakly_supported_item_count,
        unsupported_item_count: readiness_preview.unsupported_item_count,
        candidate_statement_count: readiness_preview.candidate_statement_count,
        answer_draft_present: readiness_preview.answer_draft_present,
        answer_draft_readable: readiness_preview.answer_draft_readable,
        answer_draft_claim_count: readiness_preview.answer_draft_claim_count,
        execution_consent: readiness_preview.execution_consent,
        planned_operation,
        planned_inputs,
        planned_outputs,
        planned_write_targets,
        required_inputs,
        missing_inputs,
        plan_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
        no_grounded_answer_write: true,
    })
}

pub fn preview_scholar_chat_runtime_diagnostic_bridge(
    root: impl Into<PathBuf>,
    request: ScholarChatRuntimeDiagnosticBridgePreviewRequest,
) -> AegisResult<ScholarChatRuntimeDiagnosticBridgePreview> {
    let root = root.into();
    let normalized_prompt = request.scholar_chat_request.prompt.trim().to_string();
    let (_selected_source_ids, selected_source_count) =
        normalize_selected_source_ids(request.scholar_chat_request.selected_source_ids)?;
    let smoke_execution_plan_preview =
        preview_llama_runtime_smoke_execution_plan(&root, request.smoke_execution_plan_preview_request)?;
    let smoke_execution_plan_status = smoke_execution_plan_preview.status.clone();
    let smoke_readiness_status = smoke_execution_plan_preview.smoke_readiness_status.clone();
    let capability_status = smoke_execution_plan_preview.capability_status.clone();
    let version_probe_status = smoke_execution_plan_preview.version_probe_status.clone();
    let probe_readiness_status = smoke_execution_plan_preview.probe_readiness_status.clone();
    let validation_status = smoke_execution_plan_preview.validation_status.clone();
    let adapter_contract_status = smoke_execution_plan_preview.adapter_contract_status.clone();
    let adapter_kind = smoke_execution_plan_preview.adapter_kind.clone();
    let normalized_model_family = smoke_execution_plan_preview.normalized_model_family.clone();
    let normalized_model_format = smoke_execution_plan_preview.normalized_model_format.clone();
    let safe_executable_file_name = smoke_execution_plan_preview.safe_executable_file_name.clone();
    let safe_model_file_name = smoke_execution_plan_preview.safe_model_file_name.clone();
    let diagnostic_prompt_char_count = smoke_execution_plan_preview.diagnostic_prompt_char_count;
    let max_output_tokens = smoke_execution_plan_preview.max_output_tokens;
    let timeout_ms = smoke_execution_plan_preview.timeout_ms;
    let prompt_is_blank = normalized_prompt.is_empty();
    let selected_sources_missing = selected_source_count == 0;

    let status = if prompt_is_blank || selected_sources_missing {
        ScholarChatRuntimeDiagnosticBridgeStatus::Blocked
    } else {
        match smoke_execution_plan_status {
            LocalRuntimeSmokeExecutionPlanStatus::Blocked => ScholarChatRuntimeDiagnosticBridgeStatus::Blocked,
            LocalRuntimeSmokeExecutionPlanStatus::NeedsReview => ScholarChatRuntimeDiagnosticBridgeStatus::NeedsReview,
            LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater => {
                ScholarChatRuntimeDiagnosticBridgeStatus::RuntimeDiagnosticReadyLater
            }
        }
    };

    let mut blockers = smoke_execution_plan_preview.blockers.clone();
    let mut warnings = smoke_execution_plan_preview.warnings.clone();
    let mut runtime_diagnostic_reasons = smoke_execution_plan_preview.plan_reasons.clone();
    let mut next_required_actions = smoke_execution_plan_preview.next_required_actions.clone();

    push_runtime_diagnostic_warning(
        &mut warnings,
        "boundary",
        "This is a bridge preview only; it does not run smoke diagnostics, inference, or Scholar Chat answers.",
    );

    if prompt_is_blank {
        push_runtime_diagnostic_blocker(
            &mut blockers,
            "scholar_chat_prompt_missing",
            "The Scholar Chat prompt is blank.",
        );
        push_unique_text(
            &mut runtime_diagnostic_reasons,
            "The Scholar Chat prompt is blank.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Provide a nonblank Scholar Chat prompt.",
        );
    }

    if selected_sources_missing {
        push_runtime_diagnostic_blocker(
            &mut blockers,
            "scholar_chat_sources_missing",
            "No Scholar Chat sources are selected.",
        );
        push_unique_text(
            &mut runtime_diagnostic_reasons,
            "No Scholar Chat sources are selected.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Select at least one Scholar Chat source.",
        );
    }

    match status {
        ScholarChatRuntimeDiagnosticBridgeStatus::Blocked => {
            push_unique_text(
                &mut runtime_diagnostic_reasons,
                "The llama.cpp smoke execution plan is not ready later for a Scholar Chat runtime diagnostic bridge.",
            );
            if matches!(smoke_execution_plan_status, LocalRuntimeSmokeExecutionPlanStatus::Blocked) {
                push_unique_text(
                    &mut next_required_actions,
                    "Make the llama.cpp smoke execution plan ready later first.",
                );
            }
        }
        ScholarChatRuntimeDiagnosticBridgeStatus::NeedsReview => {
            push_unique_text(
                &mut runtime_diagnostic_reasons,
                "The llama.cpp smoke execution plan still needs review before the bridge can be ready later.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Resolve the llama.cpp smoke execution plan review items before previewing a future runtime diagnostic bridge.",
            );
        }
        ScholarChatRuntimeDiagnosticBridgeStatus::RuntimeDiagnosticReadyLater => {
            push_unique_text(
                &mut runtime_diagnostic_reasons,
                "The llama.cpp smoke execution plan is ready later and the Scholar Chat runtime diagnostic bridge is ready later.",
            );
            push_unique_text(
                &mut next_required_actions,
                "A future runtime diagnostic execution step can be added later.",
            );
        }
    }

    let summary = match status {
        ScholarChatRuntimeDiagnosticBridgeStatus::Blocked => {
            if prompt_is_blank {
                "The Scholar Chat runtime diagnostic bridge is blocked until the prompt is nonblank.".to_string()
            } else if selected_sources_missing {
                "The Scholar Chat runtime diagnostic bridge is blocked until at least one Scholar Chat source is selected.".to_string()
            } else {
                "The Scholar Chat runtime diagnostic bridge is blocked until the llama.cpp smoke execution plan is ready later.".to_string()
            }
        }
        ScholarChatRuntimeDiagnosticBridgeStatus::NeedsReview => {
            "The Scholar Chat runtime diagnostic bridge still needs review because the llama.cpp smoke execution plan still needs review.".to_string()
        }
        ScholarChatRuntimeDiagnosticBridgeStatus::RuntimeDiagnosticReadyLater => {
            "The Scholar Chat runtime diagnostic bridge is ready later for a future runtime diagnostic step.".to_string()
        }
    };

    Ok(ScholarChatRuntimeDiagnosticBridgePreview {
        status,
        normalized_prompt,
        selected_source_count,
        smoke_execution_plan_status,
        smoke_readiness_status,
        capability_status,
        version_probe_status,
        probe_readiness_status,
        validation_status,
        adapter_contract_status,
        adapter_kind,
        normalized_model_family,
        normalized_model_format,
        safe_executable_file_name,
        safe_model_file_name,
        diagnostic_prompt_char_count,
        max_output_tokens,
        timeout_ms,
        runtime_diagnostic_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_smoke_execution: true,
        no_runtime_inference: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_answer_draft_created: true,
        no_grounded_answer_created: true,
        no_final_answer_created: true,
        no_grounding_applied: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_scholar_chat_runtime_answer_pipeline_gate(
    root: impl Into<PathBuf>,
    request: ScholarChatRuntimeAnswerPipelineGatePreviewRequest,
) -> AegisResult<ScholarChatRuntimeAnswerPipelineGatePreview> {
    let root = root.into();
    let grounded_answer_execution_plan_preview_request = request.grounded_answer_execution_plan_preview_request;
    let runtime_diagnostic_result_preview_request = request.runtime_diagnostic_result_preview_request;

    let grounded_answer_scholar_chat_request = grounded_answer_execution_plan_preview_request
        .execution_readiness_preview_request
        .build_preflight_preview_request
        .build_request_preview_request
        .build_intent_request
        .grounding_request
        .scholar_chat_request
        .clone();
    let runtime_diagnostic_scholar_chat_request = runtime_diagnostic_result_preview_request
        .bridge_preview_request
        .scholar_chat_request
        .clone();

    let (grounded_selected_source_ids, selected_source_count) =
        normalize_selected_source_ids(grounded_answer_scholar_chat_request.selected_source_ids.clone())?;
    let (runtime_selected_source_ids, _runtime_selected_source_count) =
        normalize_selected_source_ids(runtime_diagnostic_scholar_chat_request.selected_source_ids.clone())?;
    let normalized_prompt = grounded_answer_scholar_chat_request.prompt.trim().to_string();
    let runtime_normalized_prompt = runtime_diagnostic_scholar_chat_request.prompt.trim().to_string();
    let prompt_is_blank = normalized_prompt.is_empty();
    let sources_are_missing = selected_source_count == 0;
    let normalized_requests_match = ScholarChatRequest {
        prompt: normalized_prompt.clone(),
        mode: grounded_answer_scholar_chat_request.mode.clone(),
        grounding_policy: grounded_answer_scholar_chat_request.grounding_policy.clone(),
        selected_source_ids: grounded_selected_source_ids.clone(),
    } == ScholarChatRequest {
        prompt: runtime_normalized_prompt.clone(),
        mode: runtime_diagnostic_scholar_chat_request.mode.clone(),
        grounding_policy: runtime_diagnostic_scholar_chat_request.grounding_policy.clone(),
        selected_source_ids: runtime_selected_source_ids.clone(),
    };

    if prompt_is_blank || sources_are_missing {
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        let mut pipeline_gate_reasons = Vec::new();
        let mut next_required_actions = Vec::new();

        push_unique_text(
            &mut warnings,
            "boundary: This is a pipeline-gate preview only; it does not run smoke diagnostics, inference, or Scholar Chat answers.",
        );

        if prompt_is_blank {
            push_unique_text(
                &mut blockers,
                "scholar_chat_prompt_missing: The Scholar Chat prompt is blank.",
            );
            push_unique_text(
                &mut pipeline_gate_reasons,
                "The future runtime answer pipeline is blocked until the Scholar Chat prompt is nonblank.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Provide a nonblank Scholar Chat prompt before previewing the runtime answer pipeline gate.",
            );
        }

        if sources_are_missing {
            push_unique_text(
                &mut blockers,
                "scholar_chat_sources_missing: No Scholar Chat sources are selected.",
            );
            push_unique_text(
                &mut pipeline_gate_reasons,
                "The future runtime answer pipeline is blocked until at least one Scholar Chat source is selected.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Select at least one Scholar Chat source before previewing the runtime answer pipeline gate.",
            );
        }

        push_unique_text(
            &mut next_required_actions,
            "A future grounded-answer execution plan and runtime diagnostic result can be added later once the preview inputs are valid.",
        );

        return Ok(ScholarChatRuntimeAnswerPipelineGatePreview {
            status: ScholarChatRuntimeAnswerPipelineGateStatus::Blocked,
            grounded_answer_execution_plan_status: ScholarChatGroundedAnswerExecutionPlanStatus::Blocked,
            grounded_answer_execution_readiness_status: ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked,
            runtime_diagnostic_result_status: ScholarChatRuntimeDiagnosticResultStatus::Blocked,
            runtime_diagnostic_bridge_status: ScholarChatRuntimeDiagnosticBridgeStatus::Blocked,
            smoke_diagnostic_status: LocalRuntimeSmokeDiagnosticStatus::Blocked,
            smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus::Blocked,
            smoke_readiness_status: LocalRuntimeSmokeReadinessStatus::Blocked,
            capability_status: LocalRuntimeCapabilityStatus::Blocked,
            version_probe_status: LocalRuntimeVersionProbeStatus::Blocked,
            probe_readiness_status: LocalRuntimeProbeReadinessStatus::Blocked,
            validation_status: LocalRuntimeValidationStatus::Blocked,
            adapter_contract_status: LocalRuntimeAdapterContractStatus::Blocked,
            adapter_kind: LocalRuntimeAdapterKind::LlamaCpp,
            selected_source_count,
            normalized_model_family: None,
            normalized_model_format: "unknown".to_string(),
            safe_executable_file_name: None,
            safe_model_file_name: None,
            diagnostic_prompt_char_count: normalized_prompt.chars().count(),
            max_output_tokens: 0,
            timeout_ms: 0,
            pipeline_gate_reasons,
            blockers,
            warnings,
            next_required_actions,
            summary: if prompt_is_blank {
                "The future runtime answer pipeline is blocked until the Scholar Chat prompt is nonblank.".to_string()
            } else {
                "The future runtime answer pipeline is blocked until at least one Scholar Chat source is selected.".to_string()
            },
            preview_only: true,
            gate_only: true,
            no_smoke_execution: true,
            no_runtime_inference: true,
            no_new_process_spawn: true,
            no_llm_call: true,
            no_answer_generated: true,
            no_answer_draft_created: true,
            no_grounded_answer_created: true,
            no_final_answer_created: true,
            no_grounding_applied: true,
            no_evidence_pack_built: true,
            no_persistence: true,
            no_artifact_write: true,
            no_registry_status_change: true,
            no_audit_write: true,
        });
    }

    let grounded_answer_execution_plan_preview = preview_scholar_chat_grounded_answer_execution_plan(
        &root,
        grounded_answer_execution_plan_preview_request,
    )?;
    let runtime_diagnostic_result_preview = preview_scholar_chat_runtime_diagnostic_result(
        &root,
        runtime_diagnostic_result_preview_request,
    )?;

    let mut status = if matches!(
        grounded_answer_execution_plan_preview.status,
        ScholarChatGroundedAnswerExecutionPlanStatus::Blocked
    ) || matches!(
        runtime_diagnostic_result_preview.status,
        ScholarChatRuntimeDiagnosticResultStatus::Blocked
    ) || matches!(
        runtime_diagnostic_result_preview.status,
        ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticFailed
    ) {
        ScholarChatRuntimeAnswerPipelineGateStatus::Blocked
    } else if matches!(
        grounded_answer_execution_plan_preview.status,
        ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview
    ) || matches!(
        runtime_diagnostic_result_preview.status,
        ScholarChatRuntimeDiagnosticResultStatus::NeedsReview
    ) {
        ScholarChatRuntimeAnswerPipelineGateStatus::NeedsReview
    } else if matches!(
        grounded_answer_execution_plan_preview.status,
        ScholarChatGroundedAnswerExecutionPlanStatus::PlanReadyLater
    ) && matches!(
        runtime_diagnostic_result_preview.status,
        ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticSucceededLater
    ) && normalized_requests_match {
        ScholarChatRuntimeAnswerPipelineGateStatus::ReadyLater
    } else {
        ScholarChatRuntimeAnswerPipelineGateStatus::NeedsReview
    };

    let grounded_answer_execution_plan_status = grounded_answer_execution_plan_preview.status.clone();
    let grounded_answer_execution_readiness_status = grounded_answer_execution_plan_preview.readiness_status.clone();
    let runtime_diagnostic_result_status = runtime_diagnostic_result_preview.status.clone();
    let runtime_diagnostic_bridge_status = runtime_diagnostic_result_preview.bridge_status.clone();
    let smoke_diagnostic_status = runtime_diagnostic_result_preview.smoke_diagnostic_status.clone();
    let smoke_execution_plan_status = runtime_diagnostic_result_preview.smoke_execution_plan_status.clone();
    let smoke_readiness_status = runtime_diagnostic_result_preview.smoke_readiness_status.clone();
    let capability_status = runtime_diagnostic_result_preview.capability_status.clone();
    let version_probe_status = runtime_diagnostic_result_preview.version_probe_status.clone();
    let probe_readiness_status = runtime_diagnostic_result_preview.probe_readiness_status.clone();
    let validation_status = runtime_diagnostic_result_preview.validation_status.clone();
    let adapter_contract_status = runtime_diagnostic_result_preview.adapter_contract_status.clone();
    let adapter_kind = runtime_diagnostic_result_preview.adapter_kind.clone();
    let normalized_model_family = runtime_diagnostic_result_preview.normalized_model_family.clone();
    let normalized_model_format = runtime_diagnostic_result_preview.normalized_model_format.clone();
    let safe_executable_file_name = runtime_diagnostic_result_preview.safe_executable_file_name.clone();
    let safe_model_file_name = runtime_diagnostic_result_preview.safe_model_file_name.clone();
    let diagnostic_prompt_char_count = runtime_diagnostic_result_preview.diagnostic_prompt_char_count;
    let max_output_tokens = runtime_diagnostic_result_preview.max_output_tokens;
    let timeout_ms = runtime_diagnostic_result_preview.timeout_ms;

    let mut pipeline_gate_reasons = grounded_answer_execution_plan_preview.plan_reasons.clone();
    pipeline_gate_reasons.extend(runtime_diagnostic_result_preview.runtime_result_reasons.clone());
    let mut blockers = grounded_answer_execution_plan_preview
        .blockers
        .iter()
        .map(|blocker| format!("{}: {}", blocker.kind, blocker.message))
        .collect::<Vec<String>>();
    blockers.extend(
        runtime_diagnostic_result_preview
            .blockers
            .iter()
            .map(|blocker| format!("{}: {}", blocker.kind, blocker.message)),
    );
    let mut warnings = grounded_answer_execution_plan_preview
        .warnings
        .iter()
        .map(|warning| format!("{}: {}", warning.kind, warning.message))
        .collect::<Vec<String>>();
    warnings.extend(
        runtime_diagnostic_result_preview
            .warnings
            .iter()
            .map(|warning| format!("{}: {}", warning.kind, warning.message)),
    );
    let mut next_required_actions = grounded_answer_execution_plan_preview.next_required_actions.clone();
    next_required_actions.extend(runtime_diagnostic_result_preview.next_required_actions.clone());

    if !normalized_requests_match {
        push_unique_text(
            &mut blockers,
            "runtime_answer_pipeline_gate_metadata_mismatch: The grounded-answer execution plan preview and runtime diagnostic result preview do not use the same normalized Scholar Chat inputs.",
        );
        push_unique_text(
            &mut warnings,
            "runtime_answer_pipeline_gate_metadata_mismatch: The grounded-answer execution plan preview and runtime diagnostic result preview do not use the same normalized Scholar Chat inputs.",
        );
        push_unique_text(
            &mut pipeline_gate_reasons,
            "The grounded-answer execution plan preview and runtime diagnostic result preview do not use the same normalized Scholar Chat inputs.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Make the grounded-answer execution plan preview and runtime diagnostic result preview inputs match before retrying the pipeline gate.",
        );
        if !matches!(status, ScholarChatRuntimeAnswerPipelineGateStatus::Blocked) {
            status = ScholarChatRuntimeAnswerPipelineGateStatus::NeedsReview;
        }
    }

    match status {
        ScholarChatRuntimeAnswerPipelineGateStatus::Blocked => {
            push_unique_text(
                &mut pipeline_gate_reasons,
                "The future runtime answer pipeline cannot proceed later because one or more dependencies are blocked or failed.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Resolve the blocked dependency preview before previewing the runtime answer pipeline gate again.",
            );
        }
        ScholarChatRuntimeAnswerPipelineGateStatus::NeedsReview => {
            push_unique_text(
                &mut pipeline_gate_reasons,
                "The future runtime answer pipeline still needs review before it can be allowed later.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Review the dependency preview warnings before trying the runtime answer pipeline gate again.",
            );
        }
        ScholarChatRuntimeAnswerPipelineGateStatus::ReadyLater => {
            push_unique_text(
                &mut pipeline_gate_reasons,
                "The future runtime answer pipeline gate is satisfied and no answer was generated.",
            );
            push_unique_text(
                &mut next_required_actions,
                "A future Scholar Chat runtime answer pipeline step can be added later without changing this preview.",
            );
        }
    }

    let summary = match status {
        ScholarChatRuntimeAnswerPipelineGateStatus::Blocked => {
            if grounded_answer_execution_plan_preview.status == ScholarChatGroundedAnswerExecutionPlanStatus::Blocked {
                "The future runtime answer pipeline is blocked because the grounded-answer execution plan is blocked.".to_string()
            } else if runtime_diagnostic_result_preview.status == ScholarChatRuntimeDiagnosticResultStatus::Blocked {
                "The future runtime answer pipeline is blocked because the runtime diagnostic result is blocked.".to_string()
            } else if runtime_diagnostic_result_preview.status == ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticFailed {
                "The future runtime answer pipeline is blocked because the runtime diagnostic result failed.".to_string()
            } else {
                "The future runtime answer pipeline is blocked until the dependency previews are ready later.".to_string()
            }
        }
        ScholarChatRuntimeAnswerPipelineGateStatus::NeedsReview => {
            if !normalized_requests_match {
                "The future runtime answer pipeline needs review because the dependency previews do not use the same normalized Scholar Chat inputs.".to_string()
            } else if grounded_answer_execution_plan_preview.status == ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview {
                "The future runtime answer pipeline needs review because the grounded-answer execution plan still needs review.".to_string()
            } else if runtime_diagnostic_result_preview.status == ScholarChatRuntimeDiagnosticResultStatus::NeedsReview {
                "The future runtime answer pipeline needs review because the runtime diagnostic result still needs review.".to_string()
            } else {
                "The future runtime answer pipeline needs review before it can be allowed later.".to_string()
            }
        }
        ScholarChatRuntimeAnswerPipelineGateStatus::ReadyLater => {
            "The future runtime answer pipeline gate is ready later, but no answer was generated.".to_string()
        }
    };

    Ok(ScholarChatRuntimeAnswerPipelineGatePreview {
        status,
        grounded_answer_execution_plan_status,
        grounded_answer_execution_readiness_status,
        runtime_diagnostic_result_status,
        runtime_diagnostic_bridge_status,
        smoke_diagnostic_status,
        smoke_execution_plan_status,
        smoke_readiness_status,
        capability_status,
        version_probe_status,
        probe_readiness_status,
        validation_status,
        adapter_contract_status,
        adapter_kind,
        selected_source_count,
        normalized_model_family,
        normalized_model_format,
        safe_executable_file_name,
        safe_model_file_name,
        diagnostic_prompt_char_count,
        max_output_tokens,
        timeout_ms,
        pipeline_gate_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        gate_only: true,
        no_smoke_execution: true,
        no_runtime_inference: true,
        no_new_process_spawn: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_answer_draft_created: true,
        no_grounded_answer_created: true,
        no_final_answer_created: true,
        no_grounding_applied: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

fn runtime_diagnostic_result_metadata_mismatch_fields(
    bridge_preview: &ScholarChatRuntimeDiagnosticBridgePreview,
    diagnostic_preview: &LocalRuntimeSmokeDiagnosticPreview,
) -> Vec<&'static str> {
    let mut mismatches = Vec::new();
    if bridge_preview.smoke_execution_plan_status != diagnostic_preview.smoke_execution_plan_status {
        mismatches.push("smoke_execution_plan_status");
    }
    if bridge_preview.smoke_readiness_status != diagnostic_preview.smoke_readiness_status {
        mismatches.push("smoke_readiness_status");
    }
    if bridge_preview.capability_status != diagnostic_preview.capability_status {
        mismatches.push("capability_status");
    }
    if bridge_preview.version_probe_status != diagnostic_preview.version_probe_status {
        mismatches.push("version_probe_status");
    }
    if bridge_preview.probe_readiness_status != diagnostic_preview.probe_readiness_status {
        mismatches.push("probe_readiness_status");
    }
    if bridge_preview.validation_status != diagnostic_preview.validation_status {
        mismatches.push("validation_status");
    }
    if bridge_preview.adapter_contract_status != diagnostic_preview.adapter_contract_status {
        mismatches.push("adapter_contract_status");
    }
    if bridge_preview.adapter_kind != diagnostic_preview.adapter_kind {
        mismatches.push("adapter_kind");
    }
    if bridge_preview.normalized_model_family != diagnostic_preview.normalized_model_family {
        mismatches.push("normalized_model_family");
    }
    if bridge_preview.normalized_model_format != diagnostic_preview.normalized_model_format {
        mismatches.push("normalized_model_format");
    }
    if bridge_preview.safe_executable_file_name != diagnostic_preview.safe_executable_file_name {
        mismatches.push("safe_executable_file_name");
    }
    if bridge_preview.safe_model_file_name != diagnostic_preview.safe_model_file_name {
        mismatches.push("safe_model_file_name");
    }
    mismatches
}

pub fn preview_scholar_chat_runtime_diagnostic_result(
    root: impl Into<PathBuf>,
    request: ScholarChatRuntimeDiagnosticResultPreviewRequest,
) -> AegisResult<ScholarChatRuntimeDiagnosticResultPreview> {
    let bridge_preview = preview_scholar_chat_runtime_diagnostic_bridge(root, request.bridge_preview_request)?;
    let diagnostic_preview = request.diagnostic_preview;
    let mismatched_fields =
        runtime_diagnostic_result_metadata_mismatch_fields(&bridge_preview, &diagnostic_preview);

    let mut blockers = bridge_preview.blockers.clone();
    let mut warnings = bridge_preview.warnings.clone();
    let mut runtime_result_reasons = bridge_preview.runtime_diagnostic_reasons.clone();
    let mut next_required_actions = bridge_preview.next_required_actions.clone();

    for blocker in &diagnostic_preview.blockers {
        push_runtime_diagnostic_blocker(&mut blockers, &blocker.kind, &blocker.message);
    }
    for warning in &diagnostic_preview.warnings {
        push_runtime_diagnostic_warning(&mut warnings, &warning.kind, &warning.message);
    }

    let metadata_mismatch_message = if mismatched_fields.is_empty() {
        None
    } else {
        Some(format!(
            "Runtime diagnostic metadata mismatch between the bridge preview and the provided smoke diagnostic preview: {}.",
            mismatched_fields.join(", ")
        ))
    };
    let has_metadata_mismatch = metadata_mismatch_message.is_some();

    let status = if matches!(bridge_preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::Blocked) {
        ScholarChatRuntimeDiagnosticResultStatus::Blocked
    } else if matches!(bridge_preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::NeedsReview) {
        ScholarChatRuntimeDiagnosticResultStatus::NeedsReview
    } else if matches!(diagnostic_preview.status, LocalRuntimeSmokeDiagnosticStatus::Blocked) {
        ScholarChatRuntimeDiagnosticResultStatus::Blocked
    } else if metadata_mismatch_message.is_some() {
        ScholarChatRuntimeDiagnosticResultStatus::NeedsReview
    } else {
        match diagnostic_preview.status {
            LocalRuntimeSmokeDiagnosticStatus::SmokeFailed | LocalRuntimeSmokeDiagnosticStatus::TimedOut => {
                ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticFailed
            }
            LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded => {
                ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticSucceededLater
            }
            LocalRuntimeSmokeDiagnosticStatus::Blocked => ScholarChatRuntimeDiagnosticResultStatus::Blocked,
        }
    };

    match bridge_preview.status {
        ScholarChatRuntimeDiagnosticBridgeStatus::Blocked => {
            push_unique_text(
                &mut runtime_result_reasons,
                "The runtime diagnostic bridge preview is blocked.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Resolve the runtime diagnostic bridge preview blockers first.",
            );
        }
        ScholarChatRuntimeDiagnosticBridgeStatus::NeedsReview => {
            push_unique_text(
                &mut runtime_result_reasons,
                "The runtime diagnostic bridge preview still needs review.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Resolve the runtime diagnostic bridge preview review items first.",
            );
        }
        ScholarChatRuntimeDiagnosticBridgeStatus::RuntimeDiagnosticReadyLater => {
            push_unique_text(
                &mut runtime_result_reasons,
                "The runtime diagnostic bridge preview is ready later for a smoke diagnostic result preview.",
            );
        }
    }

    match diagnostic_preview.status {
        LocalRuntimeSmokeDiagnosticStatus::Blocked => {
            push_unique_text(
                &mut runtime_result_reasons,
                "The provided smoke diagnostic preview is blocked.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Provide a smoke diagnostic preview that is not blocked.",
            );
        }
        LocalRuntimeSmokeDiagnosticStatus::SmokeFailed => {
            push_unique_text(
                &mut runtime_result_reasons,
                "The provided smoke diagnostic preview failed.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Review the failed smoke diagnostic preview before using it for Scholar Chat.",
            );
        }
        LocalRuntimeSmokeDiagnosticStatus::TimedOut => {
            push_unique_text(
                &mut runtime_result_reasons,
                "The provided smoke diagnostic preview timed out.",
            );
            push_unique_text(
                &mut next_required_actions,
                "Review the timed-out smoke diagnostic preview before using it for Scholar Chat.",
            );
        }
        LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded => {
            push_unique_text(
                &mut runtime_result_reasons,
                "The provided smoke diagnostic preview succeeded.",
            );
            push_unique_text(
                &mut next_required_actions,
                "A future Scholar Chat runtime diagnostic step can reuse this result later.",
            );
        }
    }

    if let Some(message) = &metadata_mismatch_message {
        push_runtime_diagnostic_warning(
            &mut warnings,
            "runtime_diagnostic_metadata_mismatch",
            &message,
        );
        push_runtime_diagnostic_blocker(
            &mut blockers,
            "runtime_diagnostic_metadata_mismatch",
            &message,
        );
        push_unique_text(&mut runtime_result_reasons, &message);
        push_unique_text(
            &mut next_required_actions,
            "Make the bridge preview and smoke diagnostic preview metadata match before reusing this result.",
        );
    }

    let summary = match status {
        ScholarChatRuntimeDiagnosticResultStatus::Blocked => {
            if matches!(bridge_preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::Blocked) {
                "The runtime diagnostic result preview is blocked because the bridge preview is blocked.".to_string()
            } else if matches!(diagnostic_preview.status, LocalRuntimeSmokeDiagnosticStatus::Blocked) {
                "The runtime diagnostic result preview is blocked because the provided smoke diagnostic preview is blocked.".to_string()
            } else {
                "The runtime diagnostic result preview is blocked until the bridge preview and provided smoke diagnostic preview are ready later.".to_string()
            }
        }
        ScholarChatRuntimeDiagnosticResultStatus::NeedsReview => {
            if has_metadata_mismatch {
                "The runtime diagnostic result preview needs review because the bridge preview and provided smoke diagnostic preview do not match.".to_string()
            } else {
                "The runtime diagnostic result preview still needs review because the bridge preview still needs review.".to_string()
            }
        }
        ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticFailed => {
            "The provided smoke diagnostic preview failed, so the runtime diagnostic result cannot be used yet.".to_string()
        }
        ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticSucceededLater => {
            "The provided smoke diagnostic preview succeeded, so the runtime diagnostic result is ready later for Scholar Chat.".to_string()
        }
    };

    Ok(ScholarChatRuntimeDiagnosticResultPreview {
        status,
        bridge_status: bridge_preview.status.clone(),
        smoke_diagnostic_status: diagnostic_preview.status,
        smoke_execution_plan_status: diagnostic_preview.smoke_execution_plan_status,
        smoke_readiness_status: diagnostic_preview.smoke_readiness_status,
        capability_status: diagnostic_preview.capability_status,
        version_probe_status: diagnostic_preview.version_probe_status,
        probe_readiness_status: diagnostic_preview.probe_readiness_status,
        validation_status: diagnostic_preview.validation_status,
        adapter_contract_status: diagnostic_preview.adapter_contract_status,
        adapter_kind: diagnostic_preview.adapter_kind,
        normalized_model_family: diagnostic_preview.normalized_model_family,
        normalized_model_format: diagnostic_preview.normalized_model_format,
        safe_executable_file_name: diagnostic_preview.safe_executable_file_name,
        safe_model_file_name: diagnostic_preview.safe_model_file_name,
        diagnostic_prompt_char_count: diagnostic_preview.diagnostic_prompt_char_count,
        max_output_tokens: diagnostic_preview.max_output_tokens,
        timeout_ms: diagnostic_preview.timeout_ms,
        exit_code: diagnostic_preview.exit_code,
        stdout_preview: diagnostic_preview.stdout_preview,
        stderr_preview: diagnostic_preview.stderr_preview,
        stdout_truncated: diagnostic_preview.stdout_truncated,
        stderr_truncated: diagnostic_preview.stderr_truncated,
        runtime_result_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        diagnostic_result_only: true,
        no_smoke_execution: true,
        no_runtime_inference: true,
        no_new_process_spawn: true,
        no_llm_call: true,
        no_answer_generated: true,
        no_answer_draft_created: true,
        no_grounded_answer_created: true,
        no_final_answer_created: true,
        no_grounding_applied: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

fn grounded_answer_build_preflight_required_inputs() -> Vec<String> {
    vec![
        "build_request_ready_later".to_string(),
        "answer_draft_id".to_string(),
        "answer_draft_present".to_string(),
        "answer_draft_readable".to_string(),
    ]
}

fn grounded_answer_build_preflight_missing_inputs(
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_id: Option<&String>,
    answer_draft_present: bool,
    answer_draft_readable: bool,
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        build_request_preview.status,
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
    ) {
        missing_inputs.push("build_request_ready_later".to_string());
    } else if !matches!(status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater) {
        if !answer_draft_id_present || answer_draft_id.is_none() {
            missing_inputs.push("answer_draft_id".to_string());
        } else if !answer_draft_present {
            missing_inputs.push("answer_draft_present".to_string());
            missing_inputs.push("answer_draft_readable".to_string());
        } else if !answer_draft_readable {
            missing_inputs.push("answer_draft_readable".to_string());
        }
    }
    missing_inputs
}

fn grounded_answer_build_preflight_summary(
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_present: bool,
    answer_draft_readable: bool,
    answer_draft_claim_count: usize,
) -> String {
    match status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => {
            if !matches!(
                build_request_preview.status,
                ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
            ) {
                format!(
                    "Grounded-answer build preflight is blocked because the build request is {:?}.",
                    build_request_preview.status
                )
            } else if !answer_draft_id_present {
                "Grounded-answer build preflight is blocked until an answer draft ID is provided.".to_string()
            } else if !answer_draft_present {
                "Grounded-answer build preflight is blocked until a matching AnswerDraft artifact is found."
                    .to_string()
            } else if !answer_draft_readable {
                "Grounded-answer build preflight is blocked because the referenced AnswerDraft artifact is unreadable."
                    .to_string()
            } else {
                "Grounded-answer build preflight is blocked until the request is ready later.".to_string()
            }
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => {
            "Grounded-answer build preflight needs review because the build request still needs review."
                .to_string()
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => format!(
            "The grounded-answer build preflight is ready later: the build request is ready later, the referenced AnswerDraft is readable, and it contains {} claim(s).",
            answer_draft_claim_count
        ),
    }
}

fn grounded_answer_build_preflight_reasons(
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_present: bool,
    answer_draft_readable: bool,
    answer_draft_claim_count: usize,
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Build request status: {:?}", build_request_preview.status),
        format!("Build intent status: {:?}", build_request_preview.build_intent_status),
        format!("Write eligibility status: {:?}", build_request_preview.write_eligibility_status),
        format!("Candidate status: {:?}", build_request_preview.candidate_status),
        format!("Answer draft ID present: {}", answer_draft_id_present),
        format!("Answer draft present: {}", answer_draft_present),
        format!("Answer draft readable: {}", answer_draft_readable),
        format!("Answer draft claim count: {}", answer_draft_claim_count),
    ];
    match status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The preflight is blocked until the build request is ready later and the referenced AnswerDraft artifact is readable.",
            );
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => {
            push_unique_text(
                &mut reasons,
                "The build request still needs review before the AnswerDraft preflight can be accepted.",
            );
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => {
            push_unique_text(
                &mut reasons,
                "The referenced AnswerDraft artifact is readable and the preflight is ready later.",
            );
        }
    }
    reasons
}

fn grounded_answer_build_preflight_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_present: bool,
    answer_draft_readable: bool,
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => {
            if !matches!(
                build_request_preview.status,
                ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Bring the grounded-answer build request to request_ready_later first.",
                );
            } else if !answer_draft_id_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide an answer draft ID before a future grounded-answer build request can proceed.",
                );
            } else if !answer_draft_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Locate or create the matching AnswerDraft artifact before a future grounded-answer build request can proceed.",
                );
            } else if !answer_draft_readable {
                push_unique_text(
                    &mut next_required_actions,
                    "Create or repair the referenced AnswerDraft artifact before a future grounded-answer build request can proceed.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported or unsupported items before checking preflight again.",
            );
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future GroundedAnswer service call can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_execution_readiness_required_inputs() -> Vec<String> {
    vec![
        "build_preflight_ready_later".to_string(),
        "answer_draft_readable".to_string(),
        "execution_consent".to_string(),
    ]
}

fn grounded_answer_execution_readiness_missing_inputs(
    build_preflight_status: &ScholarChatGroundedAnswerBuildPreflightStatus,
    answer_draft_readable: bool,
    execution_consent: bool,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        build_preflight_status,
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
    ) {
        missing_inputs.push("build_preflight_ready_later".to_string());
    }
    if !answer_draft_readable {
        missing_inputs.push("answer_draft_readable".to_string());
    }
    if !execution_consent {
        missing_inputs.push("execution_consent".to_string());
    }
    missing_inputs
}

fn grounded_answer_execution_readiness_summary(
    status: &ScholarChatGroundedAnswerExecutionReadinessStatus,
    build_preflight_preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    execution_consent: bool,
) -> String {
    match status {
        ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked => {
            if !matches!(
                build_preflight_preview.status,
                ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
            ) {
                format!(
                    "Execution-readiness preview is blocked because the grounded-answer build preflight is {:?}.",
                    build_preflight_preview.status
                )
            } else if !build_preflight_preview.answer_draft_readable {
                "Execution-readiness preview is blocked because the referenced AnswerDraft artifact is unreadable."
                    .to_string()
            } else if !execution_consent {
                "Execution-readiness preview is blocked until execution consent is given.".to_string()
            } else {
                "Execution-readiness preview is blocked until the request is ready later.".to_string()
            }
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview => {
            "Execution-readiness preview needs review because the grounded-answer build preflight still needs review."
                .to_string()
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater => {
            "The execution-readiness preview is ready later: the build preflight is ready later, the referenced AnswerDraft is readable, and execution consent is true."
                .to_string()
        }
    }
}

fn grounded_answer_execution_readiness_reasons(
    build_preflight_preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    execution_consent: bool,
    status: &ScholarChatGroundedAnswerExecutionReadinessStatus,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Build preflight status: {:?}", build_preflight_preview.status),
        format!("Build request status: {:?}", build_preflight_preview.build_request_status),
        format!("Build intent status: {:?}", build_preflight_preview.build_intent_status),
        format!("Write eligibility status: {:?}", build_preflight_preview.write_eligibility_status),
        format!("Candidate status: {:?}", build_preflight_preview.candidate_status),
        format!("Answer draft present: {}", build_preflight_preview.answer_draft_present),
        format!("Answer draft readable: {}", build_preflight_preview.answer_draft_readable),
        format!("Answer draft claim count: {}", build_preflight_preview.answer_draft_claim_count),
        format!("Execution consent: {}", execution_consent),
    ];
    match status {
        ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The execution-readiness preview is blocked until the build preflight is ready later, the referenced AnswerDraft is readable, and execution consent is given.",
            );
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut reasons,
                "The build preflight still needs review before execution readiness can be accepted.",
            );
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater => {
            push_unique_text(
                &mut reasons,
                "The referenced AnswerDraft is readable, execution consent is true, and the execution-readiness preview is ready later.",
            );
        }
    }
    reasons
}

fn grounded_answer_execution_readiness_next_required_actions(
    status: &ScholarChatGroundedAnswerExecutionReadinessStatus,
    build_preflight_preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    execution_consent: bool,
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match status {
        ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked => {
            if !matches!(
                build_preflight_preview.status,
                ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Bring the grounded-answer build preflight to preflight_ready_later first.",
                );
            }
            if !build_preflight_preview.answer_draft_readable {
                push_unique_text(
                    &mut next_required_actions,
                    "Create or repair the referenced AnswerDraft artifact before execution readiness can proceed.",
                );
            }
            if !execution_consent {
                push_unique_text(
                    &mut next_required_actions,
                    "Confirm execution consent before a future grounded-answer build can proceed.",
                );
            }
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the grounded-answer build preflight before checking execution readiness again.",
            );
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future GroundedAnswer service call can be added later when execution is enabled.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_execution_plan_planned_inputs() -> Vec<String> {
    vec![
        "normalized_prompt".to_string(),
        "selected_source_ids".to_string(),
        "answer_draft_id".to_string(),
        "answer_draft_readable".to_string(),
        "execution_consent".to_string(),
    ]
}

fn grounded_answer_execution_plan_planned_outputs() -> Vec<String> {
    vec![
        "grounded_answer_execution_plan".to_string(),
        "future_grounded_answer_plan_metadata".to_string(),
        "future_grounded_answer_status".to_string(),
    ]
}

fn grounded_answer_execution_plan_planned_write_targets() -> Vec<String> {
    vec![
        "grounded_answer_artifact".to_string(),
        "registry_status_change".to_string(),
        "audit_log_entry".to_string(),
    ]
}

fn grounded_answer_execution_plan_required_inputs() -> Vec<String> {
    vec![
        "build_preflight_ready_later".to_string(),
        "answer_draft_readable".to_string(),
        "execution_consent".to_string(),
    ]
}

fn grounded_answer_execution_plan_missing_inputs(
    readiness_preview: &ScholarChatGroundedAnswerExecutionReadinessPreview,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        readiness_preview.build_preflight_status,
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
    ) {
        missing_inputs.push("build_preflight_ready_later".to_string());
    }
    if !readiness_preview.answer_draft_readable {
        missing_inputs.push("answer_draft_readable".to_string());
    }
    if !readiness_preview.execution_consent {
        missing_inputs.push("execution_consent".to_string());
    }
    missing_inputs
}

fn grounded_answer_execution_plan_summary(
    status: &ScholarChatGroundedAnswerExecutionPlanStatus,
    readiness_preview: &ScholarChatGroundedAnswerExecutionReadinessPreview,
) -> String {
    match status {
        ScholarChatGroundedAnswerExecutionPlanStatus::Blocked => {
            if !matches!(
                readiness_preview.build_preflight_status,
                ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
            ) {
                "Execution plan preview is blocked until the grounded-answer execution readiness is ready later.".to_string()
            } else if !readiness_preview.execution_consent {
                "Execution plan preview is blocked until execution consent is given.".to_string()
            } else if !readiness_preview.answer_draft_readable {
                "Execution plan preview is blocked until the referenced AnswerDraft is readable.".to_string()
            } else {
                "Execution plan preview is blocked.".to_string()
            }
        }
        ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview => {
            "Execution plan preview needs review because the grounded-answer execution readiness still needs review."
                .to_string()
        }
        ScholarChatGroundedAnswerExecutionPlanStatus::PlanReadyLater => {
            "Execution plan preview is ready later: the grounded-answer execution readiness is ready later and execution consent is true."
                .to_string()
        }
    }
}

fn grounded_answer_execution_plan_reasons(
    readiness_preview: &ScholarChatGroundedAnswerExecutionReadinessPreview,
    status: &ScholarChatGroundedAnswerExecutionPlanStatus,
    planned_operation: &str,
    planned_inputs: &[String],
    planned_outputs: &[String],
    planned_write_targets: &[String],
) -> Vec<String> {
    let mut reasons = vec![
        format!("Plan status: {:?}", status),
        format!("Readiness status: {:?}", readiness_preview.status),
        format!("Build preflight status: {:?}", readiness_preview.build_preflight_status),
        format!("Build request status: {:?}", readiness_preview.build_request_status),
        format!("Build intent status: {:?}", readiness_preview.build_intent_status),
        format!("Write eligibility status: {:?}", readiness_preview.write_eligibility_status),
        format!("Candidate status: {:?}", readiness_preview.candidate_status),
        format!("Execution consent: {}", readiness_preview.execution_consent),
        format!("Planned operation: {planned_operation}"),
        format!("Planned inputs: {}", planned_inputs.join(", ")),
        format!("Planned outputs: {}", planned_outputs.join(", ")),
        format!("Planned write targets: {}", planned_write_targets.join(", ")),
        format!("Answer draft present: {}", readiness_preview.answer_draft_present),
        format!("Answer draft readable: {}", readiness_preview.answer_draft_readable),
        format!("Answer draft claim count: {}", readiness_preview.answer_draft_claim_count),
    ];
    reasons.extend(readiness_preview.readiness_reasons.iter().cloned());
    reasons
}

fn grounded_answer_execution_plan_next_required_actions(
    status: &ScholarChatGroundedAnswerExecutionPlanStatus,
    readiness_preview: &ScholarChatGroundedAnswerExecutionReadinessPreview,
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match status {
        ScholarChatGroundedAnswerExecutionPlanStatus::Blocked => {
            if !matches!(
                readiness_preview.build_preflight_status,
                ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Bring the grounded-answer execution readiness to execution_ready_later first.",
                );
            }
            if !readiness_preview.answer_draft_readable {
                push_unique_text(
                    &mut next_required_actions,
                    "Create or repair the referenced AnswerDraft artifact before planning execution.",
                );
            }
            if !readiness_preview.execution_consent {
                push_unique_text(
                    &mut next_required_actions,
                    "Confirm execution consent before a future GroundedAnswer build can be planned.",
                );
            }
        }
        ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the grounded-answer execution readiness before planning execution again.",
            );
        }
        ScholarChatGroundedAnswerExecutionPlanStatus::PlanReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future GroundedAnswer build can be planned later when execution is enabled.",
            );
        }
    }
    next_required_actions
}

fn normalize_optional_draft_text(draft_text: Option<String>) -> Option<String> {
    draft_text
        .map(|text| text.trim().to_string())
        .and_then(|text| if text.is_empty() { None } else { Some(text) })
}

fn normalize_optional_answer_draft_id(answer_draft_id: Option<String>) -> AegisResult<Option<String>> {
    match answer_draft_id {
        None => Ok(None),
        Some(answer_draft_id) => {
            let normalized_answer_draft_id = answer_draft_id.trim().to_string();
            if normalized_answer_draft_id.is_empty() {
                Ok(None)
            } else {
                validate_answer_draft_id(&normalized_answer_draft_id)?;
                Ok(Some(normalized_answer_draft_id))
            }
        }
    }
}

fn grounded_answer_build_intent_status(
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> ScholarChatGroundedAnswerBuildIntentStatus {
    match write_eligibility_preview.status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => ScholarChatGroundedAnswerBuildIntentStatus::Blocked,
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            if explicit_user_intent && answer_draft_id_present {
                ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater
            } else {
                ScholarChatGroundedAnswerBuildIntentStatus::Blocked
            }
        }
    }
}

fn grounded_answer_build_intent_required_inputs() -> Vec<String> {
    vec![
        "write_eligible_later".to_string(),
        "explicit_user_intent".to_string(),
        "answer_draft_id".to_string(),
    ]
}

fn grounded_answer_build_intent_missing_inputs(
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        write_eligibility_preview.status,
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
    ) {
        missing_inputs.push("write_eligible_later".to_string());
    }
    if !explicit_user_intent {
        missing_inputs.push("explicit_user_intent".to_string());
    }
    if !answer_draft_id_present {
        missing_inputs.push("answer_draft_id".to_string());
    }
    missing_inputs
}

fn grounded_answer_build_intent_summary(
    status: &ScholarChatGroundedAnswerBuildIntentStatus,
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> String {
    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => match write_eligibility_preview.status {
            ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
                "Grounded-answer build intent is blocked because grounded-answer write eligibility is blocked."
                    .to_string()
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
                "Grounded-answer build intent is blocked because grounded-answer write eligibility still needs review."
                    .to_string()
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
                if !explicit_user_intent && !answer_draft_id_present {
                    "Grounded-answer build intent is blocked until explicit user intent and an answer draft ID are provided."
                        .to_string()
                } else if !explicit_user_intent {
                    "Grounded-answer build intent is blocked until explicit user intent is provided."
                        .to_string()
                } else {
                    "Grounded-answer build intent is blocked until an answer draft ID is provided."
                        .to_string()
                }
            }
        },
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer build intent because weakly supported or unsupported items remain."
                .to_string()
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            "All inputs are present. A future user-confirmed GroundedAnswer build intent can be accepted later."
                .to_string()
        }
    }
}

fn grounded_answer_build_intent_reasons(
    status: &ScholarChatGroundedAnswerBuildIntentStatus,
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Write eligibility status: {:?}", write_eligibility_preview.status),
        format!("Candidate status: {:?}", write_eligibility_preview.candidate_status),
        format!("Explicit user intent: {}", explicit_user_intent),
        format!("Answer draft ID provided: {}", answer_draft_id_present),
    ];
    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => match write_eligibility_preview.status {
            ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
                reasons.push("Grounded-answer write eligibility is blocked.".to_string());
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
                reasons.push("Grounded-answer write eligibility still needs review.".to_string());
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
                if !explicit_user_intent && !answer_draft_id_present {
                    reasons.push("Explicit user intent and answer draft ID are both missing.".to_string());
                } else if !explicit_user_intent {
                    reasons.push("Explicit user intent is missing.".to_string());
                } else {
                    reasons.push("Answer draft ID is missing.".to_string());
                }
            }
        },
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            reasons.push("Weakly supported or unsupported draft items remain.".to_string());
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            reasons.push("All inspected items were supported by local evidence.".to_string());
            reasons.push(
                "A future user-confirmed GroundedAnswer build intent can be accepted later after an explicit implementation phase."
                    .to_string(),
            );
        }
    }
    reasons
}

fn grounded_answer_build_intent_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildIntentStatus,
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> Vec<String> {
    let mut next_required_actions = write_eligibility_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => {
            if !matches!(
                write_eligibility_preview.status,
                ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Resolve grounded-answer write-eligibility blockers before any GroundedAnswer service call.",
                );
            }
            if !explicit_user_intent {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide explicit user intent before any GroundedAnswer service call.",
                );
            }
            if !answer_draft_id_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide an answer draft ID before any GroundedAnswer service call.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before any GroundedAnswer service call.",
            );
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future user-confirmed GroundedAnswer service call can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_build_intent_preview_from_write_eligibility_preview(
    write_eligibility_preview: ScholarChatGroundedAnswerWriteEligibilityPreview,
    normalized_prompt: String,
    normalized_answer_draft_id: Option<String>,
    explicit_user_intent: bool,
) -> ScholarChatGroundedAnswerBuildIntentPreview {
    let answer_draft_id_present = normalized_answer_draft_id.is_some();
    let status = grounded_answer_build_intent_status(
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let mut blockers = write_eligibility_preview.blockers.clone();
    let mut warnings = write_eligibility_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build-intent preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => {
            if !matches!(
                write_eligibility_preview.status,
                ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
            ) {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "write_eligibility_blocked",
                    "Grounded-answer build intent is blocked until grounded-answer write eligibility is available.",
                );
            }
            if !explicit_user_intent {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "explicit_user_intent_missing",
                    "Explicit user intent was not provided.",
                );
            }
            if !answer_draft_id_present {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "answer_draft_id_missing",
                    "No answer draft ID was provided.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and need review before a future GroundedAnswer build intent is added.",
            );
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "intent_ready_later",
                "All inputs are present. This is still only a grounded-answer build-intent preview.",
            );
        }
    }

    let summary = grounded_answer_build_intent_summary(
        &status,
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let required_inputs = grounded_answer_build_intent_required_inputs();
    let missing_inputs = grounded_answer_build_intent_missing_inputs(
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let intent_reasons = grounded_answer_build_intent_reasons(
        &status,
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let next_required_actions = grounded_answer_build_intent_next_required_actions(
        &status,
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );

    ScholarChatGroundedAnswerBuildIntentPreview {
        status,
        write_eligibility_status: write_eligibility_preview.status,
        candidate_status: write_eligibility_preview.candidate_status,
        normalized_prompt,
        selected_source_count: write_eligibility_preview.selected_source_count,
        evidence_candidate_count: write_eligibility_preview.evidence_candidate_count,
        inspected_item_count: write_eligibility_preview.inspected_item_count,
        supported_item_count: write_eligibility_preview.supported_item_count,
        weakly_supported_item_count: write_eligibility_preview.weakly_supported_item_count,
        unsupported_item_count: write_eligibility_preview.unsupported_item_count,
        candidate_statement_count: write_eligibility_preview.candidate_statement_count,
        required_inputs,
        missing_inputs,
        intent_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
    }
}

fn grounded_answer_build_request_status(
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
) -> ScholarChatGroundedAnswerBuildRequestStatus {
    match build_intent_preview.status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => ScholarChatGroundedAnswerBuildRequestStatus::Blocked,
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
        }
    }
}

fn grounded_answer_build_request_required_inputs() -> Vec<String> {
    vec![
        "build_intent_ready_later".to_string(),
        "answer_draft_id".to_string(),
        "selected_source_ids".to_string(),
    ]
}

fn grounded_answer_build_request_missing_inputs(
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        build_intent_preview.status,
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater
    ) {
        missing_inputs.push("build_intent_ready_later".to_string());
    }
    if answer_draft_id.is_none() {
        missing_inputs.push("answer_draft_id".to_string());
    }
    if selected_source_ids.is_empty() {
        missing_inputs.push("selected_source_ids".to_string());
    }
    missing_inputs
}

fn grounded_answer_build_request_summary(
    status: &ScholarChatGroundedAnswerBuildRequestStatus,
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> String {
    let answer_draft_id_summary = if answer_draft_id.is_some() { "present" } else { "missing" };
    let selected_source_summary = if selected_source_ids.is_empty() {
        "no selected source IDs"
    } else {
        "selected source IDs are normalized and ready"
    };
    match status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => format!(
            "Grounded-answer build request is blocked because the build intent is {:?}; answer draft ID is {}; {}.",
            build_intent_preview.status,
            answer_draft_id_summary,
            selected_source_summary,
        ),
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => format!(
            "Grounded-answer build request still needs review because the build intent is {:?}; answer draft ID is {}; {}.",
            build_intent_preview.status,
            answer_draft_id_summary,
            selected_source_summary,
        ),
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            "The grounded-answer build request is normalized and ready later for a future GroundedAnswer service call."
                .to_string()
        }
    }
}

fn grounded_answer_build_request_reasons(
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> Vec<String> {
    let mut request_reasons = vec![
        format!("Build intent status: {:?}", build_intent_preview.status),
        format!("Write eligibility status: {:?}", build_intent_preview.write_eligibility_status),
        format!("Candidate status: {:?}", build_intent_preview.candidate_status),
        format!("Answer draft ID provided: {}", answer_draft_id.is_some()),
        format!("Selected source count: {}", selected_source_ids.len()),
    ];
    match build_intent_preview.status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => {
            request_reasons.push("Grounded-answer build intent must be ready later before a request can be accepted.".to_string());
        }
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            request_reasons.push(
                "Weakly supported or unsupported draft items remain and need review before a future GroundedAnswer service call."
                    .to_string(),
            );
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            request_reasons.push(
                "The request fields are normalized and ready later for a future GroundedAnswer service call.".to_string(),
            );
        }
    }
    request_reasons
}

fn grounded_answer_build_request_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildRequestStatus,
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> Vec<String> {
    let mut next_required_actions = build_intent_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => {
            if !matches!(
                build_intent_preview.status,
                ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Resolve grounded-answer build-intent blockers before any GroundedAnswer service call.",
                );
            }
            if answer_draft_id.is_none() {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide an answer draft ID before any GroundedAnswer service call.",
                );
            }
            if selected_source_ids.is_empty() {
                push_unique_text(
                    &mut next_required_actions,
                    "Select at least one source ID before any GroundedAnswer service call.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the normalized request fields before any GroundedAnswer service call.",
            );
        }
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future user-confirmed GroundedAnswer service call can be added later using the normalized request fields.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_build_request_preview_from_build_intent_preview(
    build_intent_preview: ScholarChatGroundedAnswerBuildIntentPreview,
    normalized_prompt: String,
    normalized_answer_draft_id: Option<String>,
    normalized_selected_source_ids: Vec<String>,
) -> ScholarChatGroundedAnswerBuildRequestPreview {
    let status = grounded_answer_build_request_status(&build_intent_preview);
    let mut blockers = build_intent_preview.blockers.clone();
    let mut warnings = build_intent_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build-request preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => {
            push_grounding_inspection_blocker(
                &mut blockers,
                "build_request_blocked",
                "Grounded-answer build request is blocked until the build intent is ready later.",
            );
        }
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "request_needs_review",
                "The normalized grounded-answer build request still needs review before any GroundedAnswer service call.",
            );
        }
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "request_ready_later",
                "The normalized grounded-answer build request is ready later for a future GroundedAnswer service call.",
            );
        }
    }

    let answer_draft_id = normalized_answer_draft_id;
    let selected_source_count = normalized_selected_source_ids.len();
    let required_inputs = grounded_answer_build_request_required_inputs();
    let missing_inputs = grounded_answer_build_request_missing_inputs(
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );
    let request_reasons = grounded_answer_build_request_reasons(
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );
    let next_required_actions = grounded_answer_build_request_next_required_actions(
        &status,
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );
    let summary = grounded_answer_build_request_summary(
        &status,
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );

    ScholarChatGroundedAnswerBuildRequestPreview {
        status,
        build_intent_status: build_intent_preview.status,
        write_eligibility_status: build_intent_preview.write_eligibility_status,
        candidate_status: build_intent_preview.candidate_status,
        normalized_prompt,
        selected_source_count,
        evidence_candidate_count: build_intent_preview.evidence_candidate_count,
        inspected_item_count: build_intent_preview.inspected_item_count,
        supported_item_count: build_intent_preview.supported_item_count,
        weakly_supported_item_count: build_intent_preview.weakly_supported_item_count,
        unsupported_item_count: build_intent_preview.unsupported_item_count,
        candidate_statement_count: build_intent_preview.candidate_statement_count,
        answer_draft_id,
        selected_source_ids: normalized_selected_source_ids,
        required_inputs,
        missing_inputs,
        request_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
    }
}

fn push_grounding_inspection_warning(
    warnings: &mut Vec<ScholarChatDraftGroundingInspectionWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(ScholarChatDraftGroundingInspectionWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_unique_text(items: &mut Vec<String>, value: &str) {
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

fn push_runtime_diagnostic_warning(
    warnings: &mut Vec<LocalRuntimeProbeWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(LocalRuntimeProbeWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_runtime_diagnostic_blocker(
    blockers: &mut Vec<LocalRuntimeProbeWarning>,
    kind: &str,
    message: &str,
) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(LocalRuntimeProbeWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_grounding_inspection_blocker(
    blockers: &mut Vec<ScholarChatDraftGroundingInspectionBlocker>,
    kind: &str,
    message: &str,
) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(ScholarChatDraftGroundingInspectionBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn inspect_draft_grounding_items(
    draft_text: &str,
    evidence_candidates: &[ScholarChatEvidenceCandidate],
    max_items: usize,
) -> DraftGroundingInspectionItems {
    let mut items = Vec::new();
    let mut supported_item_count = 0;
    let mut weakly_supported_item_count = 0;
    let mut unsupported_item_count = 0;
    let warnings = Vec::new();

    let segments = split_draft_text_into_segments(draft_text);
    let was_clamped = segments.len() > max_items;
    for (item_index, segment) in segments.into_iter().take(max_items).enumerate() {
        let normalized_segment = compact_text_preview(&segment, 180);
        let item_terms = inspection_terms(&normalized_segment);
        if item_terms.is_empty() {
            continue;
        }

        let mut matched_evidence = Vec::new();
        for candidate in evidence_candidates {
            let candidate_terms = inspection_terms(&candidate.preview)
                .into_iter()
                .chain(candidate.matched_terms.iter().flat_map(|term| inspection_terms(term).into_iter()))
                .collect::<BTreeSet<_>>();
            let overlap = item_terms.intersection(&candidate_terms).count();
            if overlap > 0 {
                matched_evidence.push((overlap, candidate));
            }
        }

        let support_status = if matched_evidence.is_empty() {
            unsupported_item_count += 1;
            ScholarChatDraftGroundingSupportStatus::Unsupported
        } else {
            let best_overlap = matched_evidence.iter().map(|(overlap, _)| *overlap).max().unwrap_or(0);
            if best_overlap >= 2 {
                supported_item_count += 1;
                ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence
            } else {
                weakly_supported_item_count += 1;
                ScholarChatDraftGroundingSupportStatus::WeaklySupported
            }
        };

        let mut source_ids = BTreeSet::new();
        let mut locator_previews = BTreeSet::new();
        let matched_evidence_count = matched_evidence.len();
        for (_, candidate) in matched_evidence {
            source_ids.insert(candidate.source_id.clone());
            locator_previews.insert(locator_preview(&candidate.locator));
        }

        items.push(ScholarChatDraftGroundingInspectionItem {
            item_index,
            text_preview: normalized_segment,
            support_status,
            matched_evidence_count,
            source_ids: source_ids.into_iter().collect(),
            locator_previews: locator_previews.into_iter().collect(),
        });
    }

    DraftGroundingInspectionItems {
        items,
        supported_item_count,
        weakly_supported_item_count,
        unsupported_item_count,
        warnings,
        was_clamped,
    }
}

fn split_draft_text_into_segments(draft_text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();

    for ch in draft_text.chars() {
        if matches!(ch, '.' | '!' | '?' | '\n' | '\r') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                segments.push(trimmed.to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed.to_string());
    }

    if segments.is_empty() && !draft_text.trim().is_empty() {
        segments.push(draft_text.trim().to_string());
    }

    segments
}

fn compact_text_preview(text: &str, max_chars: usize) -> String {
    let compacted = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview = compacted.trim().to_string();
    if preview.chars().count() > max_chars {
        let mut truncated = String::new();
        for ch in preview.chars().take(max_chars.saturating_sub(1)) {
            truncated.push(ch);
        }
        truncated.push('…');
        preview = truncated;
    }
    preview
}

fn inspection_terms(text: &str) -> BTreeSet<String> {
    const MIN_MEANINGFUL_TERM_LEN: usize = 3;

    text.split(|ch: char| !ch.is_alphanumeric())
        .filter_map(|term| {
            let normalized = term.trim().to_lowercase();
            if normalized.is_empty() {
                None
            } else if is_stopword_for_draft_grounding_inspection(&normalized) {
                None
            } else if normalized.chars().all(|ch| ch.is_numeric()) {
                Some(normalized)
            } else if normalized.chars().count() < MIN_MEANINGFUL_TERM_LEN {
                None
            } else {
                Some(normalized)
            }
        })
        .collect()
}

fn is_stopword_for_draft_grounding_inspection(term: &str) -> bool {
    matches!(
        term,
        "the" | "and" | "or" | "of" | "to" | "in" | "a" | "an" | "is" | "are" | "was" | "were" | "with" | "for" | "on" | "by" | "as" | "at" | "from" | "this" | "that"
    )
}

fn locator_preview(locator: &CitationLocator) -> String {
    let mut parts = vec![format!("{:?}", locator.locator_type).to_lowercase()];
    if !locator.label.trim().is_empty() {
        parts.push(format!("label={}", locator.label.trim()));
    }
    if let Some(page) = locator.page {
        parts.push(format!("page={page}"));
    }
    if let Some(slide) = locator.slide {
        parts.push(format!("slide={slide}"));
    }
    if let Some(start) = locator.character_start {
        parts.push(format!("chars={start}-{}", locator.character_end.unwrap_or(start)));
    }
    parts.join(" | ")
}

fn render_prompt_pack_for_runtime(prompt_pack: &ScholarChatPromptPack) -> String {
    let mut lines = Vec::new();
    for section in &prompt_pack.sections {
        lines.push(format!("## {}", section.title));
        lines.extend(section.lines.iter().cloned());
        lines.push(String::new());
    }
    lines.join("\n").trim().to_string()
}

fn draft_output_classification(
    grounding_policy: &GroundingPolicy,
    blocked: bool,
) -> ScholarChatDraftOutputClassification {
    if blocked {
        return ScholarChatDraftOutputClassification::Blocked;
    }
    match grounding_policy {
        GroundingPolicy::AllowMarkedModelKnowledge => ScholarChatDraftOutputClassification::UngroundedModelDraft,
        GroundingPolicy::LocalOnly => ScholarChatDraftOutputClassification::GroundedDraftCandidate,
        GroundingPolicy::LocalFirst | GroundingPolicy::ExternalAdaptersLater => {
            ScholarChatDraftOutputClassification::SourceContextDraft
        }
    }
}

fn push_draft_warning(
    warnings: &mut Vec<ScholarChatDraftInferenceWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(ScholarChatDraftInferenceWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_draft_blocker(
    blockers: &mut Vec<ScholarChatDraftInferenceBlocker>,
    kind: &str,
    message: &str,
) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(ScholarChatDraftInferenceBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn build_draft_inference_preview(
    normalized_prompt: String,
    mode: ScholarChatMode,
    grounding_policy: GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
    prompt_pack_section_count: usize,
    prompt_pack_text: String,
    runtime_health_status: LocalModelRuntimeHealthStatus,
    invocation_plan_status: LocalRuntimeInvocationPlanStatus,
    status: ScholarChatDraftInferenceStatus,
    output_classification: ScholarChatDraftOutputClassification,
    allow_model_execution: bool,
    execution_attempted: bool,
    safe_model_file_name: Option<String>,
    safe_executable_file_name: Option<String>,
    stdout_preview: String,
    stderr_preview: String,
    duration_ms: u64,
    exit_code: Option<i32>,
    blockers: Vec<ScholarChatDraftInferenceBlocker>,
    warnings: Vec<ScholarChatDraftInferenceWarning>,
) -> ScholarChatDraftInferencePreview {
    ScholarChatDraftInferencePreview {
        status,
        output_classification,
        normalized_prompt,
        mode,
        grounding_policy,
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        prompt_pack_section_count,
        prompt_char_count: prompt_pack_text.chars().count(),
        runtime_health_status,
        invocation_plan_status,
        allow_model_execution,
        execution_attempted,
        safe_model_file_name,
        safe_executable_file_name,
        stdout_preview,
        stderr_preview,
        duration_ms,
        exit_code,
        draft_only: true,
        preview_only: true,
        not_final_answer: true,
        not_grounded_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        blockers,
        warnings,
    }
}

fn normalized_prompt_or_err(prompt: String) -> AegisResult<String> {
    let normalized_prompt = prompt.trim().to_string();
    if normalized_prompt.is_empty() {
        return Err(AegisError::ScholarChatPromptEmpty);
    }
    Ok(normalized_prompt)
}

fn normalize_selected_source_ids(source_ids: Vec<String>) -> AegisResult<(Vec<String>, usize)> {
    let mut selected_source_ids = Vec::new();
    for source_id in source_ids {
        let normalized_source_id = source_id.trim().to_string();
        validate_source_id(&normalized_source_id)?;
        selected_source_ids.push(normalized_source_id);
    }
    let selected_source_count = selected_source_ids.len();
    Ok((selected_source_ids, selected_source_count))
}

fn normalize_scientific_context_tags(tags: Option<Vec<String>>) -> Vec<String> {
    let mut normalized_tags = BTreeSet::new();
    if let Some(tags) = tags {
        for tag in tags {
            let normalized = normalize_scientific_tag_text(&tag);
            if !normalized.is_empty() {
                normalized_tags.insert(normalized);
            }
        }
    }
    normalized_tags.into_iter().collect()
}

fn normalize_scientific_tag_text(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;
    for ch in value.trim().to_lowercase().chars() {
        if ch.is_alphanumeric() {
            normalized.push(ch);
            last_was_separator = false;
        } else if matches!(ch, ' ' | '-') && !normalized.is_empty() && !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        } else if matches!(ch, ' ' | '-') && normalized.is_empty() {
            continue;
        } else if !normalized.is_empty() && !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }
    normalized.trim_matches('_').to_string()
}

fn scientific_context_mentions(context: &Option<String>, needles: &[&str]) -> bool {
    let Some(context) = context.as_deref() else {
        return false;
    };
    let context = context.to_lowercase();
    needles.iter().any(|needle| context.contains(needle))
}

fn scientific_context_tags_contain(tags: &[String], needles: &[&str]) -> bool {
    tags.iter().any(|tag| needles.iter().any(|needle| tag == needle || tag.contains(needle)))
}

fn scientific_source_registry_family(
    id: &str,
    label: &str,
    domain: &str,
    access_class: ScholarChatScientificSourceAccessClass,
    priority: ScholarChatScientificSourcePriority,
    applies_when: &str,
    active_for_current_context: bool,
    planned_use: &str,
    query_roles: &[&str],
    boundary_notes: &[&str],
) -> ScholarChatScientificSourceRegistrySourceFamily {
    ScholarChatScientificSourceRegistrySourceFamily {
        id: id.to_string(),
        label: label.to_string(),
        domain: domain.to_string(),
        access_class,
        priority,
        applies_when: applies_when.to_string(),
        active_for_current_context,
        planned_use: planned_use.to_string(),
        query_roles: query_roles.iter().map(|value| (*value).to_string()).collect(),
        boundary_notes: boundary_notes.iter().map(|value| (*value).to_string()).collect(),
    }
}

fn scientific_source_registry_access_classes(
    source_families: &[ScholarChatScientificSourceRegistrySourceFamily],
) -> Vec<ScholarChatScientificSourceAccessClass> {
    let mut access_classes = Vec::new();
    for family in source_families {
        if !access_classes.contains(&family.access_class) {
            access_classes.push(family.access_class.clone());
        }
    }
    access_classes
}

fn scientific_source_registry_plan_summary(status: &ScholarChatScientificSourceRegistryStatus, label: Option<&str>, normalized_mode: &str) -> String {
    match status {
        ScholarChatScientificSourceRegistryStatus::Blocked => {
            "Scientific source registry preview blocked because the topic is blank.".to_string()
        }
        ScholarChatScientificSourceRegistryStatus::UnknownConcept => {
            format!(
                "Scientific source registry preview could not yet map the topic '{}' in {} mode.",
                label.unwrap_or("unknown topic"),
                normalized_mode
            )
        }
        ScholarChatScientificSourceRegistryStatus::SourcePlanReady => {
            format!(
                "Scientific source registry preview is ready later for {} in {} mode.",
                label.unwrap_or("the mapped concept"),
                normalized_mode
            )
        }
    }
}

fn scientific_source_registry_plan_steps(normalized_mode: &str) -> Vec<String> {
    let mut steps = vec![
        "Normalize topic, mode, and context tags.".to_string(),
        "Map the topic through the scientific discipline registry preview.".to_string(),
        "Select active source families for the current context.".to_string(),
        "Plan metadata-only queries before later retrieval or indexing.".to_string(),
    ];
    if normalized_mode == "scientific_paper" {
        steps.push("Prefer literature-search planning, deduplication, and citation-safe follow-up phases.".to_string());
    } else if normalized_mode == "course" {
        steps.push("Prefer curriculum metadata, module context, prerequisites, and learning-path support in later phases.".to_string());
    } else {
        steps.push("Plan local evidence first before later Scholar Chat answering.".to_string());
    }
    steps
}

fn scientific_source_registry_ranking_hints(
    recognized_concept: Option<&str>,
    normalized_mode: &str,
    normalized_context_tags: &[String],
    course_context: &Option<String>,
) -> Vec<String> {
    let mut hints = Vec::new();
    match recognized_concept {
        Some("signal_detection_theory") => {
            hints.push("Prefer psychology and psychophysics sources first.".to_string());
            hints.push("Use biomedical sources only when biomedical context is explicit.".to_string());
            hints.push("Prefer method and review sources later where available.".to_string());
        }
        Some("analysis_of_variance") => {
            hints.push("Prefer statistics and methods sources first.".to_string());
            hints.push("Activate psychology-specific sources only when psychology context is explicit.".to_string());
            hints.push("Activate theory sources only when theory or statistics context is explicit.".to_string());
        }
        Some("hypothesis_testing") => {
            hints.push("Prefer methods and statistics sources first.".to_string());
            hints.push("Use psychology sources when applied psychology context is explicit.".to_string());
            hints.push("Later phases should deduplicate by DOI, title, and source identifiers.".to_string());
        }
        _ => {}
    }
    if normalized_mode == "course" {
        hints.push("Course Mode should favor curriculum metadata and local course materials later.".to_string());
    }
    if scientific_context_tags_contain(normalized_context_tags, &["biomedical", "medical", "neuroscience", "clinical", "diagnostics", "medicine"]) {
        hints.push("Biomedical context can activate biomedical source families later.".to_string());
    }
    if scientific_context_mentions(course_context, &["psychology", "psychologie"]) {
        hints.push("Course context can activate psychology-specific source families later.".to_string());
    }
    hints
}

fn scientific_source_registry_deduplication_hints(
    recognized_concept: Option<&str>,
    normalized_mode: &str,
) -> Vec<String> {
    let mut hints = vec![
        "Deduplicate later by DOI, title, and source identifiers.".to_string(),
        "Keep source-family provenance when merging overlapping metadata records.".to_string(),
    ];
    if recognized_concept == Some("signal_detection_theory") {
        hints.push("Deduplicate psychophysics records against broader psychology metadata later.".to_string());
    }
    if normalized_mode == "scientific_paper" {
        hints.push("Deduplicate review and preprint records before evidence-pack planning later.".to_string());
    }
    hints
}

#[derive(Clone, Copy)]
struct ScientificQueryAliasSpec {
    alias: &'static str,
    concept: &'static str,
    topic_label: &'static str,
    language: &'static str,
}

#[derive(Clone, Copy)]
struct ScientificQueryConceptCandidate {
    concept: &'static str,
    topic_label: &'static str,
    start_index: usize,
}

const SCIENTIFIC_QUERY_ALIAS_SPECS: &[ScientificQueryAliasSpec] = &[
    ScientificQueryAliasSpec {
        alias: "Signalentdeckung",
        concept: "signal_detection_theory",
        topic_label: "Signalentdeckung",
        language: "german",
    },
    ScientificQueryAliasSpec {
        alias: "Signalentdeckungstheorie",
        concept: "signal_detection_theory",
        topic_label: "Signalentdeckung",
        language: "german",
    },
    ScientificQueryAliasSpec {
        alias: "signal detection",
        concept: "signal_detection_theory",
        topic_label: "Signalentdeckung",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "signal detection theory",
        concept: "signal_detection_theory",
        topic_label: "Signalentdeckung",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "d prime",
        concept: "signal_detection_theory",
        topic_label: "Signalentdeckung",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "d-prime",
        concept: "signal_detection_theory",
        topic_label: "Signalentdeckung",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "dprime",
        concept: "signal_detection_theory",
        topic_label: "Signalentdeckung",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "ANOVA",
        concept: "analysis_of_variance",
        topic_label: "ANOVA",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "Varianzanalyse",
        concept: "analysis_of_variance",
        topic_label: "ANOVA",
        language: "german",
    },
    ScientificQueryAliasSpec {
        alias: "analysis of variance",
        concept: "analysis_of_variance",
        topic_label: "ANOVA",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "factorial ANOVA",
        concept: "analysis_of_variance",
        topic_label: "ANOVA",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "repeated measures ANOVA",
        concept: "analysis_of_variance",
        topic_label: "ANOVA",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "Hypothesentests",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "german",
    },
    ScientificQueryAliasSpec {
        alias: "Hypothesentest",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "german",
    },
    ScientificQueryAliasSpec {
        alias: "hypothesis testing",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "null hypothesis",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "nullhypothese",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "german",
    },
    ScientificQueryAliasSpec {
        alias: "p-value",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "p value",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "english",
    },
    ScientificQueryAliasSpec {
        alias: "p-wert",
        concept: "hypothesis_testing",
        topic_label: "Hypothesentests",
        language: "german",
    },
];

fn normalize_scientific_query_text(query: &str) -> String {
    query.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn scientific_query_contains_any(query_lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| query_lower.contains(needle))
}

fn scientific_query_detected_aliases(query: &str) -> Vec<ScholarChatScientificQueryDetectedAlias> {
    let query_lower = query.to_lowercase();
    let mut aliases = Vec::new();
    for spec in SCIENTIFIC_QUERY_ALIAS_SPECS {
        let alias_search = spec.alias.to_lowercase();
        if let Some(byte_index) = query_lower.find(&alias_search) {
            aliases.push(ScholarChatScientificQueryDetectedAlias {
                alias: spec.alias.to_string(),
                concept: spec.concept.to_string(),
                language: spec.language.to_string(),
                start_index: query[..byte_index].chars().count(),
            });
        }
    }
    aliases.sort_by(|left, right| {
        left.start_index
            .cmp(&right.start_index)
            .then_with(|| left.alias.cmp(&right.alias))
            .then_with(|| left.concept.cmp(&right.concept))
    });
    aliases
}

fn scientific_query_detected_concepts(query: &str) -> Vec<ScientificQueryConceptCandidate> {
    let query_lower = query.to_lowercase();
    let mut concepts = BTreeMap::<&'static str, ScientificQueryConceptCandidate>::new();
    for spec in SCIENTIFIC_QUERY_ALIAS_SPECS {
        let alias_search = spec.alias.to_lowercase();
        if let Some(byte_index) = query_lower.find(&alias_search) {
            let start_index = query[..byte_index].chars().count();
            let candidate = ScientificQueryConceptCandidate {
                concept: spec.concept,
                topic_label: spec.topic_label,
                start_index,
            };
            concepts
                .entry(spec.concept)
                .and_modify(|existing| {
                    if candidate.start_index < existing.start_index
                        || (candidate.start_index == existing.start_index
                            && candidate.topic_label < existing.topic_label)
                    {
                        *existing = candidate;
                    }
                })
                .or_insert(candidate);
        }
    }
    let mut detected: Vec<_> = concepts.into_values().collect();
    detected.sort_by(|left, right| {
        left.start_index
            .cmp(&right.start_index)
            .then_with(|| left.topic_label.cmp(right.topic_label))
            .then_with(|| left.concept.cmp(right.concept))
    });
    detected
}

fn scientific_query_ambiguity_warnings(
    detected_concepts: &[ScientificQueryConceptCandidate],
) -> Vec<String> {
    if detected_concepts.len() <= 1 {
        return Vec::new();
    }
    let concept_list = detected_concepts
        .iter()
        .map(|candidate| candidate.topic_label.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    vec![
        format!("Detected multiple scientific concepts: {}.", concept_list),
        format!(
            "Preferred first occurrence: {}.",
            detected_concepts
                .first()
                .map(|candidate| candidate.topic_label)
                .unwrap_or("the first inferred concept")
        ),
    ]
}

fn scientific_query_language_hints(
    query_lower: &str,
    detected_aliases: &[ScholarChatScientificQueryDetectedAlias],
) -> Vec<String> {
    let mut hints = Vec::new();
    if detected_aliases.iter().any(|alias| alias.language == "german")
        || scientific_query_contains_any(
            query_lower,
            &[
                "was ist",
                "erklär",
                "erklaer",
                "lernen",
                "klausur",
                "prüfung",
                "pruefung",
                "vorlesung",
                "seminar",
                "modul",
                "auswertung",
                "berechnen",
                "beispiel",
                "unterschied",
                "vergleich",
                "metaanalyse",
                "quelle",
                "quellen",
                "studie",
                "studien",
            ],
        )
    {
        push_unique_text(&mut hints, "german");
    }
    if detected_aliases.iter().any(|alias| alias.language == "english")
        || scientific_query_contains_any(
            query_lower,
            &[
                "what is",
                "explain",
                "define",
                "definition",
                "literature",
                "literatur",
                "paper",
                "papers",
                "review",
                "meta-analysis",
                "citation",
                "cite",
                "compare",
                "comparison",
                "example",
                "calculate",
                "dataset",
                "interpret",
                "regression",
                "experiment",
            ],
        )
    {
        push_unique_text(&mut hints, "english");
    }
    hints
}

fn scientific_query_intent(normalized_mode: &str, query_lower: &str) -> ScholarChatScientificQueryIntent {
    if normalized_mode == "scientific_paper" {
        ScholarChatScientificQueryIntent::LiteratureSearch
    } else if normalized_mode == "course" {
        ScholarChatScientificQueryIntent::CourseLearning
    } else if scientific_query_contains_any(
        query_lower,
        &[
            "literature",
            "literatur",
            "paper",
            "papers",
            "studie",
            "studien",
            "review",
            "meta-analysis",
            "metaanalyse",
            "systematic review",
            "citation",
            "cite",
            "quelle",
            "quellen",
        ],
    ) {
        ScholarChatScientificQueryIntent::LiteratureSearch
    } else if scientific_query_contains_any(
        query_lower,
        &[
            "kurs",
            "course",
            "lernen",
            "explain for exam",
            "klausur",
            "prüfung",
            "pruefung",
            "modul",
            "vorlesung",
            "seminar",
        ],
    ) {
        ScholarChatScientificQueryIntent::CourseLearning
    } else if scientific_query_contains_any(
        query_lower,
        &[
            "anwenden",
            "berechnen",
            "calculate",
            "example",
            "beispiel",
            "dataset",
            "daten",
            "auswertung",
            "interpretieren",
            "regression",
            "experiment",
        ],
    ) {
        ScholarChatScientificQueryIntent::MethodApplication
    } else if scientific_query_contains_any(
        query_lower,
        &[
            "unterschied",
            "compare",
            "comparison",
            "vs",
            "versus",
            "unterschied zwischen",
        ],
    ) {
        ScholarChatScientificQueryIntent::Comparison
    } else if scientific_query_contains_any(
        query_lower,
        &[
            "was ist",
            "erklären",
            "erklaeren",
            "explain",
            "define",
            "definition",
            "meaning",
            "bedeutung",
        ],
    ) {
        ScholarChatScientificQueryIntent::ConceptExplanation
    } else {
        ScholarChatScientificQueryIntent::Unknown
    }
}

fn scientific_query_planned_local_search_queries(recognized_concept: Option<&str>, normalized_query: &str) -> Vec<String> {
    match recognized_concept {
        Some("signal_detection_theory") => vec![
            "Signalentdeckung".to_string(),
            "Signalentdeckungstheorie".to_string(),
            "d prime".to_string(),
            "ROC".to_string(),
        ],
        Some("analysis_of_variance") => vec![
            "ANOVA".to_string(),
            "Varianzanalyse".to_string(),
            "F test".to_string(),
            "repeated measures".to_string(),
        ],
        Some("hypothesis_testing") => vec![
            "Hypothesentests".to_string(),
            "Nullhypothese".to_string(),
            "p-Wert".to_string(),
            "Fehler 1. Art".to_string(),
            "Fehler 2. Art".to_string(),
        ],
        _ if normalized_query.is_empty() => Vec::new(),
        _ => vec![normalized_query.to_string()],
    }
}

fn scientific_query_planned_expanded_queries(recognized_concept: Option<&str>) -> Vec<String> {
    match recognized_concept {
        Some("signal_detection_theory") => vec![
            "Signal Detection Theory".to_string(),
            "Signalentdeckungstheorie".to_string(),
            "d-prime".to_string(),
            "ROC analysis".to_string(),
        ],
        Some("analysis_of_variance") => vec![
            "analysis of variance".to_string(),
            "Varianzanalyse".to_string(),
            "F-test".to_string(),
            "repeated measures ANOVA".to_string(),
        ],
        Some("hypothesis_testing") => vec![
            "hypothesis testing".to_string(),
            "Hypothesentests".to_string(),
            "null hypothesis".to_string(),
            "p-value".to_string(),
        ],
        _ => Vec::new(),
    }
}

fn scientific_query_evidence_requirements(
    normalized_mode: &str,
    query_intent: &ScholarChatScientificQueryIntent,
    source_registry_status: &ScholarChatScientificSourceRegistryStatus,
) -> Vec<String> {
    let mut requirements = Vec::new();
    if !matches!(source_registry_status, ScholarChatScientificSourceRegistryStatus::SourcePlanReady) {
        push_unique_text(
            &mut requirements,
            "source_family_plan_required",
        );
    }
    if normalized_mode == "scholar_chat" || normalized_mode == "course" {
        push_unique_text(
            &mut requirements,
            "local_evidence_required_before_answer",
        );
    }
    if normalized_mode == "scientific_paper"
        || matches!(query_intent, ScholarChatScientificQueryIntent::LiteratureSearch)
    {
        push_unique_text(
            &mut requirements,
            "citation_safe_metadata_required",
        );
        push_unique_text(
            &mut requirements,
            "deduplication_required_before_literature_review",
        );
    }
    if normalized_mode == "course"
        || matches!(query_intent, ScholarChatScientificQueryIntent::CourseLearning)
    {
        push_unique_text(
            &mut requirements,
            "course_material_alignment_required",
        );
    }
    requirements
}

fn normalize_scientific_selected_local_source_ids(ids: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = ids
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn scientific_search_plan_strategy(
    status: &ScholarChatScientificSearchPlanStatus,
    normalized_mode: &str,
    query_intent: &ScholarChatScientificQueryIntent,
) -> ScholarChatScientificSearchStrategy {
    if matches!(status, ScholarChatScientificSearchPlanStatus::Blocked) {
        ScholarChatScientificSearchStrategy::Blocked
    } else if normalized_mode == "course" {
        ScholarChatScientificSearchStrategy::CourseLocalFirst
    } else if normalized_mode == "scientific_paper"
        || matches!(query_intent, ScholarChatScientificQueryIntent::LiteratureSearch)
    {
        ScholarChatScientificSearchStrategy::MetadataFirst
    } else {
        ScholarChatScientificSearchStrategy::LocalFirst
    }
}

fn scientific_search_plan_step_notes() -> Vec<String> {
    vec![
        "preview-only".to_string(),
        "no retrieval executed".to_string(),
        "no files read".to_string(),
        "no web request".to_string(),
        "no connector call".to_string(),
        "no index built".to_string(),
    ]
}

fn scientific_search_plan_step(
    kind: ScholarChatScientificSearchPlanStepKind,
    id: &str,
    label: &str,
    description: &str,
    planned_queries: Vec<String>,
    source_ids: Vec<String>,
    depends_on: Vec<String>,
    active: bool,
) -> ScholarChatScientificSearchPlanStep {
    ScholarChatScientificSearchPlanStep {
        kind,
        id: id.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        planned_queries,
        source_ids,
        depends_on,
        active,
        preview_only: true,
        boundary_notes: scientific_search_plan_step_notes(),
    }
}

fn scientific_search_plan_source_routing(
    preferred_source_ids: &[String],
    conditional_source_ids: &[String],
    excluded_source_ids: &[String],
) -> ScholarChatScientificSourceRoutingPlan {
    let mut active_routes = preferred_source_ids.to_vec();
    active_routes.sort();
    active_routes.dedup();

    let mut conditional_routes = conditional_source_ids.to_vec();
    conditional_routes.sort();
    conditional_routes.dedup();

    let mut excluded_routes = excluded_source_ids.to_vec();
    excluded_routes.sort();
    excluded_routes.dedup();

    let route_count = active_routes.len() + conditional_routes.len() + excluded_routes.len();
    ScholarChatScientificSourceRoutingPlan {
        route_count,
        active_routes,
        conditional_routes,
        excluded_routes,
        summary: if route_count == 0 {
            "No source routes are planned yet.".to_string()
        } else {
            "Source routing is planned only; no routing, connector calls, or registry writes were performed."
                .to_string()
        },
    }
}

fn scientific_search_plan_local_search(
    normalized_mode: &str,
    selected_local_source_ids: Vec<String>,
    planned_queries: Vec<String>,
) -> ScholarChatScientificLocalSearchPlan {
    let local_first = normalized_mode == "scholar_chat" || normalized_mode == "course";
    ScholarChatScientificLocalSearchPlan {
        local_source_count: selected_local_source_ids.len(),
        selected_local_source_ids,
        planned_queries,
        local_first,
        requires_local_evidence_before_answer: local_first,
        will_read_files: false,
        will_build_index: false,
        summary: if local_first {
            "Local search is planned only; no files are read and no indexes are built.".to_string()
        } else {
            "Local search is deferred; this preview does not read files or build indexes.".to_string()
        },
    }
}

fn scientific_search_plan_metadata(
    source_registry_status: &ScholarChatScientificSourceRegistryStatus,
    preferred_source_ids: Vec<String>,
    conditional_source_ids: Vec<String>,
    excluded_source_ids: Vec<String>,
    planned_queries: Vec<String>,
) -> ScholarChatScientificMetadataSearchPlan {
    let mut source_family_count = preferred_source_ids.len();
    source_family_count += conditional_source_ids.len();
    source_family_count += excluded_source_ids.len();
    ScholarChatScientificMetadataSearchPlan {
        source_family_count,
        preferred_source_ids,
        conditional_source_ids,
        excluded_source_ids,
        planned_queries,
        will_call_connectors: false,
        will_make_web_requests: false,
        summary: if matches!(source_registry_status, ScholarChatScientificSourceRegistryStatus::SourcePlanReady) {
            "Metadata search is planned only; no connectors or web requests were performed.".to_string()
        } else {
            "Metadata search is planned only; routing still depends on the discipline and source registry previews.".to_string()
        },
    }
}

fn scientific_search_plan_evidence_requirements(
    normalized_mode: &str,
    query_intent: &ScholarChatScientificQueryIntent,
    source_registry_status: &ScholarChatScientificSourceRegistryStatus,
) -> Vec<String> {
    let mut requirements = scientific_query_evidence_requirements(
        normalized_mode,
        query_intent,
        source_registry_status,
    );
    push_unique_text(&mut requirements, "local_search_plan_required");
    push_unique_text(&mut requirements, "metadata_search_plan_required");
    push_unique_text(&mut requirements, "source_routing_required");
    push_unique_text(&mut requirements, "ranking_required_before_answer");
    if normalized_mode == "scientific_paper"
        || matches!(query_intent, ScholarChatScientificQueryIntent::LiteratureSearch)
    {
        push_unique_text(
            &mut requirements,
            "deduplication_required_before_literature_review",
        );
    }
    if normalized_mode == "course" {
        push_unique_text(&mut requirements, "course_material_alignment_required");
    }
    if normalized_mode == "scholar_chat" || normalized_mode == "course" {
        push_unique_text(&mut requirements, "local_evidence_required_before_answer");
    }
    requirements
}

fn scientific_search_plan_combined_queries(
    planned_local_queries: &[String],
    planned_metadata_queries: &[String],
    planned_expanded_queries: &[String],
) -> Vec<String> {
    let mut queries = Vec::new();
    for query in planned_expanded_queries
        .iter()
        .chain(planned_local_queries.iter())
        .chain(planned_metadata_queries.iter())
    {
        push_unique_text(&mut queries, query);
    }
    queries
}

fn scientific_search_plan_steps(
    normalized_mode: &str,
    status: &ScholarChatScientificSearchPlanStatus,
    selected_local_source_ids: &[String],
    preferred_source_ids: &[String],
    conditional_source_ids: &[String],
    excluded_source_ids: &[String],
    planned_local_queries: &[String],
    planned_metadata_queries: &[String],
    planned_expanded_queries: &[String],
    evidence_requirements: &[String],
) -> Vec<ScholarChatScientificSearchPlanStep> {
    let active = !matches!(status, ScholarChatScientificSearchPlanStatus::Blocked);
    let local_step_kind = if normalized_mode == "course" {
        ScholarChatScientificSearchPlanStepKind::LocalCourseMaterialSearch
    } else {
        ScholarChatScientificSearchPlanStepKind::LocalSourceSearch
    };
    let local_step_id = if normalized_mode == "course" {
        "local_course_material_search"
    } else {
        "local_source_search"
    };
    let local_step_label = if normalized_mode == "course" {
        "Local course material search"
    } else {
        "Local source search"
    };
    let local_step_description = if normalized_mode == "course" {
        "Plan future local course-material search only; no files are read and no indexes are built."
    } else {
        "Plan future local source search only; no files are read and no indexes are built."
    };

    let routing_source_ids = preferred_source_ids
        .iter()
        .chain(conditional_source_ids.iter())
        .chain(excluded_source_ids.iter())
        .cloned()
        .collect::<Vec<_>>();

    let mut ranking_queries = scientific_search_plan_combined_queries(
        planned_local_queries,
        planned_metadata_queries,
        planned_expanded_queries,
    );
    if ranking_queries.is_empty() {
        ranking_queries = planned_metadata_queries.to_vec();
    }

    vec![
        scientific_search_plan_step(
            ScholarChatScientificSearchPlanStepKind::QueryExpansion,
            "query_expansion",
            "Query expansion",
            "Preview query expansion only; no retrieval or indexing is executed.",
            planned_expanded_queries.to_vec(),
            Vec::new(),
            Vec::new(),
            active,
        ),
        scientific_search_plan_step(
            local_step_kind,
            local_step_id,
            local_step_label,
            local_step_description,
            planned_local_queries.to_vec(),
            selected_local_source_ids.to_vec(),
            vec!["query_expansion".to_string()],
            active,
        ),
        scientific_search_plan_step(
            ScholarChatScientificSearchPlanStepKind::SourceFamilyRouting,
            "source_family_routing",
            "Source-family routing",
            "Preview source-family routing only; no registry writes or connector calls are performed.",
            planned_metadata_queries.to_vec(),
            routing_source_ids.clone(),
            vec![local_step_id.to_string()],
            active,
        ),
        scientific_search_plan_step(
            ScholarChatScientificSearchPlanStepKind::MetadataSourceSearch,
            "metadata_source_search",
            "Metadata source search",
            "Preview metadata search only; no web requests or connectors are executed.",
            planned_metadata_queries.to_vec(),
            preferred_source_ids
                .iter()
                .chain(conditional_source_ids.iter())
                .cloned()
                .collect::<Vec<_>>(),
            vec!["source_family_routing".to_string()],
            active,
        ),
        scientific_search_plan_step(
            ScholarChatScientificSearchPlanStepKind::RankingPlan,
            "ranking_plan",
            "Ranking plan",
            "Preview ranking only; no ranking is executed and no answers are generated.",
            ranking_queries.clone(),
            Vec::new(),
            vec!["metadata_source_search".to_string()],
            active,
        ),
        scientific_search_plan_step(
            ScholarChatScientificSearchPlanStepKind::DeduplicationPlan,
            "deduplication_plan",
            "Deduplication plan",
            "Preview deduplication only; no evidence or answer artifacts are created.",
            ranking_queries,
            Vec::new(),
            vec!["ranking_plan".to_string()],
            active,
        ),
        scientific_search_plan_step(
            ScholarChatScientificSearchPlanStepKind::EvidenceRequirementCheck,
            "evidence_requirement_check",
            "Evidence requirement check",
            "Preview evidence requirements only; no Evidence Packs or answers are created.",
            evidence_requirements.to_vec(),
            Vec::new(),
            vec!["deduplication_plan".to_string()],
            active,
        ),
    ]
}

fn normalize_scientific_expected_source_kinds(kinds: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = kinds
        .unwrap_or_default()
        .into_iter()
        .map(|value| normalize_scientific_tag_text(&value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn scientific_local_literature_index_status(
    search_plan_status: &ScholarChatScientificSearchPlanStatus,
    selected_local_source_ids: &[String],
) -> ScholarChatLocalLiteratureIndexStatus {
    match search_plan_status {
        ScholarChatScientificSearchPlanStatus::Blocked => ScholarChatLocalLiteratureIndexStatus::Blocked,
        ScholarChatScientificSearchPlanStatus::NeedsDisambiguation => {
            ScholarChatLocalLiteratureIndexStatus::NeedsDisambiguation
        }
        ScholarChatScientificSearchPlanStatus::UnknownConcept => {
            ScholarChatLocalLiteratureIndexStatus::UnknownConcept
        }
        ScholarChatScientificSearchPlanStatus::SearchPlanReady => {
            if selected_local_source_ids.is_empty() {
                ScholarChatLocalLiteratureIndexStatus::NeedsLocalSources
            } else {
                ScholarChatLocalLiteratureIndexStatus::IndexPlanReady
            }
        }
    }
}

fn scientific_local_literature_index_strategy(
    status: &ScholarChatLocalLiteratureIndexStatus,
    normalized_mode: &str,
) -> ScholarChatLocalLiteratureIndexStrategy {
    if matches!(status, ScholarChatLocalLiteratureIndexStatus::Blocked) {
        ScholarChatLocalLiteratureIndexStrategy::Blocked
    } else if normalized_mode == "course" {
        ScholarChatLocalLiteratureIndexStrategy::CourseMaterialLocalFirst
    } else if normalized_mode == "scientific_paper" {
        ScholarChatLocalLiteratureIndexStrategy::ScientificPaperCitationLocalFirst
    } else {
        ScholarChatLocalLiteratureIndexStrategy::ScholarChatLocalFirst
    }
}

fn scientific_local_literature_index_boundary_notes() -> Vec<String> {
    vec![
        "preview-only".to_string(),
        "no file read".to_string(),
        "no pdf extraction".to_string(),
        "no ocr".to_string(),
        "no chunking run".to_string(),
        "no embeddings generated".to_string(),
        "no index created".to_string(),
        "no artifact write".to_string(),
    ]
}

fn scientific_local_literature_index_step(
    kind: ScholarChatLocalLiteratureIndexStepKind,
    id: &str,
    label: &str,
    description: &str,
    planned_inputs: Vec<String>,
    planned_outputs: Vec<String>,
    active: bool,
) -> ScholarChatLocalLiteratureIndexStep {
    ScholarChatLocalLiteratureIndexStep {
        kind,
        id: id.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        planned_inputs,
        planned_outputs,
        active,
        preview_only: true,
        boundary_notes: scientific_local_literature_index_boundary_notes(),
    }
}

fn scientific_local_literature_index_planned_artifact_ids() -> Vec<String> {
    vec![
        "local_corpus_manifest_preview".to_string(),
        "local_literature_metadata_map_preview".to_string(),
        "lexical_index_plan_preview".to_string(),
        "vector_index_plan_preview".to_string(),
        "retrieval_readiness_plan_preview".to_string(),
    ]
}

fn scientific_local_literature_index_planned_artifact_descriptions() -> Vec<String> {
    vec![
        "Future local corpus manifest preview describing selected sources without reading files.".to_string(),
        "Future local literature metadata map preview describing source metadata only.".to_string(),
        "Future lexical index plan preview describing BM25-ready planning without index creation.".to_string(),
        "Future vector index plan preview describing embedding-ready planning without generation.".to_string(),
        "Future retrieval readiness plan preview describing later retrieval compatibility only.".to_string(),
    ]
}

fn scientific_local_literature_index_planned_index_fields() -> Vec<String> {
    vec![
        "source_id".to_string(),
        "source_kind".to_string(),
        "title".to_string(),
        "authors".to_string(),
        "year".to_string(),
        "doi".to_string(),
        "url_or_locator".to_string(),
        "course_context".to_string(),
        "discipline_concept".to_string(),
        "chunk_id_later".to_string(),
        "page_or_section_later".to_string(),
        "language_hint".to_string(),
        "local_query_terms".to_string(),
    ]
}

fn scientific_local_literature_index_chunking_policy() -> Vec<String> {
    vec![
        "chunking_not_run_in_preview".to_string(),
        "later_pdf_text_extraction_required_before_chunking".to_string(),
        "later_markdown_section_chunking_candidate".to_string(),
        "later_course_slide_section_chunking_candidate".to_string(),
        "preserve_page_or_section_locator_later".to_string(),
        "keep_citation_metadata_attached_to_chunks_later".to_string(),
    ]
}

fn scientific_local_literature_index_metadata_requirements(
    normalized_mode: &str,
) -> Vec<String> {
    let mut requirements = vec![
        "source_id_required".to_string(),
        "source_kind_required".to_string(),
        "title_required_when_available".to_string(),
        "authors_required_when_available".to_string(),
        "year_required_when_available".to_string(),
        "doi_or_stable_locator_preferred".to_string(),
        "local_file_locator_required_later_but_not_read_now".to_string(),
    ];
    if normalized_mode == "course" {
        push_unique_text(&mut requirements, "course_context_required_for_course_mode");
    }
    if normalized_mode == "scientific_paper" {
        push_unique_text(
            &mut requirements,
            "citation_metadata_required_for_scientific_paper",
        );
    }
    if normalized_mode == "scholar_chat" {
        push_unique_text(
            &mut requirements,
            "local_evidence_metadata_required_for_scholar_chat",
        );
    }
    requirements
}

fn scientific_local_literature_index_local_queries(
    planned_local_queries: &[String],
    normalized_query: &str,
) -> Vec<String> {
    if planned_local_queries.is_empty() && !normalized_query.is_empty() {
        vec![normalized_query.to_string()]
    } else {
        planned_local_queries.to_vec()
    }
}

fn normalize_course_optional_text(value: Option<String>) -> Option<String> {
    value.map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
        .and_then(|value| if value.is_empty() { None } else { Some(value) })
}

fn normalize_course_identity_segment(value: &str) -> String {
    normalize_scientific_tag_text(value)
}

fn normalize_course_material_kind_text(value: &str) -> String {
    normalize_scientific_tag_text(value)
}

fn normalize_course_material_kinds(kinds: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = kinds
        .unwrap_or_default()
        .into_iter()
        .map(|value| normalize_course_material_kind_text(&value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn course_literature_registry_boundary_notes() -> Vec<String> {
    vec![
        "preview-only".to_string(),
        "no file read".to_string(),
        "no pdf extraction".to_string(),
        "no ocr".to_string(),
        "no chunking run".to_string(),
        "no embeddings generated".to_string(),
        "no index created".to_string(),
        "no web request".to_string(),
        "no scraping".to_string(),
        "no connector call".to_string(),
        "no artifact write".to_string(),
        "no persistence".to_string(),
        "no source import".to_string(),
        "no model loading".to_string(),
        "no runtime inference".to_string(),
        "no llm call".to_string(),
        "no answer generation".to_string(),
        "no evidence pack creation".to_string(),
    ]
}

fn course_literature_registry_step(
    kind: ScholarChatCourseLiteratureRegistryStepKind,
    id: &str,
    label: &str,
    description: &str,
    planned_inputs: Vec<String>,
    planned_outputs: Vec<String>,
    active: bool,
) -> ScholarChatCourseLiteratureRegistryStep {
    ScholarChatCourseLiteratureRegistryStep {
        kind,
        id: id.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        planned_inputs,
        planned_outputs,
        active,
        preview_only: true,
        boundary_notes: course_literature_registry_boundary_notes(),
    }
}

fn course_literature_registry_status(
    local_status: &ScholarChatLocalLiteratureIndexStatus,
    normalized_course_context: &Option<String>,
    normalized_module_code: &Option<String>,
    normalized_course_title: &Option<String>,
    selected_local_source_ids: &[String],
) -> ScholarChatCourseLiteratureRegistryStatus {
    match local_status {
        ScholarChatLocalLiteratureIndexStatus::Blocked => ScholarChatCourseLiteratureRegistryStatus::Blocked,
        ScholarChatLocalLiteratureIndexStatus::NeedsDisambiguation => {
            ScholarChatCourseLiteratureRegistryStatus::NeedsDisambiguation
        }
        ScholarChatLocalLiteratureIndexStatus::UnknownConcept => {
            ScholarChatCourseLiteratureRegistryStatus::UnknownConcept
        }
        ScholarChatLocalLiteratureIndexStatus::NeedsLocalSources => {
            if normalized_course_context.is_none()
                && normalized_module_code.is_none()
                && normalized_course_title.is_none()
            {
                ScholarChatCourseLiteratureRegistryStatus::NeedsCourseContext
            } else {
                ScholarChatCourseLiteratureRegistryStatus::NeedsLocalSources
            }
        }
        ScholarChatLocalLiteratureIndexStatus::IndexPlanReady => {
            if normalized_course_context.is_none()
                && normalized_module_code.is_none()
                && normalized_course_title.is_none()
            {
                ScholarChatCourseLiteratureRegistryStatus::NeedsCourseContext
            } else if selected_local_source_ids.is_empty() {
                ScholarChatCourseLiteratureRegistryStatus::NeedsLocalSources
            } else {
                ScholarChatCourseLiteratureRegistryStatus::CourseRegistryPlanReady
            }
        }
    }
}

fn course_literature_registry_strategy(
    status: &ScholarChatCourseLiteratureRegistryStatus,
    normalized_module_code: &Option<String>,
    selected_local_source_ids: &[String],
) -> ScholarChatCourseLiteratureRegistryStrategy {
    if matches!(status, ScholarChatCourseLiteratureRegistryStatus::Blocked) {
        ScholarChatCourseLiteratureRegistryStrategy::Blocked
    } else if normalized_module_code.is_some() {
        ScholarChatCourseLiteratureRegistryStrategy::ModuleContextFirst
    } else if !selected_local_source_ids.is_empty() {
        ScholarChatCourseLiteratureRegistryStrategy::LocalSourceAlignmentFirst
    } else {
        ScholarChatCourseLiteratureRegistryStrategy::CourseMaterialAlignmentFirst
    }
}

fn course_literature_registry_course_identity(
    normalized_course_context: Option<String>,
    normalized_module_code: Option<String>,
    normalized_course_title: Option<String>,
    normalized_instructor: Option<String>,
    normalized_semester: Option<String>,
) -> ScholarChatCourseIdentityPreview {
    let identity_key_parts = [
        normalized_module_code
            .as_deref()
            .map(normalize_course_identity_segment),
        normalized_course_title
            .as_deref()
            .map(normalize_course_identity_segment),
        normalized_semester
            .as_deref()
            .map(normalize_course_identity_segment),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    let identity_key = if identity_key_parts.is_empty() {
        None
    } else {
        Some(identity_key_parts.join("::"))
    };
    let has_course_context = normalized_course_context.is_some();
    let has_module_code = normalized_module_code.is_some();
    let has_course_title = normalized_course_title.is_some();
    let summary = if identity_key.is_some() {
        "Course identity preview combines the module code, course title, and semester when available; course context and instructor remain descriptive hints only.".to_string()
    } else {
        "Course identity preview has no module code, course title, or semester yet; course context and instructor remain descriptive hints only.".to_string()
    };

    ScholarChatCourseIdentityPreview {
        course_context: normalized_course_context,
        module_code: normalized_module_code,
        course_title: normalized_course_title,
        instructor: normalized_instructor,
        semester: normalized_semester,
        identity_key,
        has_course_context,
        has_module_code,
        has_course_title,
        summary,
    }
}

fn course_literature_registry_known_material_kinds() -> BTreeSet<&'static str> {
    [
        "syllabus",
        "module_handbook",
        "lecture_slide",
        "seminar_reading",
        "exercise_sheet",
        "assignment",
        "textbook_chapter",
        "article",
        "notes",
        "exam_prep",
        "unknown",
    ]
    .into_iter()
    .collect()
}

fn course_literature_registry_course_material_plan(
    selected_local_source_ids: Vec<String>,
    expected_course_material_kinds: Vec<String>,
) -> ScholarChatCourseMaterialPlan {
    let known_material_kind_catalog = course_literature_registry_known_material_kinds();
    let known_material_kinds = expected_course_material_kinds
        .iter()
        .filter(|kind| known_material_kind_catalog.contains(kind.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let unknown_material_kinds = expected_course_material_kinds
        .iter()
        .filter(|kind| !known_material_kind_catalog.contains(kind.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    ScholarChatCourseMaterialPlan {
        selected_source_count: selected_local_source_ids.len(),
        selected_local_source_ids,
        expected_course_material_kinds,
        known_material_kinds,
        unknown_material_kinds,
        will_read_files: false,
        will_import_sources: false,
        will_create_registry: false,
        summary: "Course material planning is preview-only; no files are read, no sources are imported, and no registry is created.".to_string(),
    }
}

fn course_literature_registry_curriculum_alignment_plan(
    normalized_course_context: &Option<String>,
    normalized_module_code: &Option<String>,
) -> ScholarChatCurriculumAlignmentPlan {
    ScholarChatCurriculumAlignmentPlan {
        requires_course_context: normalized_course_context.is_none()
            && normalized_module_code.is_none(),
        requires_module_metadata: normalized_module_code.is_none(),
        requires_learning_objectives_later: true,
        requires_prerequisites_later: true,
        requires_session_or_week_mapping_later: true,
        will_scrape_curriculum_sources: false,
        will_call_connectors: false,
        summary: "Curriculum alignment planning is preview-only; no TU/ULB scraping or connector call occurs, and later phases would still need learning objectives, prerequisites, and session or week mapping.".to_string(),
    }
}

fn course_literature_registry_planned_course_metadata_requirements(
    course_identity: &ScholarChatCourseIdentityPreview,
    course_material_plan: &ScholarChatCourseMaterialPlan,
) -> Vec<String> {
    let mut requirements = vec![
        "course_context_or_module_identity_required".to_string(),
        "module_code_recommended".to_string(),
        "course_title_recommended".to_string(),
        "semester_recommended".to_string(),
        "instructor_optional".to_string(),
        "local_source_ids_required_for_course_material_alignment".to_string(),
        "material_kind_required_when_available".to_string(),
        "learning_objectives_required_later".to_string(),
        "prerequisites_required_later".to_string(),
        "session_or_week_mapping_required_later".to_string(),
        "citation_metadata_required_for_scientific_course_materials".to_string(),
        "no_curriculum_scraping_in_preview".to_string(),
    ];
    if !course_identity.has_course_context {
        requirements.push("course_context_missing".to_string());
    }
    if !course_identity.has_module_code {
        requirements.push("module_code_missing".to_string());
    }
    if !course_identity.has_course_title {
        requirements.push("course_title_missing".to_string());
    }
    if course_material_plan.selected_local_source_ids.is_empty() {
        requirements.push("local_sources_missing".to_string());
    }
    requirements
}

fn push_unique_string(items: &mut Vec<String>, value: String) {
    if !items.contains(&value) {
        items.push(value);
    }
}

fn course_literature_registry_planned_course_material_queries(
    normalized_query: &str,
    normalized_module_code: &Option<String>,
    normalized_course_title: &Option<String>,
    base_queries: &[String],
) -> Vec<String> {
    let mut planned_queries = base_queries.to_vec();
    if !normalized_query.is_empty() {
        push_unique_string(
            &mut planned_queries,
            format!("{normalized_query} course materials"),
        );
        push_unique_string(
            &mut planned_queries,
            format!("{normalized_query} lecture notes"),
        );
        push_unique_string(
            &mut planned_queries,
            format!("{normalized_query} module context"),
        );
        push_unique_string(
            &mut planned_queries,
            format!("{normalized_query} exam preparation"),
        );
        if let Some(module_code) = normalized_module_code {
            push_unique_string(
                &mut planned_queries,
                format!("{module_code} {normalized_query}"),
            );
        }
        if let Some(course_title) = normalized_course_title {
            push_unique_string(
                &mut planned_queries,
                format!("{course_title} {normalized_query}"),
            );
        }
    }
    planned_queries
}

#[derive(Clone)]
struct ScientificDisciplineRegistryEntry {
    recognized_concept: &'static str,
    label: &'static str,
    science_class: ScholarChatScientificDisciplineScienceClass,
    discipline_path: &'static [&'static str],
    parent_path: &'static [&'static str],
    related_methods: &'static [&'static str],
    appears_in: &'static [&'static str],
    preferred_sources: &'static [&'static str],
    curriculum_sources: &'static [&'static str],
    canonical_mappings: &'static [&'static str],
    planned_queries: &'static [&'static str],
}

fn normalize_scientific_topic_text(topic: &str) -> String {
    topic.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_scientific_mode(mode: Option<String>) -> String {
    let normalized_mode = mode
        .as_deref()
        .map(|value| {
            let mut result = String::new();
            let mut last_was_separator = false;
            for ch in value.trim().chars() {
                if ch.is_alphanumeric() {
                    for lower in ch.to_lowercase() {
                        result.push(lower);
                    }
                    last_was_separator = false;
                } else if !result.is_empty() && !last_was_separator {
                    result.push('_');
                    last_was_separator = true;
                }
            }
            result.trim_matches('_').to_string()
        })
        .unwrap_or_default();
    match normalized_mode.as_str() {
        "scientific_paper" | "course" | "scholar_chat" => normalized_mode,
        _ => "scholar_chat".to_string(),
    }
}

fn normalize_scientific_topic_key(topic: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;
    let normalized_topic = topic.trim().to_lowercase().replace("d'", "d prime");
    let mut chars = normalized_topic.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                normalized.push(lower);
            }
            last_was_separator = false;
        } else if !normalized.is_empty() && !last_was_separator {
            normalized.push(' ');
            last_was_separator = true;
        }
    }
    normalized.trim().to_string()
}

fn scientific_discipline_registry_entry(topic_key: &str) -> Option<ScientificDisciplineRegistryEntry> {
    match topic_key {
        "signalentdeckung"
        | "signalentdeckungstheorie"
        | "signal detection"
        | "signal detection theory"
        | "d prime" => Some(ScientificDisciplineRegistryEntry {
            recognized_concept: "signal_detection_theory",
            label: "Signalentdeckungstheorie",
            science_class: ScholarChatScientificDisciplineScienceClass::CoreScience,
            discipline_path: &[
                "psychology",
                "general_psychology",
                "perception",
                "psychophysics",
                "signal_detection_theory",
            ],
            parent_path: &["psychology", "general_psychology", "perception", "psychophysics"],
            related_methods: &["statistics", "probability_theory", "decision_theory", "roc_analysis"],
            appears_in: &[
                "perception_psychology",
                "psychophysics",
                "diagnostics",
                "neuroscience",
                "medical_testing",
                "machine_learning_evaluation",
            ],
            preferred_sources: &[
                "pubpsych",
                "psycharchives",
                "openalex",
                "crossref",
                "pubmed_if_biomedical_context",
            ],
            curriculum_sources: &["tu_darmstadt_module_handbook_candidate", "local_course_materials_later"],
            canonical_mappings: &[
                "psychology",
                "general_psychology",
                "psychophysics",
                "signal_detection_theory",
            ],
            planned_queries: &[
                "Signalentdeckungstheorie",
                "signal detection theory",
                "psychophysics signal detection",
                "d prime criterion ROC",
            ],
        }),
        "anova" | "varianzanalyse" | "analysis of variance" | "factorial anova" | "repeated measures anova" => Some(ScientificDisciplineRegistryEntry {
            recognized_concept: "analysis_of_variance",
            label: "ANOVA / Varianzanalyse",
            science_class: ScholarChatScientificDisciplineScienceClass::CoreScience,
            discipline_path: &["statistics", "inferential_statistics", "hypothesis_testing", "analysis_of_variance"],
            parent_path: &["statistics", "inferential_statistics", "hypothesis_testing"],
            related_methods: &["linear_models", "f_test", "effect_size", "post_hoc_tests", "repeated_measures"],
            appears_in: &[
                "psychology_methods",
                "experimental_design",
                "biomedical_statistics",
                "education_research",
            ],
            preferred_sources: &[
                "openalex",
                "crossref",
                "psycharchives_if_psychology_context",
                "pubpsych_if_psychology_context",
                "zbmath_if_theory_context",
                "arxiv_if_theory_context",
            ],
            curriculum_sources: &["tu_darmstadt_module_handbook_candidate", "local_course_materials_later"],
            canonical_mappings: &[
                "statistics",
                "inferential_statistics",
                "hypothesis_testing",
                "analysis_of_variance",
            ],
            planned_queries: &[
                "ANOVA",
                "Varianzanalyse",
                "analysis of variance",
                "factorial ANOVA",
                "repeated measures ANOVA",
            ],
        }),
        "hypothesentests"
        | "hypothesentest"
        | "hypothesis testing"
        | "null hypothesis"
        | "p value"
        | "null hypothesis p value"
        | "statistical power type i error type ii error"
        | "confidence intervals hypothesis testing" => Some(ScientificDisciplineRegistryEntry {
            recognized_concept: "hypothesis_testing",
            label: "Hypothesentests",
            science_class: ScholarChatScientificDisciplineScienceClass::CoreScience,
            discipline_path: &["statistics", "inferential_statistics", "hypothesis_testing"],
            parent_path: &["statistics", "inferential_statistics"],
            related_methods: &[
                "null_hypothesis",
                "alternative_hypothesis",
                "p_value",
                "type_i_error",
                "type_ii_error",
                "power",
                "confidence_intervals",
            ],
            appears_in: &["psychology_methods", "experimental_design", "biomedical_statistics", "data_analysis"],
            preferred_sources: &[
                "openalex",
                "crossref",
                "zbmath",
                "arxiv",
                "pubpsych_if_psychology_context",
                "psycharchives_if_psychology_context",
            ],
            curriculum_sources: &["tu_darmstadt_module_handbook_candidate", "local_course_materials_later"],
            canonical_mappings: &["statistics", "inferential_statistics", "hypothesis_testing"],
            planned_queries: &[
                "Hypothesentests",
                "hypothesis testing",
                "null hypothesis p value",
                "statistical power type I error type II error",
                "confidence intervals hypothesis testing",
            ],
        }),
        _ => None,
    }
}

fn preview_warnings(policy: &GroundingPolicy, selected_source_count: usize, kind: ScholarChatPreviewKind) -> Vec<String> {
    let mut warnings = Vec::new();
    if selected_source_count == 0 {
        match kind {
            ScholarChatPreviewKind::Request => {
                warnings.push("No selected sources; preview cannot plan source-scoped grounding yet.".to_string());
            }
            ScholarChatPreviewKind::Retrieval => {
                warnings.push("No selected sources; retrieval preview is unscoped.".to_string());
            }
        }
    }
    match policy {
        GroundingPolicy::LocalOnly => warnings.push("local_only requires local evidence before an answer can be presented as grounded.".to_string()),
        GroundingPolicy::ExternalAdaptersLater => warnings.push("External scholarly adapters are not implemented in this preview.".to_string()),
        GroundingPolicy::AllowMarkedModelKnowledge => warnings.push("Model knowledge is not used in this preview and would need to be clearly marked later.".to_string()),
        GroundingPolicy::LocalFirst => {}
    }
    match kind {
        ScholarChatPreviewKind::Request => warnings.push("This is a contract preview only; no retrieval, evidence-pack build, or model inference was run.".to_string()),
        ScholarChatPreviewKind::Retrieval => warnings.push("This is a retrieval preview only; no answer was generated.".to_string()),
    }
    warnings
}

fn grounding_plan(mode: &ScholarChatMode, policy: &GroundingPolicy, selected_source_count: usize) -> ScholarChatGroundingPlan {
    ScholarChatGroundingPlan {
        selected_source_count,
        local_corpus_required: matches!(policy, GroundingPolicy::LocalOnly | GroundingPolicy::LocalFirst),
        retrieval_would_run: selected_source_count > 0,
        evidence_pack_would_be_required: true,
        model_knowledge_allowed: matches!(policy, GroundingPolicy::AllowMarkedModelKnowledge),
        external_adapters_available: false,
        summary: grounding_summary(mode, policy, selected_source_count),
        steps: vec![
            "Normalize prompt and validate selected source IDs.".to_string(),
            "Resolve selected course or project context before retrieval.".to_string(),
            "Search registered local sources before any answer synthesis.".to_string(),
            "Assemble an Evidence Pack before grounded answer generation.".to_string(),
            "Return source/evidence status with any future answer.".to_string(),
        ],
    }
}

fn convert_retrieval_response(response: RetrievalResponse) -> Vec<ScholarChatRetrievalCandidate> {
    response
        .results
        .into_iter()
        .map(|result| ScholarChatRetrievalCandidate {
            source_id: result.source_id,
            version_id: result.version_id,
            chunk_id: result.chunk_id,
            score: result.score,
            matched_terms: result.matched_terms,
            preview: result.preview,
            locator: result.locator,
        })
        .collect()
}

fn convert_retrieval_candidates_from_preview(
    candidates: &[ScholarChatRetrievalCandidate],
) -> Vec<ScholarChatEvidenceCandidate> {
    candidates
        .iter()
        .map(|result| ScholarChatEvidenceCandidate {
            source_id: result.source_id.clone(),
            version_id: result.version_id.clone(),
            chunk_id: result.chunk_id.clone(),
            score: result.score,
            matched_terms: result.matched_terms.clone(),
            preview: result.preview.clone(),
            locator: result.locator.clone(),
        })
        .collect()
}

fn convert_evidence_candidates_to_prompt_context_items(
    candidates: &[ScholarChatEvidenceCandidate],
) -> Vec<ScholarChatPromptContextItem> {
    candidates
        .iter()
        .map(|result| ScholarChatPromptContextItem {
            source_id: result.source_id.clone(),
            version_id: result.version_id.clone(),
            chunk_id: result.chunk_id.clone(),
            score: result.score,
            matched_terms: result.matched_terms.clone(),
            preview: result.preview.clone(),
            locator: result.locator.clone(),
        })
        .collect()
}

fn build_prompt_pack(
    mode: &ScholarChatMode,
    policy: &GroundingPolicy,
    normalized_prompt: &str,
    selected_source_ids: &[String],
    context_items: &[ScholarChatPromptContextItem],
    evidence_candidate_count: usize,
) -> ScholarChatPromptPack {
    let sections = vec![
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::SystemOrPolicyInstructions,
            title: "System or policy instructions".to_string(),
            lines: system_or_policy_instructions(policy),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::ModeInstructions,
            title: "Mode instructions".to_string(),
            lines: mode_instructions(mode),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::GroundingInstructions,
            title: "Grounding instructions".to_string(),
            lines: grounding_pack_instructions(policy, selected_source_ids.len(), evidence_candidate_count),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::SourceContext,
            title: "Source context".to_string(),
            lines: source_context_lines(selected_source_ids, context_items),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::UserPrompt,
            title: "User prompt".to_string(),
            lines: vec![normalized_prompt.to_string()],
        },
    ];

    ScholarChatPromptPack {
        section_count: sections.len(),
        context_item_count: context_items.len(),
        estimated_input_char_count: estimate_prompt_pack_char_count(&sections, context_items),
        sections,
    }
}

fn system_or_policy_instructions(policy: &GroundingPolicy) -> Vec<String> {
    let mut lines = vec![
        "AEGIS Scholar local-first academic Scholar Chat workspace.".to_string(),
        "Preview only; no model inference or answer generation is run here.".to_string(),
    ];
    match policy {
        GroundingPolicy::LocalOnly => lines.push("Use only selected local evidence.".to_string()),
        GroundingPolicy::LocalFirst => lines.push("Prefer selected local evidence before any later fallback.".to_string()),
        GroundingPolicy::AllowMarkedModelKnowledge => lines.push("Model knowledge may be used later only if clearly marked.".to_string()),
        GroundingPolicy::ExternalAdaptersLater => lines.push("External scholarly adapters are not implemented yet.".to_string()),
    }
    lines
}

fn mode_instructions(mode: &ScholarChatMode) -> Vec<String> {
    match mode {
        ScholarChatMode::LectureLearning => vec![
            "Answer from course or lecture material first.".to_string(),
            "Prioritize what was taught and keep explanations grounded.".to_string(),
        ],
        ScholarChatMode::ThesisWriting => vec![
            "Support scientific writing, outlining, and literature synthesis.".to_string(),
            "Keep claims grounded and ready for citation.".to_string(),
        ],
        ScholarChatMode::LiteratureReview => vec![
            "Compare and synthesize papers with provenance.".to_string(),
            "Prefer source-linked evidence over general summaries.".to_string(),
        ],
        ScholarChatMode::Flashcards => vec![
            "Generate source-linked study-card candidates later.".to_string(),
            "Keep prompts compact and recall-oriented.".to_string(),
        ],
        ScholarChatMode::StatisticsMethods => vec![
            "Explain methods and support reproducible academic work.".to_string(),
            "Keep terminology precise and source-linked.".to_string(),
        ],
        ScholarChatMode::GeneralScholar => vec![
            "General academic assistant, local-first.".to_string(),
            "Use selected sources before any later fallback.".to_string(),
        ],
    }
}

fn grounding_pack_instructions(
    policy: &GroundingPolicy,
    selected_source_count: usize,
    evidence_candidate_count: usize,
) -> Vec<String> {
    let mut lines = vec![
        format!("Selected source count: {selected_source_count}."),
        format!("Evidence candidate count: {evidence_candidate_count}."),
    ];
    match policy {
        GroundingPolicy::LocalOnly => lines.push("local_only cannot answer without local evidence.".to_string()),
        GroundingPolicy::LocalFirst => lines.push("Prefer local evidence before any later fallback.".to_string()),
        GroundingPolicy::AllowMarkedModelKnowledge => lines.push("Model knowledge is only allowed when clearly marked later.".to_string()),
        GroundingPolicy::ExternalAdaptersLater => lines.push("External adapters are not implemented in this preview.".to_string()),
    }
    lines
}

fn source_context_lines(selected_source_ids: &[String], context_items: &[ScholarChatPromptContextItem]) -> Vec<String> {
    if selected_source_ids.is_empty() {
        return vec!["No selected sources; prompt pack preview is unscoped.".to_string()];
    }

    let mut counts_by_source = BTreeMap::new();
    for item in context_items {
        *counts_by_source.entry(item.source_id.clone()).or_insert(0usize) += 1;
    }

    let mut lines = vec![format!("Selected source IDs: {}.", selected_source_ids.join(", "))];
    for source_id in selected_source_ids {
        let count = counts_by_source.get(source_id).copied().unwrap_or(0);
        lines.push(format!("{source_id}: {count} evidence candidate(s)."));
    }
    lines
}

fn estimate_prompt_pack_char_count(
    sections: &[ScholarChatPromptPackSection],
    context_items: &[ScholarChatPromptContextItem],
) -> usize {
    let section_chars = sections.iter().fold(0usize, |acc, section| {
        let title_chars = section.title.chars().count();
        let line_chars = section.lines.iter().map(|line| line.chars().count()).sum::<usize>();
        let separator_chars = section.lines.len().saturating_sub(1);
        acc + title_chars + line_chars + separator_chars
    });
    let context_chars = context_items.iter().fold(0usize, |acc, item| {
        acc + item.source_id.chars().count()
            + item.version_id.chars().count()
            + item.chunk_id.chars().count()
            + item.preview.chars().count()
            + item.matched_terms.iter().map(|term| term.chars().count()).sum::<usize>()
            + locator_summary_chars(&item.locator)
    });
    section_chars + context_chars
}

fn locator_summary_chars(locator: &CitationLocator) -> usize {
    let section = locator
        .section_path
        .as_ref()
        .map(|value| value.iter().map(|part| part.chars().count()).sum::<usize>())
        .unwrap_or(0);
    let start = locator.character_start.map(|value| value.to_string().chars().count()).unwrap_or(0);
    let end = locator.character_end.map(|value| value.to_string().chars().count()).unwrap_or(0);
    locator.label.chars().count() + section + start + end
}

fn push_warning(warnings: &mut Vec<String>, message: &str) {
    if !warnings.iter().any(|warning| warning == message) {
        warnings.push(message.to_string());
    }
}

fn push_readiness_warning(
    warnings: &mut Vec<ScholarChatAnswerReadinessWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(ScholarChatAnswerReadinessWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_readiness_blocker(
    blockers: &mut Vec<ScholarChatAnswerReadinessBlocker>,
    kind: &str,
    message: &str,
) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(ScholarChatAnswerReadinessBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_readiness_action(actions: &mut Vec<String>, action: &str) {
    if !actions.iter().any(|existing| existing == action) {
        actions.push(action.to_string());
    }
}

fn readiness_status(
    policy: GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
    runtime_ready: bool,
    invocation_ready: bool,
    allow_model_execution: bool,
) -> ScholarChatAnswerReadinessStatus {
    if selected_source_count == 0 {
        if matches!(policy, GroundingPolicy::AllowMarkedModelKnowledge) && runtime_ready && invocation_ready && allow_model_execution {
            return ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater;
        }
        return ScholarChatAnswerReadinessStatus::NeedsSources;
    }
    if retrieval_candidate_count == 0 {
        return if matches!(policy, GroundingPolicy::LocalOnly) {
            ScholarChatAnswerReadinessStatus::Blocked
        } else {
            ScholarChatAnswerReadinessStatus::NeedsRetrievalIndex
        };
    }
    if evidence_candidate_count == 0 {
        return if matches!(policy, GroundingPolicy::LocalOnly) {
            ScholarChatAnswerReadinessStatus::Blocked
        } else {
            ScholarChatAnswerReadinessStatus::NeedsEvidenceCandidates
        };
    }
    if !runtime_ready {
        return ScholarChatAnswerReadinessStatus::NeedsRuntimeConfig;
    }
    if !allow_model_execution {
        return ScholarChatAnswerReadinessStatus::NeedsExecutionConsent;
    }
    if matches!(policy, GroundingPolicy::AllowMarkedModelKnowledge) {
        ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater
    } else {
        ScholarChatAnswerReadinessStatus::ReadyForGroundedDraftLater
    }
}

fn readiness_output_classification(
    status: ScholarChatAnswerReadinessStatus,
) -> ScholarChatAnswerReadinessOutputClassification {
    match status {
        ScholarChatAnswerReadinessStatus::Blocked
        | ScholarChatAnswerReadinessStatus::NeedsSources
        | ScholarChatAnswerReadinessStatus::NeedsRuntimeConfig
        | ScholarChatAnswerReadinessStatus::NeedsExecutionConsent => {
            ScholarChatAnswerReadinessOutputClassification::Blocked
        }
        ScholarChatAnswerReadinessStatus::NeedsRetrievalIndex
        | ScholarChatAnswerReadinessStatus::NeedsEvidenceCandidates => {
            ScholarChatAnswerReadinessOutputClassification::SourceContextDraft
        }
        ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater => {
            ScholarChatAnswerReadinessOutputClassification::UngroundedDraft
        }
        ScholarChatAnswerReadinessStatus::ReadyForGroundedDraftLater => {
            ScholarChatAnswerReadinessOutputClassification::GroundedDraftCandidate
        }
    }
}

fn evidence_plan(
    mode: &ScholarChatMode,
    policy: &GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
) -> ScholarChatEvidencePlan {
    ScholarChatEvidencePlan {
        retrieval_candidate_count,
        evidence_candidate_count,
        evidence_required: true,
        evidence_pack_would_be_built_later: true,
        summary: evidence_plan_summary(mode, policy, selected_source_count, retrieval_candidate_count, evidence_candidate_count),
        steps: vec![
            "Normalize prompt and validate selected source IDs.".to_string(),
            "Reuse retrieval-preview candidates over the selected local sources.".to_string(),
            "Mark retrieval candidates that would be eligible for Evidence Pack assembly later.".to_string(),
            "Return preview-only evidence readiness warnings without building an Evidence Pack.".to_string(),
        ],
    }
}

fn evidence_plan_summary(
    mode: &ScholarChatMode,
    policy: &GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
) -> String {
    format!(
        "Preview plans a {:?} request with {:?} grounding over {} selected source(s), yielding {} retrieval candidate(s) and {} evidence candidate(s); no Evidence Pack is built yet.",
        mode,
        policy,
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
    )
}

fn grounding_summary(mode: &ScholarChatMode, policy: &GroundingPolicy, selected_source_count: usize) -> String {
    format!(
        "Preview plans a {:?} request with {:?} grounding over {} selected source(s); no answer is generated.",
        mode, policy, selected_source_count
    )
}

fn validate_source_id(source_id: &str) -> AegisResult<()> {
    if source_id.trim().is_empty() {
        return Err(AegisError::ScholarChatInvalidSourceId);
    }
    if source_id.contains('/') || source_id.contains('\\') || source_id.contains("..") {
        return Err(AegisError::ScholarChatInvalidSourceId);
    }
    Ok(())
}

fn validate_answer_draft_id(answer_draft_id: &str) -> AegisResult<()> {
    if answer_draft_id.trim().is_empty() {
        return Err(AegisError::AnswerDraftInvalidId);
    }
    if answer_draft_id.contains('/') || answer_draft_id.contains('\\') || answer_draft_id.contains("..") {
        return Err(AegisError::AnswerDraftInvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_runtime::{
        LocalModelRuntimeKind,
        LocalRuntimeAdapterContractPreviewRequest,
        LocalRuntimeAdapterKind,
        LocalRuntimeCapabilityPreviewRequest,
        LocalRuntimeCapabilityStatus,
        LocalRuntimeProbeReadinessPreviewRequest,
        LocalRuntimeProbeReadinessStatus,
        LocalRuntimeSmokeExecutionPlanPreviewRequest,
        LocalRuntimeSmokeExecutionPlanStatus,
        LocalRuntimeSmokeReadinessPreviewRequest,
        LocalRuntimeSmokeReadinessStatus,
        LocalRuntimeValidationPreviewRequest,
        LocalRuntimeValidationStatus,
        LocalRuntimeVersionProbePreviewRequest,
        LocalRuntimeVersionProbeStatus,
    };
    use std::{env, fs, path::PathBuf, process::Command};

    fn request(prompt: &str) -> ScholarChatRequest {
        ScholarChatRequest {
            prompt: prompt.to_string(),
            mode: ScholarChatMode::LectureLearning,
            grounding_policy: GroundingPolicy::LocalFirst,
            selected_source_ids: vec![" src_demo ".to_string()],
        }
    }

    fn grounding_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
    ) -> ScholarChatDraftGroundingInspectionRequest {
        ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: prompt.to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids,
            },
            draft_text: draft_text.map(|value| value.to_string()),
            max_items: Some(4),
        }
    }

    fn build_intent_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
    ) -> ScholarChatGroundedAnswerBuildIntentRequest {
        ScholarChatGroundedAnswerBuildIntentRequest {
            grounding_request: grounding_request(prompt, draft_text, selected_source_ids),
            answer_draft_id: answer_draft_id.map(|value| value.to_string()),
            explicit_user_intent,
        }
    }

    fn build_request_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
    ) -> ScholarChatGroundedAnswerBuildRequestPreviewRequest {
        ScholarChatGroundedAnswerBuildRequestPreviewRequest {
            build_intent_request: build_intent_request(
                prompt,
                draft_text,
                selected_source_ids,
                answer_draft_id,
                explicit_user_intent,
            ),
        }
    }

    fn build_preflight_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
    ) -> ScholarChatGroundedAnswerBuildPreflightPreviewRequest {
        ScholarChatGroundedAnswerBuildPreflightPreviewRequest {
            build_request_preview_request: build_request_request(
                prompt,
                draft_text,
                selected_source_ids,
                answer_draft_id,
                explicit_user_intent,
            ),
        }
    }

    fn execution_readiness_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
        execution_consent: bool,
    ) -> ScholarChatGroundedAnswerExecutionReadinessPreviewRequest {
        ScholarChatGroundedAnswerExecutionReadinessPreviewRequest {
            build_preflight_preview_request: build_preflight_request(
                prompt,
                draft_text,
                selected_source_ids,
                answer_draft_id,
                explicit_user_intent,
            ),
            execution_consent,
        }
    }

    fn execution_plan_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
        execution_consent: bool,
    ) -> ScholarChatGroundedAnswerExecutionPlanPreviewRequest {
        ScholarChatGroundedAnswerExecutionPlanPreviewRequest {
            execution_readiness_preview_request: execution_readiness_request(
                prompt,
                draft_text,
                selected_source_ids,
                answer_draft_id,
                explicit_user_intent,
                execution_consent,
            ),
        }
    }

    #[test]
    fn scholar_chat_preview_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_request(temp.path(), request("   "));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_preview_trims_prompt_and_sources_deterministically() {
        let temp = tempfile::tempdir().unwrap();
        let first = preview_scholar_chat_request(temp.path(), request("  Explain retrieval  ")).unwrap();
        let second = preview_scholar_chat_request(temp.path(), request("  Explain retrieval  ")).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "Explain retrieval");
        assert_eq!(first.selected_source_ids, vec!["src_demo"]);
        assert_eq!(first.status, ScholarChatStatus::PreviewOnly);
    }

    #[test]
    fn scholar_chat_preview_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let mut request = request("Explain alpha");
            request.selected_source_ids = vec![invalid.to_string()];
            let result = preview_scholar_chat_request(temp.path(), request);
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_preview_allows_no_selected_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let mut request = request("Explain alpha");
        request.selected_source_ids = Vec::new();
        let response = preview_scholar_chat_request(temp.path(), request).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("No selected sources")));
        assert!(!response.grounding_plan.retrieval_would_run);
    }

    #[test]
    fn scholar_chat_preview_local_only_policy_is_visible_in_warnings_and_plan() {
        let temp = tempfile::tempdir().unwrap();
        let mut request = request("Explain alpha");
        request.grounding_policy = GroundingPolicy::LocalOnly;
        let response = preview_scholar_chat_request(temp.path(), request).unwrap();
        assert!(response.grounding_plan.local_corpus_required);
        assert!(response.warnings.iter().any(|warning| warning.contains("local_only")));
    }

    #[test]
    fn scholar_chat_preview_output_is_path_free_and_non_mutating() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_request(temp.path(), request("Explain alpha")).unwrap();
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    fn retrieval_request(prompt: &str, selected_source_ids: Vec<String>) -> ScholarChatRequest {
        ScholarChatRequest {
            prompt: prompt.to_string(),
            mode: ScholarChatMode::LectureLearning,
            grounding_policy: GroundingPolicy::LocalFirst,
            selected_source_ids,
        }
    }

    fn evidence_plan_request(prompt: &str, selected_source_ids: Vec<String>) -> ScholarChatRequest {
        retrieval_request(prompt, selected_source_ids)
    }

    fn prompt_pack_request(prompt: &str, selected_source_ids: Vec<String>) -> ScholarChatRequest {
        ScholarChatRequest {
            prompt: prompt.to_string(),
            mode: ScholarChatMode::ThesisWriting,
            grounding_policy: GroundingPolicy::LocalOnly,
            selected_source_ids,
        }
    }

    fn runtime_config(model_path: Option<&str>, executable_path: Option<&str>) -> LocalModelRuntimeConfig {
        LocalModelRuntimeConfig {
            runtime_kind: LocalModelRuntimeKind::LlamaCpp,
            model_path: model_path.map(|value| value.to_string()),
            executable_path: executable_path.map(|value| value.to_string()),
            context_window: Some(512),
            gpu_layers: Some(0),
            temperature: Some(0.0),
        }
    }

    fn answer_readiness_request(
        prompt: &str,
        grounding_policy: GroundingPolicy,
        selected_source_ids: Vec<String>,
        runtime_config: LocalModelRuntimeConfig,
        allow_model_execution: bool,
    ) -> ScholarChatAnswerReadinessRequest {
        ScholarChatAnswerReadinessRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: prompt.to_string(),
                mode: ScholarChatMode::ThesisWriting,
                grounding_policy,
                selected_source_ids,
            },
            runtime_config,
            allow_model_execution,
        }
    }

    fn build_source_with_index(temp: &tempfile::TempDir, text: &str) -> String {
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, text).unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();
        RetrievalService::new(temp.path())
            .build_index(&source.source_id)
            .unwrap();
        source.source_id
    }

    fn build_readable_answer_draft_fixture(temp: &tempfile::TempDir) -> (String, String, String, usize) {
        let source_id = build_source_with_index(temp, "alpha beta gamma\nalpha beta delta\n");
        let evidence = crate::evidence::EvidenceService::new(temp.path())
            .build_evidence_pack(&source_id, "alpha grounded evidence", 4)
            .unwrap();
        let draft = crate::answer_draft::AnswerDraftService::new(temp.path())
            .build_answer_draft(&source_id, &evidence.evidence_pack_id)
            .unwrap();
        (source_id, draft.answer_draft_id, draft.version_id, draft.claim_count)
    }

    fn build_runtime_fixture(temp: &tempfile::TempDir) -> LocalModelRuntimeConfig {
        let model_path = temp.path().join("ready-model.gguf");
        let executable_path = temp.path().join("ready-smoke-helper.exe");
        fs::write(&model_path, "gguf placeholder").unwrap();
        fs::write(&executable_path, "runtime placeholder").unwrap();
        runtime_config(
            Some(model_path.to_string_lossy().as_ref()),
            Some(executable_path.to_string_lossy().as_ref()),
        )
    }

    fn smoke_helper_executable(temp: &tempfile::TempDir) -> PathBuf {
        let source_path = temp.path().join("smoke_helper.rs");
        let executable_path = temp.path().join(if cfg!(windows) { "smoke_helper.exe" } else { "smoke_helper" });
        let source = r#"
use std::{env, thread, time::Duration};

fn prompt_argument(args: &[String]) -> String {
    args.windows(2)
        .find(|pair| pair[0] == "-p" || pair[0] == "--prompt")
        .map(|pair| pair[1].clone())
        .unwrap_or_default()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prompt = prompt_argument(&args);
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("{}", "E".repeat(5000));
    if prompt.contains("SLEEP") {
        thread::sleep(Duration::from_millis(700));
    }
    if prompt.contains("FAIL") {
        std::process::exit(7);
    }
}
"#;
        fs::write(&source_path, source).unwrap();
        let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        let status = Command::new(rustc)
            .arg("--crate-type")
            .arg("bin")
            .arg("--edition")
            .arg("2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .unwrap();
        assert!(status.success());
        executable_path
    }

    fn build_draft_runtime_fixture(temp: &tempfile::TempDir) -> LocalModelRuntimeConfig {
        let model_path = temp.path().join("draft-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let executable_path = smoke_helper_executable(temp);
        runtime_config(
            Some(model_path.to_string_lossy().as_ref()),
            Some(executable_path.to_string_lossy().as_ref()),
        )
    }

    fn version_probe_helper_executable_with_source(
        temp: &tempfile::TempDir,
        executable_name: &str,
        source: &str,
    ) -> PathBuf {
        let source_path = temp.path().join(format!("{executable_name}.rs"));
        let executable_path = temp.path().join(executable_name);
        let crate_name = source_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("runtime_diagnostic_bridge_helper")
            .chars()
            .map(|value| if value.is_ascii_alphanumeric() || value == '_' { value } else { '_' })
            .collect::<String>();
        fs::write(&source_path, source).unwrap();
        let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        let status = Command::new(rustc)
            .arg("--crate-name")
            .arg(&crate_name)
            .arg("--edition=2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .unwrap();
        assert!(status.success());
        executable_path
    }

    fn runtime_diagnostic_bridge_helper_executable(temp: &tempfile::TempDir, executable_name: &str) -> PathBuf {
        let counter_path = temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt");
        let marker_path = temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt");
        let counter_literal = format!("{:?}", counter_path.to_string_lossy().to_string());
        let marker_literal = format!("{:?}", marker_path.to_string_lossy().to_string());
        let source = r#"
use std::{env, fs, path::PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    if args.iter().any(|arg| matches!(arg.as_str(), "-p" | "--prompt" | "-n" | "--max_output_tokens" | "-m" | "--ctx-size" | "-ngl")) {
        let marker = PathBuf::from(MARKER_PATH);
        let _ = fs::write(marker, args.join(" | "));
        std::process::exit(91);
    }
    if args.iter().any(|arg| arg == "--version") {
        let counter = PathBuf::from(COUNTER_PATH);
        let current = fs::read_to_string(&counter)
            .ok()
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        let _ = fs::write(counter, (current + 1).to_string());
        if exe_name.contains("fail") {
            std::process::exit(7);
        }
        println!("llama.cpp version 1.2.3");
        return;
    }
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
}
"#
        .replace("COUNTER_PATH", &counter_literal)
        .replace("MARKER_PATH", &marker_literal);
        version_probe_helper_executable_with_source(temp, executable_name, &source)
    }

    fn prepare_runtime_diagnostic_bridge_spies(temp: &tempfile::TempDir) {
        fs::write(temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt"), "0").unwrap();
        fs::write(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt"), "").unwrap();
    }

    fn count_entries_recursively(path: &std::path::Path) -> usize {
        fn inner(path: &std::path::Path, count: &mut usize) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    *count += 1;
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        inner(&entry_path, count);
                    }
                }
            }
        }

        let mut count = 0;
        inner(path, &mut count);
        count
    }

    fn runtime_diagnostic_bridge_request(
        prompt: &str,
        selected_source_ids: Vec<String>,
        executable_path: Option<&str>,
        model_path: Option<&str>,
        probe_consent: bool,
        allow_probe_execution: bool,
        smoke_consent: bool,
        diagnostic_prompt: Option<&str>,
        max_output_tokens: Option<u32>,
        timeout_ms: Option<u64>,
    ) -> ScholarChatRuntimeDiagnosticBridgePreviewRequest {
        ScholarChatRuntimeDiagnosticBridgePreviewRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: prompt.to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids,
            },
            smoke_execution_plan_preview_request: LocalRuntimeSmokeExecutionPlanPreviewRequest {
                smoke_readiness_preview_request: LocalRuntimeSmokeReadinessPreviewRequest {
                    capability_preview_request: LocalRuntimeCapabilityPreviewRequest {
                        version_probe_preview_request: LocalRuntimeVersionProbePreviewRequest {
                            probe_readiness_preview_request: LocalRuntimeProbeReadinessPreviewRequest {
                                validation_preview_request: LocalRuntimeValidationPreviewRequest {
                                    adapter_contract_request: LocalRuntimeAdapterContractPreviewRequest {
                                        adapter_kind: LocalRuntimeAdapterKind::LlamaCpp,
                                        executable_path: executable_path.map(|value| value.to_string()),
                                        model_path: model_path.map(|value| value.to_string()),
                                        model_family: Some("llama".to_string()),
                                        model_format: Some("gguf".to_string()),
                                        context_window_tokens: Some(8192),
                                        gpu_layers: Some(0),
                                        threads: Some(8),
                                        batch_size: Some(256),
                                        chat_template: Some("template".to_string()),
                                    },
                                },
                                probe_consent,
                            },
                            allow_probe_execution,
                            timeout_ms,
                        },
                    },
                    smoke_consent,
                    diagnostic_prompt: diagnostic_prompt.map(|value| value.to_string()),
                    max_output_tokens,
                    timeout_ms,
                },
            },
        }
    }

    fn scientific_discipline_registry_request(
        topic: &str,
        mode: Option<&str>,
        course_context: Option<&str>,
    ) -> ScholarChatScientificDisciplineRegistryPreviewRequest {
        ScholarChatScientificDisciplineRegistryPreviewRequest {
            topic: topic.to_string(),
            mode: mode.map(|value| value.to_string()),
            course_context: course_context.map(|value| value.to_string()),
        }
    }

    fn scientific_source_registry_request(
        topic: &str,
        mode: Option<&str>,
        course_context: Option<&str>,
        context_tags: Option<Vec<&str>>,
    ) -> ScholarChatScientificSourceRegistryPreviewRequest {
        ScholarChatScientificSourceRegistryPreviewRequest {
            topic: topic.to_string(),
            mode: mode.map(|value| value.to_string()),
            course_context: course_context.map(|value| value.to_string()),
            context_tags: context_tags.map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
        }
    }

    fn scientific_query_understanding_request(
        query: &str,
        mode: Option<&str>,
        course_context: Option<&str>,
        context_tags: Option<Vec<&str>>,
    ) -> ScholarChatScientificQueryUnderstandingPreviewRequest {
        ScholarChatScientificQueryUnderstandingPreviewRequest {
            query: query.to_string(),
            mode: mode.map(|value| value.to_string()),
            course_context: course_context.map(|value| value.to_string()),
            context_tags: context_tags.map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
        }
    }

    fn scientific_search_plan_request(
        query: &str,
        mode: Option<&str>,
        course_context: Option<&str>,
        context_tags: Option<Vec<&str>>,
        selected_local_source_ids: Option<Vec<&str>>,
    ) -> ScholarChatScientificSearchPlanRequest {
        ScholarChatScientificSearchPlanRequest {
            query: query.to_string(),
            mode: mode.map(|value| value.to_string()),
            course_context: course_context.map(|value| value.to_string()),
            context_tags: context_tags
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
            selected_local_source_ids: selected_local_source_ids
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
        }
    }

    fn scientific_local_literature_index_request(
        query: &str,
        mode: Option<&str>,
        course_context: Option<&str>,
        context_tags: Option<Vec<&str>>,
        selected_local_source_ids: Option<Vec<&str>>,
        expected_source_kinds: Option<Vec<&str>>,
    ) -> ScholarChatLocalLiteratureIndexRequest {
        ScholarChatLocalLiteratureIndexRequest {
            query: query.to_string(),
            mode: mode.map(|value| value.to_string()),
            course_context: course_context.map(|value| value.to_string()),
            context_tags: context_tags
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
            selected_local_source_ids: selected_local_source_ids
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
            expected_source_kinds: expected_source_kinds
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
        }
    }

    fn course_literature_registry_request(
        query: &str,
        course_context: Option<&str>,
        module_code: Option<&str>,
        course_title: Option<&str>,
        instructor: Option<&str>,
        semester: Option<&str>,
        context_tags: Option<Vec<&str>>,
        selected_local_source_ids: Option<Vec<&str>>,
        expected_course_material_kinds: Option<Vec<&str>>,
    ) -> ScholarChatCourseLiteratureRegistryPreviewRequest {
        ScholarChatCourseLiteratureRegistryPreviewRequest {
            query: query.to_string(),
            course_context: course_context.map(|value| value.to_string()),
            module_code: module_code.map(|value| value.to_string()),
            course_title: course_title.map(|value| value.to_string()),
            instructor: instructor.map(|value| value.to_string()),
            semester: semester.map(|value| value.to_string()),
            context_tags: context_tags
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
            selected_local_source_ids: selected_local_source_ids
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
            expected_course_material_kinds: expected_course_material_kinds
                .map(|tags| tags.into_iter().map(|value| value.to_string()).collect()),
        }
    }

    fn runtime_diagnostic_result_request(
        bridge_preview_request: ScholarChatRuntimeDiagnosticBridgePreviewRequest,
        diagnostic_preview: LocalRuntimeSmokeDiagnosticPreview,
    ) -> ScholarChatRuntimeDiagnosticResultPreviewRequest {
        ScholarChatRuntimeDiagnosticResultPreviewRequest {
            bridge_preview_request,
            diagnostic_preview,
        }
    }

    fn runtime_answer_pipeline_gate_request(
        grounded_answer_execution_plan_preview_request: ScholarChatGroundedAnswerExecutionPlanPreviewRequest,
        runtime_diagnostic_result_preview_request: ScholarChatRuntimeDiagnosticResultPreviewRequest,
    ) -> ScholarChatRuntimeAnswerPipelineGatePreviewRequest {
        ScholarChatRuntimeAnswerPipelineGatePreviewRequest {
            grounded_answer_execution_plan_preview_request,
            runtime_diagnostic_result_preview_request,
        }
    }

    fn runtime_diagnostic_preview_like_bridge(
        status: LocalRuntimeSmokeDiagnosticStatus,
        stdout_preview: &str,
        stderr_preview: &str,
    ) -> LocalRuntimeSmokeDiagnosticPreview {
        let _execution_attempted = matches!(
            status,
            LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded
                | LocalRuntimeSmokeDiagnosticStatus::SmokeFailed
                | LocalRuntimeSmokeDiagnosticStatus::TimedOut
        );
        let exit_code = match status {
            LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded => Some(0),
            LocalRuntimeSmokeDiagnosticStatus::SmokeFailed => Some(1),
            LocalRuntimeSmokeDiagnosticStatus::TimedOut | LocalRuntimeSmokeDiagnosticStatus::Blocked => None,
        };
        LocalRuntimeSmokeDiagnosticPreview {
            status: status.clone(),
            smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater,
            smoke_readiness_status: LocalRuntimeSmokeReadinessStatus::SmokeReadyLater,
            capability_status: LocalRuntimeCapabilityStatus::CapabilityReadyLater,
            version_probe_status: LocalRuntimeVersionProbeStatus::ProbeSucceeded,
            probe_readiness_status: LocalRuntimeProbeReadinessStatus::ProbeReadyLater,
            validation_status: LocalRuntimeValidationStatus::ValidationReadyLater,
            adapter_contract_status: LocalRuntimeAdapterContractStatus::ContractReadyLater,
            adapter_kind: LocalRuntimeAdapterKind::LlamaCpp,
            normalized_model_family: Some("llama".to_string()),
            normalized_model_format: "gguf".to_string(),
            safe_executable_file_name: Some("runtime_diagnostic_bridge_ready.exe".to_string()),
            safe_model_file_name: Some("runtime-diagnostic-bridge-model.gguf".to_string()),
            probe_consent: true,
            allow_probe_execution: true,
            smoke_consent: true,
            allow_smoke_execution: true,
            execution_attempted: matches!(
                status,
                LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded
                    | LocalRuntimeSmokeDiagnosticStatus::SmokeFailed
                    | LocalRuntimeSmokeDiagnosticStatus::TimedOut
            ),
            normalized_diagnostic_prompt: "Diagnostic smoke prompt.".to_string(),
            diagnostic_prompt_char_count: "Diagnostic smoke prompt.".chars().count(),
            max_output_tokens: 32,
            timeout_ms: 1_500,
            duration_ms: 12,
            exit_code,
            stdout_preview: stdout_preview.to_string(),
            stderr_preview: stderr_preview.to_string(),
            stdout_truncated: false,
            stderr_truncated: false,
            blockers: vec![],
            warnings: vec![],
            next_required_actions: vec![],
            summary: "Diagnostic preview only.".to_string(),
            diagnostic_only: true,
            not_scholar_chat_answer: true,
            no_answer_generated: true,
            no_grounding_applied: true,
            no_evidence_pack_used: true,
            no_persistence: true,
            no_artifact_write: true,
            no_registry_status_change: true,
            no_audit_write: true,
        }
    }

    fn draft_inference_request(
        prompt: &str,
        grounding_policy: GroundingPolicy,
        selected_source_ids: Vec<String>,
        runtime_config: LocalModelRuntimeConfig,
        allow_model_execution: bool,
        timeout_ms: Option<u64>,
        max_output_tokens: Option<u32>,
    ) -> ScholarChatDraftInferenceRequest {
        ScholarChatDraftInferenceRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: prompt.to_string(),
                mode: ScholarChatMode::ThesisWriting,
                grounding_policy,
                selected_source_ids,
            },
            runtime_config,
            allow_model_execution,
            timeout_ms,
            max_output_tokens,
        }
    }

    fn assert_readiness_boundary_fields(preview: &ScholarChatAnswerReadinessPreview) {
        assert!(!preview.would_generate_answer_now);
        assert!(!preview.would_build_evidence_pack_now);
        assert!(!preview.would_create_final_answer_now);
        assert!(preview.prompt_pack_ready);
    }

    fn assert_draft_boundary_fields(preview: &ScholarChatDraftInferencePreview) {
        assert!(preview.draft_only);
        assert!(preview.preview_only);
        assert!(preview.not_final_answer);
        assert!(preview.not_grounded_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
    }

    fn assert_draft_grounding_inspection_boundary_fields(
        preview: &ScholarChatDraftGroundingInspectionPreview,
    ) {
        assert!(preview.inspection_only);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_draft_readiness_boundary_fields(
        preview: &ScholarChatGroundedDraftReadinessPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_draft_readiness_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedDraftReadinessPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_draft_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_draft_readiness(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_draft_readiness_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_plan_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildPlanPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_answer_candidate_boundary_fields(
        preview: &ScholarChatGroundedAnswerCandidatePreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_answer_write_eligibility_boundary_fields(
        preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_grounded_answer_build_intent_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
    }

    fn assert_grounded_answer_build_request_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
    }

    fn assert_grounded_answer_build_preflight_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
        assert!(preview.no_grounded_answer_write);
    }

    fn assert_grounded_answer_execution_readiness_boundary_fields(
        preview: &ScholarChatGroundedAnswerExecutionReadinessPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
        assert!(preview.no_grounded_answer_write);
    }

    fn assert_grounded_answer_build_plan_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedAnswerBuildPlanPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_plan(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_plan(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_plan_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_candidate_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedAnswerCandidatePreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_candidate(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_candidate(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_candidate_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_write_eligibility_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedAnswerWriteEligibilityPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_write_eligibility(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_write_eligibility(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_write_eligibility_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_intent_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerBuildIntentRequest,
    ) -> ScholarChatGroundedAnswerBuildIntentPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_intent(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_intent(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_intent_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_execution_readiness_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerExecutionReadinessPreviewRequest,
    ) -> ScholarChatGroundedAnswerExecutionReadinessPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_execution_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_execution_readiness(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_execution_readiness_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_execution_plan_boundary_fields(
        preview: &ScholarChatGroundedAnswerExecutionPlanPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
        assert!(preview.no_grounded_answer_write);
    }

    fn assert_grounded_answer_execution_plan_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerExecutionPlanPreviewRequest,
    ) -> ScholarChatGroundedAnswerExecutionPlanPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_execution_plan(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_execution_plan(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_execution_plan_boundary_fields(preview);
        }
        first
    }

    fn assert_scientific_discipline_registry_boundary_fields(
        preview: &ScholarChatScientificDisciplineRegistryPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.registry_preview_only);
        assert!(preview.no_web_request);
        assert!(preview.no_scraping);
        assert!(preview.no_connector_call);
        assert!(preview.no_source_import);
        assert!(preview.no_local_file_indexing);
        assert!(preview.no_bm25_index);
        assert!(preview.no_vector_index);
        assert!(preview.no_model_loading);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_evidence_pack_created);
        assert!(preview.no_artifact_write);
        assert!(preview.no_persistence);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_scientific_source_registry_boundary_fields(
        preview: &ScholarChatScientificSourceRegistryPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.source_registry_preview_only);
        assert!(preview.no_web_request);
        assert!(preview.no_scraping);
        assert!(preview.no_connector_call);
        assert!(preview.no_source_import);
        assert!(preview.no_local_file_indexing);
        assert!(preview.no_bm25_index);
        assert!(preview.no_vector_index);
        assert!(preview.no_model_loading);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_evidence_pack_created);
        assert!(preview.no_artifact_write);
        assert!(preview.no_persistence);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_scientific_query_understanding_boundary_fields(
        preview: &ScholarChatScientificQueryUnderstandingPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.query_understanding_preview_only);
        assert!(preview.no_web_request);
        assert!(preview.no_scraping);
        assert!(preview.no_connector_call);
        assert!(preview.no_source_import);
        assert!(preview.no_local_file_indexing);
        assert!(preview.no_bm25_index);
        assert!(preview.no_vector_index);
        assert!(preview.no_model_loading);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_evidence_pack_created);
        assert!(preview.no_artifact_write);
        assert!(preview.no_persistence);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_scientific_query_understanding_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatScientificQueryUnderstandingPreviewRequest,
    ) -> ScholarChatScientificQueryUnderstandingPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_scientific_query_understanding(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_scientific_query_understanding(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_scientific_query_understanding_boundary_fields(preview);
        }
        first
    }

    fn assert_scientific_search_plan_boundary_fields(preview: &ScholarChatScientificSearchPlanPreview) {
        assert!(preview.preview_only);
        assert!(preview.scientific_search_plan_preview_only);
        assert!(preview.no_web_request);
        assert!(preview.no_scraping);
        assert!(preview.no_connector_call);
        assert!(preview.no_source_import);
        assert!(preview.no_local_file_indexing);
        assert!(preview.no_bm25_index);
        assert!(preview.no_vector_index);
        assert!(preview.no_model_loading);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_evidence_pack_created);
        assert!(preview.no_artifact_write);
        assert!(preview.no_persistence);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_scientific_search_plan_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatScientificSearchPlanRequest,
    ) -> ScholarChatScientificSearchPlanPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_scientific_search_plan(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_scientific_search_plan(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_scientific_search_plan_boundary_fields(preview);
        }
        first
    }

    fn assert_scientific_search_plan_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_scientific_search_plan")
            .unwrap();
        let end = source[start..]
            .find("pub fn preview_scholar_chat_answer_readiness")
            .unwrap();
        let body = &source[start..start + end];
        assert_eq!(body.matches("preview_scholar_chat_scientific_query_understanding").count(), 1);
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("reqwest::"));
        assert!(!body.contains("ureq::"));
        assert!(!body.contains("std::fs"));
        assert!(!body.contains("fs::"));
        assert!(!body.contains("RetrievalService::new"));
        assert!(!body.contains("SourceRegistry::"));
        assert!(!body.contains("preview_scholar_chat_retrieval"));
        assert!(!body.contains("preview_scholar_chat_evidence_plan"));
        assert!(!body.contains("preview_scholar_chat_prompt_pack"));
        assert!(!body.contains("preview_scholar_chat_answer_readiness"));
        assert!(!body.contains("preview_scholar_chat_draft_inference"));
        assert!(!body.contains("preview_scholar_chat_grounded_answer"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    fn assert_runtime_diagnostic_bridge_boundary_fields(
        preview: &ScholarChatRuntimeDiagnosticBridgePreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.no_smoke_execution);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_answer_draft_created);
        assert!(preview.no_grounded_answer_created);
        assert!(preview.no_final_answer_created);
        assert!(preview.no_grounding_applied);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_runtime_diagnostic_bridge_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatRuntimeDiagnosticBridgePreviewRequest,
    ) -> ScholarChatRuntimeDiagnosticBridgePreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_runtime_diagnostic_bridge(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_runtime_diagnostic_bridge(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(
            fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt"))
                .unwrap(),
            ""
        );
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_runtime_diagnostic_bridge_boundary_fields(preview);
        }
        first
    }

    fn assert_runtime_diagnostic_result_boundary_fields(
        preview: &ScholarChatRuntimeDiagnosticResultPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.diagnostic_result_only);
        assert!(preview.no_smoke_execution);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_new_process_spawn);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_answer_draft_created);
        assert!(preview.no_grounded_answer_created);
        assert!(preview.no_final_answer_created);
        assert!(preview.no_grounding_applied);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_runtime_answer_pipeline_gate_boundary_fields(
        preview: &ScholarChatRuntimeAnswerPipelineGatePreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.gate_only);
        assert!(preview.no_smoke_execution);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_new_process_spawn);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_answer_draft_created);
        assert!(preview.no_grounded_answer_created);
        assert!(preview.no_final_answer_created);
        assert!(preview.no_grounding_applied);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_runtime_diagnostic_result_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatRuntimeDiagnosticResultPreviewRequest,
    ) -> ScholarChatRuntimeDiagnosticResultPreview {
        let before_entries = count_entries_recursively(temp.path());
        let before_aegis_exists = temp.path().join(".aegis").exists();
        let first = preview_scholar_chat_runtime_diagnostic_result(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_runtime_diagnostic_result(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        let after_aegis_exists = temp.path().join(".aegis").exists();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert_eq!(before_aegis_exists, after_aegis_exists);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_runtime_diagnostic_result_boundary_fields(preview);
        }
        first
    }

    fn assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatRuntimeAnswerPipelineGatePreviewRequest,
    ) -> ScholarChatRuntimeAnswerPipelineGatePreview {
        let before_entries = count_entries_recursively(temp.path());
        let before_aegis_exists = temp.path().join(".aegis").exists();
        let first = preview_scholar_chat_runtime_answer_pipeline_gate(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_runtime_answer_pipeline_gate(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        let after_aegis_exists = temp.path().join(".aegis").exists();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert_eq!(before_aegis_exists, after_aegis_exists);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_runtime_answer_pipeline_gate_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_request_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerBuildRequestPreviewRequest,
    ) -> ScholarChatGroundedAnswerBuildRequestPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_request(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_request(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_request_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_preflight_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest,
    ) -> ScholarChatGroundedAnswerBuildPreflightPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_preflight(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_preflight(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_preflight_boundary_fields(preview);
        }
        first
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_drops_stopwords_and_short_noise_terms() {
        let terms = inspection_terms("the and of to in a an is are was were with for on by as at from this that 12 alpha 2024 x y");
        assert!(!terms.contains("the"));
        assert!(!terms.contains("and"));
        assert!(!terms.contains("x"));
        assert!(terms.contains("12"));
        assert!(terms.contains("2024"));
        assert!(terms.contains("alpha"));
    }

    #[test]
    fn scholar_chat_retrieval_preview_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_retrieval(temp.path(), retrieval_request("   ", vec![]));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_retrieval_preview_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_retrieval(temp.path(), retrieval_request("Explain alpha", vec![invalid.to_string()]));
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_retrieval_preview_allows_empty_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_retrieval(temp.path(), retrieval_request("Explain alpha", vec![])).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert_eq!(response.candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("unscoped")));
        assert_eq!(response.status, ScholarChatStatus::PreviewOnly);
    }

    #[test]
    fn scholar_chat_retrieval_preview_does_not_build_missing_indexes() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, crate::source_metadata::SourceMetadataInput {
            title: "Notes".to_string(),
            source_type: crate::source_metadata::SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }).unwrap();
        crate::extraction::ExtractionService::new(temp.path()).extract_source(&source.source_id).unwrap();
        crate::chunking::ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        let source_id = source.source_id.clone();
        let version_id = source.version_id.clone();

        let response = preview_scholar_chat_retrieval(
            temp.path(),
            retrieval_request("alpha", vec![source_id.clone()]),
        )
        .unwrap();

        assert_eq!(response.selected_source_ids, vec![source_id.clone()]);
        assert_eq!(response.candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("not ready")));
        let index_path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(&version_id)
            .join("retrieval")
            .join("index.json");
        assert!(!index_path.exists());
    }

    #[test]
    fn scholar_chat_retrieval_preview_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let source_b = build_source_with_index(&temp, "alpha delta\n\nalpha epsilon\n");
        let request = retrieval_request("  alpha  ", vec![source_b.clone(), source_a.clone()]);
        let first = preview_scholar_chat_retrieval(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_retrieval(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "alpha");
        assert_eq!(first.selected_source_ids, vec![source_b, source_a]);
        assert!(first.candidates.windows(2).all(|pair| pair[0].score >= pair[1].score));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").join("corpus").join("sources").join("missing").exists());
    }

    #[test]
    fn scholar_chat_evidence_plan_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_evidence_plan(temp.path(), evidence_plan_request("   ", vec![]));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_evidence_plan_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_evidence_plan(temp.path(), evidence_plan_request("Explain alpha", vec![invalid.to_string()]));
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_evidence_plan_allows_no_selected_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_evidence_plan(temp.path(), evidence_plan_request("Explain alpha", vec![])).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert_eq!(response.retrieval_candidate_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("No selected sources")));
        assert!(response.warnings.iter().any(|warning| warning.contains("Evidence Pack was built")));
        assert_eq!(response.status, ScholarChatEvidencePlanStatus::EvidencePlanPreview);
        assert!(response.evidence_plan.evidence_required);
        assert!(response.evidence_plan.evidence_pack_would_be_built_later);
    }

    #[test]
    fn scholar_chat_evidence_plan_does_not_build_missing_indexes() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, crate::source_metadata::SourceMetadataInput {
            title: "Notes".to_string(),
            source_type: crate::source_metadata::SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }).unwrap();
        crate::extraction::ExtractionService::new(temp.path()).extract_source(&source.source_id).unwrap();
        crate::chunking::ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        let source_id = source.source_id.clone();
        let version_id = source.version_id.clone();

        let response = preview_scholar_chat_evidence_plan(
            temp.path(),
            evidence_plan_request("alpha", vec![source_id.clone()]),
        )
        .unwrap();

        assert_eq!(response.selected_source_ids, vec![source_id.clone()]);
        assert_eq!(response.retrieval_candidate_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("not ready")));
        let index_path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(&version_id)
            .join("retrieval")
            .join("index.json");
        assert!(!index_path.exists());
    }

    #[test]
    fn scholar_chat_evidence_plan_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let source_b = build_source_with_index(&temp, "alpha delta\n\nalpha epsilon\n");
        let request = evidence_plan_request("  alpha  ", vec![source_b.clone(), source_a.clone()]);
        let first = preview_scholar_chat_evidence_plan(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_evidence_plan(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "alpha");
        assert_eq!(first.selected_source_ids, vec![source_b, source_a]);
        assert_eq!(first.retrieval_candidate_count, first.candidates.len());
        assert_eq!(first.evidence_candidate_count, first.candidates.len());
        assert!(first.candidates.windows(2).all(|pair| pair[0].score >= pair[1].score));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").join("corpus").join("sources").join("missing").exists());
    }

    #[test]
    fn scholar_chat_prompt_pack_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_prompt_pack(temp.path(), prompt_pack_request("   ", vec![]));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_prompt_pack_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_prompt_pack(temp.path(), prompt_pack_request("Explain alpha", vec![invalid.to_string()]));
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_prompt_pack_allows_no_selected_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_prompt_pack(temp.path(), prompt_pack_request("Explain alpha", vec![])).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert_eq!(response.context_items.len(), 0);
        assert_eq!(response.status, ScholarChatPromptPackStatus::PromptPackPreview);
        assert!(response.warnings.iter().any(|warning| warning.contains("unscoped")));
        assert!(response.warnings.iter().any(|warning| warning.contains("no model inference")));
    }

    #[test]
    fn scholar_chat_prompt_pack_includes_mode_and_grounding_sections() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let response = preview_scholar_chat_prompt_pack(
            temp.path(),
            prompt_pack_request("  alpha  ", vec![source_id.clone()]),
        )
        .unwrap();
        assert_eq!(response.normalized_prompt, "alpha");
        assert_eq!(response.selected_source_ids, vec![source_id.clone()]);
        assert_eq!(response.prompt_pack.section_count, 5);
        assert_eq!(response.prompt_pack.context_item_count, response.context_items.len());
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::SystemOrPolicyInstructions));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::ModeInstructions));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::GroundingInstructions));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::SourceContext));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::UserPrompt));
        assert!(response.prompt_pack.sections.iter().any(|section| section.lines.iter().any(|line| line.contains("local evidence"))));
        assert!(response.warnings.iter().any(|warning| warning.contains("local evidence")));
    }

    #[test]
    fn scholar_chat_prompt_pack_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let source_b = build_source_with_index(&temp, "alpha delta\n\nalpha epsilon\n");
        let request = prompt_pack_request("  alpha  ", vec![source_b.clone(), source_a.clone()]);
        let first = preview_scholar_chat_prompt_pack(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_prompt_pack(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.selected_source_ids, vec![source_b, source_a]);
        assert_eq!(first.prompt_pack.section_count, first.prompt_pack.sections.len());
        assert_eq!(first.prompt_pack.context_item_count, first.context_items.len());
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").join("corpus").join("sources").join("missing").exists());
    }

    #[test]
    fn scholar_chat_answer_readiness_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "   ",
                GroundingPolicy::LocalOnly,
                vec![],
                runtime_config(None, None),
                false,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_answer_readiness_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_answer_readiness(
                temp.path(),
                answer_readiness_request(
                    "Explain alpha",
                    GroundingPolicy::LocalOnly,
                    vec![invalid.to_string()],
                    runtime_config(None, None),
                    false,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_answer_readiness_requires_sources_for_local_only() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalOnly,
                vec![],
                runtime_config,
                true,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::NeedsSources);
        assert_eq!(response.future_output_classification, ScholarChatAnswerReadinessOutputClassification::Blocked);
        assert_eq!(response.selected_source_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(response.next_required_actions.iter().any(|action| action.contains("Select one or more Scholar Chat sources")));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_blocks_local_only_without_retrieval_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, crate::source_metadata::SourceMetadataInput {
            title: "Notes".to_string(),
            source_type: crate::source_metadata::SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }).unwrap();
        crate::extraction::ExtractionService::new(temp.path()).extract_source(&source.source_id).unwrap();
        crate::chunking::ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalOnly,
                vec![source.source_id.clone()],
                runtime_config,
                true,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::Blocked);
        assert_eq!(response.future_output_classification, ScholarChatAnswerReadinessOutputClassification::Blocked);
        assert_eq!(response.selected_source_count, 1);
        assert_eq!(response.retrieval_candidate_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "blocked"));
        assert!(response.next_required_actions.iter().any(|action| action.contains("retrieval index")));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_needs_runtime_config_when_local_runtime_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                runtime_config(None, None),
                true,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::NeedsRuntimeConfig);
        assert_eq!(response.runtime_health_status, LocalModelRuntimeHealthStatus::ConfigPresent);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_runtime_config"));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_requires_execution_consent_when_runtime_is_ready() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                runtime_config,
                false,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::NeedsExecutionConsent);
        assert_eq!(response.invocation_plan_status, LocalRuntimeInvocationPlanStatus::ReadyToInvokeLater);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_execution_consent"));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_can_be_ready_for_draft_inference_later() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let request = answer_readiness_request(
            "  Explain alpha  ",
            GroundingPolicy::AllowMarkedModelKnowledge,
            vec![source_id.clone()],
            runtime_config,
            true,
        );
        let first = preview_scholar_chat_answer_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_answer_readiness(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(first.status, ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater);
        assert_eq!(first.future_output_classification, ScholarChatAnswerReadinessOutputClassification::UngroundedDraft);
        assert_eq!(first.normalized_prompt, "Explain alpha");
        assert_eq!(first.mode, ScholarChatMode::ThesisWriting);
        assert_eq!(first.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge);
        assert_eq!(first.selected_source_count, 1);
        assert!(first.prompt_pack_ready);
        assert!(first.would_generate_answer_now == false);
        assert!(first.would_build_evidence_pack_now == false);
        assert!(first.would_create_final_answer_now == false);
        assert!(first.warnings.iter().any(|warning| warning.kind == "future_draft_marking_required"));
        assert!(first.next_required_actions.iter().any(|action| action.contains("prompt pack")));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&first);
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let result = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "   ",
                GroundingPolicy::AllowMarkedModelKnowledge,
                vec![],
                runtime_config,
                true,
                None,
                None,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_draft_inference_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_draft_inference(
                temp.path(),
                draft_inference_request(
                    "Explain alpha",
                    GroundingPolicy::AllowMarkedModelKnowledge,
                    vec![invalid.to_string()],
                    runtime_config.clone(),
                    true,
                    None,
                    None,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
    }

    #[test]
    fn scholar_chat_draft_inference_blocks_when_execution_is_disabled() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let request = draft_inference_request(
            "Explain alpha",
            GroundingPolicy::AllowMarkedModelKnowledge,
            vec![],
            runtime_config,
            false,
            None,
            None,
        );
        let first = preview_scholar_chat_draft_inference(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_draft_inference(temp.path(), request).unwrap();
        let response = first.clone();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsExecutionConsent);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert_eq!(response.runtime_health_status, LocalModelRuntimeHealthStatus::NotConfigured);
        assert_eq!(response.invocation_plan_status, LocalRuntimeInvocationPlanStatus::PreviewOnly);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_execution_consent"));
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_blocks_local_only_without_evidence_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note-no-index.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "Explain alpha",
                GroundingPolicy::LocalOnly,
                vec![source.source_id.clone()],
                runtime_config,
                true,
                None,
                None,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsEvidence);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_evidence"));
        assert!(response.warnings.iter().any(|warning| warning.kind == "evidence_required"));
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_needs_runtime_config_when_runtime_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                LocalModelRuntimeConfig {
                    runtime_kind: LocalModelRuntimeKind::None,
                    model_path: None,
                    executable_path: None,
                    context_window: Some(512),
                    gpu_layers: Some(0),
                    temperature: Some(0.0),
                },
                true,
                None,
                None,
            ),
        )
        .unwrap();
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsRuntimeConfig);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_runtime_config"));
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_reports_missing_model_and_executable_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                runtime_config(
                    Some(temp.path().join("missing-model.gguf").to_string_lossy().as_ref()),
                    Some(temp.path().join("missing-draft-helper.exe").to_string_lossy().as_ref()),
                ),
                true,
                None,
                None,
            ),
        )
        .unwrap();
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsRuntimeConfig);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert_eq!(response.safe_model_file_name.as_deref(), Some("missing-model.gguf"));
        assert!(response.safe_executable_file_name.is_none());
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let request = draft_inference_request(
            "  Explain alpha  ",
            GroundingPolicy::AllowMarkedModelKnowledge,
            vec![],
            runtime_config,
            true,
            None,
            None,
        );
        let first = preview_scholar_chat_draft_inference(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_draft_inference(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        let mut first_sanitized = first.clone();
        let mut second_sanitized = second.clone();
        first_sanitized.duration_ms = 0;
        second_sanitized.duration_ms = 0;
        assert_eq!(first_sanitized, second_sanitized);
        assert_eq!(first.status, ScholarChatDraftInferenceStatus::InferenceSucceeded);
        assert_eq!(first.output_classification, ScholarChatDraftOutputClassification::UngroundedModelDraft);
        assert_eq!(first.normalized_prompt, "Explain alpha");
        assert_eq!(first.mode, ScholarChatMode::ThesisWriting);
        assert_eq!(first.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge);
        assert_eq!(first.selected_source_count, 0);
        assert_eq!(first.retrieval_candidate_count, 0);
        assert_eq!(first.evidence_candidate_count, 0);
        assert!(first.prompt_pack_section_count > 0);
        assert!(first.execution_attempted);
        assert_eq!(first.allow_model_execution, true);
        assert!(first.stdout_preview.contains("stdout marker"));
        assert!(first.stderr_preview.contains("stderr marker"));
        assert_draft_boundary_fields(&first);
        assert_eq!(before_entries, after_entries);
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: request("   "),
                draft_text: Some("Draft text".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_draft_grounding_inspection(
                temp.path(),
                ScholarChatDraftGroundingInspectionRequest {
                    scholar_chat_request: ScholarChatRequest {
                        prompt: "Explain grounded text".to_string(),
                        mode: ScholarChatMode::LectureLearning,
                        grounding_policy: GroundingPolicy::LocalFirst,
                        selected_source_ids: vec![invalid.to_string()],
                    },
                    draft_text: Some("alpha beta.".to_string()),
                    max_items: Some(4),
                },
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_reports_no_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: request("Explain grounded text"),
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::NoDraftText);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "Explain grounded text".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("alpha beta.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("No Scholar Chat source context selected")));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_reports_no_evidence_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note-no-index.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();

        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "Explain grounded text".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("alpha beta.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("No local evidence candidates")));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let before_entries = count_entries_recursively(temp.path());
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "  alpha beta grounded evidence  ".to_string(),
                mode: ScholarChatMode::ThesisWriting,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id.clone()],
            },
            draft_text: Some("Alpha beta. Gamma delta? Alpha beta gamma.".to_string()),
            max_items: Some(8),
        };
        let first = preview_scholar_chat_draft_grounding_inspection(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_draft_grounding_inspection(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "alpha beta grounded evidence");
        assert_eq!(first.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(first.selected_source_count, 1);
        assert!(!first.items.is_empty());
        assert!(first.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(before_entries, after_entries);
        assert_draft_grounding_inspection_boundary_fields(&first);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_supports_local_evidence_and_clamps_items() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some(
                    "Alpha beta support. Gamma. Theta. Alpha beta gamma evidence. Delta alpha beta. More alpha beta. Another alpha beta.".to_string(),
                ),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 4);
        assert_eq!(
            result.supported_item_count + result.weakly_supported_item_count + result.unsupported_item_count,
            result.items.len()
        );
        assert!(result.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence));
        assert!(result.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::WeaklySupported));
        assert!(result.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::Unsupported));
        assert!(result.items.iter().all(|item| !item.text_preview.contains("  ")));
        assert!(result.items.iter().all(|item| !item.locator_previews.iter().any(|preview| preview.contains("section_path"))));
        assert!(result.warnings.iter().any(|warning| warning.kind == "inspection_clamped"));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_local_only_support_needs_clear_overlap() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::AllowMarkedModelKnowledge,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Gamma.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::WeaklySupported);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("Model knowledge is not used in this preview")));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_marks_single_meaningful_overlap_as_weakly_supported() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha beta grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::WeaklySupported);
        assert_eq!(result.items[0].matched_evidence_count, 1);
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_requires_two_meaningful_terms_for_supported_overlap() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha beta grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("The alpha beta.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 1);
        assert_eq!(result.weakly_supported_item_count, 0);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence);
        assert!(result.items[0].matched_evidence_count >= 1);
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_leaves_unrelated_items_unsupported() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha beta grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 0);
        assert_eq!(result.unsupported_item_count, 1);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::Unsupported);
        assert_eq!(result.items[0].matched_evidence_count, 0);
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_draft_readiness(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "   ".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert_eq!(result.inspection_status, ScholarChatDraftGroundingInspectionStatus::NoDraftText);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert_eq!(result.inspection_status, ScholarChatDraftGroundingInspectionStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_blocks_without_evidence_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note-no-index.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();

        let result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert_eq!(result.inspection_status, ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_marks_weak_or_unsupported_items_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let weak_result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id.clone()],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(weak_result.status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(weak_result.inspection_status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(weak_result.weakly_supported_item_count, 1);
        assert_eq!(weak_result.unsupported_item_count, 0);
        assert!(weak_result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));

        let unsupported_result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(unsupported_result.status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(unsupported_result.inspection_status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(unsupported_result.supported_item_count, 0);
        assert_eq!(unsupported_result.weakly_supported_item_count, 0);
        assert_eq!(unsupported_result.unsupported_item_count, 1);
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_is_ready_only_when_every_item_has_local_support() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "alpha grounded evidence".to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id],
            },
            draft_text: Some("Alpha beta.".to_string()),
            max_items: Some(4),
        };
        let first = assert_grounded_draft_readiness_deterministic_and_path_free(&temp, request);
        assert_eq!(first.status, ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater);
        assert_eq!(first.inspection_status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(first.inspected_item_count, 1);
        assert_eq!(first.supported_item_count, 1);
        assert_eq!(first.weakly_supported_item_count, 0);
        assert_eq!(first.unsupported_item_count, 0);
        assert!(first.summary.contains("All inspected items were supported by local evidence"));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_plan(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "   ".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_blocks_without_evidence_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note-no-index.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();

        let result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_marks_weak_or_unsupported_items_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let weak_result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id.clone()],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(weak_result.status, ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview);
        assert_eq!(weak_result.readiness_status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(weak_result.weakly_supported_item_count, 1);
        assert_eq!(weak_result.unsupported_item_count, 0);
        assert!(weak_result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));

        let unsupported_result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(unsupported_result.status, ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview);
        assert_eq!(unsupported_result.readiness_status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(unsupported_result.supported_item_count, 0);
        assert_eq!(unsupported_result.weakly_supported_item_count, 0);
        assert_eq!(unsupported_result.unsupported_item_count, 1);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_is_ready_only_when_every_item_has_local_support() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "alpha grounded evidence".to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id],
            },
            draft_text: Some("Alpha beta.".to_string()),
            max_items: Some(4),
        };
        let first = assert_grounded_answer_build_plan_deterministic_and_path_free(&temp, request);
        assert_eq!(first.status, ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater);
        assert_eq!(
            first.readiness_status,
            ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater
        );
        assert_eq!(first.inspected_item_count, 1);
        assert_eq!(first.supported_item_count, 1);
        assert_eq!(first.weakly_supported_item_count, 0);
        assert_eq!(first.unsupported_item_count, 0);
        assert!(first.summary.contains("All inspected items were supported by local evidence"));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_candidate(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "   ".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.candidate_items.is_empty());
        assert_eq!(result.inspected_item_count, 0);
        assert_eq!(result.supported_item_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.candidate_items.is_empty());
        assert_eq!(result.inspected_item_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_blocks_without_evidence_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note-no-index.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();

        let result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.candidate_items.is_empty());
        assert_eq!(result.evidence_candidate_count, 0);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_marks_weak_or_unsupported_items_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let weak_result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id.clone()],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(weak_result.status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(weak_result.candidate_statement_count, 0);
        assert!(weak_result.candidate_items.is_empty());
        assert_eq!(weak_result.weakly_supported_item_count, 1);
        assert_eq!(weak_result.unsupported_item_count, 0);
        assert!(weak_result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));

        let unsupported_result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(unsupported_result.status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(unsupported_result.candidate_statement_count, 0);
        assert!(unsupported_result.candidate_items.is_empty());
        assert_eq!(unsupported_result.supported_item_count, 0);
        assert_eq!(unsupported_result.weakly_supported_item_count, 0);
        assert_eq!(unsupported_result.unsupported_item_count, 1);
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_is_ready_only_when_every_item_has_local_support() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "alpha grounded evidence".to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id.clone()],
            },
            draft_text: Some("Alpha beta. Alpha beta gamma.".to_string()),
            max_items: Some(4),
        };
        let candidate_preview = assert_grounded_answer_candidate_deterministic_and_path_free(&temp, request.clone());
        let build_plan_preview = preview_scholar_chat_grounded_answer_build_plan(temp.path(), request.clone()).unwrap();
        assert_eq!(build_plan_preview.status, ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater);
        assert_eq!(candidate_preview.selected_source_count, build_plan_preview.selected_source_count);
        assert_eq!(candidate_preview.evidence_candidate_count, build_plan_preview.evidence_candidate_count);
        assert_eq!(candidate_preview.inspected_item_count, build_plan_preview.inspected_item_count);
        assert_eq!(candidate_preview.supported_item_count, build_plan_preview.supported_item_count);
        assert_eq!(candidate_preview.weakly_supported_item_count, build_plan_preview.weakly_supported_item_count);
        assert_eq!(candidate_preview.unsupported_item_count, build_plan_preview.unsupported_item_count);
        assert_eq!(candidate_preview.status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(candidate_preview.candidate_statement_count, 2);
        assert_eq!(candidate_preview.candidate_statement_count, candidate_preview.candidate_items.len());
        assert_eq!(candidate_preview.inspected_item_count, 2);
        assert_eq!(candidate_preview.supported_item_count, 2);
        assert_eq!(candidate_preview.weakly_supported_item_count, 0);
        assert_eq!(candidate_preview.unsupported_item_count, 0);
        assert!(candidate_preview.summary.contains("All inspected items were supported by local evidence"));
        let inspection_preview = preview_scholar_chat_draft_grounding_inspection(temp.path(), request).unwrap();
        assert_eq!(inspection_preview.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(candidate_preview.candidate_items.len(), inspection_preview.items.len());
        for (candidate_item, inspection_item) in candidate_preview.candidate_items.iter().zip(inspection_preview.items.iter()) {
            assert_eq!(candidate_item.item_index, inspection_item.item_index);
            assert_eq!(candidate_item.statement_preview, inspection_item.text_preview);
            assert_eq!(candidate_item.support_status, inspection_item.support_status);
            assert_eq!(candidate_item.matched_evidence_count, inspection_item.matched_evidence_count);
            assert_eq!(candidate_item.source_ids, inspection_item.source_ids);
            assert_eq!(candidate_item.locator_previews, inspection_item.locator_previews);
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_intent(
            temp.path(),
            build_intent_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_rejects_invalid_answer_draft_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_build_intent(
                temp.path(),
                build_intent_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_treats_whitespace_answer_draft_id_as_missing_input() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("   "),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()));
        assert!(!result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "answer_draft_id_invalid"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("   "),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"write_eligible_later".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "write_eligibility_blocked"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta."),
                vec![],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"write_eligible_later".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "write_eligibility_blocked"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_needs_review_when_candidate_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(result.selected_source_count, 1);
        assert_eq!(result.evidence_candidate_count, 1);
        assert_eq!(result.inspected_item_count, 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result
            .intent_reasons
            .iter()
            .any(|reason| reason.contains("Weakly supported or unsupported draft items remain.")));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_explicit_user_intent() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("draft-1"),
                false,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"explicit_user_intent".to_string()));
        assert!(result.missing_inputs.contains(&"explicit_user_intent".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "explicit_user_intent_missing"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Provide explicit user intent before any GroundedAnswer service call.")));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_answer_draft_id() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                None,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "answer_draft_id_missing"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Provide an answer draft ID before any GroundedAnswer service call.")));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_is_ready_only_when_all_inputs_present() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = build_intent_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![source_id.clone()],
            Some("draft-1"),
            true,
        );
        let build_intent_preview = assert_grounded_answer_build_intent_deterministic_and_path_free(&temp, request.clone());
        let write_eligibility_preview = assert_grounded_answer_write_eligibility_deterministic_and_path_free(&temp, request.grounding_request);
        assert_eq!(build_intent_preview.status, ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater);
        assert_eq!(build_intent_preview.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(build_intent_preview.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(build_intent_preview.selected_source_count, write_eligibility_preview.selected_source_count);
        assert_eq!(build_intent_preview.evidence_candidate_count, write_eligibility_preview.evidence_candidate_count);
        assert_eq!(build_intent_preview.inspected_item_count, write_eligibility_preview.inspected_item_count);
        assert_eq!(build_intent_preview.supported_item_count, write_eligibility_preview.supported_item_count);
        assert_eq!(build_intent_preview.weakly_supported_item_count, write_eligibility_preview.weakly_supported_item_count);
        assert_eq!(build_intent_preview.unsupported_item_count, write_eligibility_preview.unsupported_item_count);
        assert_eq!(build_intent_preview.candidate_statement_count, write_eligibility_preview.candidate_statement_count);
        assert_eq!(
            build_intent_preview.required_inputs,
            vec![
                "write_eligible_later".to_string(),
                "explicit_user_intent".to_string(),
                "answer_draft_id".to_string(),
            ]
        );
        assert!(build_intent_preview.missing_inputs.is_empty());
        assert!(build_intent_preview
            .intent_reasons
            .iter()
            .any(|reason| reason.contains("All inspected items were supported by local evidence")));
        assert!(build_intent_preview
            .warnings
            .iter()
            .any(|warning| warning.kind == "intent_ready_later"));
        assert!(build_intent_preview
            .next_required_actions
            .iter()
            .any(|action| action.contains("A future user-confirmed GroundedAnswer service call can be added later without changing this preview.")));
        assert_grounded_answer_build_intent_boundary_fields(&build_intent_preview);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_request(
            temp.path(),
            build_request_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_rejects_invalid_answer_draft_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_build_request(
                temp.path(),
                build_request_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_build_request(
                temp.path(),
                build_request_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.required_inputs.contains(&"selected_source_ids".to_string()));
        assert!(result.missing_inputs.contains(&"selected_source_ids".to_string()));
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_treats_whitespace_answer_draft_id_as_missing_input() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("   "),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.answer_draft_id.is_none());
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_needs_review_when_build_intent_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result
            .request_reasons
            .iter()
            .any(|reason| reason.contains("Weakly supported or unsupported draft items remain")));
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_is_ready_only_when_build_intent_is_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let source_b = build_source_with_index(&temp, "alpha beta gamma\nalpha beta epsilon\n");
        let request = build_request_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![format!("  {source_b}  "), format!("  {source_a}  ")],
            Some("  draft-1  "),
            true,
        );
        let first = assert_grounded_answer_build_request_deterministic_and_path_free(&temp, request.clone());
        let build_intent_preview = preview_scholar_chat_grounded_answer_build_intent(
            temp.path(),
            request.build_intent_request.clone(),
        )
        .unwrap();
        assert_eq!(first.status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert_eq!(first.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater);
        assert_eq!(first.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(first.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(first.selected_source_count, 2);
        assert_eq!(first.selected_source_ids, vec![source_b.clone(), source_a.clone()]);
        assert_eq!(first.answer_draft_id.as_deref(), Some("draft-1"));
        assert_eq!(first.selected_source_count, build_intent_preview.selected_source_count);
        assert_eq!(first.evidence_candidate_count, build_intent_preview.evidence_candidate_count);
        assert_eq!(first.inspected_item_count, build_intent_preview.inspected_item_count);
        assert_eq!(first.supported_item_count, build_intent_preview.supported_item_count);
        assert_eq!(first.weakly_supported_item_count, build_intent_preview.weakly_supported_item_count);
        assert_eq!(first.unsupported_item_count, build_intent_preview.unsupported_item_count);
        assert_eq!(first.candidate_statement_count, build_intent_preview.candidate_statement_count);
        assert!(first
            .request_reasons
            .iter()
            .any(|reason| reason.contains("ready later for a future GroundedAnswer service call")));
        assert_grounded_answer_build_request_boundary_fields(&first);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_allows_missing_answer_draft_file_when_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("missing-draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some("missing-draft-1"));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()) == false);
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_preflight(
            temp.path(),
            build_preflight_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_invalid_answer_draft_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_build_preflight(
                temp.path(),
                build_preflight_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_build_preflight(
                temp.path(),
                build_preflight_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_invalid_selected_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_build_preflight(
                temp.path(),
                build_preflight_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_when_build_request_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"build_request_ready_later".to_string()));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_needs_review_when_build_request_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert!(result
            .preflight_reasons
            .iter()
            .any(|reason| reason.contains("needs review")));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_without_answer_draft_id() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                None,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_id_missing"));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_treats_whitespace_answer_draft_id_as_missing_input() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("   "),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_id_missing"));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_when_answer_draft_missing() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("missing-draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert_eq!(result.answer_draft_claim_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_missing"));
        assert!(result.missing_inputs.contains(&"answer_draft_present".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_readable".to_string()));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_when_answer_draft_malformed() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, version_id, _) = build_readable_answer_draft_fixture(&temp);
        let draft_path = crate::corpus_paths::CorpusPaths::new(temp.path())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts")
            .join(format!("{answer_draft_id}.json"));
        fs::write(&draft_path, "{not valid json").unwrap();
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some(&answer_draft_id),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert!(result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert_eq!(result.answer_draft_claim_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_unreadable"));
        assert!(result.missing_inputs.contains(&"answer_draft_readable".to_string()));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_is_ready_only_when_build_request_is_ready_and_answer_draft_readable() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, claim_count) = build_readable_answer_draft_fixture(&temp);
        let request = build_preflight_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![format!("  {source_id}  ")],
            Some(&answer_draft_id),
            true,
        );
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(&temp, request);
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some(answer_draft_id.as_str()));
        assert!(result.answer_draft_present);
        assert!(result.answer_draft_readable);
        assert_eq!(result.answer_draft_claim_count, claim_count);
        assert_eq!(result.selected_source_ids, vec![source_id]);
        assert!(result
            .preflight_reasons
            .iter()
            .any(|reason| reason.contains("Answer draft readable: true")));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "preflight_ready_later"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_execution_readiness(
            temp.path(),
            execution_readiness_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_rejects_invalid_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_execution_readiness(
                temp.path(),
                execution_readiness_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_execution_readiness(
                temp.path(),
                execution_readiness_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_when_build_preflight_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"build_preflight_ready_later".to_string()));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_needs_review_when_build_preflight_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert!(result
            .readiness_reasons
            .iter()
            .any(|reason| reason.contains("needs review")));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_when_answer_draft_missing() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let draft_path = crate::corpus_paths::CorpusPaths::new(temp.path())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts")
            .join(format!("{answer_draft_id}.json"));
        fs::remove_file(&draft_path).unwrap();
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_missing"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_when_answer_draft_malformed() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let draft_path = crate::corpus_paths::CorpusPaths::new(temp.path())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts")
            .join(format!("{answer_draft_id}.json"));
        fs::write(&draft_path, "{not valid json").unwrap();
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert!(result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_unreadable"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_without_execution_consent() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, claim_count) = build_readable_answer_draft_fixture(&temp);
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "  alpha grounded evidence  ",
                Some("  Alpha beta. Alpha beta gamma.  "),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                false,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some(answer_draft_id.as_str()));
        assert_eq!(result.selected_source_ids, vec![source_id]);
        assert_eq!(result.answer_draft_claim_count, claim_count);
        assert_eq!(result.missing_inputs, vec!["execution_consent".to_string()]);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "execution_consent_missing"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_is_ready_later_only_when_execution_consent_is_given() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, claim_count) = build_readable_answer_draft_fixture(&temp);
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "  alpha grounded evidence  ",
                Some("  Alpha beta. Alpha beta gamma.  "),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some(answer_draft_id.as_str()));
        assert_eq!(result.selected_source_ids, vec![source_id]);
        assert_eq!(result.answer_draft_claim_count, claim_count);
        assert!(result.missing_inputs.is_empty());
        assert!(result
            .readiness_reasons
            .iter()
            .any(|reason| reason.contains("Execution consent: true")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("future GroundedAnswer service call")));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_plan_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_execution_plan(
            temp.path(),
            execution_plan_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_plan_rejects_invalid_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_execution_plan(
                temp.path(),
                execution_plan_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_execution_plan(
                temp.path(),
                execution_plan_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_plan_blocks_when_readiness_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_execution_plan_deterministic_and_path_free(
            &temp,
            execution_plan_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"build_preflight_ready_later".to_string()));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_plan_needs_review_when_readiness_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_execution_plan_deterministic_and_path_free(
            &temp,
            execution_plan_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview);
        assert_eq!(result.readiness_status, ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert!(result
            .plan_reasons
            .iter()
            .any(|reason| reason.contains("still needs review")));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_plan_is_ready_later_only_when_execution_consent_is_given() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, claim_count) = build_readable_answer_draft_fixture(&temp);
        let result = assert_grounded_answer_execution_plan_deterministic_and_path_free(
            &temp,
            execution_plan_request(
                "  alpha grounded evidence  ",
                Some("  Alpha beta. Alpha beta gamma.  "),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionPlanStatus::PlanReadyLater);
        assert_eq!(result.readiness_status, ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some(answer_draft_id.as_str()));
        assert_eq!(result.selected_source_ids, vec![source_id]);
        assert_eq!(result.answer_draft_claim_count, claim_count);
        assert!(result.missing_inputs.is_empty());
        assert_eq!(result.planned_operation, "future_grounded_answer_build");
        assert!(result.planned_inputs.contains(&"execution_consent".to_string()));
        assert!(result.planned_outputs.contains(&"future_grounded_answer_status".to_string()));
        assert!(result.planned_write_targets.contains(&"audit_log_entry".to_string()));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("planned later")));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_plan_blocks_when_execution_consent_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let result = assert_grounded_answer_execution_plan_deterministic_and_path_free(
            &temp,
            execution_plan_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some(&answer_draft_id),
                true,
                false,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.missing_inputs, vec!["execution_consent".to_string()]);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "execution_consent_missing"));
    }

    #[test]
    fn scholar_chat_scientific_discipline_registry_blocks_when_topic_is_blank() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_discipline_registry(
            temp.path(),
            scientific_discipline_registry_request("   ", Some("course"), Some("Module handbook")),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificDisciplineRegistryStatus::Blocked);
        assert!(result.recognized_concept.is_none());
        assert!(result.discipline_path.is_empty());
        assert!(result.planned_queries.is_empty());
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.contains("topic_missing")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Provide a scientific topic")));
        assert_scientific_discipline_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_discipline_registry_maps_signalentdeckung() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_discipline_registry(
            temp.path(),
            scientific_discipline_registry_request(" Signalentdeckung ", None, None),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificDisciplineRegistryStatus::ConceptMapped);
        assert_eq!(result.normalized_topic, "Signalentdeckung");
        assert_eq!(result.normalized_mode, "scholar_chat");
        assert_eq!(result.recognized_concept.as_deref(), Some("signal_detection_theory"));
        assert_eq!(result.label.as_deref(), Some("Signalentdeckungstheorie"));
        assert_eq!(
            result.discipline_path,
            vec![
                "psychology".to_string(),
                "general_psychology".to_string(),
                "perception".to_string(),
                "psychophysics".to_string(),
                "signal_detection_theory".to_string(),
            ]
        );
        assert_eq!(
            result.parent_path,
            vec![
                "psychology".to_string(),
                "general_psychology".to_string(),
                "perception".to_string(),
                "psychophysics".to_string(),
            ]
        );
        assert_eq!(
            result.related_methods,
            vec![
                "statistics".to_string(),
                "probability_theory".to_string(),
                "decision_theory".to_string(),
                "roc_analysis".to_string(),
            ]
        );
        assert_eq!(
            result.preferred_sources,
            vec![
                "pubpsych".to_string(),
                "psycharchives".to_string(),
                "openalex".to_string(),
                "crossref".to_string(),
                "pubmed_if_biomedical_context".to_string(),
            ]
        );
        assert_eq!(
            result.curriculum_sources,
            vec![
                "tu_darmstadt_module_handbook_candidate".to_string(),
                "local_course_materials_later".to_string(),
            ]
        );
        assert_eq!(
            result.canonical_mappings,
            vec![
                "psychology".to_string(),
                "general_psychology".to_string(),
                "psychophysics".to_string(),
                "signal_detection_theory".to_string(),
            ]
        );
        assert_eq!(
            result.planned_queries,
            vec![
                "Signalentdeckungstheorie".to_string(),
                "signal detection theory".to_string(),
                "psychophysics signal detection".to_string(),
                "d prime criterion ROC".to_string(),
            ]
        );
        assert_eq!(result.science_class, Some(ScholarChatScientificDisciplineScienceClass::CoreScience));
        assert_scientific_discipline_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_discipline_registry_maps_anova() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_discipline_registry(
            temp.path(),
            scientific_discipline_registry_request("ANOVA", Some("scientific_paper"), None),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificDisciplineRegistryStatus::ConceptMapped);
        assert_eq!(result.recognized_concept.as_deref(), Some("analysis_of_variance"));
        assert_eq!(result.label.as_deref(), Some("ANOVA / Varianzanalyse"));
        assert_eq!(
            result.discipline_path,
            vec![
                "statistics".to_string(),
                "inferential_statistics".to_string(),
                "hypothesis_testing".to_string(),
                "analysis_of_variance".to_string(),
            ]
        );
        assert_eq!(result.science_class, Some(ScholarChatScientificDisciplineScienceClass::CoreScience));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("Scientific Paper Mode")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("literature search")));
        assert_scientific_discipline_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_discipline_registry_maps_hypothesentests() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_discipline_registry(
            temp.path(),
            scientific_discipline_registry_request("Hypothesentests", Some("course"), Some("Module 123")),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificDisciplineRegistryStatus::ConceptMapped);
        assert_eq!(result.recognized_concept.as_deref(), Some("hypothesis_testing"));
        assert_eq!(result.label.as_deref(), Some("Hypothesentests"));
        assert_eq!(
            result.discipline_path,
            vec![
                "statistics".to_string(),
                "inferential_statistics".to_string(),
                "hypothesis_testing".to_string(),
            ]
        );
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("Course Mode")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("course materials")));
        assert_scientific_discipline_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_discipline_registry_returns_unknown_concept_for_unmapped_topic() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_discipline_registry(
            temp.path(),
            scientific_discipline_registry_request("Signal graph theory", None, None),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificDisciplineRegistryStatus::UnknownConcept);
        assert_eq!(result.normalized_topic, "Signal graph theory");
        assert!(result.recognized_concept.is_none());
        assert!(result.discipline_path.is_empty());
        assert_eq!(result.planned_queries, vec!["Signal graph theory".to_string()]);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("not yet in the local preview registry")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add a discipline registry mapping")));
        assert_eq!(result.planned_queries, vec!["Signal graph theory".to_string()]);
        assert_scientific_discipline_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_discipline_registry_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_scientific_discipline_registry(
            temp.path(),
            scientific_discipline_registry_request("  ANOVA  ", Some("scientific paper"), Some("Linear models")),
        )
        .unwrap();
        let second = preview_scholar_chat_scientific_discipline_registry(
            temp.path(),
            scientific_discipline_registry_request("  ANOVA  ", Some("scientific paper"), Some("Linear models")),
        )
        .unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_scientific_discipline_registry_boundary_fields(preview);
        }
    }

    #[test]
    fn scholar_chat_scientific_discipline_registry_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_scientific_discipline_registry")
            .unwrap();
        let end = source[start..]
            .find("pub fn preview_scholar_chat_answer_readiness")
            .unwrap();
        let body = &source[start..start + end];
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("reqwest"));
        assert!(!body.contains("ureq"));
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    #[test]
    fn scholar_chat_scientific_source_registry_blocks_when_topic_is_blank() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request("   ", Some("course"), Some("Module handbook"), Some(vec!["psychology"])),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificSourceRegistryStatus::Blocked);
        assert_eq!(result.discipline_status, ScholarChatScientificDisciplineRegistryStatus::Blocked);
        assert!(result.recognized_concept.is_none());
        assert!(result.source_families.is_empty());
        assert!(result.preferred_source_ids.is_empty());
        assert!(result.conditional_source_ids.is_empty());
        assert!(result.planned_metadata_queries.is_empty());
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.contains("topic_missing")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Provide a scientific topic")));
        assert_scientific_source_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_source_registry_maps_signalentdeckung() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request("Signalentdeckung", None, None, None),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificSourceRegistryStatus::SourcePlanReady);
        assert_eq!(result.discipline_status, ScholarChatScientificDisciplineRegistryStatus::ConceptMapped);
        assert_eq!(result.normalized_mode, "scholar_chat");
        assert_eq!(result.recognized_concept.as_deref(), Some("signal_detection_theory"));
        assert_eq!(
            result.preferred_source_ids,
            vec![
                "pubpsych".to_string(),
                "psycharchives".to_string(),
                "openalex".to_string(),
                "crossref".to_string(),
            ]
        );
        assert_eq!(result.conditional_source_ids, vec!["pubmed_if_biomedical_context".to_string()]);
        assert_eq!(result.excluded_source_ids, vec!["pubmed_if_biomedical_context".to_string()]);
        assert_eq!(result.planned_metadata_queries[0], "Signalentdeckungstheorie");
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "pubpsych" && family.active_for_current_context));
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "psycharchives" && family.active_for_current_context));
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "openalex" && family.active_for_current_context));
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "crossref" && family.active_for_current_context));
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "pubmed_if_biomedical_context" && !family.active_for_current_context));
        assert_eq!(result.access_classes, vec![ScholarChatScientificSourceAccessClass::OpenMetadata]);
        assert_eq!(result.source_plan.source_family_count, 5);
        assert_eq!(result.source_plan.active_source_family_count, 4);
        assert_eq!(result.source_plan.conditional_source_family_count, 1);
        assert!(result
            .ranking_hints
            .iter()
            .any(|hint| hint.contains("psychology and psychophysics")));
        assert!(result
            .deduplication_hints
            .iter()
            .any(|hint| hint.contains("DOI, title, and source identifiers")));
        assert_scientific_source_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_source_registry_activates_pubmed_for_biomedical_context() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request(
                "Signalentdeckung",
                None,
                None,
                Some(vec!["BiomEdical", " neuroscience ", "medical"]),
            ),
        )
        .unwrap();
        let pubmed = result
            .source_families
            .iter()
            .find(|family| family.id == "pubmed_if_biomedical_context")
            .unwrap();
        assert!(pubmed.active_for_current_context);
        assert_eq!(result.excluded_source_ids, Vec::<String>::new());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("biomedical")));
        assert_scientific_source_registry_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_scientific_source_registry_maps_anova_and_activates_theory_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request(
                "ANOVA",
                Some("scientific_paper"),
                Some("Psychology statistics seminar"),
                None,
            ),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificSourceRegistryStatus::SourcePlanReady);
        assert_eq!(result.discipline_status, ScholarChatScientificDisciplineRegistryStatus::ConceptMapped);
        assert_eq!(
            result.preferred_source_ids,
            vec!["openalex".to_string(), "crossref".to_string()]
        );
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "pubpsych_if_psychology_context" && family.active_for_current_context));
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "psycharchives_if_psychology_context" && family.active_for_current_context));
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "zbmath_if_theory_context" && family.active_for_current_context));
        assert!(result
            .source_families
            .iter()
            .any(|family| family.id == "arxiv_if_theory_context" && family.active_for_current_context));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("literature search planning")));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("Science Paper Mode"))
            || result
                .warnings
                .iter()
                .any(|warning| warning.contains("Scientific Paper Mode")));
        assert_eq!(result.source_plan.conditional_source_family_count, 4);
        assert_eq!(result.source_plan.active_source_family_count, 6);
        assert_scientific_source_registry_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_scientific_source_registry_activates_zbmath_and_arxiv_for_theory_context() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request(
                "ANOVA",
                Some("course"),
                Some("Linear models"),
                Some(vec![" theory ", "statistics", "statistics", "math"]),
            ),
        )
        .unwrap();
        let zbmath = result
            .source_families
            .iter()
            .find(|family| family.id == "zbmath_if_theory_context")
            .unwrap();
        let arxiv = result
            .source_families
            .iter()
            .find(|family| family.id == "arxiv_if_theory_context")
            .unwrap();
        assert!(zbmath.active_for_current_context);
        assert!(arxiv.active_for_current_context);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("Course Mode")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("local course materials")));
        assert_scientific_source_registry_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_scientific_source_registry_maps_hypothesentests() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request("Hypothesentests", None, None, None),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificSourceRegistryStatus::SourcePlanReady);
        assert_eq!(result.discipline_status, ScholarChatScientificDisciplineRegistryStatus::ConceptMapped);
        assert_eq!(
            result.preferred_source_ids,
            vec![
                "openalex".to_string(),
                "crossref".to_string(),
                "zbmath".to_string(),
                "arxiv".to_string(),
            ]
        );
        assert_eq!(
            result.conditional_source_ids,
            vec![
                "pubpsych_if_psychology_context".to_string(),
                "psycharchives_if_psychology_context".to_string(),
            ]
        );
        assert_eq!(
            result.excluded_source_ids,
            vec![
                "pubpsych_if_psychology_context".to_string(),
                "psycharchives_if_psychology_context".to_string(),
            ]
        );
        assert_eq!(result.planned_metadata_queries[0], "Hypothesentests");
        assert_eq!(result.access_classes, vec![ScholarChatScientificSourceAccessClass::OpenMetadata]);
        assert_scientific_source_registry_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_scientific_source_registry_returns_unknown_concept_for_unmapped_topic() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request("Signal graph theory", Some("scholar_chat"), None, Some(vec!["unknown"])),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificSourceRegistryStatus::UnknownConcept);
        assert_eq!(result.discipline_status, ScholarChatScientificDisciplineRegistryStatus::UnknownConcept);
        assert!(result.recognized_concept.is_none());
        assert!(result.source_families.is_empty());
        assert_eq!(result.planned_metadata_queries, vec!["Signal graph theory".to_string()]);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("not yet mapped through the discipline registry")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("adding discipline mapping")));
        assert_scientific_source_registry_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_scientific_source_registry_normalizes_context_tags() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request(
                "ANOVA",
                None,
                None,
                Some(vec!["  Psychology  ", "theory", "psychology", "statistics", "theory-methods", ""]),
            ),
        )
        .unwrap();
        assert_eq!(
            result.normalized_context_tags,
            vec![
                "psychology".to_string(),
                "statistics".to_string(),
                "theory".to_string(),
                "theory_methods".to_string(),
            ]
        );
        assert_scientific_source_registry_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_scientific_source_registry_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request(
                "  ANOVA  ",
                Some("scientific paper"),
                Some("Psychology statistics seminar"),
                Some(vec!["statistics", "psychology", "theory"]),
            ),
        )
        .unwrap();
        let second = preview_scholar_chat_scientific_source_registry(
            temp.path(),
            scientific_source_registry_request(
                "  ANOVA  ",
                Some("scientific paper"),
                Some("Psychology statistics seminar"),
                Some(vec!["statistics", "psychology", "theory"]),
            ),
        )
        .unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_scientific_source_registry_boundary_fields(preview);
        }
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_blocks_blank_query_and_keeps_boundary_fields() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_scientific_query_understanding(
            temp.path(),
            scientific_query_understanding_request("   ", None, None, None),
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatScientificQueryUnderstandingStatus::Blocked);
        assert!(result.normalized_query.is_empty());
        assert!(result.inferred_topic.is_none());
        assert!(result.recognized_concept.is_none());
        assert!(result.label.is_none());
        assert!(result.detected_aliases.is_empty());
        assert!(result.planned_metadata_queries.is_empty());
        assert!(result.planned_local_search_queries.is_empty());
        assert!(result.planned_expanded_queries.is_empty());
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.contains("query_missing")));
        assert_scientific_query_understanding_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(count_entries_recursively(temp.path()), 0);
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_maps_signalentdeckung_and_includes_registry_ids() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("Signalentdeckung", None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificQueryUnderstandingStatus::Understood);
        assert_eq!(result.inferred_topic.as_deref(), Some("Signalentdeckung"));
        assert_eq!(result.recognized_concept.as_deref(), Some("signal_detection_theory"));
        assert_eq!(result.label.as_deref(), Some("Signalentdeckungstheorie"));
        assert_eq!(result.source_registry_status, ScholarChatScientificSourceRegistryStatus::SourcePlanReady);
        assert!(result.preferred_source_ids.iter().any(|value| value == "pubpsych"));
        assert!(result.preferred_source_ids.iter().any(|value| value == "psycharchives"));
        assert!(result.preferred_source_ids.iter().any(|value| value == "openalex"));
        assert!(result.preferred_source_ids.iter().any(|value| value == "crossref"));
        assert!(result
            .planned_metadata_queries
            .iter()
            .any(|value| value == "Signalentdeckungstheorie"));
        assert!(result
            .planned_local_search_queries
            .iter()
            .any(|value| value == "Signalentdeckung"));
        assert!(result
            .detected_aliases
            .iter()
            .any(|alias| alias.alias == "Signalentdeckung" && alias.language == "german"));
        assert!(result.language_hints.iter().any(|hint| hint == "german"));
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_detects_english_signal_detection_alias_and_bridge_queries() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("Explain signal detection theory and d-prime", None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificQueryUnderstandingStatus::Understood);
        assert_eq!(result.query_intent, ScholarChatScientificQueryIntent::ConceptExplanation);
        assert!(result
            .detected_aliases
            .iter()
            .any(|alias| alias.alias == "signal detection theory" && alias.language == "english"));
        assert!(result.language_hints.iter().any(|hint| hint == "english"));
        assert!(result
            .planned_expanded_queries
            .iter()
            .any(|value| value == "Signal Detection Theory"));
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_anova_literature_query_prefers_literature_search_and_evidence_requirements() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("ANOVA literature review", Some("scholar_chat"), None, None),
        );
        assert_eq!(result.status, ScholarChatScientificQueryUnderstandingStatus::Understood);
        assert_eq!(result.query_intent, ScholarChatScientificQueryIntent::LiteratureSearch);
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "citation_safe_metadata_required"));
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "deduplication_required_before_literature_review"));
        assert!(result
            .planned_metadata_queries
            .iter()
            .any(|value| value == "ANOVA"));
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_hypothesentests_course_query_prefers_course_learning_and_course_requirements() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("Hypothesentests für die Klausur", Some("course"), None, None),
        );
        assert_eq!(result.status, ScholarChatScientificQueryUnderstandingStatus::Understood);
        assert_eq!(result.query_intent, ScholarChatScientificQueryIntent::CourseLearning);
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "course_material_alignment_required"));
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "local_evidence_required_before_answer"));
        assert!(result.no_local_file_indexing);
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_marks_mixed_concepts_ambiguous_and_prefers_first_occurrence() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("Hypothesentests und ANOVA Vergleich", None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificQueryUnderstandingStatus::Ambiguous);
        assert_eq!(result.ambiguity_level, ScholarChatScientificAmbiguityLevel::Medium);
        assert_eq!(result.inferred_topic.as_deref(), Some("Hypothesentests"));
        assert!(result
            .ambiguity_warnings
            .iter()
            .any(|warning| warning.contains("Hypothesentests")));
        assert!(result
            .ambiguity_warnings
            .iter()
            .any(|warning| warning.contains("ANOVA")));
        assert_eq!(result.detected_aliases.first().map(|alias| alias.alias.as_str()), Some("Hypothesentest"));
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_returns_unknown_concept_with_normalized_query_and_local_search() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("  Signal graph theory  ", None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificQueryUnderstandingStatus::UnknownConcept);
        assert_eq!(result.inferred_topic.as_deref(), Some("Signal graph theory"));
        assert!(result.recognized_concept.is_none());
        assert!(result.label.is_none());
        assert_eq!(result.planned_metadata_queries, vec!["Signal graph theory".to_string()]);
        assert_eq!(result.planned_local_search_queries, vec!["Signal graph theory".to_string()]);
        assert!(result.planned_expanded_queries.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("does not yet map to a known scientific concept")));
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_normalizes_context_tags_and_passes_them_through() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request(
                "ANOVA",
                None,
                Some("Psychology statistics seminar"),
                Some(vec!["  neuroscience  ", "Psychology", "neuroscience", "clinical-science"]),
            ),
        );
        assert_eq!(
            result.normalized_context_tags,
            vec![
                "clinical_science".to_string(),
                "neuroscience".to_string(),
                "psychology".to_string(),
            ]
        );
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_scientific_paper_mode_forces_literature_search() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("x", Some("scientific_paper"), None, None),
        );
        assert_eq!(result.query_intent, ScholarChatScientificQueryIntent::LiteratureSearch);
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "citation_safe_metadata_required"));
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "deduplication_required_before_literature_review"));
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_course_mode_forces_course_learning_and_no_local_file_indexing() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_query_understanding_deterministic_and_path_free(
            &temp,
            scientific_query_understanding_request("x", Some("course"), None, None),
        );
        assert_eq!(result.query_intent, ScholarChatScientificQueryIntent::CourseLearning);
        assert!(result.no_local_file_indexing);
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "course_material_alignment_required"));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_blocks_blank_query_and_keeps_boundary_fields() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request("   ", None, None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificSearchPlanStatus::Blocked);
        assert_eq!(result.search_strategy, ScholarChatScientificSearchStrategy::Blocked);
        assert_eq!(result.query_understanding_status, ScholarChatScientificQueryUnderstandingStatus::Blocked);
        assert!(result.normalized_query.is_empty());
        assert!(result.inferred_topic.is_none());
        assert!(result.recognized_concept.is_none());
        assert!(result.label.is_none());
        assert!(result.selected_local_source_ids.is_empty());
        assert!(result.planned_local_queries.is_empty());
        assert!(result.planned_metadata_queries.is_empty());
        assert!(result.planned_expanded_queries.is_empty());
        assert!(result.blockers.iter().any(|blocker| blocker.contains("query_missing")));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_maps_signalentdeckung_to_local_first_and_includes_source_routing() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request("Signalentdeckung", None, None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificSearchPlanStatus::SearchPlanReady);
        assert_eq!(result.search_strategy, ScholarChatScientificSearchStrategy::LocalFirst);
        assert_eq!(result.query_understanding_status, ScholarChatScientificQueryUnderstandingStatus::Understood);
        assert_eq!(result.inferred_topic.as_deref(), Some("Signalentdeckung"));
        assert_eq!(result.local_search_plan.local_source_count, 0);
        assert!(result.local_search_plan.local_first);
        assert!(result
            .source_routing_plan
            .active_routes
            .iter()
            .any(|value| value == "pubpsych"));
        assert!(result
            .source_routing_plan
            .active_routes
            .iter()
            .any(|value| value == "psycharchives"));
        assert!(result
            .source_routing_plan
            .active_routes
            .iter()
            .any(|value| value == "openalex"));
        assert!(result
            .source_routing_plan
            .active_routes
            .iter()
            .any(|value| value == "crossref"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("No local sources selected")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select or import local sources")));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_normalizes_selected_local_source_ids_without_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request(
                "Signalentdeckung",
                None,
                None,
                None,
                Some(vec!["  local-b  ", "local-a", "local-b", ""]),
            ),
        );
        assert_eq!(
            result.selected_local_source_ids,
            vec!["local-a".to_string(), "local-b".to_string()]
        );
        assert_eq!(result.local_search_plan.local_source_count, 2);
        assert!(result
            .local_search_plan
            .selected_local_source_ids
            .iter()
            .all(|value| value == "local-a" || value == "local-b"));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_uses_metadata_first_for_scientific_paper_mode() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request("ANOVA", Some("scientific_paper"), None, None, None),
        );
        assert_eq!(result.search_strategy, ScholarChatScientificSearchStrategy::MetadataFirst);
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "citation_safe_metadata_required"));
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "deduplication_required_before_literature_review"));
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "ranking_required_before_answer"));
        assert!(!result.metadata_search_plan.will_call_connectors);
        assert!(!result.metadata_search_plan.will_make_web_requests);
    }

    #[test]
    fn scholar_chat_scientific_search_plan_uses_course_local_first_and_course_alignment() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request("Hypothesentests", Some("course"), Some("Module 123"), None, None),
        );
        assert_eq!(result.search_strategy, ScholarChatScientificSearchStrategy::CourseLocalFirst);
        assert!(result.local_search_plan.local_first);
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "course_material_alignment_required"));
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "local_evidence_required_before_answer"));
        assert!(!result.local_search_plan.will_read_files);
        assert!(!result.local_search_plan.will_build_index);
    }

    #[test]
    fn scholar_chat_scientific_search_plan_marks_mixed_concepts_needs_disambiguation_and_keeps_first_concept() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request("ANOVA und Hypothesentests Vergleich", None, None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificSearchPlanStatus::NeedsDisambiguation);
        assert_eq!(result.query_understanding_status, ScholarChatScientificQueryUnderstandingStatus::Ambiguous);
        assert!(result.inferred_topic.is_some());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("multiple scientific concepts")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Narrow the scientific concept")));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_returns_unknown_concept_with_search_requirements() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request("Signal graph theory", None, None, None, None),
        );
        assert_eq!(result.status, ScholarChatScientificSearchPlanStatus::UnknownConcept);
        assert_eq!(result.query_understanding_status, ScholarChatScientificQueryUnderstandingStatus::UnknownConcept);
        assert_eq!(result.normalized_query, "Signal graph theory");
        assert_eq!(result.planned_local_queries, vec!["Signal graph theory".to_string()]);
        assert_eq!(result.planned_metadata_queries, vec!["Signal graph theory".to_string()]);
        assert!(result
            .evidence_requirements
            .iter()
            .any(|value| value == "source_family_plan_required"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add discipline and source registry mappings")));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_emits_stable_step_order_and_preview_only_constraints() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_scientific_search_plan_deterministic_and_path_free(
            &temp,
            scientific_search_plan_request(
                "Signalentdeckung",
                None,
                None,
                None,
                Some(vec!["local-b", "local-a"]),
            ),
        );
        let kinds = result
            .planned_search_steps
            .iter()
            .map(|step| step.kind.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                ScholarChatScientificSearchPlanStepKind::QueryExpansion,
                ScholarChatScientificSearchPlanStepKind::LocalSourceSearch,
                ScholarChatScientificSearchPlanStepKind::SourceFamilyRouting,
                ScholarChatScientificSearchPlanStepKind::MetadataSourceSearch,
                ScholarChatScientificSearchPlanStepKind::RankingPlan,
                ScholarChatScientificSearchPlanStepKind::DeduplicationPlan,
                ScholarChatScientificSearchPlanStepKind::EvidenceRequirementCheck,
            ]
        );
        assert!(result
            .planned_search_steps
            .iter()
            .all(|step| step.preview_only && step.boundary_notes.iter().any(|note| note == "preview-only")));
        assert!(!result.metadata_search_plan.will_call_connectors);
        assert!(!result.metadata_search_plan.will_make_web_requests);
        assert!(!result.local_search_plan.will_read_files);
        assert!(!result.local_search_plan.will_build_index);
    }

    #[test]
    fn scholar_chat_local_literature_index_blocks_blank_query_and_keeps_boundary_fields() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request("   ", None, None, None, None, None),
        );
        assert_eq!(result.status, ScholarChatLocalLiteratureIndexStatus::Blocked);
        assert_eq!(result.local_index_strategy, ScholarChatLocalLiteratureIndexStrategy::Blocked);
        assert_eq!(result.search_plan_status, ScholarChatScientificSearchPlanStatus::Blocked);
        assert_eq!(result.query_understanding_status, ScholarChatScientificQueryUnderstandingStatus::Blocked);
        assert!(result.normalized_query.is_empty());
        assert!(result.selected_local_source_ids.is_empty());
        assert!(result.expected_source_kinds.is_empty());
        assert!(result.blockers.iter().any(|blocker| blocker.contains("query_missing")));
    }

    #[test]
    fn scholar_chat_local_literature_index_maps_signalentdeckung_to_scholar_chat_local_first() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request(
                "Signalentdeckung",
                None,
                None,
                None,
                Some(vec!["  source-b  ", "source-a", "source-b"]),
                Some(vec![" pdf ", "markdown"]),
            ),
        );
        assert_eq!(result.status, ScholarChatLocalLiteratureIndexStatus::IndexPlanReady);
        assert_eq!(result.local_index_strategy, ScholarChatLocalLiteratureIndexStrategy::ScholarChatLocalFirst);
        assert_eq!(result.search_plan_status, ScholarChatScientificSearchPlanStatus::SearchPlanReady);
        assert_eq!(
            result.selected_local_source_ids,
            vec!["source-a".to_string(), "source-b".to_string()]
        );
        assert!(result
            .local_corpus_plan
            .selected_local_source_ids
            .iter()
            .any(|value| value == "source-a"));
        assert_eq!(
            result.expected_source_kinds,
            vec!["markdown".to_string(), "pdf".to_string()]
        );
        assert!(result
            .planned_index_steps
            .iter()
            .all(|step| step.active));
    }

    #[test]
    fn scholar_chat_local_literature_index_reports_needs_local_sources_without_reading_files() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request("Signalentdeckung", None, None, None, None, None),
        );
        assert_eq!(result.status, ScholarChatLocalLiteratureIndexStatus::NeedsLocalSources);
        assert_eq!(result.ingestion_readiness, ScholarChatLocalLiteratureIngestionReadiness::NeedsSources);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("No local sources selected")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select or import local sources")));
        assert!(!result.local_corpus_plan.corpus_manifest_would_be_required);
        assert!(!result.local_corpus_plan.will_create_corpus);
        assert!(!result.local_corpus_plan.will_read_files);
    }

    #[test]
    fn scholar_chat_local_literature_index_normalizes_expected_source_kinds_and_warns_on_unknown_kind() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request(
                "Signalentdeckung",
                None,
                None,
                None,
                Some(vec!["source-a"]),
                Some(vec!["  book-chapter  ", "notes", "alien-kind", "book_chapter", ""]),
            ),
        );
        assert_eq!(
            result.expected_source_kinds,
            vec![
                "alien_kind".to_string(),
                "book_chapter".to_string(),
                "notes".to_string(),
            ]
        );
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("Unknown expected source kinds")));
    }

    #[test]
    fn scholar_chat_local_literature_index_uses_scientific_paper_mode_for_citation_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request(
                "ANOVA",
                Some("scientific_paper"),
                None,
                None,
                Some(vec!["source-a"]),
                Some(vec!["article"]),
            ),
        );
        assert_eq!(
            result.local_index_strategy,
            ScholarChatLocalLiteratureIndexStrategy::ScientificPaperCitationLocalFirst
        );
        assert!(result
            .planned_metadata_requirements
            .iter()
            .any(|value| value == "citation_metadata_required_for_scientific_paper"));
        assert!(result
            .planned_metadata_requirements
            .iter()
            .any(|value| value == "doi_or_stable_locator_preferred"));
    }

    #[test]
    fn scholar_chat_local_literature_index_uses_course_mode_for_course_context_requirements() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request(
                "Hypothesentests",
                Some("course"),
                Some("Module 123"),
                Some(vec![" lecture notes ", "course-work"]),
                Some(vec!["source-a"]),
                Some(vec!["course_material", "lecture-slide"]),
            ),
        );
        assert_eq!(
            result.local_index_strategy,
            ScholarChatLocalLiteratureIndexStrategy::CourseMaterialLocalFirst
        );
        assert!(result
            .planned_metadata_requirements
            .iter()
            .any(|value| value == "course_context_required_for_course_mode"));
        assert_eq!(
            result.planned_chunking_policy,
            vec![
                "chunking_not_run_in_preview".to_string(),
                "later_pdf_text_extraction_required_before_chunking".to_string(),
                "later_markdown_section_chunking_candidate".to_string(),
                "later_course_slide_section_chunking_candidate".to_string(),
                "preserve_page_or_section_locator_later".to_string(),
                "keep_citation_metadata_attached_to_chunks_later".to_string(),
            ]
        );
    }

    #[test]
    fn scholar_chat_local_literature_index_marks_ambiguous_and_unknown_concept_statuses() {
        let temp = tempfile::tempdir().unwrap();
        let ambiguous = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request(
                "ANOVA und Hypothesentests Vergleich",
                None,
                None,
                None,
                Some(vec!["source-a"]),
                None,
            ),
        );
        assert_eq!(
            ambiguous.status,
            ScholarChatLocalLiteratureIndexStatus::NeedsDisambiguation
        );
        assert_eq!(
            ambiguous.ingestion_readiness,
            ScholarChatLocalLiteratureIngestionReadiness::NeedsDisambiguation
        );

        let unknown = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request(
                "Signal graph theory",
                None,
                None,
                None,
                Some(vec!["source-a"]),
                None,
            ),
        );
        assert_eq!(unknown.status, ScholarChatLocalLiteratureIndexStatus::UnknownConcept);
        assert_eq!(
            unknown.ingestion_readiness,
            ScholarChatLocalLiteratureIngestionReadiness::UnknownConceptMappingNeeded
        );
    }

    #[test]
    fn scholar_chat_local_literature_index_emits_stable_step_order_and_preview_only_constraints() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_local_literature_index_deterministic_and_path_free(
            &temp,
            scientific_local_literature_index_request(
                "Signalentdeckung",
                None,
                None,
                None,
                Some(vec!["source-b", "source-a"]),
                Some(vec!["pdf"]),
            ),
        );
        let kinds = result
            .planned_index_steps
            .iter()
            .map(|step| step.kind.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                ScholarChatLocalLiteratureIndexStepKind::SourceSelectionReview,
                ScholarChatLocalLiteratureIndexStepKind::MetadataRequirementCheck,
                ScholarChatLocalLiteratureIndexStepKind::CorpusManifestPlan,
                ScholarChatLocalLiteratureIndexStepKind::ExtractionPlan,
                ScholarChatLocalLiteratureIndexStepKind::ChunkingPolicyPlan,
                ScholarChatLocalLiteratureIndexStepKind::LexicalIndexPlan,
                ScholarChatLocalLiteratureIndexStepKind::VectorIndexPlan,
                ScholarChatLocalLiteratureIndexStepKind::DeduplicationPlan,
                ScholarChatLocalLiteratureIndexStepKind::RetrievalReadinessCheck,
            ]
        );
        assert!(result
            .planned_index_steps
            .iter()
            .all(|step| step.preview_only && step.boundary_notes.iter().any(|note| note == "preview-only")));
    }

    #[test]
    fn scholar_chat_local_literature_index_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_local_literature_index")
            .unwrap();
        let end = source[start..]
            .find("pub fn preview_scholar_chat_answer_readiness")
            .unwrap();
        let body = &source[start..start + end];
        assert_eq!(body.matches("preview_scholar_chat_scientific_search_plan").count(), 1);
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("reqwest::"));
        assert!(!body.contains("ureq::"));
        assert!(!body.contains("std::fs"));
        assert!(!body.contains("fs::"));
        assert!(!body.contains("CorpusAuthority::"));
        assert!(!body.contains("SourceRegistry::"));
        assert!(!body.contains("RetrievalService::new"));
        assert!(!body.contains("extract_source"));
        assert!(!body.contains("chunk_source"));
        assert!(!body.contains("build_retrieval_index"));
        assert!(!body.contains("preview_scholar_chat_retrieval"));
        assert!(!body.contains("preview_scholar_chat_evidence_plan"));
        assert!(!body.contains("preview_scholar_chat_prompt_pack"));
        assert!(!body.contains("preview_scholar_chat_answer_readiness"));
        assert!(!body.contains("preview_scholar_chat_draft_inference"));
        assert!(!body.contains("preview_scholar_chat_grounded_answer"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    #[test]
    fn scholar_chat_course_literature_registry_blocks_blank_query_and_keeps_boundary_fields() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_course_literature_registry_deterministic_and_path_free(
            &temp,
            course_literature_registry_request(
                "   ",
                Some("Psychology course"),
                Some("PSY-201"),
                Some("Signalentdeckung"),
                Some("Prof. Example"),
                Some("WS 2025"),
                None,
                None,
                None,
            ),
        );
        assert_eq!(
            result.status,
            ScholarChatCourseLiteratureRegistryStatus::Blocked
        );
        assert_eq!(
            result.course_registry_strategy,
            ScholarChatCourseLiteratureRegistryStrategy::Blocked
        );
        assert!(result.normalized_query.is_empty());
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.contains("query_missing")));
    }

    #[test]
    fn scholar_chat_course_literature_registry_maps_signalentdeckung_to_course_registry_plan_ready() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_course_literature_registry_deterministic_and_path_free(
            &temp,
            course_literature_registry_request(
                "Signalentdeckung",
                Some("  Psychology   course  "),
                Some(" PSY-201 "),
                Some(" Signalentdeckung "),
                Some("  Prof. Example  "),
                Some(" WS 2025 "),
                Some(vec![" lecture-material ", "psychology", "lecture-material"]),
                Some(vec!["  source-b  ", "source-a", "source-b"]),
                Some(vec![" lecture-slide ", "seminar-reading", "unknown-kind", "module handbook", "lecture_slide"]),
            ),
        );
        assert_eq!(
            result.status,
            ScholarChatCourseLiteratureRegistryStatus::CourseRegistryPlanReady
        );
        assert_eq!(
            result.course_registry_strategy,
            ScholarChatCourseLiteratureRegistryStrategy::ModuleContextFirst
        );
        assert_eq!(
            result.local_literature_index_status,
            ScholarChatLocalLiteratureIndexStatus::IndexPlanReady
        );
        assert_eq!(
            result.search_plan_status,
            ScholarChatScientificSearchPlanStatus::SearchPlanReady
        );
        assert_eq!(
            result.query_understanding_status,
            ScholarChatScientificQueryUnderstandingStatus::Understood
        );
        assert_eq!(result.normalized_query, "Signalentdeckung");
        assert_eq!(result.normalized_course_context.as_deref(), Some("Psychology course"));
        assert_eq!(result.normalized_module_code.as_deref(), Some("PSY-201"));
        assert_eq!(result.normalized_course_title.as_deref(), Some("Signalentdeckung"));
        assert_eq!(result.normalized_instructor.as_deref(), Some("Prof. Example"));
        assert_eq!(result.normalized_semester.as_deref(), Some("WS 2025"));
        assert_eq!(
            result.selected_local_source_ids,
            vec!["source-a".to_string(), "source-b".to_string()]
        );
        assert_eq!(
            result.expected_course_material_kinds,
            vec![
                "lecture_slide".to_string(),
                "module_handbook".to_string(),
                "seminar_reading".to_string(),
                "unknown_kind".to_string(),
            ]
        );
        assert_eq!(
            result.course_identity.identity_key.as_deref(),
            Some("psy_201::signalentdeckung::ws_2025")
        );
        assert!(result.course_identity.has_course_context);
        assert!(result.course_identity.has_module_code);
        assert!(result.course_identity.has_course_title);
        assert_eq!(
            result.course_material_plan.known_material_kinds,
            vec![
                "lecture_slide".to_string(),
                "module_handbook".to_string(),
                "seminar_reading".to_string(),
            ]
        );
        assert_eq!(
            result.course_material_plan.unknown_material_kinds,
            vec!["unknown_kind".to_string()]
        );
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("Unknown course material kinds")));
        assert!(!result.course_material_plan.will_read_files);
        assert!(!result.course_material_plan.will_import_sources);
        assert!(!result.course_material_plan.will_create_registry);
        assert!(!result.curriculum_alignment_plan.requires_course_context);
        assert!(!result.curriculum_alignment_plan.requires_module_metadata);
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "module_code_recommended"));
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "local_source_ids_required_for_course_material_alignment"));
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "no_curriculum_scraping_in_preview"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "Signalentdeckung course materials"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "Signalentdeckung lecture notes"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "PSY-201 Signalentdeckung"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "Signalentdeckung exam preparation"));
        assert_eq!(
            result
                .planned_registry_steps
                .iter()
                .map(|step| step.kind.clone())
                .collect::<Vec<_>>(),
            vec![
                ScholarChatCourseLiteratureRegistryStepKind::CourseIdentityReview,
                ScholarChatCourseLiteratureRegistryStepKind::ModuleContextReview,
                ScholarChatCourseLiteratureRegistryStepKind::CourseMaterialKindPlan,
                ScholarChatCourseLiteratureRegistryStepKind::LocalSourceAlignmentPlan,
                ScholarChatCourseLiteratureRegistryStepKind::CurriculumMetadataRequirementCheck,
                ScholarChatCourseLiteratureRegistryStepKind::LocalLiteratureIndexAlignment,
                ScholarChatCourseLiteratureRegistryStepKind::RetrievalReadinessCheck,
                ScholarChatCourseLiteratureRegistryStepKind::LearningPathAlignmentPlan,
            ]
        );
        assert!(result.summary.contains("ready later"));
    }

    #[test]
    fn scholar_chat_course_literature_registry_reports_needs_course_context_when_identity_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_course_literature_registry_deterministic_and_path_free(
            &temp,
            course_literature_registry_request(
                "Signalentdeckung",
                None,
                None,
                None,
                None,
                None,
                Some(vec!["psychology"]),
                Some(vec!["source-a"]),
                Some(vec!["lecture-slide"]),
            ),
        );
        assert_eq!(
            result.status,
            ScholarChatCourseLiteratureRegistryStatus::NeedsCourseContext
        );
        assert_eq!(
            result.course_registry_strategy,
            ScholarChatCourseLiteratureRegistryStrategy::LocalSourceAlignmentFirst
        );
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("No course context, module code, or course title")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Provide course context, module code, or course title")));
    }

    #[test]
    fn scholar_chat_course_literature_registry_reports_needs_local_sources_when_identity_exists() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_course_literature_registry_deterministic_and_path_free(
            &temp,
            course_literature_registry_request(
                "Hypothesentests",
                Some("Statistics course"),
                Some("STAT-101"),
                Some("Hypothesentests"),
                Some("Prof. Example"),
                Some("WS 2025"),
                Some(vec!["statistics"]),
                None,
                Some(vec!["exam-prep"]),
            ),
        );
        assert_eq!(
            result.status,
            ScholarChatCourseLiteratureRegistryStatus::NeedsLocalSources
        );
        assert_eq!(
            result.course_registry_strategy,
            ScholarChatCourseLiteratureRegistryStrategy::ModuleContextFirst
        );
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("No local course sources selected")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select or import local course material sources later")));
        assert_eq!(result.course_material_plan.selected_source_count, 0);
        assert_eq!(result.course_material_plan.selected_local_source_ids, Vec::<String>::new());
    }

    #[test]
    fn scholar_chat_course_literature_registry_marks_ambiguous_and_unknown_concepts() {
        let temp = tempfile::tempdir().unwrap();
        let ambiguous = assert_course_literature_registry_deterministic_and_path_free(
            &temp,
            course_literature_registry_request(
                "ANOVA und Hypothesentests Vergleich",
                Some("Statistics course"),
                Some("STAT-101"),
                Some("ANOVA"),
                Some("Prof. Example"),
                Some("WS 2025"),
                Some(vec!["statistics"]),
                Some(vec!["source-a"]),
                Some(vec!["module-handbook"]),
            ),
        );
        assert_eq!(
            ambiguous.status,
            ScholarChatCourseLiteratureRegistryStatus::NeedsDisambiguation
        );

        let unknown = assert_course_literature_registry_deterministic_and_path_free(
            &temp,
            course_literature_registry_request(
                "Signal graph theory",
                Some("Theory course"),
                Some("THE-201"),
                Some("Graph theory"),
                Some("Prof. Example"),
                Some("WS 2025"),
                Some(vec!["theory"]),
                Some(vec!["source-a"]),
                Some(vec!["notes"]),
            ),
        );
        assert_eq!(
            unknown.status,
            ScholarChatCourseLiteratureRegistryStatus::UnknownConcept
        );
    }

    #[test]
    fn scholar_chat_course_literature_registry_includes_course_metadata_requirements_and_planned_queries() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_course_literature_registry_deterministic_and_path_free(
            &temp,
            course_literature_registry_request(
                "ANOVA",
                Some("Statistics course"),
                Some("STAT-201"),
                Some("ANOVA seminar"),
                Some("Prof. Example"),
                Some("SS 2025"),
                Some(vec!["statistics"]),
                Some(vec!["source-a"]),
                Some(vec!["module-handbook", "article", "exam_prep"]),
            ),
        );
        assert_eq!(
            result.status,
            ScholarChatCourseLiteratureRegistryStatus::CourseRegistryPlanReady
        );
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "course_context_or_module_identity_required"));
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "module_code_recommended"));
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "course_title_recommended"));
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "semester_recommended"));
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "instructor_optional"));
        assert!(result
            .planned_course_metadata_requirements
            .iter()
            .any(|requirement| requirement == "learning_objectives_required_later"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "ANOVA course materials"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "ANOVA lecture notes"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "STAT-201 ANOVA"));
        assert!(result
            .planned_course_material_queries
            .iter()
            .any(|query| query == "ANOVA seminar ANOVA"));
    }

    #[test]
    fn scholar_chat_course_literature_registry_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_course_literature_registry")
            .unwrap();
        let end = source[start..]
            .find("pub fn preview_scholar_chat_answer_readiness")
            .unwrap();
        let body = &source[start..start + end];
        assert_eq!(body.matches("preview_scholar_chat_local_literature_index").count(), 1);
        assert!(body.contains("mode: Some(\"course\".to_string())"));
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("reqwest::"));
        assert!(!body.contains("ureq::"));
        assert!(!body.contains("std::fs"));
        assert!(!body.contains("fs::"));
        assert!(!body.contains("CorpusAuthority::"));
        assert!(!body.contains("SourceRegistry::"));
        assert!(!body.contains("RetrievalService::new"));
        assert!(!body.contains("extract_source"));
        assert!(!body.contains("chunk_source"));
        assert!(!body.contains("build_retrieval_index"));
        assert!(!body.contains("preview_scholar_chat_retrieval"));
        assert!(!body.contains("preview_scholar_chat_evidence_plan"));
        assert!(!body.contains("preview_scholar_chat_prompt_pack"));
        assert!(!body.contains("preview_scholar_chat_answer_readiness"));
        assert!(!body.contains("preview_scholar_chat_draft_inference"));
        assert!(!body.contains("preview_scholar_chat_grounded_answer"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_scientific_search_plan")
            .unwrap();
        let end = source[start..]
            .find("pub fn preview_scholar_chat_answer_readiness")
            .unwrap();
        let body = &source[start..start + end];
        assert_eq!(body.matches("preview_scholar_chat_scientific_query_understanding").count(), 1);
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("reqwest::"));
        assert!(!body.contains("ureq::"));
        assert!(!body.contains("std::fs"));
        assert!(!body.contains("fs::"));
        assert!(!body.contains("RetrievalService::new"));
        assert!(!body.contains("SourceRegistry::"));
        assert!(!body.contains("preview_scholar_chat_retrieval"));
        assert!(!body.contains("preview_scholar_chat_evidence_plan"));
        assert!(!body.contains("preview_scholar_chat_prompt_pack"));
        assert!(!body.contains("preview_scholar_chat_answer_readiness"));
        assert!(!body.contains("preview_scholar_chat_draft_inference"));
        assert!(!body.contains("preview_scholar_chat_grounded_answer"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    #[test]
    fn scholar_chat_scientific_search_plan_body_guard_helper_is_still_valid() {
        assert_scientific_search_plan_body_does_not_call_execution_functions();
    }

    fn assert_local_literature_index_boundary_fields(
        preview: &ScholarChatLocalLiteratureIndexPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.local_literature_index_preview_only);
        assert!(preview.no_file_read);
        assert!(preview.no_pdf_extraction);
        assert!(preview.no_ocr);
        assert!(preview.no_chunking_run);
        assert!(preview.no_embedding_generation);
        assert!(preview.no_index_created);
        assert!(preview.no_bm25_index);
        assert!(preview.no_vector_index);
        assert!(preview.no_retrieval_execution);
        assert!(preview.no_web_request);
        assert!(preview.no_scraping);
        assert!(preview.no_connector_call);
        assert!(preview.no_source_import);
        assert!(preview.no_local_file_indexing);
        assert!(preview.no_model_loading);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_evidence_pack_created);
        assert!(preview.no_artifact_write);
        assert!(preview.no_persistence);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_local_literature_index_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatLocalLiteratureIndexRequest,
    ) -> ScholarChatLocalLiteratureIndexPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_local_literature_index(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_local_literature_index(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_local_literature_index_boundary_fields(preview);
        }
        first
    }

    fn assert_course_literature_registry_boundary_fields(
        preview: &ScholarChatCourseLiteratureRegistryPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.course_literature_registry_preview_only);
        assert!(preview.no_file_read);
        assert!(preview.no_pdf_extraction);
        assert!(preview.no_ocr);
        assert!(preview.no_chunking_run);
        assert!(preview.no_embedding_generation);
        assert!(preview.no_index_created);
        assert!(preview.no_retrieval_execution);
        assert!(preview.no_web_request);
        assert!(preview.no_scraping);
        assert!(preview.no_connector_call);
        assert!(preview.no_source_import);
        assert!(preview.no_local_file_indexing);
        assert!(preview.no_model_loading);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_answer_generated);
        assert!(preview.no_evidence_pack_created);
        assert!(preview.no_artifact_write);
        assert!(preview.no_persistence);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_course_literature_registry_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatCourseLiteratureRegistryPreviewRequest,
    ) -> ScholarChatCourseLiteratureRegistryPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_course_literature_registry(temp.path(), request.clone())
            .unwrap();
        let second = preview_scholar_chat_course_literature_registry(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_course_literature_registry_boundary_fields(preview);
        }
        first
    }

    #[test]
    fn scholar_chat_scientific_query_understanding_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_scientific_query_understanding")
            .unwrap();
        let end = source[start..]
            .find("pub fn preview_scholar_chat_answer_readiness")
            .unwrap();
        let body = &source[start..start + end];
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("reqwest::"));
        assert!(!body.contains("ureq::"));
        assert!(!body.contains("std::fs"));
        assert!(!body.contains("fs::"));
        assert!(!body.contains("RetrievalService::new"));
        assert!(!body.contains("SourceRegistry::"));
        assert!(!body.contains("preview_scholar_chat_retrieval"));
        assert!(!body.contains("preview_scholar_chat_evidence_plan"));
        assert!(!body.contains("preview_scholar_chat_prompt_pack"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    #[test]
    fn scholar_chat_scientific_source_registry_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_scientific_source_registry")
            .unwrap();
        let end = source[start..]
            .find("pub fn preview_scholar_chat_answer_readiness")
            .unwrap();
        let body = &source[start..start + end];
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("reqwest::"));
        assert!(!body.contains("ureq::"));
        assert!(!body.contains("std::fs"));
        assert!(!body.contains("fs::"));
        assert!(!body.contains("RetrievalService::new"));
        assert!(!body.contains("SourceRegistry::"));
        assert!(!body.contains("preview_scholar_chat_retrieval"));
        assert!(!body.contains("preview_scholar_chat_evidence_plan"));
        assert!(!body.contains("preview_scholar_chat_prompt_pack"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_write_eligibility(
            temp.path(),
            grounding_request("   ", Some("Alpha beta."), vec![]),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request("alpha grounded evidence", Some("Alpha beta."), vec![]),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_write_eligibility(
                temp.path(),
                grounding_request("alpha grounded evidence", Some("Alpha beta."), vec![invalid.to_string()]),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request("alpha grounded evidence", Some("   "), vec![source_id]),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_blocks_without_evidence_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note-no-index.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();

        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request(
                "alpha grounded evidence",
                Some("Alpha beta."),
                vec![source.source_id.clone()],
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert_eq!(result.evidence_candidate_count, 0);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_needs_review_when_candidate_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(result.candidate_statement_count, 0);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));
        assert!(result
            .eligibility_reasons
            .iter()
            .any(|reason| reason.contains("Weakly supported or unsupported")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_is_ready_only_when_candidate_ready_later_and_statements_exist() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = grounding_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![source_id.clone()],
        );
        let candidate_preview = assert_grounded_answer_candidate_deterministic_and_path_free(&temp, request.clone());
        let write_eligibility_preview = assert_grounded_answer_write_eligibility_deterministic_and_path_free(&temp, request);
        assert_eq!(candidate_preview.status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(write_eligibility_preview.status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(write_eligibility_preview.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(write_eligibility_preview.candidate_statement_count > 0);
        assert_eq!(write_eligibility_preview.candidate_statement_count, candidate_preview.candidate_statement_count);
        assert_eq!(write_eligibility_preview.selected_source_count, candidate_preview.selected_source_count);
        assert_eq!(write_eligibility_preview.evidence_candidate_count, candidate_preview.evidence_candidate_count);
        assert_eq!(write_eligibility_preview.inspected_item_count, candidate_preview.inspected_item_count);
        assert_eq!(write_eligibility_preview.supported_item_count, candidate_preview.supported_item_count);
        assert_eq!(write_eligibility_preview.weakly_supported_item_count, candidate_preview.weakly_supported_item_count);
        assert_eq!(write_eligibility_preview.unsupported_item_count, candidate_preview.unsupported_item_count);
        assert!(write_eligibility_preview
            .eligibility_reasons
            .iter()
            .any(|reason| reason.contains("All inspected items were supported by local evidence")));
        assert!(write_eligibility_preview
            .next_required_actions
            .iter()
            .any(|action| action.contains("A GroundedAnswer write implementation can be added later")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_rejects_zero_candidate_statements_even_when_candidate_status_is_ready_later() {
        let candidate_preview = ScholarChatGroundedAnswerCandidatePreview {
            status: ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater,
            normalized_prompt: "alpha grounded evidence".to_string(),
            selected_source_count: 1,
            evidence_candidate_count: 1,
            inspected_item_count: 1,
            supported_item_count: 1,
            weakly_supported_item_count: 0,
            unsupported_item_count: 0,
            candidate_statement_count: 0,
            summary: "candidate preview".to_string(),
            candidate_items: Vec::new(),
            preview_only: true,
            not_answer_draft: true,
            not_grounded_answer: true,
            not_final_answer: true,
            no_answer_artifact_created: true,
            no_evidence_pack_built: true,
            no_persistence: true,
            no_llm_call: true,
            no_runtime_execution: true,
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_required_actions: Vec::new(),
        };
        let preview = grounded_answer_write_eligibility_preview_from_candidate_preview(candidate_preview);
        assert_eq!(preview.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(preview.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(preview.candidate_statement_count, 0);
        assert!(preview
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "candidate_statements_missing"));
        assert!(preview
            .eligibility_reasons
            .iter()
            .any(|reason| reason.contains("No candidate statements were available")));
        assert_grounded_answer_write_eligibility_boundary_fields(&preview);
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_bridge_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        for invalid in ["..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_runtime_diagnostic_bridge(
                temp.path(),
                runtime_diagnostic_bridge_request(
                    "Bridge preview prompt.",
                    vec![invalid.to_string()],
                    Some(helper.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    true,
                    true,
                    true,
                    Some("Diagnostic smoke prompt."),
                    Some(128),
                    Some(1_500),
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_bridge_blocks_when_prompt_is_blank() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_bridge_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_bridge_request(
                "   ",
                vec!["src_demo".to_string()],
                Some(helper.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                true,
                true,
                true,
                Some("Diagnostic smoke prompt."),
                Some(128),
                Some(1_500),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::Blocked);
        assert_eq!(preview.selected_source_count, 1);
        assert_eq!(preview.normalized_prompt, "");
        assert_eq!(preview.smoke_execution_plan_status, LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater);
        assert_eq!(preview.smoke_readiness_status, LocalRuntimeSmokeReadinessStatus::SmokeReadyLater);
        assert_eq!(preview.capability_status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert_eq!(preview.probe_readiness_status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater);
        assert_eq!(preview.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert_eq!(preview.adapter_contract_status, LocalRuntimeAdapterContractStatus::ContractReadyLater);
        assert_eq!(preview.adapter_kind, LocalRuntimeAdapterKind::LlamaCpp);
        assert_eq!(preview.version_probe_status, LocalRuntimeVersionProbeStatus::ProbeSucceeded);
        assert!(preview
            .runtime_diagnostic_reasons
            .iter()
            .any(|reason| reason.contains("Scholar Chat prompt is blank")));
        assert!(preview
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "scholar_chat_prompt_missing"));
        assert_eq!(fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt")).unwrap().trim(), "2");
        assert_eq!(
            fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt"))
                .unwrap(),
            ""
        );
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_bridge_blocks_when_no_sources_selected() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_bridge_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_bridge_request(
                "Bridge preview prompt.",
                vec![],
                Some(helper.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                true,
                true,
                true,
                Some("Diagnostic smoke prompt."),
                Some(128),
                Some(1_500),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::Blocked);
        assert_eq!(preview.selected_source_count, 0);
        assert!(preview
            .runtime_diagnostic_reasons
            .iter()
            .any(|reason| reason.contains("No Scholar Chat sources are selected")));
        assert!(preview
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "scholar_chat_sources_missing"));
        assert_eq!(preview.probe_readiness_status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater);
        assert_eq!(preview.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert_eq!(preview.adapter_contract_status, LocalRuntimeAdapterContractStatus::ContractReadyLater);
        assert_eq!(preview.adapter_kind, LocalRuntimeAdapterKind::LlamaCpp);
        assert_eq!(fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt")).unwrap().trim(), "2");
        assert_eq!(
            fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt"))
                .unwrap(),
            ""
        );
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_bridge_blocks_when_smoke_execution_plan_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_bridge_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_bridge_request(
                "Bridge preview prompt.",
                vec!["src_demo".to_string()],
                Some(helper.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                true,
                false,
                true,
                Some("Diagnostic smoke prompt."),
                Some(128),
                Some(1_500),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::Blocked);
        assert_eq!(preview.smoke_execution_plan_status, LocalRuntimeSmokeExecutionPlanStatus::Blocked);
        assert_eq!(preview.smoke_readiness_status, LocalRuntimeSmokeReadinessStatus::Blocked);
        assert_eq!(preview.capability_status, LocalRuntimeCapabilityStatus::Blocked);
        assert_eq!(preview.probe_readiness_status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater);
        assert_eq!(preview.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert_eq!(preview.adapter_contract_status, LocalRuntimeAdapterContractStatus::ContractReadyLater);
        assert_eq!(preview.adapter_kind, LocalRuntimeAdapterKind::LlamaCpp);
        assert_eq!(preview.version_probe_status, LocalRuntimeVersionProbeStatus::Blocked);
        assert!(preview
            .runtime_diagnostic_reasons
            .iter()
            .any(|reason| reason.contains("smoke execution plan is not ready later")));
        assert!(preview
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "probe_execution_not_allowed"));
        assert_eq!(fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt")).unwrap().trim(), "0");
        assert_eq!(
            fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt"))
                .unwrap(),
            ""
        );
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_bridge_needs_review_when_smoke_execution_plan_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_fail.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_bridge_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_bridge_request(
                "Bridge preview prompt.",
                vec!["src_demo".to_string()],
                Some(helper.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                true,
                true,
                true,
                Some("Diagnostic smoke prompt."),
                Some(128),
                Some(1_500),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::NeedsReview);
        assert_eq!(preview.smoke_execution_plan_status, LocalRuntimeSmokeExecutionPlanStatus::NeedsReview);
        assert_eq!(preview.smoke_readiness_status, LocalRuntimeSmokeReadinessStatus::NeedsReview);
        assert_eq!(preview.capability_status, LocalRuntimeCapabilityStatus::NeedsReview);
        assert_eq!(preview.probe_readiness_status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater);
        assert_eq!(preview.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert_eq!(preview.adapter_contract_status, LocalRuntimeAdapterContractStatus::ContractReadyLater);
        assert_eq!(preview.adapter_kind, LocalRuntimeAdapterKind::LlamaCpp);
        assert_eq!(preview.version_probe_status, LocalRuntimeVersionProbeStatus::ProbeFailed);
        assert!(preview
            .runtime_diagnostic_reasons
            .iter()
            .any(|reason| reason.contains("smoke execution plan still needs review")));
        assert_eq!(fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt")).unwrap().trim(), "2");
        assert_eq!(
            fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt"))
                .unwrap(),
            ""
        );
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_bridge_is_ready_later_when_prompt_sources_and_smoke_execution_plan_are_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let before_entries = count_entries_recursively(temp.path());
        let preview = preview_scholar_chat_runtime_diagnostic_bridge(
            temp.path(),
            runtime_diagnostic_bridge_request(
                "Bridge preview prompt.",
                vec!["src_demo".to_string()],
                Some(helper.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                true,
                true,
                true,
                Some("Diagnostic smoke prompt."),
                Some(128),
                Some(1_500),
            ),
        )
        .unwrap();
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticBridgeStatus::RuntimeDiagnosticReadyLater);
        assert_eq!(preview.smoke_execution_plan_status, LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater);
        assert_eq!(preview.smoke_readiness_status, LocalRuntimeSmokeReadinessStatus::SmokeReadyLater);
        assert_eq!(preview.capability_status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert_eq!(preview.probe_readiness_status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater);
        assert_eq!(preview.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert_eq!(preview.adapter_contract_status, LocalRuntimeAdapterContractStatus::ContractReadyLater);
        assert_eq!(preview.adapter_kind, LocalRuntimeAdapterKind::LlamaCpp);
        assert_eq!(preview.version_probe_status, LocalRuntimeVersionProbeStatus::ProbeSucceeded);
        assert!(preview
            .runtime_diagnostic_reasons
            .iter()
            .any(|reason| reason.contains("runtime diagnostic bridge is ready later")));
        assert_eq!(preview.normalized_prompt, "Bridge preview prompt.");
        assert_eq!(preview.selected_source_count, 1);
        assert_eq!(preview.normalized_model_format, "gguf");
        assert_eq!(preview.safe_model_file_name.as_deref(), Some("runtime-diagnostic-bridge-model.gguf"));
        assert_eq!(preview.safe_executable_file_name.as_deref(), Some("runtime_diagnostic_bridge_ready.exe"));
        assert_eq!(preview.diagnostic_prompt_char_count, "Diagnostic smoke prompt.".chars().count());
        assert_eq!(preview.max_output_tokens, 32);
        assert_eq!(preview.timeout_ms, 1_500);
        assert_eq!(fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt")).unwrap().trim(), "1");
        assert!(!temp.path().join(".aegis").exists());
        assert_runtime_diagnostic_bridge_boundary_fields(&preview);
        assert_eq!(before_entries, count_entries_recursively(temp.path()));
        assert_eq!(
            fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt"))
                .unwrap(),
            ""
        );
        let debug = format!("{preview:?}");
        let json = serde_json::to_string(&preview).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(model_path.to_string_lossy().as_ref()));
        assert!(!json.contains(model_path.to_string_lossy().as_ref()));
        assert!(!debug.contains(helper.to_string_lossy().as_ref()));
        assert!(!json.contains(helper.to_string_lossy().as_ref()));
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_blocks_when_bridge_preview_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let preview = assert_runtime_diagnostic_result_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_result_request(
                runtime_diagnostic_bridge_request(
                    "   ",
                    vec![],
                    None,
                    None,
                    false,
                    false,
                    false,
                    None,
                    None,
                    None,
                ),
                runtime_diagnostic_preview_like_bridge(
                    LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                    "stdout ok",
                    "stderr ok",
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticResultStatus::Blocked);
        assert_eq!(preview.bridge_status, ScholarChatRuntimeDiagnosticBridgeStatus::Blocked);
        assert!(preview
            .runtime_result_reasons
            .iter()
            .any(|reason| reason.contains("bridge preview is blocked")));
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_blocks_when_bridge_preview_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_fail.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_result_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_result_request(
                runtime_diagnostic_bridge_request(
                    "Bridge preview prompt.",
                    vec!["src_demo".to_string()],
                    Some(helper.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    true,
                    true,
                    true,
                    Some("Diagnostic smoke prompt."),
                    Some(128),
                    Some(1_500),
                ),
                runtime_diagnostic_preview_like_bridge(
                    LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                    "stdout ok",
                    "stderr ok",
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticResultStatus::NeedsReview);
        assert_eq!(preview.bridge_status, ScholarChatRuntimeDiagnosticBridgeStatus::NeedsReview);
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_blocks_when_smoke_diagnostic_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_result_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_result_request(
                runtime_diagnostic_bridge_request(
                    "Bridge preview prompt.",
                    vec!["src_demo".to_string()],
                    Some(helper.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    true,
                    true,
                    true,
                    Some("Diagnostic smoke prompt."),
                    Some(128),
                    Some(1_500),
                ),
                runtime_diagnostic_preview_like_bridge(
                    LocalRuntimeSmokeDiagnosticStatus::Blocked,
                    "",
                    "",
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticResultStatus::Blocked);
        assert_eq!(preview.smoke_diagnostic_status, LocalRuntimeSmokeDiagnosticStatus::Blocked);
        assert!(preview
            .runtime_result_reasons
            .iter()
            .any(|reason| reason.contains("smoke diagnostic preview is blocked")));
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_fails_when_smoke_diagnostic_fails() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_result_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_result_request(
                runtime_diagnostic_bridge_request(
                    "Bridge preview prompt.",
                    vec!["src_demo".to_string()],
                    Some(helper.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    true,
                    true,
                    true,
                    Some("Diagnostic smoke prompt."),
                    Some(128),
                    Some(1_500),
                ),
                runtime_diagnostic_preview_like_bridge(
                    LocalRuntimeSmokeDiagnosticStatus::SmokeFailed,
                    "stdout failed",
                    "stderr failed",
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticFailed);
        assert_eq!(preview.smoke_diagnostic_status, LocalRuntimeSmokeDiagnosticStatus::SmokeFailed);
        assert!(preview.runtime_result_reasons.iter().any(|reason| reason.contains("failed")));
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_fails_when_smoke_diagnostic_times_out() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_diagnostic_result_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_result_request(
                runtime_diagnostic_bridge_request(
                    "Bridge preview prompt.",
                    vec!["src_demo".to_string()],
                    Some(helper.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    true,
                    true,
                    true,
                    Some("Diagnostic smoke prompt."),
                    Some(128),
                    Some(1_500),
                ),
                runtime_diagnostic_preview_like_bridge(
                    LocalRuntimeSmokeDiagnosticStatus::TimedOut,
                    "stdout timeout",
                    "stderr timeout",
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticFailed);
        assert_eq!(preview.smoke_diagnostic_status, LocalRuntimeSmokeDiagnosticStatus::TimedOut);
        assert!(preview.runtime_result_reasons.iter().any(|reason| reason.contains("timed out")));
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_succeeds_later_when_bridge_and_diagnostic_match() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let before_entries = count_entries_recursively(temp.path());
        let preview = assert_runtime_diagnostic_result_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_result_request(
                runtime_diagnostic_bridge_request(
                    "Bridge preview prompt.",
                    vec!["src_demo".to_string()],
                    Some(helper.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    true,
                    true,
                    true,
                    Some("Diagnostic smoke prompt."),
                    Some(128),
                    Some(1_500),
                ),
                runtime_diagnostic_preview_like_bridge(
                    LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                    "stdout ok",
                    "stderr ok",
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticSucceededLater);
        assert_eq!(preview.bridge_status, ScholarChatRuntimeDiagnosticBridgeStatus::RuntimeDiagnosticReadyLater);
        assert_eq!(preview.smoke_diagnostic_status, LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded);
        assert_eq!(preview.normalized_model_format, "gguf");
        assert_eq!(preview.safe_model_file_name.as_deref(), Some("runtime-diagnostic-bridge-model.gguf"));
        assert_eq!(preview.safe_executable_file_name.as_deref(), Some("runtime_diagnostic_bridge_ready.exe"));
        assert_eq!(preview.stdout_preview, "stdout ok");
        assert_eq!(preview.stderr_preview, "stderr ok");
        assert_eq!(before_entries, count_entries_recursively(temp.path()));
        assert_eq!(fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_version_probe_count.txt")).unwrap().trim(), "2");
        assert_eq!(fs::read_to_string(temp.path().join("runtime_diagnostic_bridge_unexpected_call.txt")).unwrap(), "");
        assert!(preview.runtime_result_reasons.iter().any(|reason| reason.contains("succeeded")));
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_needs_review_on_metadata_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let mut diagnostic_preview = runtime_diagnostic_preview_like_bridge(
            LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
            "stdout ok",
            "stderr ok",
        );
        diagnostic_preview.normalized_model_format = "ggml".to_string();
        let preview = assert_runtime_diagnostic_result_deterministic_and_path_free(
            &temp,
            runtime_diagnostic_result_request(
                runtime_diagnostic_bridge_request(
                    "Bridge preview prompt.",
                    vec!["src_demo".to_string()],
                    Some(helper.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    true,
                    true,
                    true,
                    Some("Diagnostic smoke prompt."),
                    Some(128),
                    Some(1_500),
                ),
                diagnostic_preview,
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeDiagnosticResultStatus::NeedsReview);
        assert!(preview
            .runtime_result_reasons
            .iter()
            .any(|reason| reason.contains("metadata mismatch")));
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.message.contains("metadata mismatch")));
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_blocks_when_prompt_is_blank() {
        let temp = tempfile::tempdir().unwrap();
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "   ",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some("draft-1"),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "   ",
                        vec!["src_demo".to_string()],
                        None,
                        None,
                        false,
                        false,
                        false,
                        None,
                        None,
                        None,
                    ),
                    runtime_diagnostic_preview_like_bridge(
                        LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                        "stdout ok",
                        "stderr ok",
                    ),
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::Blocked);
        assert_eq!(preview.selected_source_count, 1);
        assert_eq!(preview.grounded_answer_execution_plan_status, ScholarChatGroundedAnswerExecutionPlanStatus::Blocked);
        assert_eq!(preview.runtime_diagnostic_result_status, ScholarChatRuntimeDiagnosticResultStatus::Blocked);
        assert!(preview
            .pipeline_gate_reasons
            .iter()
            .any(|reason| reason.contains("prompt is nonblank")));
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_blocks_when_no_sources_are_selected() {
        let temp = tempfile::tempdir().unwrap();
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "Bridge preview prompt.",
                    Some("Alpha beta."),
                    vec![],
                    Some("draft-1"),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "Bridge preview prompt.",
                        vec![],
                        None,
                        None,
                        false,
                        false,
                        false,
                        None,
                        None,
                        None,
                    ),
                    runtime_diagnostic_preview_like_bridge(
                        LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                        "stdout ok",
                        "stderr ok",
                    ),
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::Blocked);
        assert_eq!(preview.selected_source_count, 0);
        assert!(preview
            .pipeline_gate_reasons
            .iter()
            .any(|reason| reason.contains("at least one Scholar Chat source")));
        assert!(preview
            .blockers
            .iter()
            .any(|blocker| blocker.contains("scholar_chat_sources_missing")));
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_blocks_when_grounded_plan_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "   ",
                    Some("The alpha."),
                    vec![source_id.clone()],
                    Some(&answer_draft_id),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "   ",
                        vec![source_id],
                        Some(helper.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        true,
                        true,
                        true,
                        Some("Diagnostic smoke prompt."),
                        Some(128),
                        Some(1_500),
                    ),
                    runtime_diagnostic_preview_like_bridge(
                        LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                        "stdout ok",
                        "stderr ok",
                    ),
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::Blocked);
        assert_eq!(preview.grounded_answer_execution_plan_status, ScholarChatGroundedAnswerExecutionPlanStatus::Blocked);
        assert_eq!(preview.runtime_diagnostic_result_status, ScholarChatRuntimeDiagnosticResultStatus::Blocked);
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_blocks_when_runtime_diagnostic_result_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "Bridge preview prompt.",
                    Some("Alpha beta. Alpha beta gamma."),
                    vec![source_id.clone()],
                    Some(&answer_draft_id),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "Bridge preview prompt.",
                        vec![source_id],
                        Some(helper.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        true,
                        true,
                        true,
                        Some("Diagnostic smoke prompt."),
                        Some(128),
                        Some(1_500),
                    ),
                    runtime_diagnostic_preview_like_bridge(
                        LocalRuntimeSmokeDiagnosticStatus::Blocked,
                        "",
                        "",
                    ),
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::Blocked);
        assert_eq!(preview.runtime_diagnostic_result_status, ScholarChatRuntimeDiagnosticResultStatus::Blocked);
        assert!(preview
            .pipeline_gate_reasons
            .iter()
            .any(|reason| reason.contains("provided smoke diagnostic preview is blocked")));
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_blocks_when_runtime_diagnostic_result_failed() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "  alpha grounded evidence  ",
                    Some("  Alpha beta. Alpha beta gamma.  "),
                    vec![format!("  {source_id}  ")],
                    Some(&format!("  {answer_draft_id}  ")),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "  alpha grounded evidence  ",
                        vec![source_id],
                        Some(helper.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        true,
                        true,
                        true,
                        Some("Diagnostic smoke prompt."),
                        Some(128),
                        Some(1_500),
                    ),
                    runtime_diagnostic_preview_like_bridge(
                        LocalRuntimeSmokeDiagnosticStatus::SmokeFailed,
                        "stdout failed",
                        "stderr failed",
                    ),
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::Blocked);
        assert_eq!(preview.runtime_diagnostic_result_status, ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticFailed);
        assert!(preview
            .pipeline_gate_reasons
            .iter()
            .any(|reason| reason.contains("provided smoke diagnostic preview failed")));
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_needs_review_when_grounded_plan_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "alpha grounded evidence",
                    Some("The alpha."),
                    vec![source_id.clone()],
                    Some("draft-1"),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "alpha grounded evidence",
                        vec![source_id],
                        Some(helper.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        true,
                        true,
                        true,
                        Some("Diagnostic smoke prompt."),
                        Some(128),
                        Some(1_500),
                    ),
                    runtime_diagnostic_preview_like_bridge(
                        LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                        "stdout ok",
                        "stderr ok",
                    ),
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::NeedsReview);
        assert_eq!(preview.grounded_answer_execution_plan_status, ScholarChatGroundedAnswerExecutionPlanStatus::NeedsReview);
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_needs_review_when_runtime_diagnostic_result_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let mut diagnostic_preview = runtime_diagnostic_preview_like_bridge(
            LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
            "stdout ok",
            "stderr ok",
        );
        diagnostic_preview.normalized_model_format = "ggml".to_string();
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "  alpha grounded evidence  ",
                    Some("  Alpha beta. Alpha beta gamma.  "),
                    vec![format!("  {source_id}  ")],
                    Some(&format!("  {answer_draft_id}  ")),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "  alpha grounded evidence  ",
                        vec![source_id],
                        Some(helper.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        true,
                        true,
                        true,
                        Some("Diagnostic smoke prompt."),
                        Some(128),
                        Some(1_500),
                    ),
                    diagnostic_preview,
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::NeedsReview);
        assert_eq!(preview.runtime_diagnostic_result_status, ScholarChatRuntimeDiagnosticResultStatus::NeedsReview);
        assert!(preview
            .pipeline_gate_reasons
            .iter()
            .any(|reason| reason.contains("metadata mismatch")));
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_is_ready_later_only_when_plan_and_result_are_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let helper = runtime_diagnostic_bridge_helper_executable(&temp, "runtime_diagnostic_bridge_ready.exe");
        let model_path = temp.path().join("runtime-diagnostic-bridge-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        prepare_runtime_diagnostic_bridge_spies(&temp);
        let preview = assert_runtime_answer_pipeline_gate_deterministic_and_path_free(
            &temp,
            runtime_answer_pipeline_gate_request(
                execution_plan_request(
                    "  alpha grounded evidence  ",
                    Some("  Alpha beta. Alpha beta gamma.  "),
                    vec![format!("  {source_id}  ")],
                    Some(&format!("  {answer_draft_id}  ")),
                    true,
                    true,
                ),
                runtime_diagnostic_result_request(
                    runtime_diagnostic_bridge_request(
                        "  alpha grounded evidence  ",
                        vec![source_id],
                        Some(helper.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        true,
                        true,
                        true,
                        Some("Diagnostic smoke prompt."),
                        Some(128),
                        Some(1_500),
                    ),
                    runtime_diagnostic_preview_like_bridge(
                        LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded,
                        "stdout ok",
                        "stderr ok",
                    ),
                ),
            ),
        );
        assert_eq!(preview.status, ScholarChatRuntimeAnswerPipelineGateStatus::ReadyLater);
        assert_eq!(preview.grounded_answer_execution_plan_status, ScholarChatGroundedAnswerExecutionPlanStatus::PlanReadyLater);
        assert_eq!(preview.runtime_diagnostic_result_status, ScholarChatRuntimeDiagnosticResultStatus::RuntimeDiagnosticSucceededLater);
        assert!(preview.next_required_actions.iter().any(|action| action.contains("A future Scholar Chat runtime answer pipeline step can be added later")));
        assert!(preview.summary.contains("ready later"));
    }

    #[test]
    fn scholar_chat_runtime_answer_pipeline_gate_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_runtime_answer_pipeline_gate")
            .unwrap();
        let end = source[start..]
            .find("fn runtime_diagnostic_result_metadata_mismatch_fields")
            .unwrap();
        let body = &source[start..start + end];
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("Command::new"));
        assert!(!body.contains("build_answer_draft"));
        assert!(!body.contains("build_grounded_answer"));
        assert!(!body.contains("build_final_answer"));
        assert!(!body.contains("build_evidence_pack"));
        assert!(!body.contains("export_answer_artifacts"));
    }

    #[test]
    fn scholar_chat_runtime_diagnostic_result_body_does_not_call_execution_functions() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scholar_chat.rs"));
        let start = source
            .find("pub fn preview_scholar_chat_runtime_diagnostic_result")
            .unwrap();
        let end = source[start..]
            .find("fn grounded_answer_build_preflight_required_inputs")
            .unwrap();
        let body = &source[start..start + end];
        assert!(!body.contains("run_llama_runtime_smoke_diagnostic"));
        assert!(!body.contains("smoke_test_local_runtime_inference"));
        assert!(!body.contains("run_smoke_inference_probe"));
        assert!(!body.contains("Command::new"));
    }
}
