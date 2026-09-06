"""Tests for direct bindings of core configuration types."""

import code2prompt_rs
import pytest
from code2prompt_rs import (
    Code2PromptConfig,
    Code2PromptSession,
    FileProcessorsConfig,
    FileSortMethod,
    IpynbProcessorConfig,
    OutputFormat,
    TokenFormat,
    TokenizerType,
)


def test_core_defaults_are_exposed(project):
    config = Code2PromptConfig(project)

    assert config.path == project
    assert config.include_patterns == []
    assert config.exclude_patterns == []
    assert config.output_format == OutputFormat.Markdown
    assert config.encoding == TokenizerType.Cl100kBase
    assert config.token_format == TokenFormat.Raw
    assert config.sort_method is None
    assert config.processors.ipynb.max_code_cells == 3
    assert config.processors.ipynb.include_outputs is False
    assert config.processors.ipynb.include_markdown is False


def test_all_user_facing_core_options_round_trip_through_session(project):
    processors = FileProcessorsConfig(
        ipynb=IpynbProcessorConfig(
            max_code_cells=7,
            include_outputs=True,
            include_markdown=True,
        )
    )
    config = Code2PromptConfig(
        str(project),
        include_patterns=["**/*.py"],
        exclude_patterns=["**/tests/**"],
        line_numbers=True,
        absolute_path=True,
        full_directory_tree=True,
        no_codeblock=True,
        follow_symlinks=True,
        hidden=True,
        no_ignore=True,
        sort_method=FileSortMethod.NameDesc,
        output_format=OutputFormat.Xml,
        custom_template="custom-template-path",
        encoding=TokenizerType.O200kBase,
        token_format=TokenFormat.Format,
        diff_enabled=True,
        diff_branches=("main", "feature"),
        log_branches=("main", "feature"),
        template_name="custom",
        template_str="{{absolute_code_path}}",
        user_variables={"audience": "maintainers"},
        token_map_enabled=True,
        deselected=True,
        processors=processors,
    )

    snapshot = Code2PromptSession(config).config
    assert snapshot.include_patterns == ["**/*.py"]
    assert snapshot.exclude_patterns == ["**/tests/**"]
    assert snapshot.line_numbers is True
    assert snapshot.absolute_path is True
    assert snapshot.full_directory_tree is True
    assert snapshot.no_codeblock is True
    assert snapshot.follow_symlinks is True
    assert snapshot.hidden is True
    assert snapshot.no_ignore is True
    assert snapshot.sort_method == FileSortMethod.NameDesc
    assert snapshot.output_format == OutputFormat.Xml
    assert snapshot.custom_template == "custom-template-path"
    assert snapshot.encoding == TokenizerType.O200kBase
    assert snapshot.token_format == TokenFormat.Format
    assert snapshot.diff_enabled is True
    assert snapshot.diff_branches == ("main", "feature")
    assert snapshot.log_branches == ("main", "feature")
    assert snapshot.template_name == "custom"
    assert snapshot.template_str == "{{absolute_code_path}}"
    assert snapshot.user_variables == {"audience": "maintainers"}
    assert snapshot.token_map_enabled is True
    assert snapshot.deselected is True
    assert snapshot.processors.ipynb.max_code_cells == 7


def test_configuration_requires_native_enums(project):
    with pytest.raises(TypeError):
        Code2PromptConfig(str(project), output_format="xml")


def test_removed_v3_facade_is_not_exported():
    assert not hasattr(code2prompt_rs, "Code2Prompt")
    assert hasattr(code2prompt_rs, "Code2PromptConfig")
    assert hasattr(code2prompt_rs, "Code2PromptSession")
