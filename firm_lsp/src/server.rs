//! LSP server implementation for Firm DSL.

use std::path::PathBuf;

use log::info;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use firm_lang::diagnostics::{self, Diagnostic, DiagnosticSeverity, SourceSpan};
use firm_lang::parser::dsl::parse_source;

/// The Firm language server.
pub struct FirmLspServer {
    client: Client,
    workspace_path: PathBuf,
}

impl FirmLspServer {
    pub fn new(client: Client, workspace_path: PathBuf) -> Self {
        Self {
            client,
            workspace_path,
        }
    }

    /// Start the language server on stdio.
    pub async fn serve_stdio(workspace_path: PathBuf) -> std::result::Result<(), String> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) =
            LspService::new(|client| FirmLspServer::new(client, workspace_path));
        Server::new(stdin, stdout, socket).serve(service).await;
        Ok(())
    }

    /// Publish diagnostics for a single document.
    async fn publish_diagnostics(&self, uri: Uri, text: &str) {
        let file_path = uri_to_path(&uri).unwrap_or_default();

        let parsed = match parse_source(text.to_string(), Some(file_path)) {
            Ok(parsed) => parsed,
            Err(e) => {
                log::error!("Failed to parse source: {e}");
                return;
            }
        };

        let firm_diagnostics = diagnostics::collect_syntax_errors(&parsed);
        let lsp_diagnostics: Vec<tower_lsp_server::lsp_types::Diagnostic> = firm_diagnostics
            .iter()
            .map(to_lsp_diagnostic)
            .collect();

        self.client
            .publish_diagnostics(uri, lsp_diagnostics, None)
            .await;
    }
}

impl LanguageServer for FirmLspServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("Firm language server initializing");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Firm language server initialized");
        self.client
            .log_message(MessageType::INFO, "Firm language server ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Firm language server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.publish_diagnostics(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // FULL sync mode: last content change has the full text
        if let Some(change) = params.content_changes.last() {
            self.publish_diagnostics(params.text_document.uri, &change.text)
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = &params.text {
            self.publish_diagnostics(params.text_document.uri, text)
                .await;
        }
    }
}

/// Convert a firm Diagnostic to an LSP Diagnostic.
fn to_lsp_diagnostic(d: &Diagnostic) -> tower_lsp_server::lsp_types::Diagnostic {
    tower_lsp_server::lsp_types::Diagnostic {
        range: span_to_range(&d.span),
        severity: Some(match d.severity {
            DiagnosticSeverity::Error => tower_lsp_server::lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticSeverity::Warning => {
                tower_lsp_server::lsp_types::DiagnosticSeverity::WARNING
            }
        }),
        source: Some("firm".to_string()),
        message: d.message.clone(),
        ..Default::default()
    }
}

/// Convert a SourceSpan to an LSP Range.
fn span_to_range(span: &SourceSpan) -> Range {
    Range {
        start: Position {
            line: span.start_line,
            character: span.start_col,
        },
        end: Position {
            line: span.end_line,
            character: span.end_col,
        },
    }
}

/// Extract a file path from a document URI.
fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    let s = uri.as_str();
    if let Some(path) = s.strip_prefix("file://") {
        Some(PathBuf::from(path))
    } else {
        None
    }
}
