use assert_cmd::Command;
use predicates::str::contains;
use std::env;
use std::fs;

#[test]
fn test_main_invalid_config_path() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("--config")
        .arg("/nonexistent/path/config.toml")
        .arg("config");
    cmd.assert()
        .success()
        .stdout(contains("Use --show to display current configuration"));
}

#[test]
fn test_main_save_config_error() {
    // Путь в несуществующую директорию (скорее всего вызовет ошибку)
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("--save-config")
        .arg("/nonexistent_dir/out.toml")
        .arg("config");
    cmd.assert()
        .failure()
        .stderr(contains("Failed to save configuration"));
}

#[test]
fn test_main_command_validation_error() {
    // Неизвестная команда вызовет ошибку валидации (стандартное сообщение clap)
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("not-a-command");
    cmd.assert()
        .failure()
        .stderr(contains("unrecognized subcommand"));
}

#[test]
fn test_main_command_execution_error() {
    // Отправка сообщения без токена вызовет ошибку выполнения
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("send-text")
        .arg("-u")
        .arg("user123")
        .arg("-m")
        .arg("hi");
    // Очищаем переменные окружения, чтобы не было токена
    cmd.env_remove("VKTEAMS_BOT_API_TOKEN");
    cmd.env_remove("VKTEAMS_BOT_API_URL");
    cmd.assert().failure().stderr(contains("Error:"));
}
