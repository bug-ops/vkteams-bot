//! Integration tests for runtime shell completion functionality

use std::fs;
use std::path::Path;
use tempfile::tempdir;
use vkteams_bot_cli::completion::{
    CompletionShell, generate_all_completions, generate_completion, get_default_completion_dir,
};

#[test]
fn test_generate_bash_completion_to_stdout() {
    // Test that runtime bash completion generation doesn't panic when writing to stdout
    let result = generate_completion(CompletionShell::Bash, None);
    assert!(result.is_ok(), "Bash completion generation should succeed");
}

#[test]
fn test_generate_zsh_completion_to_stdout() {
    // Test that runtime zsh completion generation doesn't panic when writing to stdout
    let result = generate_completion(CompletionShell::Zsh, None);
    assert!(result.is_ok(), "Zsh completion generation should succeed");
}

#[test]
fn test_generate_fish_completion_to_stdout() {
    // Test that runtime fish completion generation doesn't panic when writing to stdout
    let result = generate_completion(CompletionShell::Fish, None);
    assert!(result.is_ok(), "Fish completion generation should succeed");
}

#[test]
fn test_generate_powershell_completion_to_stdout() {
    // Test that runtime PowerShell completion generation doesn't panic when writing to stdout
    let result = generate_completion(CompletionShell::PowerShell, None);
    assert!(
        result.is_ok(),
        "PowerShell completion generation should succeed"
    );
}

#[test]
fn test_generate_completion_to_file() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("test_completion.bash");

    let result = generate_completion(CompletionShell::Bash, Some(&output_path));
    assert!(
        result.is_ok(),
        "Runtime completion generation to file should succeed"
    );
    assert!(output_path.exists(), "Completion file should be created");

    // Verify file has content
    let content = fs::read_to_string(&output_path).expect("Failed to read completion file");
    assert!(!content.is_empty(), "Completion file should not be empty");
    assert!(
        content.contains("vkteams-bot-cli"),
        "Completion should reference the CLI name"
    );
}

#[test]
fn test_generate_all_completions() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    let result = generate_all_completions(temp_dir.path());
    assert!(
        result.is_ok(),
        "Runtime generating all completions should succeed"
    );

    // Check that all expected files were created
    let expected_files = [
        "vkteams-bot-cli.bash",
        "_vkteams-bot-cli",
        "vkteams-bot-cli.fish",
        "vkteams-bot-cli.ps1",
    ];

    for filename in &expected_files {
        let file_path = temp_dir.path().join(filename);
        assert!(
            file_path.exists(),
            "Runtime completion file {filename} should exist"
        );

        let content =
            fs::read_to_string(&file_path).unwrap_or_else(|_| panic!("Failed to read {filename}"));
        assert!(
            !content.is_empty(),
            "Runtime completion file {filename} should not be empty"
        );
    }
}

#[test]
fn test_generate_completion_nonexistent_directory() {
    let nonexistent_path = Path::new("/nonexistent/directory/completion.bash");

    let result = generate_completion(CompletionShell::Bash, Some(nonexistent_path));
    assert!(
        result.is_err(),
        "Should fail when parent directory doesn't exist"
    );
}

#[test]
fn test_get_default_completion_dir() {
    let default_dir = get_default_completion_dir();

    // Should return Some path on most systems
    if let Some(dir) = default_dir {
        assert!(dir.to_string_lossy().contains("vkteams-bot-cli"));
        assert!(dir.to_string_lossy().contains("completions"));
    }
    // Note: On some CI environments, this might return None, which is acceptable
}

#[test]
fn test_completion_shell_conversion() {
    use clap_complete::Shell;

    // Test conversion from CompletionShell to clap_complete::Shell
    let bash_shell: Shell = CompletionShell::Bash.into();
    assert!(matches!(bash_shell, Shell::Bash));

    let zsh_shell: Shell = CompletionShell::Zsh.into();
    assert!(matches!(zsh_shell, Shell::Zsh));

    let fish_shell: Shell = CompletionShell::Fish.into();
    assert!(matches!(fish_shell, Shell::Fish));

    let powershell_shell: Shell = CompletionShell::PowerShell.into();
    assert!(matches!(powershell_shell, Shell::PowerShell));
}

#[test]
fn test_completion_content_quality() {
    let temp_dir = tempdir().expect("Failed to create temp directory");

    // Test bash completion content
    let bash_path = temp_dir.path().join("test.bash");
    generate_completion(CompletionShell::Bash, Some(&bash_path)).unwrap();

    let bash_content = fs::read_to_string(&bash_path).unwrap();

    // Check for essential completion elements
    assert!(
        bash_content.contains("vkteams-bot-cli"),
        "Should contain CLI name"
    );
    assert!(
        bash_content.contains("complete"),
        "Should contain completion setup"
    );

    // Test that it references main commands
    let commands = [
        "send-text",
        "send-file",
        "get-chat-info",
        "schedule",
        "config",
    ];
    for command in &commands {
        assert!(
            bash_content.contains(command),
            "Completion should reference command: {command}"
        );
    }
}

