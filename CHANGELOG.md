# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **BREAKING**: Upgraded to Todoist API v1 from v2/v9
- **BREAKING**: Updated API endpoint from `https://api.todoist.com/rest/v2` to `https://api.todoist.com/api/v1`
- **BREAKING**: Todo field `is_completed` renamed to `checked` to match API v1
- **BREAKING**: Removed `comment_count` and `url` fields from Todo struct
- **BREAKING**: Updated Todo struct to match API v1 response format with new fields:
  - Added: `user_id`, `deadline`, `duration`, `is_deleted`, `added_at`, `completed_at`, `updated_at`
  - Added: `child_order`, `day_order`, `is_collapsed`, `added_by_uid`, `assigned_by_uid`, `responsible_uid`
- Updated response handling to use paginated format with `results` wrapper
- Improved error handling for API v1 JSON error responses

### Added
- **NEW**: `get_todos_by_filter()` method using the new `/api/v1/tasks/filter` endpoint
- **NEW**: Optional `query` parameter in `get_all_todos(query: Option<&str>)` for unified filtering
- Support for advanced filter query syntax (e.g., `"today | overdue"`, `"p1 & @work"`)
- Added `parent_id` parameter to `get_todos_with_filters()` method
- Created comprehensive example: `examples/fetch_todos.rs`
- Added integration tests for new filter endpoint

### Removed
- **BREAKING**: Removed `filter` and `lang` parameters from `get_todos_with_filters()`
  - Use the new `get_todos_by_filter()` method instead for filter queries
- Removed deprecated fields from Todo struct to match API v1

### Fixed
- API compatibility with Todoist's latest v1 endpoints
- Pagination support for large result sets
- Updated documentation and examples to reflect API changes

## Migration Guide

### For existing users upgrading from the previous version:

1. **Update field names in your code:**
   ```rust
   // Old (v2):
   if todo.is_completed { ... }
   
   // New (v1):
   if todo.checked { ... }
   ```

2. **Replace filter calls:**
   ```rust
   // Old:
   client.get_todos_with_filters(None, None, None, Some("today"), Some("en"), None).await?;
   
   // New (option 1 - unified method):
   client.get_all_todos(Some("today")).await?;
   
   // New (option 2 - dedicated filter method):
   client.get_todos_by_filter("today", Some("en")).await?;
   ```

3. **Update get_all_todos calls:**
   ```rust
   // Old:
   client.get_all_todos().await?;
   
   // New:
   client.get_all_todos(None).await?;
   ```

3. **Update Todo field access:**
   - `todo.is_completed` → `todo.checked`
   - `todo.comment_count` → removed (no replacement)
   - `todo.url` → removed (no replacement)
   - `todo.creator_id` → `todo.added_by_uid`
   - `todo.assignee_id` → `todo.responsible_uid`

4. **Filter parameters:**
   - Use `get_all_todos(Some("query"))` for simple filtering with the unified method
   - Use `get_todos_by_filter()` for complex queries with language control
   - `get_todos_with_filters()` now supports `parent_id` but removed `filter` and `lang`

### Benefits of the upgrade:
- ✅ Better performance with optimized API endpoints
- ✅ Advanced filtering with powerful query syntax
- ✅ Unified method interface with optional parameters for cleaner code
- ✅ Future-proof compatibility with Todoist's latest API
- ✅ Improved error handling and response format
- ✅ Cursor-based pagination for large datasets