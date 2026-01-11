//! Adapter tests
//!
//! This module contains tests for all adapter implementations including:
//! - Database connection pooling
//! - HTTP client pool
//! - Hybrid search (BM25 + semantic)
//! - Provider routing (circuit breaker, failover, health, cost, metrics)
//! - Vector store providers (EdgeVec, filesystem, encrypted)
//! - Embedding providers (FastEmbed)

mod database;
mod http_client;
mod hybrid_search;
mod providers;
