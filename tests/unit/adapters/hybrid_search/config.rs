//! Hybrid search configuration tests

use mcp_context_browser::adapters::hybrid_search::config::HybridSearchConfig;

#[test]
fn test_hybrid_search_config() {
    let config = HybridSearchConfig::default();
    assert!(config.enabled);
    assert_eq!(config.bm25_weight, 0.4);
    assert_eq!(config.semantic_weight, 0.6);
    assert_eq!(config.bm25_k1, 1.2);
    assert_eq!(config.bm25_b, 0.75);
}
