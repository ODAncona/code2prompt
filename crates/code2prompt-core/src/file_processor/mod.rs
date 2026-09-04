//! File processor module for handling different file types intelligently.
//!
//! This module provides a strategy pattern for processing file contents based on their extension
//! in order to optimize for LLM token usage. The main idea is to extract the schema rather than
//! raw data where applicable. (e.g., schema + sample for CSV, code cells for Jupyter notebooks).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

mod csv;
mod default;
mod ipynb;
mod jsonl;
mod tsv;

pub use csv::CsvProcessor;
pub use default::DefaultTextProcessor;
pub use ipynb::JupyterNotebookProcessor;
pub use jsonl::JsonLinesProcessor;
pub use tsv::TsvProcessor;

/// Configuration for the Jupyter notebook (`.ipynb`) file processor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct IpynbProcessorConfig {
    /// Maximum number of code cells to include in the processed output.
    ///
    /// Default: `3`
    pub max_code_cells: usize,

    /// When true, include cell outputs (stdout / text/plain / errors) after each code cell.
    ///
    /// Default: `false`
    pub include_outputs: bool,

    /// When true, include markdown cells in the processed output.
    ///
    /// Default: `false`
    pub include_markdown: bool,
}

impl Default for IpynbProcessorConfig {
    fn default() -> Self {
        Self {
            max_code_cells: 3,
            include_outputs: false,
            include_markdown: false,
        }
    }
}

/// Configuration for all file processors.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct FileProcessorsConfig {
    /// Jupyter notebook processor settings.
    pub ipynb: IpynbProcessorConfig,
}

/// Trait for processing file contents into LLM-optimized string representations.
///
/// Each processor takes raw bytes and produces a formatted string suitable for
/// inclusion in an LLM prompt. Processors may extract schemas, truncate content,
/// or apply other transformations to reduce token usage while preserving semantic value.
pub trait FileProcessor: Send + Sync {
    /// Process file content and return a formatted string.
    ///
    /// # Arguments
    ///
    /// * `content` - Raw file bytes
    /// * `path` - File path for context and error messages
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Processed content or error
    fn process(&self, content: &[u8], path: &Path) -> Result<String>;
}

/// Factory function to get the appropriate processor for a file extension.
///
/// # Arguments
///
/// * `extension` - File extension (without dot)
/// * `processors` - Processor configuration (used by configurable processors such as ipynb)
///
/// # Returns
///
/// * `Box<dyn FileProcessor>` - Processor instance for the given extension
pub fn get_processor_for_extension(
    extension: &str,
    processors: &FileProcessorsConfig,
) -> Box<dyn FileProcessor> {
    match extension.to_lowercase().as_str() {
        "csv" => Box::new(CsvProcessor),
        "tsv" => Box::new(TsvProcessor),
        "jsonl" | "ndjson" => Box::new(JsonLinesProcessor),
        "ipynb" => Box::new(JupyterNotebookProcessor::new(processors.ipynb.clone())),
        // Future processors can be added here:
        // "parquet" => Box::new(ParquetProcessor),
        // "xml" => Box::new(XmlProcessor),
        _ => Box::new(DefaultTextProcessor),
    }
}
