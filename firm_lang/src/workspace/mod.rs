mod build;
mod io;
mod workspace_errors;

use std::{collections::HashMap, path::PathBuf};

pub use build::WorkspaceBuild;
pub use workspace_errors::WorkspaceError;

use crate::parser::dsl::ParsedSource;

/// Represents a collection of files to be processed by Firm.
///
/// Initally, we collect DSL files in the workspace, parsing the source.
/// Afterwards, the workspace can be "built", converting that to core entities and schemas.
#[derive(Debug)]
pub struct Workspace {
    files: HashMap<PathBuf, WorkspaceFile>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Gets the number of files currently in the workspace.
    pub fn num_files(&self) -> usize {
        self.files.len()
    }
}

/// Represents a parsed file in the workspace.
#[derive(Debug)]
pub struct WorkspaceFile {
    parsed: ParsedSource,
}

impl WorkspaceFile {
    pub fn new(parsed: ParsedSource) -> Self {
        Self { parsed }
    }
}
