//! Jupyter Notebook (.ipynb) file processor.
//!
//! This processor parses Jupyter notebook JSON and extracts:
//! - Total number of cells and their types
//! - Code cells (and optionally markdown cells)
//! - A configurable number of sample cells
//! - Optional cell outputs
//!
//! This provides LLMs with notebook structure context without overwhelming them with all cells.

use super::{DefaultTextProcessor, FileProcessor, IpynbProcessorConfig};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

/// Jupyter Notebook processor that extracts code cells and metadata.
pub struct JupyterNotebookProcessor {
    config: IpynbProcessorConfig,
}

impl Default for JupyterNotebookProcessor {
    fn default() -> Self {
        Self {
            config: IpynbProcessorConfig::default(),
        }
    }
}

impl JupyterNotebookProcessor {
    /// Create a processor with the given configuration.
    pub fn new(config: IpynbProcessorConfig) -> Self {
        Self { config }
    }

    fn extract_source(source: &Value) -> String {
        match source {
            Value::String(s) => s.clone(),
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(""),
            _ => String::from("(Unable to extract source)"),
        }
    }

    fn append_outputs(output: &mut String, cell: &Value) {
        let Some(outputs) = cell.get("outputs").and_then(|v| v.as_array()) else {
            return;
        };
        if outputs.is_empty() {
            return;
        }

        output.push_str("Output:\n");
        for out in outputs {
            let output_type = out.get("output_type").and_then(|v| v.as_str()).unwrap_or("");
            match output_type {
                "stream" => {
                    if let Some(text) = out.get("text") {
                        let text = Self::extract_source(text);
                        output.push_str(&text);
                        if !text.ends_with('\n') {
                            output.push('\n');
                        }
                    }
                }
                "execute_result" | "display_data" => {
                    if let Some(data) = out.get("data")
                        && let Some(text) = data.get("text/plain")
                    {
                        let text = Self::extract_source(text);
                        output.push_str(&text);
                        if !text.ends_with('\n') {
                            output.push('\n');
                        }
                    }
                }
                "error" => {
                    let ename = out
                        .get("ename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Error");
                    let evalue = out.get("evalue").and_then(|v| v.as_str()).unwrap_or("");
                    output.push_str(&format!("{}: {}\n", ename, evalue));
                }
                _ => {}
            }
        }
        output.push('\n');
    }
}

impl FileProcessor for JupyterNotebookProcessor {
    fn process(&self, content: &[u8], _path: &Path) -> Result<String> {
        // Parse notebook JSON
        let notebook: Value =
            serde_json::from_slice(content).context("Failed to parse .ipynb file as JSON")?;

        // Extract cells array
        let cells = notebook
            .get("cells")
            .and_then(|v| v.as_array())
            .context("Notebook has no 'cells' array")?;

        // Count cell types and collect code / markdown cells
        let mut code_cells = Vec::new();
        let mut markdown_cells = Vec::new();
        let mut markdown_count = 0;
        let mut raw_count = 0;

        for cell in cells {
            let cell_type = cell
                .get("cell_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            match cell_type {
                "code" => code_cells.push(cell),
                "markdown" => {
                    markdown_count += 1;
                    markdown_cells.push(cell);
                }
                "raw" => raw_count += 1,
                _ => {}
            }
        }

        let total_cells = cells.len();

        // Format output
        let mut output = String::new();
        output.push_str("Jupyter Notebook Summary:\n");
        output.push_str(&format!(
            "Total cells: {} ({} code, {} markdown, {} raw)\n\n",
            total_cells,
            code_cells.len(),
            markdown_count,
            raw_count
        ));

        if self.config.include_markdown {
            for (idx, cell) in markdown_cells.iter().enumerate() {
                output.push_str(&format!("Markdown Cell #{}:\n", idx + 1));
                if let Some(source) = cell.get("source") {
                    let text = Self::extract_source(source);
                    output.push_str(&text);
                    if !text.ends_with('\n') {
                        output.push('\n');
                    }
                    output.push('\n');
                }
            }
        }

        if code_cells.is_empty() {
            output.push_str("(No code cells found)\n");
            return Ok(output);
        }

        let max_cells_to_show = self.config.max_code_cells.min(code_cells.len());

        for (idx, cell) in code_cells.iter().take(max_cells_to_show).enumerate() {
            output.push_str(&format!("Code Cell #{}:\n", idx + 1));

            // Extract source code
            if let Some(source) = cell.get("source") {
                let code = Self::extract_source(source);

                output.push_str("```python\n");
                output.push_str(&code);
                if !code.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str("```\n\n");
            }

            if self.config.include_outputs {
                Self::append_outputs(&mut output, cell);
            }
        }

        if code_cells.len() > max_cells_to_show {
            output.push_str(&format!(
                "... [{} more code cells omitted]\n",
                code_cells.len() - max_cells_to_show
            ));
        }

        Ok(output)
    }
}

impl JupyterNotebookProcessor {
    /// Process with fallback to raw text on error.
    pub fn process_with_fallback(&self, content: &[u8], path: &Path) -> Result<String> {
        match self.process(content, path) {
            Ok(result) => Ok(result),
            Err(e) => {
                log::warn!(
                    "Jupyter notebook parsing failed for {:?}: {}. Using raw text fallback.",
                    path,
                    e
                );
                let fallback = DefaultTextProcessor;
                fallback.process(content, path)
            }
        }
    }
}
