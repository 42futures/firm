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

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
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

    /// Gets all file paths in the workspace.
    pub fn file_paths(&self) -> Vec<&PathBuf> {
        self.files.keys().collect()
    }

    /// Finds the source file path for an entity by its type and ID.
    ///
    /// This performs a linear search through all parsed files in the workspace,
    /// returning the absolute path of the first file containing a matching entity.
    ///
    /// Returns None if no matching entity is found.
    pub fn find_entity_source(&self, entity_type: &str, entity_id: &str) -> Option<PathBuf> {
        for (path, file) in &self.files {
            for entity in file.parsed.entities() {
                if entity.entity_type() == Some(entity_type) && entity.id() == Some(entity_id) {
                    return Some(path.clone());
                }
            }
        }
        None
    }

    /// Adds a pre-parsed source file to the workspace.
    /// Useful for testing or when source is parsed externally.
    pub fn add_parsed_source(&mut self, parsed: ParsedSource) {
        let path = parsed.path.clone();
        self.files.insert(path, WorkspaceFile::new(parsed));
    }

    /// Returns all parsed source files in the workspace.
    pub fn parsed_sources(&self) -> Vec<&ParsedSource> {
        self.files.values().map(|f| &f.parsed).collect()
    }

    /// Finds the source file path for a schema by its name.
    ///
    /// This performs a linear search through all parsed files in the workspace,
    /// returning the absolute path of the first file containing a matching schema.
    ///
    /// Returns None if no matching schema is found.
    pub fn find_schema_source(&self, schema_name: &str) -> Option<PathBuf> {
        for (path, file) in &self.files {
            for schema in file.parsed.schemas() {
                if schema.name() == Some(schema_name) {
                    return Some(path.clone());
                }
            }
        }
        None
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
