//! Tool Registry Tests

use mcb_server::tools::registry::create_tool_list;

#[test]
fn test_tool_definitions_create_valid_tools() {
    let tools = create_tool_list().expect("should create tool list");
    assert_eq!(tools.len(), 4);

    let names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(names.contains(&"index_codebase"));
    assert!(names.contains(&"search_code"));
    assert!(names.contains(&"get_indexing_status"));
    assert!(names.contains(&"clear_index"));
}

#[test]
fn test_each_tool_has_description() {
    let tools = create_tool_list().expect("should create tool list");
    for tool in tools {
        assert!(
            tool.description.is_some(),
            "Tool {} should have description",
            tool.name
        );
    }
}
