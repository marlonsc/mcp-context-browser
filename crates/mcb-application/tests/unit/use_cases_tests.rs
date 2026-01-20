//! Basic compilation tests for application use cases

use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};

// Test that the use cases can be imported and type names are correct
#[test]
fn test_use_cases_can_be_imported() {
    // Verify type names exist using type_name
    let context_type = std::any::type_name::<ContextServiceImpl>();
    let indexing_type = std::any::type_name::<IndexingServiceImpl>();
    let search_type = std::any::type_name::<SearchServiceImpl>();

    assert!(
        context_type.contains("ContextServiceImpl"),
        "ContextServiceImpl type should be available"
    );
    assert!(
        indexing_type.contains("IndexingServiceImpl"),
        "IndexingServiceImpl type should be available"
    );
    assert!(
        search_type.contains("SearchServiceImpl"),
        "SearchServiceImpl type should be available"
    );
}

// Test that types have expected traits implemented
#[test]
fn test_types_are_send_sync() {
    // Use cases must be Send + Sync for async runtime
    fn assert_send_sync<T: Send + Sync>() {}

    // This will fail to compile if types don't implement Send + Sync
    assert_send_sync::<ContextServiceImpl>();
    assert_send_sync::<IndexingServiceImpl>();
    assert_send_sync::<SearchServiceImpl>();

    // Types are Send + Sync - compilation is sufficient proof
    // Instantiation requires dependencies, so we only verify traits at compile time
    // The assert_send_sync calls above verify the traits at compile time
}
