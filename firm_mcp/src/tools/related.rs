//! Related tool implementation.

use firm_core::compose_entity_id;
use firm_core::graph::{Direction, EntityGraph};
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars;

/// Parameters for the related tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RelatedParams {
    /// Entity type (e.g., "person", "organization").
    pub r#type: String,

    /// Entity ID (e.g., "john_doe").
    pub id: String,

    /// Direction of relationships to return.
    /// - "incoming": entities that reference this entity
    /// - "outgoing": entities that this entity references
    /// - omit or null: both directions
    #[serde(default)]
    pub direction: Option<RelatedDirection>,
}

/// Direction for related entity lookup.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RelatedDirection {
    Incoming,
    Outgoing,
}

impl From<RelatedDirection> for Direction {
    fn from(dir: RelatedDirection) -> Direction {
        match dir {
            RelatedDirection::Incoming => Direction::Incoming,
            RelatedDirection::Outgoing => Direction::Outgoing,
        }
    }
}

/// Execute the related tool.
///
/// Returns IDs of entities related to the specified entity.
pub fn execute(graph: &EntityGraph, params: &RelatedParams) -> CallToolResult {
    let id = compose_entity_id(&params.r#type, &params.id);

    match graph.get_related(&id, params.direction.clone().map(|d| d.into())) {
        Some(entities) => {
            if entities.is_empty() {
                let direction_text = match &params.direction {
                    Some(RelatedDirection::Incoming) => " (incoming)",
                    Some(RelatedDirection::Outgoing) => " (outgoing)",
                    None => "",
                };
                return CallToolResult::success(vec![Content::text(format!(
                    "No related entities found{}.",
                    direction_text
                ))]);
            }

            // Return just the IDs, like list does
            let ids: Vec<&str> = entities.iter().map(|e| e.id.as_str()).collect();
            CallToolResult::success(vec![Content::text(ids.join("\n"))])
        }
        None => CallToolResult::error(vec![Content::text(format!(
            "Entity '{}' with type '{}' not found. Use list with type='{}' to see available IDs.",
            params.id, params.r#type, params.r#type
        ))]),
    }
}
