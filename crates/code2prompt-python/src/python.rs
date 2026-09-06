use std::collections::HashMap;
use std::path::PathBuf;

use code2prompt_core::analysis::{
    CodebaseAnalysis, EntryMetadata as AnalysisEntryMetadata, ExtensionStat, TokenMapEntry,
    TokenMapOptions,
};
use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::file_processor::{FileProcessorsConfig, IpynbProcessorConfig};
use code2prompt_core::path::{EntryMetadata, FileEntry};
use code2prompt_core::session::{Code2PromptSession, RenderedPrompt, SessionData};
use code2prompt_core::sort::FileSortMethod;
use code2prompt_core::template::OutputFormat;
use code2prompt_core::tokenizer::{TokenFormat, TokenizerType};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

fn runtime_error(error: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

// -----------------------------------------------------------------------------
// Core enums
// -----------------------------------------------------------------------------

#[pyclass(name = "OutputFormat", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyOutputFormat {
    Markdown,
    Json,
    Xml,
}

impl From<PyOutputFormat> for OutputFormat {
    fn from(value: PyOutputFormat) -> Self {
        match value {
            PyOutputFormat::Markdown => Self::Markdown,
            PyOutputFormat::Json => Self::Json,
            PyOutputFormat::Xml => Self::Xml,
        }
    }
}

impl From<OutputFormat> for PyOutputFormat {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Markdown => Self::Markdown,
            OutputFormat::Json => Self::Json,
            OutputFormat::Xml => Self::Xml,
        }
    }
}

#[pyclass(name = "TokenizerType", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyTokenizerType {
    O200kBase,
    Cl100kBase,
    P50kBase,
    P50kEdit,
    R50kBase,
}

impl From<PyTokenizerType> for TokenizerType {
    fn from(value: PyTokenizerType) -> Self {
        match value {
            PyTokenizerType::O200kBase => Self::O200kBase,
            PyTokenizerType::Cl100kBase => Self::Cl100kBase,
            PyTokenizerType::P50kBase => Self::P50kBase,
            PyTokenizerType::P50kEdit => Self::P50kEdit,
            PyTokenizerType::R50kBase => Self::R50kBase,
        }
    }
}

impl From<TokenizerType> for PyTokenizerType {
    fn from(value: TokenizerType) -> Self {
        match value {
            TokenizerType::O200kBase => Self::O200kBase,
            TokenizerType::Cl100kBase => Self::Cl100kBase,
            TokenizerType::P50kBase => Self::P50kBase,
            TokenizerType::P50kEdit => Self::P50kEdit,
            TokenizerType::R50kBase => Self::R50kBase,
        }
    }
}

#[pyclass(name = "TokenFormat", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyTokenFormat {
    Raw,
    Format,
}

impl From<PyTokenFormat> for TokenFormat {
    fn from(value: PyTokenFormat) -> Self {
        match value {
            PyTokenFormat::Raw => Self::Raw,
            PyTokenFormat::Format => Self::Format,
        }
    }
}

impl From<TokenFormat> for PyTokenFormat {
    fn from(value: TokenFormat) -> Self {
        match value {
            TokenFormat::Raw => Self::Raw,
            TokenFormat::Format => Self::Format,
        }
    }
}

#[pyclass(name = "FileSortMethod", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyFileSortMethod {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
}

impl From<PyFileSortMethod> for FileSortMethod {
    fn from(value: PyFileSortMethod) -> Self {
        match value {
            PyFileSortMethod::NameAsc => Self::NameAsc,
            PyFileSortMethod::NameDesc => Self::NameDesc,
            PyFileSortMethod::DateAsc => Self::DateAsc,
            PyFileSortMethod::DateDesc => Self::DateDesc,
        }
    }
}

