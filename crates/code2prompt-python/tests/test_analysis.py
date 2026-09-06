"""Tests for owned Python projections of core analysis results."""

from code2prompt_rs import (
    Code2PromptConfig,
    Code2PromptSession,
    CodebaseAnalysis,
    ExtensionStat,
    TokenMapEntry,
    TokenMapOptions,
)


def test_raw_and_contextual_analysis(project):
    session = Code2PromptSession(
        Code2PromptConfig(str(project), include_patterns=["**/*.py"])
    )
    session.load_codebase()

    raw = session.raw_analysis()
    assert isinstance(raw, CodebaseAnalysis)
    assert all(file.token_count > 0 for file in raw.raw_files())

    rendered = session.generate_prompt()
    contextual = session.contextual_analysis(rendered)
    assert isinstance(contextual, CodebaseAnalysis)
    assert len(contextual.raw_files()) == len(rendered.files)


def test_extension_and_token_map_models(project):
    session = Code2PromptSession(
        Code2PromptConfig(
            str(project),
            include_patterns=["**/*.py", "README.md"],
        )
    )
    session.load_codebase()
    analysis = session.raw_analysis()
    assert analysis is not None

    extension_stats = analysis.by_extension()
    assert all(isinstance(item, ExtensionStat) for item in extension_stats)
    assert {item.extension for item in extension_stats} == {"py", "md"}

    entries = analysis.token_map(TokenMapOptions(max_lines=4, min_percent=0.0))
    assert entries
    assert all(isinstance(item, TokenMapEntry) for item in entries)
    assert all(item.tokens >= 0 for item in entries)
    assert all(isinstance(item.metadata.is_dir, bool) for item in entries)
