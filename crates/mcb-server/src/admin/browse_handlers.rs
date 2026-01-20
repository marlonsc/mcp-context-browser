//! Browse handlers for Admin UI
//!
//! REST API handlers for browsing indexed collections, files, and code chunks.
//! Provides navigation capabilities for the Admin UI code browser.
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! |------|--------|-------------|
//! | `/collections` | GET | List all indexed collections |
//! | `/collections/:name/files` | GET | List files in a collection |
//! | `/collections/:name/files/*path/chunks` | GET | Get chunks for a file |

use mcb_domain::ports::providers::VectorStoreBrowser;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get};
use std::sync::Arc;

use super::auth::AdminAuth;
use super::models::{
    ChunkDetailResponse, ChunkListResponse, CollectionInfoResponse, CollectionListResponse,
    FileInfoResponse, FileListResponse,
};

/// Browse handler state containing the vector store browser
#[derive(Clone)]
pub struct BrowseState {
    /// Vector store browser for collection/file navigation
    pub browser: Arc<dyn VectorStoreBrowser>,
}

/// Error response for browse operations
#[derive(serde::Serialize)]
pub struct BrowseErrorResponse {
    /// Error message
    pub error: String,
    /// Error code for programmatic handling
    pub code: String,
}

impl BrowseErrorResponse {
    fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
        }
    }

    /// Creates a not found error response
    pub fn not_found(resource: &str) -> Self {
        Self::new(format!("{} not found", resource), "NOT_FOUND")
    }

    /// Creates an internal error response
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(message, "INTERNAL_ERROR")
    }
}

/// List all indexed collections
///
/// Returns a list of all collections with their statistics including
/// vector count, file count, and provider information.
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/collections")]
pub async fn list_collections(
    _auth: AdminAuth,
    state: &State<BrowseState>,
) -> Result<Json<CollectionListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let collections = state.browser.list_collections().await.map_err(|e| {
        (
            Status::InternalServerError,
            Json(BrowseErrorResponse::internal(e.to_string())),
        )
    })?;

    let collection_responses: Vec<CollectionInfoResponse> = collections
        .into_iter()
        .map(|c| CollectionInfoResponse {
            name: c.name,
            vector_count: c.vector_count,
            file_count: c.file_count,
            last_indexed: c.last_indexed,
            provider: c.provider,
        })
        .collect();

    let total = collection_responses.len();
    Ok(Json(CollectionListResponse {
        collections: collection_responses,
        total,
    }))
}

/// List files in a collection
///
/// Returns a list of all indexed files in the specified collection,
/// including chunk counts and language information.
///
/// # Arguments
///
/// * `name` - Collection name
/// * `limit` - Maximum number of files to return (default: 100)
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/collections/<name>/files?<limit>")]
pub async fn list_collection_files(
    _auth: AdminAuth,
    state: &State<BrowseState>,
    name: &str,
    limit: Option<usize>,
) -> Result<Json<FileListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let limit = limit.unwrap_or(100);

    let files = state
        .browser
        .list_file_paths(name, limit)
        .await
        .map_err(|e| {
            // Check if it's a collection not found error
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                (
                    Status::NotFound,
                    Json(BrowseErrorResponse::not_found("Collection")),
                )
            } else {
                (
                    Status::InternalServerError,
                    Json(BrowseErrorResponse::internal(error_msg)),
                )
            }
        })?;

    let file_responses: Vec<FileInfoResponse> = files
        .into_iter()
        .map(|f| FileInfoResponse {
            path: f.path,
            chunk_count: f.chunk_count,
            language: f.language,
            size_bytes: f.size_bytes,
        })
        .collect();

    let total = file_responses.len();
    Ok(Json(FileListResponse {
        files: file_responses,
        total,
        collection: name.to_string(),
    }))
}

/// Get code chunks for a specific file
///
/// Returns all code chunks that were extracted from a specific file,
/// ordered by line number. Useful for displaying the full indexed
/// content of a file.
///
/// # Arguments
///
/// * `name` - Collection name
/// * `path` - File path (URL-encoded, can contain slashes)
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/collections/<name>/chunks/<path..>")]
pub async fn get_file_chunks(
    _auth: AdminAuth,
    state: &State<BrowseState>,
    name: &str,
    path: std::path::PathBuf,
) -> Result<Json<ChunkListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let file_path = path.to_string_lossy().to_string();

    let chunks = state
        .browser
        .get_chunks_by_file(name, &file_path)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                (
                    Status::NotFound,
                    Json(BrowseErrorResponse::not_found("File or collection")),
                )
            } else {
                (
                    Status::InternalServerError,
                    Json(BrowseErrorResponse::internal(error_msg)),
                )
            }
        })?;

    let chunk_responses: Vec<ChunkDetailResponse> = chunks
        .into_iter()
        .map(|c| {
            // Estimate end line from content
            let line_count = c.content.lines().count() as u32;
            let end_line = c.start_line.saturating_add(line_count.saturating_sub(1));

            ChunkDetailResponse {
                id: c.id,
                content: c.content,
                file_path: c.file_path,
                start_line: c.start_line,
                end_line,
                language: c.language,
                score: c.score,
            }
        })
        .collect();

    let total = chunk_responses.len();
    Ok(Json(ChunkListResponse {
        chunks: chunk_responses,
        file_path,
        collection: name.to_string(),
        total,
    }))
}

// Tests moved to tests/unit/browse_handlers_tests.rs per test organization standards