impl From<FileSortMethod> for PyFileSortMethod {
    fn from(value: FileSortMethod) -> Self {
        match value {
            FileSortMethod::NameAsc => Self::NameAsc,
            FileSortMethod::NameDesc => Self::NameDesc,
            FileSortMethod::DateAsc => Self::DateAsc,
            FileSortMethod::DateDesc => Self::DateDesc,
        }
    }
}

// -----------------------------------------------------------------------------
// Configuration
// -----------------------------------------------------------------------------

#[pyclass(name = "IpynbProcessorConfig", get_all, set_all, from_py_object)]
#[derive(Clone)]
struct PyIpynbProcessorConfig {
    max_code_cells: usize,
    include_outputs: bool,
    include_markdown: bool,
}

#[pymethods]
impl PyIpynbProcessorConfig {
    #[new]
    #[pyo3(signature = (*, max_code_cells=3, include_outputs=false, include_markdown=false))]
    fn new(max_code_cells: usize, include_outputs: bool, include_markdown: bool) -> Self {
        Self {
            max_code_cells,
            include_outputs,
            include_markdown,
        }
    }
}

impl From<&PyIpynbProcessorConfig> for IpynbProcessorConfig {
    fn from(value: &PyIpynbProcessorConfig) -> Self {
        Self {
            max_code_cells: value.max_code_cells,
            include_outputs: value.include_outputs,
            include_markdown: value.include_markdown,
        }
    }
}

impl From<&IpynbProcessorConfig> for PyIpynbProcessorConfig {
    fn from(value: &IpynbProcessorConfig) -> Self {
        Self {
            max_code_cells: value.max_code_cells,
            include_outputs: value.include_outputs,
            include_markdown: value.include_markdown,
        }
    }
}

#[pyclass(name = "FileProcessorsConfig", get_all, set_all, from_py_object)]
#[derive(Clone)]
struct PyFileProcessorsConfig {
    ipynb: PyIpynbProcessorConfig,
}

#[pymethods]
impl PyFileProcessorsConfig {
    #[new]
    #[pyo3(signature = (*, ipynb=None))]
    fn new(ipynb: Option<PyIpynbProcessorConfig>) -> Self {
        Self {
            ipynb: ipynb
                .unwrap_or_else(|| PyIpynbProcessorConfig::from(&IpynbProcessorConfig::default())),
        }
    }
}

impl From<&PyFileProcessorsConfig> for FileProcessorsConfig {
    fn from(value: &PyFileProcessorsConfig) -> Self {
        Self {
            ipynb: IpynbProcessorConfig::from(&value.ipynb),
        }
    }
}

impl From<&FileProcessorsConfig> for PyFileProcessorsConfig {
    fn from(value: &FileProcessorsConfig) -> Self {
        Self {
            ipynb: PyIpynbProcessorConfig::from(&value.ipynb),
        }
    }
}

#[pyclass(name = "Code2PromptConfig", get_all, set_all, from_py_object)]
#[derive(Clone)]
struct PyCode2PromptConfig {
    path: PathBuf,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    line_numbers: bool,
    absolute_path: bool,
    full_directory_tree: bool,
    no_codeblock: bool,
    follow_symlinks: bool,
    hidden: bool,
    no_ignore: bool,
    sort_method: Option<PyFileSortMethod>,
    output_format: PyOutputFormat,
    custom_template: Option<String>,
    encoding: PyTokenizerType,
    token_format: PyTokenFormat,
    diff_enabled: bool,
    diff_branches: Option<(String, String)>,
    log_branches: Option<(String, String)>,
    template_name: String,
    template_str: String,
    user_variables: HashMap<String, String>,
    token_map_enabled: bool,
    deselected: bool,
    processors: PyFileProcessorsConfig,
}

