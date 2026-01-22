//! Core MCP server implementation for Firm.
//!
//! This module contains the MCP protocol handling and delegates to the
//! tools module for actual business logic.

use std::path::PathBuf;
use std::sync::Arc;

use log::debug;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_handler,
    tool_router, transport::stdio,
};
use tokio::sync::Mutex;

use firm_core::graph::EntityGraph;
use firm_lang::workspace::{Workspace, WorkspaceBuild, WorkspaceError};

use crate::resources;
use crate::tools::{
    self, AddEntityParams, BuildParams, DslReferenceParams, FindSourceParams, GetParams,
    ListParams, QueryParams, ReadSourceParams, RelatedParams, WriteSourceParams,
};

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
pub struct ServerState {
    pub workspace: Workspace,
    pub build: WorkspaceBuild,
    pub graph: EntityGraph,
}

/// MCP server for a Firm workspace.
///
/// Exposes workspace operations (query, list, get, etc.) as MCP tools,
/// and source files as MCP resources.
#[derive(Clone)]
pub struct FirmMcpServer {
    workspace_path: PathBuf,
    state: Arc<Mutex<ServerState>>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<FirmMcpServer>,
}

#[tool_router]
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

        // Build the entity graph for query support
        let mut graph = EntityGraph::new();
        graph.add_entities(build.entities.clone()).map_err(|e| {
            WorkspaceError::ValidationError(workspace_path.clone(), format!("{:?}", e))
        })?;
        graph.build();

        debug!(
            "Workspace loaded: {} entities, {} schemas",
            build.entities.len(),
            build.schemas.len()
        );

        Ok(Self {
            workspace_path,
            state: Arc::new(Mutex::new(ServerState {
                workspace,
                build,
                graph,
            })),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(
        description = "List all entity IDs of a given type, or all schema names if type is 'schema'. \
        Returns only IDs/names for discovery purposes. Use 'get' to retrieve full details for a specific entity or schema, \
        or use 'query' to fetch details for multiple entities matching search criteria."
    )]
    async fn list(
        &self,
        Parameters(params): Parameters<ListParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: list, type={}", params.r#type);
        let state = self.state.lock().await;
        Ok(tools::list::execute(&state.build, &params))
    }

    #[tool(description = "Get full details of a single entity or schema. \
        For entities: provide the entity type (e.g., 'person') and ID (e.g., 'john_doe'). \
        For schemas: use type='schema' and id=<schema_name> (e.g., id='person'). \
        Returns all fields and their values. Use 'list' first to discover available IDs.")]
    async fn get(
        &self,
        Parameters(params): Parameters<GetParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: get, type={}, id={}", params.r#type, params.id);
        let state = self.state.lock().await;
        Ok(tools::get::execute(&state.build, &params))
    }

    #[tool(
        description = "Query entities using the Firm query language. Returns full details for all matching entities. \
        Examples: 'from person', 'from task | where is_completed == false', 'from person | where name contains \"John\" | limit 5'. \
        Use 'list' for a simple ID overview, or 'get' for a single entity's details."
    )]
    async fn query(
        &self,
        Parameters(params): Parameters<QueryParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: query, query={}", params.query);
        let state = self.state.lock().await;
        Ok(tools::query::execute(&state.graph, &params))
    }

    #[tool(description = "Get IDs of entities related to a specific entity. \
        Returns entity IDs that reference or are referenced by the given entity. \
        Use 'direction' to filter: 'incoming' (entities that reference this one), \
        'outgoing' (entities this one references), or omit for both.")]
    async fn related(
        &self,
        Parameters(params): Parameters<RelatedParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Tool: related, type={}, id={}, direction={:?}",
            params.r#type, params.id, params.direction
        );
        let state = self.state.lock().await;
        Ok(tools::related::execute(&state.graph, &params))
    }

    #[tool(description = "Add a new entity to the workspace. \
        Provide the entity type, ID, and a map of field values (JSON types). \
        The tool validates the entity against the schema, generates the DSL, and writes it to a file. \
        Use this to safely create new entities without writing raw DSL.")]
    async fn add_entity(
        &self,
        Parameters(params): Parameters<AddEntityParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: add_entity, type={}, id={}", params.r#type, params.id);
        let state = self.state.lock().await;
        Ok(
            tools::add_entity::execute(&self.workspace_path, &state.build, &state.graph, &params)
                .map(tools::add_entity::success_result)
                .unwrap_or_else(|e| tools::build::error_result(&e)),
        )
    }

    #[tool(description = "Find the source file path for an entity or schema. \
        Returns the relative path to the .firm file containing the definition. \
        Use this to locate where an entity or schema is defined before reading or editing the source file.")]
    async fn find_source(
        &self,
        Parameters(params): Parameters<FindSourceParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Tool: find_source, type={}, id={}",
            params.r#type, params.id
        );
        let state = self.state.lock().await;
        Ok(tools::find_source::execute(
            &state.workspace,
            &self.workspace_path,
            &params,
        ))
    }

    #[tool(description = "Read the raw DSL content of a .firm source file. \
        Provide the relative path to the file (e.g., 'schemas/person.firm', 'core/main.firm'). \
        Use 'find_source' first to locate the file path for a specific entity or schema.")]
    async fn read_source(
        &self,
        Parameters(params): Parameters<ReadSourceParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: read_source, path={}", params.path);
        Ok(tools::read_source::execute(&self.workspace_path, &params))
    }

    #[tool(description = "Write DSL content to a .firm source file. \
        The content is validated for correct syntax and semantics (references, schema conformance). \
        If validation fails, changes are rolled back unless 'force' is true. \
        Use 'find_source' to locate existing files, or provide a new path to create a new file. \
        Use 'force: true' to fix a broken workspace where normal writes would be rolled back.")]
    async fn write_source(
        &self,
        Parameters(params): Parameters<WriteSourceParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Tool: write_source, path={}, content_len={}, force={}",
            params.path,
            params.content.len(),
            params.force
        );

        // Validate syntax and write the file
        let write_result =
            match tools::write_source::validate_and_write(&self.workspace_path, &params) {
                Ok(result) => result,
                Err(error_result) => return Ok(error_result),
            };

        // Try to rebuild the workspace (semantic validation)
        match self.rebuild().await {
            Ok(_) => {
                // Success - workspace is valid
                Ok(tools::write_source::success_result(
                    &params.path,
                    params.content.len(),
                    write_result.file_existed,
                ))
            }
            Err(e) => {
                if params.force {
                    // Force mode: keep the file, report the validation error
                    Ok(tools::write_source::force_success_result(
                        &params.path,
                        params.content.len(),
                        write_result.file_existed,
                        &e.to_string(),
                    ))
                } else {
                    // Normal mode: rollback the file change
                    let rollback_success = tools::write_source::rollback(
                        &self.workspace_path,
                        &params.path,
                        write_result.original_content,
                    );
                    Ok(tools::write_source::validation_error_result(
                        &e.to_string(),
                        rollback_success,
                    ))
                }
            }
        }
    }

    #[tool(description = "Rebuild and validate the workspace. \
        Returns the current status: number of entities and schemas if valid, \
        or validation errors if the workspace is broken. \
        Use this to check workspace health or refresh state after external changes.")]
    async fn build(
        &self,
        #[allow(unused_variables)] Parameters(params): Parameters<BuildParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: build");

        match self.rebuild().await {
            Ok(_) => {
                let state = self.state.lock().await;
                Ok(tools::build::success_result(
                    state.build.entities.len(),
                    state.build.schemas.len(),
                ))
            }
            Err(e) => Ok(tools::build::error_result(&e.to_string())),
        }
    }

    #[tool(
        description = "Get reference documentation for the Firm DSL syntax and query language. \
        Use 'topic' parameter: 'dsl' for DSL syntax (entities, schemas, field types), \
        'query' for query language (from, where, related, order, limit), \
        or 'all' for both (default). \
        Call this before writing or modifying .firm files to understand the correct syntax."
    )]
    async fn dsl_reference(
        &self,
        Parameters(params): Parameters<DslReferenceParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: dsl_reference, topic={}", params.topic);
        Ok(tools::dsl_reference::execute(&params))
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
    async fn rebuild(&self) -> Result<(), WorkspaceError> {
        debug!("Rebuilding workspace");
        let mut state = self.state.lock().await;

        let mut workspace = Workspace::new();
        workspace.load_directory(&self.workspace_path)?;
        let build = workspace.build()?;

        // Rebuild the entity graph
        let mut graph = EntityGraph::new();
        graph.add_entities(build.entities.clone()).map_err(|e| {
            WorkspaceError::ValidationError(self.workspace_path.clone(), format!("{:?}", e))
        })?;
        graph.build();

        state.workspace = workspace;
        state.build = build;
        state.graph = graph;

        debug!(
            "Workspace rebuilt: {} entities, {} schemas",
            state.build.entities.len(),
            state.build.schemas.len()
        );

        Ok(())
    }
}

