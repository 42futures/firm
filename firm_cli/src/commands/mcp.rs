//! MCP server command implementation.

use std::path::Path;

use firm_mcp::FirmMcpServer;

use crate::errors::CliError;
use crate::ui;

/// Start the MCP server on stdio.
pub fn serve(workspace_path: &Path) -> Result<(), CliError> {
    // Create a tokio runtime for the async MCP server
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        ui::error_with_details("Failed to create async runtime", &e.to_string());
        CliError::BuildError
    })?;

    rt.block_on(async {
        // Create the MCP server
        let server = FirmMcpServer::new(workspace_path.to_path_buf()).map_err(|e| {
            ui::error_with_details("Failed to load workspace", &e.to_string());
            CliError::BuildError
        })?;

        // Serve over stdio (blocks until connection closes)
        server.serve_stdio().await.map_err(|e| {
            ui::error_with_details("MCP server error", &e.to_string());
            CliError::BuildError
        })
    })
}