#[pymethods]
impl PyCode2PromptConfig {
    #[new]
    #[pyo3(signature = (
        path,
        *,
        include_patterns=None,
        exclude_patterns=None,
        line_numbers=false,
        absolute_path=false,
        full_directory_tree=false,
        no_codeblock=false,
        follow_symlinks=false,
        hidden=false,
        no_ignore=false,
        sort_method=None,
        output_format=None,
        custom_template=None,
        encoding=None,
        token_format=None,
        diff_enabled=false,
        diff_branches=None,
        log_branches=None,
        template_name=None,
        template_str=None,
        user_variables=None,
        token_map_enabled=false,
        deselected=false,
        processors=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        path: PathBuf,
        include_patterns: Option<Vec<String>>,
        exclude_patterns: Option<Vec<String>>,
        line_numbers: bool,
        absolute_path: bool,
        full_directory_tree: bool,
        no_codeblock: bool,
        follow_symlinks: bool,
        hidden: bool,
        no_ignore: bool,
        sort_method: Option<PyFileSortMethod>,
        output_format: Option<PyOutputFormat>,
        custom_template: Option<String>,
        encoding: Option<PyTokenizerType>,
        token_format: Option<PyTokenFormat>,
        diff_enabled: bool,
        diff_branches: Option<(String, String)>,
        log_branches: Option<(String, String)>,
        template_name: Option<String>,
        template_str: Option<String>,
        user_variables: Option<HashMap<String, String>>,
        token_map_enabled: bool,
        deselected: bool,
        processors: Option<PyFileProcessorsConfig>,
    ) -> Self {
        Self {
            path,
            include_patterns: include_patterns.unwrap_or_default(),
            exclude_patterns: exclude_patterns.unwrap_or_default(),
            line_numbers,
            absolute_path,
            full_directory_tree,
            no_codeblock,
            follow_symlinks,
            hidden,
            no_ignore,
            sort_method,
            output_format: output_format.unwrap_or(PyOutputFormat::Markdown),
            custom_template,
            encoding: encoding.unwrap_or(PyTokenizerType::Cl100kBase),
            token_format: token_format.unwrap_or(PyTokenFormat::Raw),
            diff_enabled,
            diff_branches,
            log_branches,
            template_name: template_name.unwrap_or_default(),
            template_str: template_str.unwrap_or_default(),
            user_variables: user_variables.unwrap_or_default(),
            token_map_enabled,
            deselected,
            processors: processors
                .unwrap_or_else(|| PyFileProcessorsConfig::from(&FileProcessorsConfig::default())),
        }
    }
}

impl From<&PyCode2PromptConfig> for Code2PromptConfig {
    fn from(value: &PyCode2PromptConfig) -> Self {
        Self {
            path: value.path.clone(),
            include_patterns: value.include_patterns.clone(),
            exclude_patterns: value.exclude_patterns.clone(),
            line_numbers: value.line_numbers,
            absolute_path: value.absolute_path,
            full_directory_tree: value.full_directory_tree,
            no_codeblock: value.no_codeblock,
            follow_symlinks: value.follow_symlinks,
            entity_map: false,
            hidden: value.hidden,
            no_ignore: value.no_ignore,
            sort_method: value.sort_method.map(Into::into),
            output_format: value.output_format.into(),
            custom_template: value.custom_template.clone(),
            encoding: value.encoding.into(),
            token_format: value.token_format.into(),
            diff_enabled: value.diff_enabled,
            diff_branches: value.diff_branches.clone(),
            diff_files: None,
            log_branches: value.log_branches.clone(),
            template_name: value.template_name.clone(),
            template_str: value.template_str.clone(),
            user_variables: value.user_variables.clone(),
            token_map_enabled: value.token_map_enabled,
            deselected: value.deselected,
            processors: FileProcessorsConfig::from(&value.processors),
        }
    }
}

