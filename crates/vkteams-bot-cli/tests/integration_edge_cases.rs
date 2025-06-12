use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_send_text_invalid_chat_id() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("send-text")
        .arg("-u")
        .arg("!!!invalid!!!")
        .arg("-m")
        .arg("hi");
    cmd.env_remove("VKTEAMS_BOT_API_TOKEN");
    cmd.env_remove("VKTEAMS_BOT_API_URL");
    cmd.assert().failure().stderr(contains("Error:"));
}

#[test]
fn test_send_text_empty_message() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("send-text")
        .arg("-u")
        .arg("user123")
        .arg("-m")
        .arg("");
    cmd.env_remove("VKTEAMS_BOT_API_TOKEN");
    cmd.env_remove("VKTEAMS_BOT_API_URL");
    cmd.assert().failure().stderr(contains("Error:"));
}

#[test]
fn test_send_file_invalid_path() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("send-file")
        .arg("-u")
        .arg("user123")
        .arg("-p")
        .arg("/nonexistent/file.txt");
    cmd.env_remove("VKTEAMS_BOT_API_TOKEN");
    cmd.env_remove("VKTEAMS_BOT_API_URL");
    cmd.assert().failure().stderr(contains("Error:"));
}

#[test]
fn test_edit_message_invalid_message_id() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("edit-message")
        .arg("-c")
        .arg("chat123")
        .arg("-m")
        .arg("!!!invalid!!!")
        .arg("-t")
        .arg("new text");
    cmd.env_remove("VKTEAMS_BOT_API_TOKEN");
    cmd.env_remove("VKTEAMS_BOT_API_URL");
    cmd.assert().failure().stderr(contains("Error:"));
}

#[test]
fn test_delete_message_empty_message_id() {
    let mut cmd = Command::cargo_bin("vkteams-bot-cli").unwrap();
    cmd.arg("delete-message")
        .arg("-c")
        .arg("chat123")
        .arg("-m")
        .arg("");
    cmd.env_remove("VKTEAMS_BOT_API_TOKEN");
    cmd.env_remove("VKTEAMS_BOT_API_URL");
    cmd.assert().failure().stderr(contains("Error:"));
}
