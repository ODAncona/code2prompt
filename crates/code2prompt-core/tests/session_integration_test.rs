//! Integration tests for the session with simplified file selection

use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::session::Code2PromptSession;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test directory structure
        fs::create_dir_all(base_path.join("src")).unwrap();
        fs::create_dir_all(base_path.join("tests")).unwrap();

        // Create test files
        fs::write(base_path.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(base_path.join("src/lib.rs"), "pub mod utils;").unwrap();
        fs::write(base_path.join("src/utils.rs"), "pub fn helper() {}").unwrap();
        fs::write(base_path.join("tests/test_main.rs"), "#[test] fn test() {}").unwrap();
        fs::write(base_path.join("README.md"), "# Test Project").unwrap();

        temp_dir
    }

    #[test]
    fn test_session_select_deselect_file() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .exclude_patterns(vec!["*".to_string()]) // Exclude everything initially
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");

        // Initially, no files should be selected (excluded by pattern)
        assert!(!session.is_file_selected(&main_rs_relative));
        assert!(session.get_selected_files().unwrap().is_empty());

        // Select the file using relative path (user action overrides pattern)
        session.select_file(main_rs_relative.clone());
        assert!(session.is_file_selected(&main_rs_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 1);

        // Deselect the file
        session.deselect_file(main_rs_relative.clone());
        assert!(!session.is_file_selected(&main_rs_relative));
        assert!(session.get_selected_files().unwrap().is_empty());
    }

    #[test]
    fn test_session_multiple_files() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");
        let utils_rs_relative = std::path::PathBuf::from("src/utils.rs");
        let readme_relative = std::path::PathBuf::from("README.md");

        // Select multiple files using relative paths
        session.select_file(main_rs_relative.clone());
        session.select_file(utils_rs_relative.clone());
        session.select_file(readme_relative.clone());

        assert!(session.is_file_selected(&main_rs_relative));
        assert!(session.is_file_selected(&utils_rs_relative));
        assert!(session.is_file_selected(&readme_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 3);

        // Deselect one file
        session.deselect_file(utils_rs_relative.clone());
        assert!(session.is_file_selected(&main_rs_relative));
        assert!(!session.is_file_selected(&utils_rs_relative));
        assert!(session.is_file_selected(&readme_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 2);
    }

    #[test]
    fn test_session_multiple_file_selection() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");
        let utils_rs_relative = std::path::PathBuf::from("src/utils.rs");

        // Select multiple files individually using relative paths
        session.select_file(main_rs_relative.clone());
        session.select_file(utils_rs_relative.clone());

        assert!(session.is_file_selected(&main_rs_relative));
        assert!(session.is_file_selected(&utils_rs_relative));
        assert_eq!(session.get_selected_files().unwrap().len(), 2);
    }

    #[test]
    fn test_session_clear_user_actions() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .exclude_patterns(vec!["*".to_string()]) // Exclude everything initially
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");
        let utils_rs_relative = std::path::PathBuf::from("src/utils.rs");

        // Select some files using relative paths (user actions override exclude patterns)
        session.select_file(main_rs_relative.clone());
        session.select_file(utils_rs_relative.clone());
        assert_eq!(session.get_selected_files().unwrap().len(), 2);

        // Clear all user actions (reset to pattern-only behavior)
        session.clear_user_actions();
        // After clearing user actions, files should be excluded by the exclude pattern
        assert!(session.get_selected_files().unwrap().is_empty());
    }

    #[test]
    fn test_session_add_patterns() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);

        // Initially no patterns
        assert!(session.config.include_patterns.is_empty());
        assert!(session.config.exclude_patterns.is_empty());

        // Add patterns
        session.add_include_pattern("*.rs".to_string());
        session.add_exclude_pattern("**/test*".to_string());

        assert_eq!(session.config.include_patterns.len(), 1);
        assert_eq!(session.config.exclude_patterns.len(), 1);
        assert_eq!(session.config.include_patterns[0], "*.rs");
        assert_eq!(session.config.exclude_patterns[0], "**/test*");
    }

    #[test]
    fn test_session_relative_path_handling() {
        let temp_dir = create_test_project();
        let config = Code2PromptConfig::builder()
            .path(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        let main_rs_absolute = temp_dir.path().join("src/main.rs");
        let main_rs_relative = std::path::PathBuf::from("src/main.rs");

        // Select using absolute path
        session.select_file(main_rs_absolute.clone());

        // Should be found using both absolute and relative paths
        assert!(session.is_file_selected(&main_rs_absolute));
        assert!(session.is_file_selected(&main_rs_relative));

        // The stored path should be relative
        let selected_files = session.get_selected_files().unwrap();
        assert_eq!(selected_files.len(), 1);
        assert_eq!(selected_files[0], main_rs_relative);
    }

    /// Regression test for https://github.com/mufeedvh/code2prompt/issues/176
    ///
    /// With `--git-diff-branch`, the source tree (and file content) must be
    /// pruned down to only the files that actually changed between the two
    /// branches, the same way `--include` filters both the tree and content.
    #[test]
    fn test_git_diff_branch_prunes_source_tree_to_changed_files() {
        use git2::{Repository, RepositoryInitOptions, Signature};

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let mut binding = RepositoryInitOptions::new();
        let init_options = binding.initial_head("master");
        let repo = Repository::init_opts(repo_path, init_options)
            .expect("Failed to initialize repository");

        fs::create_dir_all(repo_path.join("src")).unwrap();
        fs::write(repo_path.join("src/changed.rs"), "fn changed() {}").unwrap();
        fs::write(repo_path.join("src/unchanged.rs"), "fn unchanged() {}").unwrap();

        let signature = Signature::now("Test", "test@example.com").unwrap();

        // Commit both files on master
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("src/changed.rs")).unwrap();
        index.add_path(Path::new("src/unchanged.rs")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let master_commit = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
            .expect("Failed to commit");

        // Create a feature branch and only modify one of the two files there
        repo.branch("feature", &repo.find_commit(master_commit).unwrap(), false)
            .expect("Failed to create new branch");
        repo.set_head("refs/heads/feature").unwrap();
        repo.checkout_head(None).unwrap();

        fs::write(
            repo_path.join("src/changed.rs"),
            "fn changed() { /* updated */ }",
        )
        .unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("src/changed.rs")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Update changed.rs",
            &tree,
            &[&repo.find_commit(master_commit).unwrap()],
        )
        .expect("Failed to commit on feature branch");

        let config = Code2PromptConfig::builder()
            .path(repo_path.to_path_buf())
            .diff_branches(Some(("master".to_string(), "feature".to_string())))
            .build()
            .unwrap();

        let mut session = Code2PromptSession::new(config);
        session.load_codebase().expect("Failed to load codebase");

        let source_tree = session.data.source_tree.clone().unwrap_or_default();
        assert!(
            source_tree.contains("changed.rs"),
            "expected diffed file in source tree, got:\n{source_tree}"
        );
        assert!(
            !source_tree.contains("unchanged.rs"),
            "expected unchanged file to be pruned from source tree, got:\n{source_tree}"
        );

        let files = session.data.files.clone().unwrap_or_default();
        assert!(
            files.iter().any(|f| f.path.contains("changed.rs")),
            "expected diffed file in file content list"
        );
        assert!(
            !files.iter().any(|f| f.path.contains("unchanged.rs")),
            "expected unchanged file to be excluded from file content list"
        );
    }
}