impl From<&Code2PromptConfig> for PyCode2PromptConfig {
    fn from(value: &Code2PromptConfig) -> Self {
        Self {
            path: value.path.clone(),
            include_patterns: value.include_patterns.clone(),
            exclude_patterns: value.exclude_patterns.clone(),
            line_numbers: value.line_numbers,
            absolute_path: value.absolute_path,
            full_directory_tree: value.full_directory_tree,
            no_codeblock: value.no_codeblock,
            follow_symlinks: value.follow_symlinks,
            hidden: value.hidden,
            no_ignore: value.no_ignore,
            sort_method: value.sort_method.map(Into::into),
            output_format: value.output_format.into(),
            custom_template: value.custom_template.clone(),
            encoding: value.encoding.into(),
            token_format: value.token_format.into(),
            diff_enabled: value.diff_enabled,
            diff_branches: value.diff_branches.clone(),
            log_branches: value.log_branches.clone(),
            template_name: value.template_name.clone(),
            template_str: value.template_str.clone(),
            user_variables: value.user_variables.clone(),
            token_map_enabled: value.token_map_enabled,
            deselected: value.deselected,
            processors: PyFileProcessorsConfig::from(&value.processors),
        }
    }
}

// -----------------------------------------------------------------------------
// Session data and generated results
// -----------------------------------------------------------------------------

#[pyclass(name = "EntryMetadata", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
struct PyEntryMetadata {
    is_dir: bool,
    is_symlink: bool,
}

impl From<EntryMetadata> for PyEntryMetadata {
    fn from(value: EntryMetadata) -> Self {
        Self {
            is_dir: value.is_dir,
            is_symlink: value.is_symlink,
        }
    }
}

#[pyclass(name = "FileEntry", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
struct PyFileEntry {
    path: String,
    extension: String,
    code: String,
    token_count: usize,
    metadata: PyEntryMetadata,
    mod_time: Option<u64>,
}

impl From<&FileEntry> for PyFileEntry {
    fn from(value: &FileEntry) -> Self {
        Self {
            path: value.path.clone(),
            extension: value.extension.clone(),
            code: value.code.clone(),
            token_count: value.token_count,
            metadata: value.metadata.into(),
            mod_time: value.mod_time,
        }
    }
}

#[pyclass(name = "SessionData", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
struct PySessionData {
    absolute_code_path: Option<String>,
    source_tree: Option<String>,
    files: Option<Vec<PyFileEntry>>,
    git_diff: Option<String>,
    git_diff_branch: Option<String>,
    git_log_branch: Option<String>,
}

impl From<&SessionData> for PySessionData {
    fn from(value: &SessionData) -> Self {
        Self {
            absolute_code_path: value.absolute_code_path.clone(),
            source_tree: value.source_tree.clone(),
            files: value
                .files
                .as_deref()
                .map(|files| files.iter().map(PyFileEntry::from).collect()),
            git_diff: value.git_diff.clone(),
            git_diff_branch: value.git_diff_branch.clone(),
            git_log_branch: value.git_log_branch.clone(),
        }
    }
}

#[pyclass(name = "RenderedPrompt", frozen, from_py_object)]
#[derive(Clone)]
struct PyRenderedPrompt {
    inner: RenderedPrompt,
}

#[pymethods]
impl PyRenderedPrompt {
    #[getter]
    fn prompt(&self) -> &str {
        &self.inner.prompt
    }

    #[getter]
    fn directory_name(&self) -> &str {
        &self.inner.directory_name
    }

    #[getter]
    fn token_count(&self) -> usize {
        self.inner.token_count
    }

    #[getter]
    fn model_info(&self) -> &str {
        self.inner.model_info
    }

    #[getter]
    fn files(&self) -> Vec<String> {
        self.inner.files.clone()
    }
}

impl From<RenderedPrompt> for PyRenderedPrompt {
    fn from(value: RenderedPrompt) -> Self {
        Self { inner: value }
    }
}

// -----------------------------------------------------------------------------
// Analysis
// -----------------------------------------------------------------------------

#[pyclass(name = "TokenMapOptions", get_all, set_all, from_py_object)]
#[derive(Clone)]
struct PyTokenMapOptions {
    max_lines: usize,
    min_percent: f64,
}

