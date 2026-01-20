//! Core MCP server implementation for Firm.

use std::path::PathBuf;
use std::sync::Arc;

use log::debug;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt, model::*,
    service::RequestContext, transport::stdio,
};
use tokio::sync::Mutex;

use firm_lang::workspace::{Workspace, WorkspaceBuild, WorkspaceError};

/// Error type for MCP server operations.
#[derive(Debug)]
pub enum ServerError {
    /// Workspace loading or validation error
    Workspace(WorkspaceError),
    /// MCP protocol error
    Mcp(String),
}

impl From<WorkspaceError> for ServerError {
    fn from(err: WorkspaceError) -> Self {
        ServerError::Workspace(err)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::Workspace(err) => write!(f, "Workspace error: {}", err),
            ServerError::Mcp(msg) => write!(f, "MCP error: {}", msg),
        }
    }
}

impl std::error::Error for ServerError {}

/// Internal state of the MCP server.
struct ServerState {
    workspace: Workspace,
    build: WorkspaceBuild,
}

/// MCP server for a Firm workspace.
///
/// Exposes workspace operations (query, list, get, etc.) as MCP tools,
/// and source files as MCP resources.
#[derive(Clone)]
pub struct FirmMcpServer {
    workspace_path: PathBuf,
    state: Arc<Mutex<ServerState>>,
}

impl FirmMcpServer {
    /// Create a new MCP server for the given workspace path.
    ///
    /// This will load and build the workspace. Returns an error if the
    /// workspace cannot be loaded or has validation errors.
    pub fn new(workspace_path: PathBuf) -> Result<Self, WorkspaceError> {
        debug!("Creating MCP server for workspace: {:?}", workspace_path);

        let mut workspace = Workspace::new();
        workspace.load_directory(&workspace_path)?;
        let build = workspace.build()?;

        debug!(
            "Workspace loaded: {} entities, {} schemas",
            build.entities.len(),
            build.schemas.len()
        );

        Ok(Self {
            workspace_path,
            state: Arc::new(Mutex::new(ServerState { workspace, build })),
        })
    }

    /// Serve MCP over stdio (stdin/stdout).
    ///
    /// This method blocks until the connection is closed.
    pub async fn serve_stdio(self) -> Result<(), ServerError> {
        debug!("Starting MCP server on stdio");
        let service = self
            .serve(stdio())
            .await
            .map_err(|e| ServerError::Mcp(format!("Failed to start server: {}", e)))?;
        service
            .waiting()
            .await
            .map_err(|e| ServerError::Mcp(format!("Server error: {}", e)))?;
        Ok(())
    }

    /// Rebuild the workspace from disk.
    ///
    /// Called after write operations to ensure the in-memory state is fresh.
    #[allow(dead_code)]
    async fn rebuild(&self) -> Result<(), WorkspaceError> {
        debug!("Rebuilding workspace");
        let mut state = self.state.lock().await;

        let mut workspace = Workspace::new();
        workspace.load_directory(&self.workspace_path)?;
        let build = workspace.build()?;

        state.workspace = workspace;
        state.build = build;

        debug!(
            "Workspace rebuilt: {} entities, {} schemas",
            state.build.entities.len(),
            state.build.schemas.len()
        );

        Ok(())
    }
}

impl ServerHandler for FirmMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Firm MCP server. Use tools to query, list, and modify entities in the workspace. \
                 Use resources to read source files directly."
                    .into(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        // TODO: Implement resource listing
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        // TODO: Implement resource reading
        Err(McpError::resource_not_found(
            "Resource reading not yet implemented",
            None,
        ))
    }
}
