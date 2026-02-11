//! Language server for the Firm DSL.
//!
//! Provides LSP support for `.firm` files, including syntax error
//! diagnostics with inline editor feedback.

mod server;

pub use server::FirmLspServer;