#[pymethods]
impl PyTokenMapOptions {
    #[new]
    #[pyo3(signature = (*, max_lines=20, min_percent=0.1))]
    fn new(max_lines: usize, min_percent: f64) -> Self {
        Self {
            max_lines,
            min_percent,
        }
    }
}

impl From<&PyTokenMapOptions> for TokenMapOptions {
    fn from(value: &PyTokenMapOptions) -> Self {
        Self {
            max_lines: value.max_lines,
            min_percent: value.min_percent,
        }
    }
}

#[pyclass(name = "TokenMapEntryMetadata", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
struct PyTokenMapEntryMetadata {
    is_dir: bool,
}

impl From<AnalysisEntryMetadata> for PyTokenMapEntryMetadata {
    fn from(value: AnalysisEntryMetadata) -> Self {
        Self {
            is_dir: value.is_dir,
        }
    }
}

#[pyclass(name = "TokenMapEntry", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
struct PyTokenMapEntry {
    path: String,
    name: String,
    tokens: usize,
    percentage: f64,
    depth: usize,
    is_last_child: bool,
    has_children: bool,
    metadata: PyTokenMapEntryMetadata,
}

impl From<TokenMapEntry> for PyTokenMapEntry {
    fn from(value: TokenMapEntry) -> Self {
        Self {
            path: value.path,
            name: value.name,
            tokens: value.tokens,
            percentage: value.percentage,
            depth: value.depth,
            is_last_child: value.is_last_child,
            has_children: value.has_children,
            metadata: value.metadata.into(),
        }
    }
}

#[pyclass(name = "ExtensionStat", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
struct PyExtensionStat {
    extension: String,
    file_count: usize,
    tokens: usize,
    percentage: f64,
}

impl From<ExtensionStat> for PyExtensionStat {
    fn from(value: ExtensionStat) -> Self {
        Self {
            extension: value.extension,
            file_count: value.file_count,
            tokens: value.tokens,
            percentage: value.percentage,
        }
    }
}

#[pyclass(name = "CodebaseAnalysis", frozen)]
struct PyCodebaseAnalysis {
    files: Vec<FileEntry>,
    total_tokens: usize,
}

#[pymethods]
impl PyCodebaseAnalysis {
    fn token_map(&self, options: &PyTokenMapOptions) -> Vec<PyTokenMapEntry> {
        CodebaseAnalysis::new(&self.files, self.total_tokens)
            .token_map(options.into())
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn by_extension(&self) -> Vec<PyExtensionStat> {
        CodebaseAnalysis::new(&self.files, self.total_tokens)
            .by_extension()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn raw_files(&self) -> Vec<PyFileEntry> {
        self.files.iter().map(PyFileEntry::from).collect()
    }
}

// -----------------------------------------------------------------------------
// Stateful session
// -----------------------------------------------------------------------------

#[pyclass(name = "Code2PromptSession")]
struct PyCode2PromptSession {
    inner: Code2PromptSession,
}

#[pymethods]
impl PyCode2PromptSession {
    #[new]
    fn new(config: &PyCode2PromptConfig) -> Self {
        Self {
            inner: Code2PromptSession::new(config.into()),
        }
    }

    #[getter]
    fn config(&self) -> PyCode2PromptConfig {
        PyCode2PromptConfig::from(&self.inner.config)
    }

    #[getter]
    fn data(&self) -> PySessionData {
        PySessionData::from(&self.inner.data)
    }

    fn add_include_pattern<'py>(
        mut slf: PyRefMut<'py, Self>,
        pattern: String,
    ) -> PyRefMut<'py, Self> {
        slf.inner.add_include_pattern(pattern);
        slf
    }

