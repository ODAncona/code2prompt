"""Tests for prompt generation through the bound Rust session."""

import json

from code2prompt_rs import (
    Code2PromptConfig,
    Code2PromptSession,
    OutputFormat,
    RenderedPrompt,
    TokenizerType,
)


def generate(project, **options):
    return Code2PromptSession(Code2PromptConfig(str(project), **options)).generate_prompt()


def test_generate_prompt_returns_core_result(project):
    result = generate(project, include_patterns=["**/*.py"])

    assert isinstance(result, RenderedPrompt)
    assert isinstance(result.prompt, str)
    assert result.token_count > 0
    assert result.directory_name == project.name
    assert result.model_info
    assert sorted(result.files) == ["src/main.py", "src/utils.py", "tests/test_main.py"]


def test_filtering_hidden_ignore_and_path_modes(project):
    default_result = generate(project)
    assert "secret.py" not in default_result.prompt
    assert "ignored.txt" not in default_result.files

    expanded_result = generate(
        project,
        hidden=True,
        no_ignore=True,
        absolute_path=True,
        include_patterns=["**/*.py", "**/*.txt"],
    )
    assert "secret.py" in expanded_result.prompt
    assert "ignored.txt" in expanded_result.prompt
    assert str(project / "src" / "main.py") in expanded_result.files


def test_line_numbers_and_code_block_control(project):
    with_blocks = generate(
        project,
        include_patterns=["src/main.py"],
        line_numbers=True,
    )
    assert "   1 | def main():" in with_blocks.prompt
    assert "```py" in with_blocks.prompt

    without_blocks = generate(
        project,
        include_patterns=["src/main.py"],
        no_codeblock=True,
    )
    assert "def main():" in without_blocks.prompt
    assert "```py" not in without_blocks.prompt


def test_custom_handlebars_template_and_variables(project):
    result = generate(
        project,
        include_patterns=["src/main.py"],
        template_name="custom",
        template_str="Audience={{audience}};{{#each files}}{{path}}={{code}}{{/each}}",
        user_variables={"audience": "maintainers"},
    )
    assert result.prompt.startswith("Audience=maintainers;")
    assert "src/main.py=def main():" in result.prompt


def test_xml_and_json_output_formats(project):
    xml_result = generate(
        project,
        include_patterns=["src/main.py"],
        output_format=OutputFormat.Xml,
    )
    assert "<directory>" in xml_result.prompt
    assert '<file path="src/main.py">' in xml_result.prompt

    json_result = generate(
        project,
        include_patterns=["src/main.py"],
        output_format=OutputFormat.Json,
    )
    document = json.loads(json_result.prompt)
    assert document["files"] == ["src/main.py"]
    assert document["token_count"] == json_result.token_count


def test_all_tokenizers_generate_counts(project):
    counts = [
        generate(
            project,
            include_patterns=["src/main.py"],
            encoding=tokenizer,
        ).token_count
        for tokenizer in (
            TokenizerType.Cl100kBase,
            TokenizerType.O200kBase,
            TokenizerType.P50kBase,
            TokenizerType.P50kEdit,
            TokenizerType.R50kBase,
        )
    ]
    assert all(count > 0 for count in counts)
