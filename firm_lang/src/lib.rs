//! Parsing and workspace management for the Firm DSL.
//!
//! This crate handles loading `.firm` files, parsing them into an abstract
//! representation, and converting them to Firm's core data structures.

pub mod convert;
pub mod defaults;
pub mod generate;
pub mod parser;
pub mod workspace;
