use std::path::PathBuf;

use crate::errors::CliError;
use crate::files::load_current_graph;

/// Export format for the graph.
#[derive(Clone, Debug, Default, PartialEq, clap::ValueEnum)]
pub enum ExportFormat {
    /// GEXF format (Gephi)
    #[default]
    Gexf,
    /// DOT format (Graphviz)
    Dot,
}

/// Exports the entity graph in the specified format.
pub fn export_graph(workspace_path: &PathBuf, format: ExportFormat) -> Result<(), CliError> {
    let graph = load_current_graph(workspace_path)?;

    let output = match format {
        ExportFormat::Gexf => graph.to_gexf(),
        ExportFormat::Dot => graph.to_dot(),
    };

    print!("{}", output);
    Ok(())
}
