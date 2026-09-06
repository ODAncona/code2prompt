"""Tests for configurable file processors exposed through the core config."""

from code2prompt_rs import (
    Code2PromptConfig,
    Code2PromptSession,
    FileProcessorsConfig,
    IpynbProcessorConfig,
)


def test_notebook_processor_configuration_reaches_core(project):
    processors = FileProcessorsConfig(
        ipynb=IpynbProcessorConfig(
            max_code_cells=1,
            include_outputs=True,
            include_markdown=True,
        )
    )
    session = Code2PromptSession(
        Code2PromptConfig(
            str(project),
            include_patterns=["notebook.ipynb"],
            processors=processors,
        )
    )

    result = session.generate_prompt()
    assert "Markdown Cell #1:" in result.prompt
    assert "Code Cell #1:" in result.prompt
    assert "Output:\nhello" in result.prompt
    assert "[1 more code cells omitted]" in result.prompt
