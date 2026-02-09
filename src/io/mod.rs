//! Input output source and target definitions
//!
//! This module defines strongly typed enums for all supported
//! input sources and output targets.

pub mod input;
pub mod output;

pub use input::{FileInput, InputSource, NetworkInput};
pub use output::{FileOutput, NetworkOutput, OutputTarget};
