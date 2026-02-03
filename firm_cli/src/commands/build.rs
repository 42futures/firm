use firm_core::graph::{EntityGraph, GraphError};
use firm_lang::workspace::{Workspace, WorkspaceBuild, WorkspaceError};
use std::path::PathBuf;

use crate::errors::CliError;
use crate::files::save_graph_with_backup;
use crate::ui::{self};

/// Builds the selected workspace and saves the resulting entity graph.
pub fn build_and_save_graph(workspace_path: &PathBuf) -> Result<(), CliError> {
    ui::header("Building graph");

    // First load and build the workspace from DSL
    let mut workspace = Workspace::new();
    load_workspace_files(workspace_path, &mut workspace).map_err(|_| CliError::BuildError)?;
    let build = build_workspace(workspace).map_err(|_| CliError::BuildError)?;

    // Then build and save the entity graph
    let graph = build_graph(&build).map_err(|_| CliError::BuildError)?;
    save_graph_with_backup(workspace_path, &graph).map_err(|_| CliError::BuildError)?;

    ui::success("Graph was built and saved");

    Ok(())
}

/// Loads files in the workspace with progress indicator.
pub fn load_workspace_files(
    path: &PathBuf,
    workspace: &mut Workspace,
) -> Result<(), WorkspaceError> {
    let spinner = ui::spinner("Loading workspace files");

    match workspace.load_directory(path) {
        Ok(_) => {
            spinner.finish_with_message("Workspace files loaded successfully");
            Ok(())
        },
        Err(e) => {
            spinner.finish_and_clear();
            ui::error_with_details(
                &format!("Failed to load directory '{}'", path.display()),
                &e.to_string(),
            );

            Err(e)
        }
    }
}

/// Builds a workspace with progress indicator.
pub fn build_workspace(mut workspace: Workspace) -> Result<WorkspaceBuild, WorkspaceError> {
    let progress = ui::progress_bar(workspace.num_files().try_into().unwrap());

    match workspace.build_with_progress(|total, curent, phase| {
        progress.set_length(total.try_into().unwrap());
        progress.set_position(curent.try_into().unwrap());
        progress.set_message(phase.to_string());
    }) {
        Ok(build) => {
            progress.finish_with_message("Workspace built successfully");
            Ok(build)
        }
        Err(e) => {
            progress.finish_and_clear();
            ui::error_with_details("Failed to build workspace", &e.to_string());
            Err(e)
        }
    }
}

/// Builds the entity graph from a workspace with progress indicator.
pub fn build_graph(build: &WorkspaceBuild) -> Result<EntityGraph, CliError> {
    let spinner = ui::spinner("Creating graph from workspace");
    let mut graph = EntityGraph::new();

    let entity_result = graph.add_entities(build.entities.clone());
    if let Err(e) = entity_result {
        spinner.finish_and_clear();

        if let GraphError::EntityAlreadyExists(entity_id) = e {
            ui::error(&format!(
                "Entities with duplicate IDs '{}' cannot be added to the graph",
                entity_id
            ));
        }

        return Err(CliError::BuildError);
    }

    spinner.set_message("Building graph relationships");
    graph.build();

    spinner.finish_with_message("Graph built successfully");
    Ok(graph)
}
