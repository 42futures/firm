//! Language server command implementation.

use std::path::Path;

use firm_lsp::FirmLspServer;

use crate::errors::CliError;
use crate::ui;

/// Start the language server on stdio.
pub fn serve(workspace_path: &Path) -> Result<(), CliError> {
    ui::debug("Starting language server...");

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        ui::error_with_details("Failed to create async runtime", &e.to_string());
        CliError::BuildError
    })?;

    rt.block_on(async {
        FirmLspServer::serve_stdio(workspace_path.to_path_buf())
            .await
            .map_err(|e| {
                ui::error_with_details("Language server error", &e);
                CliError::BuildError
            })
    })
}
