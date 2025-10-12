//! The command-line interface for interacting with a Firm workspace.
//!
//! This crate provides a set of commands to manage and query entities
//! defined in `.firm` files. It uses `firm_lang` to load the workspace
//! and `firm_core` to build and query the entity graph.

mod cli;
mod commands;
mod errors;
mod files;
mod logging;
mod query;
mod ui;

use clap::Parser;
use std::process::ExitCode;

use cli::{FirmCli, FirmCliCommand};
use commands::build_and_save_graph;
use files::get_workspace_path;

fn main() -> ExitCode {
    let cli = FirmCli::parse();

    // Set up logging
    if let Err(e) = logging::initialize(cli.verbose) {
        ui::error_with_details("Failed to initialize logging", &e.to_string());
        return ExitCode::FAILURE;
    }

    // Get the workspace
    let workspace_path = match get_workspace_path(&cli.workspace) {
        Ok(path) => path,
        Err(_) => return ExitCode::FAILURE,
    };

    // Pre-build the graph unless we're using cache or doing a build command
    if !cli.cached && cli.command != FirmCliCommand::Build {
        match build_and_save_graph(&workspace_path) {
            Ok(_) => (),
            Err(_) => return ExitCode::FAILURE,
        }
    }

    // Handle CLI subcommands
    let result = match cli.command {
        FirmCliCommand::Build => build_and_save_graph(&workspace_path),
        FirmCliCommand::Get {
            entity_type,
            entity_id,
        } => commands::get_entity_by_id(&workspace_path, entity_type, entity_id, cli.format),
        FirmCliCommand::List { entity_type } => {
            if entity_type == "schema" {
                commands::list_schemas(&workspace_path, cli.format)
            } else {
                commands::list_entities_by_type(&workspace_path, entity_type, cli.format)
            }
        }
        FirmCliCommand::Related {
            entity_type,
            entity_id,
            direction,
        } => commands::get_related_entities(
            &workspace_path,
            entity_type,
            entity_id,
            direction,
            cli.format,
        ),
        FirmCliCommand::Add { to_file } => {
            commands::add_entity(&workspace_path, to_file, cli.format)
        }
    };

    result.map_or(ExitCode::FAILURE, |_| ExitCode::SUCCESS)
}
