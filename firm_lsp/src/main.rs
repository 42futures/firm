mod server;

use log::info;
use server::FirmLanguageServer;
use tower_lsp_server::{LspService, Server};

#[tokio::main]
async fn main() {
    initialize_logging();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| FirmLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn initialize_logging() {
    let is_console = atty::is(atty::Stream::Stdin) || atty::is(atty::Stream::Stdout);

    if is_console {
        let mut builder = env_logger::Builder::from_default_env();
        builder
            .filter_level(log::LevelFilter::Info)
            .format_timestamp_secs()
            .init();

        info!("Firm LSP running in console");
    } else {
        let mut builder = env_logger::Builder::new();
        builder
            .filter_level(log::LevelFilter::Info)
            .format_timestamp_millis()
            .target(env_logger::Target::Stderr)
            .init();

        info!("Firm LSP running as language server");
    }
}
