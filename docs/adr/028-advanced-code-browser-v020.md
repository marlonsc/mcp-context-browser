# ADR 028: Advanced Code Browser UI v0.2.0

## Status

**Proposed** (Planned for v0.2.0)

> Not yet implemented. Key dependencies:
>
> - v0.1.2 basic browse feature (REST APIs + UI pages)
> - Syntax highlighting library integration
> - Optional: Language Server Protocol support
>
> **v0.1.2 Foundation (Implemented)**:
>
> - `crates/mcb-domain/src/ports/providers/vector_store.rs` - VectorStoreBrowser trait
> - `crates/mcb-domain/src/value_objects/browse.rs` - CollectionInfo struct
> - `crates/mcb-providers/src/vector_store/*.rs` - 6 provider implementations
> - `crates/mcb-server/src/admin/browse_handlers.rs` - REST API handlers
> - `crates/mcb-server/src/admin/web/templates/browse*.html` - UI pages

## Context

v0.1.2 provides basic code browsing with file listing and chunk display. Users need IDE-like capabilities for deep code exploration.

**Current v0.1.2 capabilities:**

- List indexed collections with stats (vector count, file count, provider)
- List files in a collection with language badges
- View code chunks with Prism.js syntax highlighting
- Navigate via breadcrumbs and nav links

**User demand for v0.2.0:**

- Tree view navigation for large codebases
- Full syntax highlighting with line numbers
- Inline search result highlighting
- Chunk metadata panel with embedding stats
- Diff view between indexed versions
- Keyboard shortcuts for power users

## Decision

Implement advanced code browser in v0.2.0 with IDE-like capabilities:

### 1. Navigation Features

- **Tree view** with collapsible directories
- **Breadcrumb navigation** for file paths
- **Quick file search** (fuzzy match on indexed files)
- **Recent files** and **favorites**
- **Keyboard shortcuts** (vim-like optional)

### 2. Code Display

- **Full syntax highlighting** (tree-sitter based, not regex)
- **Line numbers** with clickable links
- **Chunk boundaries** visually marked
- **Minimap** for large files
- **Word wrap** toggle

### 3. Search Integration

- **Inline semantic search** results highlighted in code
- **Jump to chunk** from search results
- **Related chunks** suggestions
- **Similarity score** visualization

### 4. Metadata Panel

- **Chunk details**: ID, lines, language, embedding stats
- **File metadata**: size, modification time, chunk count
- **Collection stats**: total vectors, dimensions, provider

### 5. Advanced Features

- **Diff view** between indexed versions (if git-aware indexing enabled)
- **Export** chunks as JSON/Markdown
- **Share link** to specific chunk/line
- **Dark mode** toggle
- **Mobile responsive** layout

### 6. Performance

- **Virtual scrolling** for large files
- **Lazy loading** chunks on scroll
- **Client-side caching** of viewed files
- **WebSocket** for real-time index updates

## Consequences

### Positive

- **Better UX**: Developers can explore indexed code naturally
- **Discovery**: Related code surfaces through navigation
- **Debugging**: Easier to understand what's indexed
- **Adoption**: Lower barrier to entry

### Negative

- **Complexity**: ~3000+ LOC for full implementation
- **Dependencies**: Additional JS libraries
- **Maintenance**: UI requires more testing
- **Performance**: Tree rendering for huge codebases

## Alternatives Considered

### Alternative 1: Monaco Editor Integration

- **Pros**: Full IDE experience, LSP support
- **Cons**: Heavy dependency (~2MB), complex integration
- **Deferred**: Consider for v0.3.0

### Alternative 2: CodeMirror 6

- **Pros**: Lightweight, extensible
- **Cons**: Still requires significant JS
- **Partially accepted**: Good fallback option

### Alternative 3: Keep Basic Browse

- **Pros**: Simplicity, minimal maintenance
- **Cons**: Does not meet user demand for exploration
- **Rejected**: Essential for adoption

## Implementation Notes

### Phase 1: Tree View (Essential)

**Create**: `crates/mcb-server/src/admin/web/templates/browse_tree.html`

```rust
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Vec<FileTreeNode>,
    pub chunk_count: u32,
}
```

**New REST endpoint:**

```rust
#[get("/collections/<name>/tree")]
pub async fn get_file_tree(
    name: &str,
    state: &State<BrowseState>,
) -> Result<Json<FileTreeResponse>, (Status, Json<BrowseErrorResponse>)>
```

### Phase 2: Enhanced Code Display

