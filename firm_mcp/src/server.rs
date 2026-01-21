//! Core MCP server implementation for Firm.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use log::debug;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_handler,
    tool_router, transport::stdio,
};
use tokio::sync::Mutex;

use firm_lang::workspace::{Workspace, WorkspaceBuild, WorkspaceError};

use firm_core::compose_entity_id;
use firm_core::graph::{EntityGraph, Query};
use firm_lang::parser::query::parse_query;

use crate::resources;
use crate::tools::{
    BuildParams, FindSourceParams, GetParams, ListParams, QueryParams, ReadSourceParams,
    WriteSourceParams,
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
struct ServerState {
    workspace: Workspace,
    build: WorkspaceBuild,
    graph: EntityGraph,
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

        let result = if params.r#type == "schema" {
            // List all schema names
            let names: Vec<&str> = state
                .build
                .schemas
                .iter()
                .map(|s| s.entity_type.as_str())
                .collect();
            names.join("\n")
        } else {
            // List all entity IDs of the given type
            let ids: Vec<&str> = state
                .build
                .entities
                .iter()
                .filter(|e| e.entity_type.as_str() == params.r#type)
                .map(|e| e.id.as_str())
                .collect();
            ids.join("\n")
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
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

        if params.r#type == "schema" {
            // Get schema by name
            let schema = state
                .build
                .schemas
                .iter()
                .find(|s| s.entity_type.as_str() == params.id);

            match schema {
                Some(schema) => Ok(CallToolResult::success(vec![Content::text(
                    schema.to_string(),
                )])),
                None => Ok(CallToolResult::error(vec![Content::text(format!(
                    "Schema '{}' not found. Use list with type='schema' to see available schemas.",
                    params.id
                ))])),
            }
        } else {
            // Get entity by type and ID
            let id = compose_entity_id(&params.r#type, &params.id);
            match state.build.entities.iter().find(|e| e.id == id) {
                Some(entity) => Ok(CallToolResult::success(vec![Content::text(
                    entity.to_string(),
                )])),
                None => Ok(CallToolResult::error(vec![Content::text(format!(
                    "Entity '{}' with type '{}' not found. Use list with type='{}' to see available IDs.",
                    params.id, params.r#type, params.r#type
                ))])),
            }
        }
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

        // Parse the query
        let parsed_query = match parse_query(&params.query) {
            Ok(q) => q,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to parse query: {}",
                    e
                ))]));
            }
        };

        // Convert to executable query
        let query: Query = match parsed_query.try_into() {
            Ok(q) => q,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to convert query: {}",
                    e
                ))]));
            }
        };

        // Execute the query
        let results = query.execute(&state.graph);

        // Format results
        if results.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No entities found matching the query.",
            )]));
        }

        let output: Vec<String> = results.iter().map(|e| e.to_string()).collect();
        Ok(CallToolResult::success(vec![Content::text(
            output.join("\n---\n"),
        )]))
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

        let source_path = if params.r#type == "schema" {
            state.workspace.find_schema_source(&params.id)
        } else {
            state
                .workspace
                .find_entity_source(&params.r#type, &params.id)
        };

        match source_path {
            Some(path) => {
                let relative = resources::to_relative_path(&self.workspace_path, &path)
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                Ok(CallToolResult::success(vec![Content::text(relative)]))
            }
            None => {
                let msg = if params.r#type == "schema" {
                    format!(
                        "Schema '{}' not found. Use list with type='schema' to see available schemas.",
                        params.id
                    )
                } else {
                    format!(
                        "Entity '{}' with type '{}' not found. Use list with type='{}' to see available IDs.",
                        params.id, params.r#type, params.r#type
                    )
                };
                Ok(CallToolResult::error(vec![Content::text(msg)]))
            }
        }
    }

    #[tool(description = "Read the raw DSL content of a .firm source file. \
        Provide the relative path to the file (e.g., 'schemas/person.firm', 'core/main.firm'). \
        Use 'find_source' first to locate the file path for a specific entity or schema.")]
    async fn read_source(
        &self,
        Parameters(params): Parameters<ReadSourceParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Tool: read_source, path={}", params.path);

        match resources::read_source_file(&self.workspace_path, &params.path) {
            Ok(contents) => Ok(CallToolResult::success(vec![Content::text(contents)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
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

        // First, validate the content by parsing it (syntax check - always required)
        let parsed = match firm_lang::parser::dsl::parse_source(params.content.clone(), None) {
            Ok(parsed) => parsed,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to parse DSL: {}",
                    e
                ))]));
            }
        };

        // Check for syntax errors in the parse tree (always required, even with force)
        if parsed.has_error() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Invalid DSL syntax: the content contains parse errors. \
                 Please check for unclosed braces, missing values, or malformed references.",
            )]));
        }

        // Get absolute path for the file
        let absolute_path = self.workspace_path.join(&params.path);

        // Read existing file content for potential rollback (None if file doesn't exist)
        let original_content = fs::read_to_string(&absolute_path).ok();
        let file_existed = original_content.is_some();

        // Write the new content
        if let Err(e) =
            resources::write_source_file(&self.workspace_path, &params.path, &params.content)
        {
            return Ok(CallToolResult::error(vec![Content::text(e)]));
        }

        // Try to rebuild the workspace (semantic validation)
        let action = if file_existed { "Updated" } else { "Created" };

        match self.rebuild().await {
            Ok(_) => {
                // Success - workspace is valid
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "{} {} ({} bytes). Workspace is valid.",
                    action,
                    params.path,
                    params.content.len()
                ))]))
            }
            Err(e) => {
                if params.force {
                    // Force mode: keep the file, report the validation error
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "{} {} ({} bytes). Warning: workspace has validation errors: {}. \
                         Use 'build' to check status after making more changes.",
                        action,
                        params.path,
                        params.content.len(),
                        e
                    ))]))
                } else {
                    // Normal mode: rollback the file change
                    let rollback_result = if let Some(original) = original_content {
                        fs::write(&absolute_path, original)
                    } else {
                        fs::remove_file(&absolute_path)
                    };

                    let rollback_msg = match rollback_result {
                        Ok(_) => "Changes have been rolled back.",
                        Err(_) => "Warning: Failed to rollback changes.",
                    };

                    Ok(CallToolResult::error(vec![Content::text(format!(
                        "Validation failed: {}. {} Use 'force: true' to write anyway.",
                        e, rollback_msg
                    ))]))
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
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Workspace is valid. {} entities, {} schemas.",
                    state.build.entities.len(),
                    state.build.schemas.len()
                ))]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace validation failed: {}",
                e
            ))])),
        }
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
