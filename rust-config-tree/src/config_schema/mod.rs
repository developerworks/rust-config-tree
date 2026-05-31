//! JSON Schema generation and section-schema splitting.
//!
//! `schemars` produces one full schema for the root config type. This module
//! removes constraints that do not fit partial config files, strips internal
//! marker metadata from the emitted JSON, and optionally emits separate schemas
//! for marked nested sections.

pub mod adapt;
pub mod generate;
pub mod marker;
pub mod paths;
pub mod reference;
pub mod target;
pub mod write;
