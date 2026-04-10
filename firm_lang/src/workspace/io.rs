use std::{
    fs,
    path::{Path, PathBuf},
};

use ignore::{overrides::OverrideBuilder, WalkBuilder};

use crate::{parser::dsl::parse_source, workspace::WorkspaceFile};

use super::{Workspace, WorkspaceError};

const FIRM_FILE_EXTENSION: &str = "firm";

impl Workspace {
    pub fn load_file(
        &mut self,
        path: &PathBuf,
        workspace_path: &PathBuf,
    ) -> Result<(), WorkspaceError> {
        let text = fs::read_to_string(path).map_err(WorkspaceError::IoError)?;

        let relative_path = path
            .strip_prefix(workspace_path)
            .map_err(|err| WorkspaceError::ParseError(path.clone(), err.to_string()))?;

        let parsed = parse_source(text.clone(), Some(relative_path.to_path_buf()))
            .map_err(|err| WorkspaceError::ParseError(path.clone(), err.to_string()))?;

        self.files.insert(path.clone(), WorkspaceFile::new(parsed));
        Ok(())
    }

    pub fn load_directory(&mut self, directory_path: &PathBuf) -> Result<(), WorkspaceError> {
        // 1. Always load schemas from .firm/schemas/ explicitly - never subject to ignore rules
        let schemas_dir = directory_path.join(".firm").join("schemas");
        if schemas_dir.is_dir() {
            for entry in fs::read_dir(&schemas_dir).map_err(WorkspaceError::IoError)? {
                let entry = entry.map_err(WorkspaceError::IoError)?;
                let path = entry.path();
                if path.is_file() && self.is_firm_file(&path) {
                    self.load_file(&path, directory_path)?;
                }
            }
        }

        // 2. Walk the rest of the workspace for entity files, respecting .firmignore / .gitignore
        let ignore_path = directory_path.join(".firmignore");
        let gitignore_path = directory_path.join(".gitignore");

        let mut binding = WalkBuilder::new(directory_path);
        let walker_builder = binding
            .hidden(true)  // skip hidden dirs (like .firm, .citadel) during entity walk
            .require_git(false)
            .filter_entry(|entry| {
                let path = entry.path();
                if path.is_file() {
                    return path
                        .extension()
                        .map_or(false, |ext| ext == FIRM_FILE_EXTENSION);
                }
                true
            });

        if ignore_path.exists() {
            let _ = walker_builder.add_ignore(&ignore_path);
        } else if gitignore_path.exists() {
            let _ = walker_builder.git_ignore(true);
        }

        for result in walker_builder.build() {
            match result {
                Ok(entry) => {
                    let path = entry.path().to_path_buf();
                    if self.is_firm_file(&path) {
                        self.load_file(&path, directory_path)?;
                    }
                }
                Err(e) => {
                    let io_err = std::io::Error::other(e.to_string());
                    return Err(WorkspaceError::IoError(io_err));
                }
            }
        }

        Ok(())
    }

    fn is_firm_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == FIRM_FILE_EXTENSION)
            .unwrap_or(false)
    }
}
