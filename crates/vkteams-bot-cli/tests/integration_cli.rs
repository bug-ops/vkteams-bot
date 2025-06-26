use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicates::str::contains(
        "A powerful command-line interface for interacting with VK Teams Bot API",
    ));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("vkteams-bot-cli"));
}

#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("no-such-command");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("error"));
}

#[test]
fn test_cli_rate_limit_test_invalid_requests() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.args(["rate-limit-test", "--requests", "0"]);
    cmd.assert().failure().stderr(predicates::str::contains(
        "Number of requests must be between 1 and 1000",
    ));
}

#[test]
fn test_cli_config_show() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.args(["config", "--show"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Success"));
}

#[test]
fn test_cli_completion_stdout() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.args(["completion", "zsh"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Success"));
}

#[test]
fn test_cli_validate() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("validate");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Success"));
}

#[test]
fn test_cli_list_commands() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("list-commands");
    cmd.assert().success().stdout(predicates::str::contains(
        "Success",
    ));
}

#[test]
fn test_cli_config_init() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.args(["config", "--init"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Success"));
}

#[test]
fn test_cli_config_wizard() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.args(["config", "--wizard"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Configuration updated successfully"));
}

#[test]
fn test_cli_setup() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("setup");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Setup complete"));
}

#[test]
fn test_cli_examples() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("examples");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Success"));
}
