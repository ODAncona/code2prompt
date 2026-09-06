"""Tests for the stateful selection API exposed by Code2PromptSession."""

from pathlib import Path

from code2prompt_rs import Code2PromptConfig, Code2PromptSession, SessionData


def test_select_deselect_and_toggle_use_the_same_session(project):
    session = Code2PromptSession(
        Code2PromptConfig(str(project), exclude_patterns=["**/*"])
    )

    assert session.is_file_selected("src/main.py") is False
    assert session.get_selected_files() == []

    assert session.select_file("src/main.py") is session
    assert session.has_user_actions() is True
    assert session.is_file_selected("src/main.py") is True
    assert session.get_selected_files() == [Path("src/main.py")]

    assert session.toggle_file_selection("src/main.py") is session
    assert session.is_file_selected("src/main.py") is False
    assert session.deselect_file("src/utils.py") is session


def test_absolute_selection_is_normalized_to_relative_path(project):
    session = Code2PromptSession(Code2PromptConfig(str(project), deselected=True))
    absolute = str(project / "src" / "utils.py")

    session.select_file(absolute)
    assert session.is_file_selected(absolute) is True
    assert session.is_file_selected("src/utils.py") is True
    assert session.get_selected_files() == [Path("src/utils.py")]


def test_clear_actions_restores_pattern_selection(project):
    session = Code2PromptSession(
        Code2PromptConfig(str(project), exclude_patterns=["**/*"])
    )
    session.select_file("src/main.py")
    assert session.clear_user_actions() is session
    assert session.has_user_actions() is False
    assert session.get_selected_files() == []


def test_pattern_and_deselected_mutators_update_core_config(project):
    session = Code2PromptSession(Code2PromptConfig(str(project)))

    assert session.add_include_pattern("**/*.py") is session
    assert session.add_exclude_pattern("**/tests/**") is session
    assert session.set_deselected(True) is session

    assert session.config.include_patterns == ["**/*.py"]
    assert session.config.exclude_patterns == ["**/tests/**"]
    assert session.config.deselected is True


def test_loaded_session_data_contains_typed_file_entries(project):
    session = Code2PromptSession(
        Code2PromptConfig(str(project), include_patterns=["src/main.py"])
    )
    assert session.raw_analysis() is None

    session.load_codebase()
    data = session.data
    assert isinstance(data, SessionData)
    assert data.absolute_code_path == project.name
    assert data.source_tree is not None
    assert data.files is not None
    assert len(data.files) == 1
    assert data.files[0].path == "src/main.py"
    assert data.files[0].extension == "py"
    assert data.files[0].metadata.is_dir is False
