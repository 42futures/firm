use std::{fmt, io, path::PathBuf};

use firm_core::EntityType;

use crate::defaults;

/// Defines the errors you might encounter using a workspace.
#[derive(Debug)]
pub enum WorkspaceError {
    IoError(io::Error),
    ParseError(PathBuf, String),
    ValidationError(PathBuf, String),
    MissingSchemaError(PathBuf, EntityType),
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceError::IoError(error) => {
                write!(f, "There was a problem reading workspace files: {}", error)
            }
            WorkspaceError::ParseError(path_buf, error) => write!(
                f,
                "Workspace file at {} could not be parsed: {}",
                path_buf.display(),
                error
            ),
            WorkspaceError::ValidationError(path_buf, error) => write!(
                f,
                "Workspace file at {} was invalid: {}",
                path_buf.display(),
                error
            ),
            WorkspaceError::MissingSchemaError(path_buf, entity_type) => {
                let is_default_schema = is_default_schema_type(entity_type);

                if is_default_schema {
                    write!(
                        f,
                        "No schema found for entity type '{}' in {}\n\nRun 'firm init' to create default schemas, or define your own schema in your workspace.",
                        entity_type,
                        path_buf.display()
                    )
                } else {
                    write!(
                        f,
                        "No schema found for entity type '{}' in {}\n\nDefine a schema for this type in your workspace.",
                        entity_type,
                        path_buf.display()
                    )
                }
            }
        }
    }
}

/// Check if an entity type matches one of the default schemas.
fn is_default_schema_type(entity_type: &EntityType) -> bool {
    defaults::all_default_schemas()
        .iter()
        .any(|schema| &schema.entity_type == entity_type)
}
