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

    // Pre-build the graph unless we're using cache or doing a build/init/source/mcp command
    let skip_build = cli.cached
        || matches!(
            cli.command,
            FirmCliCommand::Build
                | FirmCliCommand::Init
                | FirmCliCommand::Source { .. }
                | FirmCliCommand::Export { .. }
                | FirmCliCommand::Mcp
        );

    if !skip_build {
        match build_and_save_graph(&workspace_path) {
            Ok(_) => (),
            Err(_) => return ExitCode::FAILURE,
        }
    }

    // Handle CLI subcommands
    let result = match cli.command {
        FirmCliCommand::Init => commands::init_workspace(&workspace_path),
        FirmCliCommand::Build => build_and_save_graph(&workspace_path),
        FirmCliCommand::Get {
            target_type,
            target_id,
        } => commands::get_item(&workspace_path, target_type, target_id, cli.format),
        FirmCliCommand::List { target_type } => {
            commands::list_items(&workspace_path, target_type, cli.format)
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
        FirmCliCommand::Add {
            to_file,
            r#type,
            id,
            fields,
            lists,
            list_values,
        } => commands::add_entity(
            &workspace_path,
            to_file,
            r#type,
            id,
            fields,
            lists,
            list_values,
            cli.format,
        ),
        FirmCliCommand::Query { query } => {
            commands::query_entities(&workspace_path, query, cli.format)
        }
        FirmCliCommand::Source {
            target_type,
            target_id,
        } => commands::find_item_source(&workspace_path, target_type, target_id, cli.format),
        FirmCliCommand::Export { export_format } => {
            commands::export_graph(&workspace_path, export_format)
        }
        FirmCliCommand::Mcp => commands::mcp::serve(&workspace_path),
    };

    result.map_or(ExitCode::FAILURE, |_| ExitCode::SUCCESS)
}
