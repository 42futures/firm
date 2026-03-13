use std::{fmt, io, path::PathBuf};

use firm_core::EntityType;

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
                write!(f, "No schema found for entity type '{}' in {}", entity_type, path_buf.display())
            }
        }
    }
}