    fn add_exclude_pattern<'py>(
        mut slf: PyRefMut<'py, Self>,
        pattern: String,
    ) -> PyRefMut<'py, Self> {
        slf.inner.add_exclude_pattern(pattern);
        slf
    }

    fn select_file<'py>(mut slf: PyRefMut<'py, Self>, path: PathBuf) -> PyRefMut<'py, Self> {
        slf.inner.select_file(path);
        slf
    }

    fn deselect_file<'py>(mut slf: PyRefMut<'py, Self>, path: PathBuf) -> PyRefMut<'py, Self> {
        slf.inner.deselect_file(path);
        slf
    }

    fn toggle_file_selection<'py>(
        mut slf: PyRefMut<'py, Self>,
        path: PathBuf,
    ) -> PyRefMut<'py, Self> {
        slf.inner.toggle_file_selection(path);
        slf
    }

    fn is_file_selected(&mut self, path: PathBuf) -> bool {
        self.inner.is_file_selected(&path)
    }

    fn get_selected_files(&mut self) -> PyResult<Vec<PathBuf>> {
        self.inner.get_selected_files().map_err(runtime_error)
    }

    fn clear_user_actions<'py>(mut slf: PyRefMut<'py, Self>) -> PyRefMut<'py, Self> {
        slf.inner.clear_user_actions();
        slf
    }

    fn has_user_actions(&self) -> bool {
        self.inner.has_user_actions()
    }

    fn set_deselected<'py>(mut slf: PyRefMut<'py, Self>, value: bool) -> PyRefMut<'py, Self> {
        slf.inner.set_deselected(value);
        slf
    }

    fn load_codebase(&mut self) -> PyResult<()> {
        self.inner.load_codebase().map_err(runtime_error)
    }

    fn load_git_diff(&mut self) -> PyResult<()> {
        self.inner.load_git_diff().map_err(runtime_error)
    }

    fn load_git_diff_between_branches(&mut self) -> PyResult<()> {
        self.inner
            .load_git_diff_between_branches()
            .map_err(runtime_error)
    }

    fn load_git_log_between_branches(&mut self) -> PyResult<()> {
        self.inner
            .load_git_log_between_branches()
            .map_err(runtime_error)
    }

    fn raw_analysis(&self) -> Option<PyCodebaseAnalysis> {
        self.inner.raw_analysis().map(|analysis| {
            let files = analysis.raw_files().to_vec();
            let total_tokens = files.iter().map(|file| file.token_count).sum();
            PyCodebaseAnalysis {
                files,
                total_tokens,
            }
        })
    }

    fn contextual_analysis(&self, prompt: &PyRenderedPrompt) -> Option<PyCodebaseAnalysis> {
        self.inner
            .contextual_analysis(&prompt.inner)
            .map(|analysis| PyCodebaseAnalysis {
                files: analysis.raw_files().to_vec(),
                total_tokens: prompt.inner.token_count,
            })
    }

    fn generate_prompt(&mut self) -> PyResult<PyRenderedPrompt> {
        self.inner
            .generate_prompt()
            .map(Into::into)
            .map_err(runtime_error)
    }
}

#[pymodule(name = "code2prompt_rs")]
fn code2prompt_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyOutputFormat>()?;
    m.add_class::<PyTokenizerType>()?;
    m.add_class::<PyTokenFormat>()?;
    m.add_class::<PyFileSortMethod>()?;
    m.add_class::<PyIpynbProcessorConfig>()?;
    m.add_class::<PyFileProcessorsConfig>()?;
    m.add_class::<PyCode2PromptConfig>()?;
    m.add_class::<PyEntryMetadata>()?;
    m.add_class::<PyFileEntry>()?;
    m.add_class::<PySessionData>()?;
    m.add_class::<PyRenderedPrompt>()?;
    m.add_class::<PyTokenMapOptions>()?;
    m.add_class::<PyTokenMapEntryMetadata>()?;
    m.add_class::<PyTokenMapEntry>()?;
    m.add_class::<PyExtensionStat>()?;
    m.add_class::<PyCodebaseAnalysis>()?;
    m.add_class::<PyCode2PromptSession>()?;
    Ok(())
}
