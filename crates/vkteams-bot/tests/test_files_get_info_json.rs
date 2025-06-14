use vkteams_bot::prelude::ResponseFilesGetInfo;

/// Integration test: deserializes ResponseFilesGetInfo from a real JSON file and checks key fields.
#[test]
fn test_response_files_get_info_from_real_json() {
    let path = std::path::Path::new("tests/responds/files_get_info.json");
    let data = std::fs::read_to_string(path).expect("Failed to read files_get_info.json");
    let resp: ResponseFilesGetInfo =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseFilesGetInfo");
    assert_eq!(resp.file_type, "image/png");
    assert_eq!(resp.file_size, 123456);
    assert_eq!(resp.file_name, "photo.png");
    assert_eq!(resp.url, "https://files.vk.com/photo.png");
}
