//! Check command: validates workspace files and reports diagnostics.

use std::path::{Path, PathBuf};

use firm_lang::diagnostics;
use firm_lang::parser::dsl::parse_source;
use firm_lang::workspace::Workspace;

use crate::errors::CliError;
use crate::ui;

/// Check for errors. Checks the whole workspace, or a single file if provided.
pub fn check(workspace_path: &Path, file: Option<PathBuf>) -> Result<(), CliError> {
    match file {
        Some(file_path) => check_file(&file_path),
        None => check_workspace(workspace_path),
    }
}

/// Check a single .firm file for syntax errors.
fn check_file(file_path: &Path) -> Result<(), CliError> {
    let source = std::fs::read_to_string(file_path).map_err(|e| {
        ui::error_with_details(
            &format!("Failed to read '{}'", file_path.display()),
            &e.to_string(),
        );
        CliError::FileError
    })?;

    let parsed = parse_source(source, Some(file_path.to_path_buf())).map_err(|e| {
        ui::error_with_details("Failed to parse file", &e.to_string());
        CliError::BuildError
    })?;

    let file_diagnostics = diagnostics::collect_syntax_errors(&parsed);
    report_diagnostics(&file_diagnostics)
}

/// Check all files in the workspace for syntax errors.
fn check_workspace(workspace_path: &Path) -> Result<(), CliError> {
    let mut workspace = Workspace::new();

    super::load_workspace_files(&workspace_path.to_path_buf(), &mut workspace)
        .map_err(|_| CliError::BuildError)?;

    let mut all_diagnostics = Vec::new();
    for parsed in workspace.parsed_sources() {
        all_diagnostics.extend(diagnostics::collect_syntax_errors(parsed));
    }

    // Only run workspace-level diagnostics if there are no syntax errors,
    // since syntax errors may cause incomplete parse trees.
    if all_diagnostics.is_empty() {
        all_diagnostics.extend(diagnostics::collect_workspace_diagnostics(&workspace));
    }

    report_diagnostics(&all_diagnostics)
}

/// Print diagnostics and return Ok if none, Err if any errors found.
fn report_diagnostics(diagnostics: &[diagnostics::Diagnostic]) -> Result<(), CliError> {
    for diagnostic in diagnostics {
        ui::error(&diagnostic.to_string());
    }

    if diagnostics.is_empty() {
        ui::success("No errors found");
        Ok(())
    } else {
        ui::error(&format!("\nFound {} error(s)", diagnostics.len()));
        Err(CliError::BuildError)
    }
}
