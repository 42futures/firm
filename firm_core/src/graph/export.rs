use std::collections::HashMap;
use std::fmt::Write;

use petgraph::visit::EdgeRef;

use super::{EntityGraph, Relationship};
use crate::{EntityType, decompose_entity_id};

/// A palette of RGB colors for entity types.
const TYPE_COLORS: &[(u8, u8, u8)] = &[
    (78, 121, 167),   // #4e79a7
    (242, 142, 43),   // #f28e2b
    (225, 87, 89),    // #e15759
    (118, 183, 178),  // #76b7b2
    (89, 161, 79),    // #59a14f
    (237, 201, 72),   // #edc948
    (176, 122, 161),  // #b07aa1
    (255, 157, 167),  // #ff9da7
    (156, 117, 95),   // #9c755f
    (186, 176, 172),  // #bab0ac
];

impl EntityGraph {
    /// Exports the graph to GEXF format (Gephi-compatible).
    pub fn to_gexf(&self) -> String {
        let mut out = String::new();
        let type_colors = self.build_type_color_map();

        // Header
        writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
        writeln!(
            out,
            r#"<gexf xmlns="http://gexf.net/1.3" xmlns:viz="http://gexf.net/1.3/viz" version="1.3">"#
        )
        .unwrap();
        writeln!(out, "  <meta>").unwrap();
        writeln!(out, "    <creator>Firm</creator>").unwrap();
        writeln!(out, "  </meta>").unwrap();
        writeln!(out, r#"  <graph defaultedgetype="directed">"#).unwrap();

        // Node attributes
        writeln!(out, r#"    <attributes class="node">"#).unwrap();
        writeln!(
            out,
            r#"      <attribute id="0" title="entity_type" type="string"/>"#
        )
        .unwrap();
        writeln!(out, "    </attributes>").unwrap();

        // Nodes
        writeln!(out, "    <nodes>").unwrap();
        for (entity_type, node_indices) in &self.entity_type_map {
            let (r, g, b) = type_colors
                .get(entity_type)
                .copied()
                .unwrap_or((204, 204, 204));

            for &node_index in node_indices {
                if let Some(entity) = self.graph.node_weight(node_index) {
                    let (_, short_id) = decompose_entity_id(entity.id.as_str());
                    writeln!(
                        out,
                        r#"      <node id="{}" label="{}">"#,
                        escape_xml(entity.id.as_str()),
                        escape_xml(short_id)
                    )
                    .unwrap();
                    writeln!(out, "        <attvalues>").unwrap();
                    writeln!(
                        out,
                        r#"          <attvalue for="0" value="{}"/>"#,
                        escape_xml(entity_type.as_str())
                    )
                    .unwrap();
                    writeln!(out, "        </attvalues>").unwrap();
                    writeln!(
                        out,
                        r#"        <viz:color r="{}" g="{}" b="{}"/>"#,
                        r, g, b
                    )
                    .unwrap();
                    writeln!(out, "      </node>").unwrap();
                }
            }
        }
        writeln!(out, "    </nodes>").unwrap();

        // Edges
        writeln!(out, "    <edges>").unwrap();
        for (i, edge) in self.graph.edge_references().enumerate() {
            let source = &self.graph[edge.source()];
            let target = &self.graph[edge.target()];
            let label = match edge.weight() {
                Relationship::EntityReference { from_field } => {
                    strip_ref_suffix(&from_field.to_string())
                }
                Relationship::FieldReference {
                    from_field,
                    to_field,
                } => format!(
                    "{} -> {}",
                    strip_ref_suffix(&from_field.to_string()),
                    strip_ref_suffix(&to_field.to_string())
                ),
            };

            writeln!(
                out,
                r#"      <edge id="{}" source="{}" target="{}" label="{}"/>"#,
                i,
                escape_xml(source.id.as_str()),
                escape_xml(target.id.as_str()),
                escape_xml(&label)
            )
            .unwrap();
        }
        writeln!(out, "    </edges>").unwrap();

        writeln!(out, "  </graph>").unwrap();
        writeln!(out, "</gexf>").unwrap();
        out
    }

    /// Exports the graph to DOT (Graphviz) format.
    pub fn to_dot(&self) -> String {
        let mut out = String::new();
        let type_colors = self.build_type_color_map();

        let hex_colors: HashMap<&EntityType, String> = type_colors
            .iter()
            .map(|(t, &(r, g, b))| (*t, format!("#{:02x}{:02x}{:02x}", r, g, b)))
            .collect();

        writeln!(out, "digraph firm {{").unwrap();
        writeln!(out, "  rankdir=LR;").unwrap();
        writeln!(out, "  node [shape=box, style=\"filled,rounded\", fontname=\"Helvetica\", fontsize=11];").unwrap();
        writeln!(out, "  edge [fontname=\"Helvetica\", fontsize=9, color=\"#888888\"];").unwrap();
        writeln!(out).unwrap();

        for (entity_type, node_indices) in &self.entity_type_map {
            let color = hex_colors
                .get(entity_type)
                .map(|s| s.as_str())
                .unwrap_or("#cccccc");

            writeln!(out, "  // {}", entity_type).unwrap();
            for &node_index in node_indices {
                if let Some(entity) = self.graph.node_weight(node_index) {
                    let (_, short_id) = decompose_entity_id(entity.id.as_str());
                    let label = escape_dot(&format!("{}\\n{}", entity.entity_type, short_id));
                    writeln!(
                        out,
                        "  \"{}\" [label=\"{}\", fillcolor=\"{}\", fontcolor=\"white\"];",
                        entity.id, label, color
                    )
                    .unwrap();
                }
            }
            writeln!(out).unwrap();
        }

        writeln!(out, "  // Relationships").unwrap();
        for edge in self.graph.edge_references() {
            let source = &self.graph[edge.source()];
            let target = &self.graph[edge.target()];
            let label = match edge.weight() {
                Relationship::EntityReference { from_field } => {
                    strip_ref_suffix(&from_field.to_string())
                }
                Relationship::FieldReference {
                    from_field,
                    to_field,
                } => format!(
                    "{} -> {}",
                    strip_ref_suffix(&from_field.to_string()),
                    strip_ref_suffix(&to_field.to_string())
                ),
            };

            writeln!(
                out,
                "  \"{}\" -> \"{}\" [label=\"{}\"];",
                source.id,
                target.id,
                escape_dot(&label)
            )
            .unwrap();
        }

        writeln!(out, "}}").unwrap();
        out
    }

    fn build_type_color_map(&self) -> HashMap<&EntityType, (u8, u8, u8)> {
        let mut type_colors = HashMap::new();
        let mut sorted_types: Vec<&EntityType> = self.entity_type_map.keys().collect();
        sorted_types.sort_by_key(|t| t.to_string());
        for (i, entity_type) in sorted_types.into_iter().enumerate() {
            type_colors.insert(entity_type, TYPE_COLORS[i % TYPE_COLORS.len()]);
        }
        type_colors
    }
}

/// Escapes special characters for XML attribute values.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Escapes special characters for DOT label strings.
fn escape_dot(s: &str) -> String {
    s.replace('"', "\\\"")
}

/// Strips the `_ref` suffix from field names used as edge labels.
fn strip_ref_suffix(s: &str) -> String {
    s.strip_suffix("_ref").unwrap_or(s).to_string()
}

#[cfg(test)]
mod tests {
    use crate::graph::EntityGraph;
    use crate::{Entity, EntityId, EntityType, FieldId, FieldValue, ReferenceValue};

    fn sample_graph() -> EntityGraph {
        let mut graph = EntityGraph::new();

        let org = Entity::new(EntityId::new("megacorp"), EntityType::new("organization"))
            .with_field(FieldId::new("name"), "MegaCorp Inc.");

        let person = Entity::new(EntityId::new("john_doe"), EntityType::new("person"))
            .with_field(FieldId::new("name"), "John Doe")
            .with_field(
                FieldId::new("employer"),
                FieldValue::Reference(ReferenceValue::Entity(EntityId::new("megacorp"))),
            );

        graph.add_entities(vec![org, person]).unwrap();
        graph.build();
        graph
    }

    #[test]
    fn test_to_gexf_basic() {
        let graph = sample_graph();
        let gexf = graph.to_gexf();

        assert!(gexf.contains(r#"<gexf xmlns="http://gexf.net/1.3""#));
        assert!(gexf.contains(r#"id="megacorp""#));
        assert!(gexf.contains(r#"id="john_doe""#));
        assert!(gexf.contains(r#"source="john_doe""#));
        assert!(gexf.contains(r#"target="megacorp""#));
        assert!(gexf.contains(r#"label="employer""#));
        assert!(gexf.contains("viz:color"));
    }

    #[test]
    fn test_to_gexf_empty_graph() {
        let graph = EntityGraph::new();
        let gexf = graph.to_gexf();
        assert!(gexf.contains("<nodes>"));
        assert!(gexf.contains("</nodes>"));
        assert!(gexf.contains("<edges>"));
        assert!(gexf.contains("</edges>"));
    }

    #[test]
    fn test_to_dot_basic() {
        let graph = sample_graph();
        let dot = graph.to_dot();

        assert!(dot.contains("digraph firm {"));
        assert!(dot.contains("\"megacorp\""));
        assert!(dot.contains("\"john_doe\""));
        assert!(dot.contains("\"john_doe\" -> \"megacorp\""));
        assert!(dot.contains("employer"));
    }

    #[test]
    fn test_to_dot_empty_graph() {
        let graph = EntityGraph::new();
        let dot = graph.to_dot();
        assert!(dot.contains("digraph firm {"));
        assert!(dot.contains("}"));
    }
}
