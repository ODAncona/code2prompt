"""Shared fixtures for the native Python bindings."""

import json
import subprocess
from pathlib import Path

import pytest


@pytest.fixture()
def project(tmp_path: Path) -> Path:
    """Create a small repository-like tree with ignored and hidden files."""
    (tmp_path / "src").mkdir()
    (tmp_path / "tests").mkdir()
    (tmp_path / ".secret").mkdir()

    (tmp_path / "src" / "main.py").write_text(
        "def main():\n    return 'main'\n", encoding="utf-8"
    )
    (tmp_path / "src" / "utils.py").write_text(
        "def helper():\n    return 42\n", encoding="utf-8"
    )
    (tmp_path / "tests" / "test_main.py").write_text(
        "def test_main():\n    assert True\n", encoding="utf-8"
    )
    (tmp_path / "README.md").write_text("# Test project\n", encoding="utf-8")
    (tmp_path / "ignored.txt").write_text("ignored by gitignore\n", encoding="utf-8")
    (tmp_path / ".secret" / "secret.py").write_text("SECRET = True\n", encoding="utf-8")
    (tmp_path / ".gitignore").write_text("*.txt\n", encoding="utf-8")

    notebook = {
        "cells": [
            {"cell_type": "markdown", "source": ["# Notes\n"]},
            {
                "cell_type": "code",
                "source": ["print('hello')\n"],
                "outputs": [
                    {"output_type": "stream", "name": "stdout", "text": ["hello\n"]}
                ],
            },
            {"cell_type": "code", "source": ["x = 2\n"], "outputs": []},
        ]
    }
    (tmp_path / "notebook.ipynb").write_text(json.dumps(notebook), encoding="utf-8")
    run_git(tmp_path, "init")
    return tmp_path


def run_git(repository: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
    ).stdout


@pytest.fixture()
def git_project(tmp_path: Path) -> Path:
    """Create a two-branch Git repository for session Git tests."""
    run_git(tmp_path, "init", "-b", "main")
    run_git(tmp_path, "config", "user.name", "Code2Prompt Tests")
    run_git(tmp_path, "config", "user.email", "tests@code2prompt.dev")

    (tmp_path / "changed.py").write_text("VALUE = 1\n", encoding="utf-8")
    (tmp_path / "unchanged.py").write_text("STABLE = True\n", encoding="utf-8")
    run_git(tmp_path, "add", ".")
    run_git(tmp_path, "commit", "-m", "initial")

    run_git(tmp_path, "switch", "-c", "feature")
    (tmp_path / "changed.py").write_text("VALUE = 2\n", encoding="utf-8")
    run_git(tmp_path, "add", "changed.py")
    run_git(tmp_path, "commit", "-m", "change value")
    return tmp_path
