//! MCP tool implementations for Firm.
//!
//! Each tool module contains:
//! - A `*Params` struct defining the tool's input parameters
//! - An `execute()` function (or similar) containing the tool's business logic
//! - Helper functions for constructing results
//!
//! The server.rs file contains thin wrappers that handle MCP protocol concerns
//! and delegate to these modules for the actual work.

pub mod add_entity;
pub mod build;
pub mod delete_source;
pub mod dsl_reference;
mod dsl_reference_content;
pub mod find_source;
pub mod get;
pub mod list;
pub mod query;
pub mod read_source;
pub mod related;
pub mod replace_source;
pub mod search_source;
pub mod source_tree;
pub mod write_source;

// Re-export param structs for convenience
pub use add_entity::AddEntityParams;
pub use build::BuildParams;
pub use delete_source::DeleteSourceParams;
pub use dsl_reference::DslReferenceParams;
pub use find_source::FindSourceParams;
pub use get::GetParams;
pub use list::ListParams;
pub use query::QueryParams;
pub use read_source::ReadSourceParams;
pub use related::RelatedParams;
pub use replace_source::ReplaceSourceParams;
pub use search_source::SearchSourceParams;
pub use source_tree::SourceTreeParams;
pub use write_source::WriteSourceParams;
