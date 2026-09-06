"""Select files and generate a prompt through the native session API."""

from code2prompt_rs import Code2PromptConfig, Code2PromptSession


def main() -> None:
    config = Code2PromptConfig(
        ".",
        include_patterns=["**/*.py", "**/*.rs"],
        exclude_patterns=["**/tests/**"],
        line_numbers=True,
        deselected=True,
    )
    session = Code2PromptSession(config)
    session.select_file("src/main.rs").select_file("src/lib.rs")

    result = session.generate_prompt()
    print(f"Generated {result.token_count} tokens from {len(result.files)} files")
    print(result.prompt)


if __name__ == "__main__":
    main()
