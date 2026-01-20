//! MCP server for Firm workspaces.
//!
//! This crate provides an MCP (Model Context Protocol) server that exposes
//! Firm workspace operations to AI assistants like Claude.

mod server;

pub use server::FirmMcpServer;
