use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::query::CliDirection;
use super::ui::OutputFormat;

/// Defines the top-level interface for the Firm CLI with clap.
#[derive(Parser, Debug)]
#[command(name = "firm")]
#[command(version, about = "Firm CLI: Work management in the terminal.")]
pub struct FirmCli {
    /// Path to firm workspace directory.
    #[arg(short, long, global = true)]
    pub workspace: Option<PathBuf>,

    /// Use cached firm graph?
    #[arg(short, long, global = true)]
    pub cached: bool,

    /// Enable verbose output?
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format
    #[arg(short, long, global = true, default_value_t = OutputFormat::default())]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: FirmCliCommand,
}

/// Defines the available subcommands of the Firm CLI.
#[derive(Subcommand, Debug, PartialEq)]
pub enum FirmCliCommand {
    /// Initialize a new Firm workspace with default schemas and files.
    Init,
    /// Build workspace and entity graph.
    Build,
    /// Get an entity by ID.
    Get {
        /// Entity type (e.g. person, organization or project)
        entity_type: String,
        /// Entity ID (e.g. john_doe)
        entity_id: String,
    },
    /// List entities of type.
    List {
        /// An entity type (e.g. "person") or "schema" to list schemas
        entity_type: String,
    },
    /// Gets entities related to a given entity.
    Related {
        /// Entity type (e.g. person)
        entity_type: String,
        /// Entity ID (e.g. john_doe)
        entity_id: String,
        /// Direction of relationships (incoming, outgoing, or both if not specified)
        #[arg(short, long)]
        direction: Option<CliDirection>,
    },
    /// Interactively adds a new entity to a file in the workspace.
    Add {
        /// Target firm file.
        to_file: Option<PathBuf>,
    },
}