#[tool_handler]
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
                "Firm MCP server. Use 'list schema' to explore available entity types. \
                 Use 'add_entity' to create new entities. \
                 Use 'query', 'list', and 'get' to explore existing data. \
                 Use 'read_source' and 'write_source' for low-level file operations."
                    .into(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        debug!("Listing resources");
        let state = self.state.lock().await;

        // Collect all source file resources
        let mut resource_list: Vec<Resource> = state
            .workspace
            .file_paths()
            .iter()
            .filter_map(|path| {
                resources::to_relative_path(&self.workspace_path, path)
                    .map(|rel| resources::source_file_resource(&rel))
            })
            .collect();

        // Sort by name for consistent ordering
        resource_list.sort_by(|a, b| a.name.cmp(&b.name));

        debug!("Found {} source file resources", resource_list.len());

        Ok(ListResourcesResult {
            resources: resource_list,
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;
        debug!("Reading resource: {}", uri);

        // Parse the URI to get the relative path
        let relative_path = resources::parse_source_uri(uri).ok_or_else(|| {
            McpError::resource_not_found(format!("Invalid resource URI: {}", uri), None)
        })?;

        // Read the file contents
        let contents = resources::read_source_file(&self.workspace_path, &relative_path)
            .map_err(|e| McpError::resource_not_found(e, None))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(contents, uri.clone())],
        })
    }
}
