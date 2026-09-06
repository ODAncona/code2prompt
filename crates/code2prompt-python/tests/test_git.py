"""Tests for Git operations delegated to Code2PromptSession."""

from code2prompt_rs import Code2PromptConfig, Code2PromptSession

from .conftest import run_git


def test_branch_diff_log_and_tree_pruning(git_project):
    session = Code2PromptSession(
        Code2PromptConfig(
            str(git_project),
            diff_branches=("main", "feature"),
            log_branches=("main", "feature"),
        )
    )

    session.load_codebase()
    session.load_git_diff_between_branches()
    session.load_git_log_between_branches()

    data = session.data
    assert "changed.py" in data.source_tree
    assert "unchanged.py" not in data.source_tree
    assert [file.path for file in data.files] == ["changed.py"]
    assert "VALUE = 1" in data.git_diff_branch
    assert "VALUE = 2" in data.git_diff_branch
    assert "change value" in data.git_log_branch


def test_staged_git_diff(git_project):
    (git_project / "changed.py").write_text("VALUE = 3\n", encoding="utf-8")
    run_git(git_project, "add", "changed.py")

    session = Code2PromptSession(Code2PromptConfig(str(git_project)))
    session.load_git_diff()
    assert "VALUE = 3" in session.data.git_diff
