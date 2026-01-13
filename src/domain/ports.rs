pub mod chunking;
pub mod embedding;
pub mod events;
pub mod hybrid_search;
pub mod infrastructure;
pub mod repository;
pub mod services;
pub mod sync;
pub mod vector_store;

pub use chunking::{ChunkingOptions, ChunkingResult, CodeChunker, SharedCodeChunker};
pub use embedding::EmbeddingProvider;
pub use events::{DomainEvent, EventPublisher, SharedEventPublisher};
pub use hybrid_search::HybridSearchProvider;
pub use infrastructure::{SnapshotProvider, SyncProvider};
pub use repository::{ChunkRepository, SearchRepository};
pub use services::{
    ChunkingOrchestratorInterface, ContextServiceInterface, IndexingResult,
    IndexingServiceInterface, IndexingStatus, SearchServiceInterface,
};
pub use sync::{SharedSyncCoordinator, SyncCoordinator, SyncOptions, SyncResult};
pub use vector_store::VectorStoreProvider;