**Create**: `crates/mcb-server/src/admin/web/templates/browse_code.html`

```rust
pub struct CodeView {
    pub file_path: String,
    pub content: String,
    pub chunks: Vec<ChunkHighlight>,
    pub language: Language,
}

pub struct ChunkHighlight {
    pub id: String,
    pub start_line: u32,
    pub end_line: u32,
    pub score: Option<f64>, // if from search
}
```

**Features:**

- Full syntax highlighting with tree-sitter
- Chunk boundary markers (CSS borders/backgrounds)
- Line linking (click to share URL)

### Phase 3: Search Integration

**Modify**: `crates/mcb-server/src/admin/browse_handlers.rs`

```rust
#[get("/collections/<name>/search?<query>&<limit>")]
pub async fn search_in_collection(
    name: &str,
    query: &str,
    limit: Option<usize>,
    state: &State<BrowseState>,
) -> Result<Json<SearchResultsResponse>, (Status, Json<BrowseErrorResponse>)>
```

**Features:**

- Inline result highlighting
- Related chunks sidebar
- Similarity score badges

### Phase 4: Advanced UX

- Keyboard navigation (j/k for scroll, Enter to open)
- Minimap for large files
- Dark mode with CSS variables
- Responsive layout for mobile

### Phase 5: Real-time Features

**Create**: `crates/mcb-server/src/admin/browse_sse.rs`

```rust
#[get("/collections/<name>/events")]
pub fn browse_events(name: &str) -> EventStream![Event + '_]
```

**Events:**

- `IndexingProgress` - Show progress during indexing
- `ChunkAdded` - Update file view when new chunks indexed
- `CollectionUpdated` - Refresh stats

## Files to Create (v0.2.0)

| File | Purpose |
|------|---------|
| `crates/mcb-server/src/admin/web/templates/browse_tree.html` | Tree view component |
| `crates/mcb-server/src/admin/web/templates/browse_code.html` | Enhanced code view |
| `crates/mcb-server/src/admin/web/templates/browse_search.html` | Search results page |
| `crates/mcb-server/src/admin/browse_sse.rs` | SSE events for browse |
| `crates/mcb-server/src/admin/web/assets/tree-view.js` | Tree view JS component |
| `crates/mcb-server/src/admin/web/assets/keyboard-nav.js` | Keyboard navigation |

## Files to Modify (v0.2.0)

| File | Change |
|------|--------|
| `crates/mcb-server/src/admin/browse_handlers.rs` | Add tree and search endpoints |
| `crates/mcb-server/src/admin/routes.rs` | Mount new routes |
| `crates/mcb-server/src/admin/web/handlers.rs` | Add page handlers |
| `crates/mcb-server/src/admin/web/router.rs` | Mount UI routes |
| `crates/mcb-domain/src/ports/providers/vector_store.rs` | Extend VectorStoreBrowser trait |

## Success Metrics

| Metric | v0.1.2 | Target v0.2.0 |
|--------|--------|---------------|
| File navigation | List view | Tree view |
| Code display | Basic Prism.js | Tree-sitter + chunks |
| Search integration | None | Inline highlighting |
| Keyboard nav | None | Full vim-like |
| Real-time updates | None | SSE events |

## Dependencies

**JavaScript Libraries (CDN):**

- Alpine.js (client state, tree view)
- Prism.js (syntax highlighting - already in v0.1.2)
- Optional: Monaco Editor (v0.3.0)

**Rust Crates (existing):**

- rocket (web framework)
- serde (JSON serialization)

## Technical Approach

### Frontend

- **HTMX** for server-driven updates (keep consistency with v0.1.2)
- **Alpine.js** for complex client state (tree view, keyboard nav)
- **Prism.js** or **highlight.js** for syntax highlighting
- **CSS Variables** for theming (dark mode)

### Backend

- New endpoints for tree structure
- Chunk streaming for large files
- WebSocket endpoint for live updates

## Related ADRs

- [ADR-007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - Base admin UI
- [ADR-008: Git-Aware Semantic Indexing](008-git-aware-semantic-indexing-v0.2.0.md) - Git metadata for diff view
- [ADR-026: Routing Refactor Rocket](026-routing-refactor-rocket-poem.md) - Rocket web framework

## References

- [HTMX Documentation](https://htmx.org/docs/)
- [Alpine.js Guide](https://alpinejs.dev/start-here)
- [Prism.js](https://prismjs.com/)
- [Tree-sitter](https://tree-sitter.github.io/tree-sitter/)