#[test]
fn test_zsh_completion_content() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let zsh_path = temp_dir.path().join("_test");

    generate_completion(CompletionShell::Zsh, Some(&zsh_path)).unwrap();

    let zsh_content = fs::read_to_string(&zsh_path).unwrap();

    // Check for zsh-specific completion elements
    assert!(
        zsh_content.contains("#compdef"),
        "Should contain zsh compdef"
    );
    assert!(
        zsh_content.contains("vkteams-bot-cli"),
        "Should contain CLI name"
    );
}

#[test]
fn test_fish_completion_content() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let fish_path = temp_dir.path().join("test.fish");

    generate_completion(CompletionShell::Fish, Some(&fish_path)).unwrap();

    let fish_content = fs::read_to_string(&fish_path).unwrap();

    // Check for fish-specific completion elements
    assert!(
        fish_content.contains("complete"),
        "Should contain fish complete commands"
    );
    assert!(
        fish_content.contains("vkteams-bot-cli"),
        "Should contain CLI name"
    );
}

#[test]
fn test_powershell_completion_content() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let ps_path = temp_dir.path().join("test.ps1");

    generate_completion(CompletionShell::PowerShell, Some(&ps_path)).unwrap();

    let ps_content = fs::read_to_string(&ps_path).unwrap();

    // Check for PowerShell-specific completion elements
    assert!(
        ps_content.contains("Register-ArgumentCompleter") || ps_content.contains("TabExpansion"),
        "Should contain PowerShell completion setup"
    );
    assert!(
        ps_content.contains("vkteams-bot-cli"),
        "Should contain CLI name"
    );
}

#[test]
fn test_completion_file_permissions() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let completion_path = temp_dir.path().join("test_completion.bash");

    generate_completion(CompletionShell::Bash, Some(&completion_path)).unwrap();

    // Check that file was created and is readable
    assert!(completion_path.exists());

    let metadata = fs::metadata(&completion_path).unwrap();
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);
}

#[cfg(unix)]
#[test]
fn test_completion_file_permissions_unix() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let completion_path = temp_dir.path().join("test_completion.bash");

    generate_completion(CompletionShell::Bash, Some(&completion_path)).unwrap();

    let metadata = fs::metadata(&completion_path).unwrap();
    let permissions = metadata.permissions();

    // Check that file is readable
    assert!(
        permissions.mode() & 0o400 != 0,
        "File should be readable by owner"
    );
}

#[test]
fn test_completion_with_invalid_path() {
    // Test with a path that should fail (e.g., trying to write to root on Unix)
    #[cfg(unix)]
    let invalid_path = Path::new("/root/test_completion.bash");
    #[cfg(windows)]
    let invalid_path = Path::new("C:\\Windows\\System32\\test_completion.bash");

    let result = generate_completion(CompletionShell::Bash, Some(invalid_path));
    // This should either succeed (if running as root/admin) or fail gracefully
    // We don't assert failure because it depends on permissions
    match result {
        Ok(_) => {
            // If it succeeded, clean up
            let _ = fs::remove_file(invalid_path);
        }
        Err(_) => {
            // Expected in most cases due to permissions
        }
    }
}

#[test]
fn test_completion_overwrite_existing_file() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let completion_path = temp_dir.path().join("existing_completion.bash");

    // Create an existing file
    fs::write(&completion_path, "# Old completion").unwrap();
    assert!(completion_path.exists());

    // Generate new completion (should overwrite)
    let result = generate_completion(CompletionShell::Bash, Some(&completion_path));
    assert!(
        result.is_ok(),
        "Should be able to overwrite existing file with runtime generation"
    );

    // Verify it was overwritten
    let content = fs::read_to_string(&completion_path).unwrap();
    assert!(!content.contains("# Old completion"));
    assert!(content.contains("vkteams-bot-cli"));
}

#[test]
fn test_generate_all_completions_creates_directory() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let subdir = temp_dir.path().join("new_completions_dir");

    // Directory doesn't exist yet
    assert!(!subdir.exists());

    // Generate completions (should create directory)
    let result = generate_all_completions(&subdir);
    assert!(
        result.is_ok(),
        "Should create directory and generate runtime completions"
    );

    // Verify directory was created
    assert!(subdir.exists());
    assert!(subdir.is_dir());
}

#[test]
fn test_completion_commands_coverage() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let bash_path = temp_dir.path().join("coverage_test.bash");

    generate_completion(CompletionShell::Bash, Some(&bash_path)).unwrap();

    let content = fs::read_to_string(&bash_path).unwrap();

    // Verify that major command categories are covered
    let expected_commands = [
        "send-text",
        "send-file",
        "send-voice",
        "get-chat-info",
        "schedule",
        "scheduler",
        "task",
        "config",
        "setup",
        "validate",
        "completion",
        "examples",
    ];

    for command in &expected_commands {
        assert!(
            content.contains(command),
            "Completion should cover command: {command}"
        );
    }
}
