use std::{fs, path::{Path, PathBuf}};

use crate::{parser::dsl::parse_source, workspace::WorkspaceFile};

use super::{Workspace, WorkspaceError};

const FIRM_FILE_EXTENSION: &str = "firm";

impl Workspace {
    /// Load a single firm source file.
    pub fn load_file(
        &mut self,
        path: &PathBuf,
        workspace_path: &PathBuf,
    ) -> Result<(), WorkspaceError> {
        // Read the source text
        let text = fs::read_to_string(path).map_err(WorkspaceError::IoError)?;

        // Make the source path relative to the workspace
        let relative_path = path
            .strip_prefix(workspace_path)
            .map_err(|err| WorkspaceError::ParseError(path.clone(), err.to_string()))?;

        // Parse the source text
        let parsed = parse_source(text.clone(), Some(relative_path.to_path_buf()))
            .map_err(|err| WorkspaceError::ParseError(path.clone(), err.to_string()))?;

        self.files.insert(path.clone(), WorkspaceFile::new(parsed));
        Ok(())
    }

    /// Loads all firm files in a directory and its subdirectories.
    pub fn load_directory(&mut self, directory_path: &PathBuf) -> Result<(), WorkspaceError> {
        self.load_directory_recursive(directory_path, directory_path)
    }

    /// Load all firm files in a directory recursively.
    fn load_directory_recursive(
        &mut self,
        directory_path: &PathBuf,
        root_directory_path: &PathBuf,
    ) -> Result<(), WorkspaceError> {
        let entries = fs::read_dir(directory_path).map_err(WorkspaceError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(WorkspaceError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                self.load_directory_recursive(&path, root_directory_path)?;
            } else if path.is_file() && self.is_firm_file(&path) {
                self.load_file(&path, root_directory_path)?;
            }
        }

        Ok(())
    }

    /// Returns true if a path has the .firm extension
    fn is_firm_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == FIRM_FILE_EXTENSION)
            .unwrap_or(false)
    }
}
