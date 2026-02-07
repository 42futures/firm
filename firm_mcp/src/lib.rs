//! MCP server for Firm workspaces.
//!
//! This crate provides an MCP (Model Context Protocol) server that exposes
//! Firm workspace operations to AI assistants like Claude.

pub mod resources;
mod server;
pub mod tools;

pub use server::FirmMcpServer;
