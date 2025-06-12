use crate::config::CONFIG;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;
use std::path::Path;
use std::time::Duration;

/// Creates a progress bar for file download operations
pub fn create_download_progress_bar(total_size: u64, file_name: &str) -> Option<ProgressBar> {
    if !&CONFIG.ui.show_progress {
        return None;
    }
    let cfg = &CONFIG.ui;

    let pb = ProgressBar::new(total_size);
    pb.set_style(create_progress_style("⬇️ "));
    pb.set_message(format!("Downloading {}", file_name));
    pb.enable_steady_tick(Duration::from_millis(cfg.progress_refresh_rate));
    Some(pb)
}

/// Creates a progress bar for file upload operations
pub fn create_upload_progress_bar(total_size: u64, file_path: &str) -> Option<ProgressBar> {
    if !&CONFIG.ui.show_progress {
        return None;
    }

    let cfg = &CONFIG.ui;

    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path);

    let pb = ProgressBar::new(total_size);
    pb.set_style(create_progress_style("⬆️ "));
    pb.set_message(format!("Uploading {}", file_name));
    pb.enable_steady_tick(Duration::from_millis(cfg.progress_refresh_rate));
    Some(pb)
}

/// Creates a progress style based on the configuration
fn create_progress_style(prefix: &str) -> ProgressStyle {
    let prefix_owned = prefix.to_string();
    let cfg = &CONFIG.ui;
    match cfg.progress_style.as_str() {
        "ascii" => ProgressStyle::with_template(
            "{prefix}{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("##-"),
        "unicode" => ProgressStyle::with_template(
            "{prefix}{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("━━╾─"),
        _ => ProgressStyle::with_template(
            "{prefix}{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        }),
    }
    .with_key("prefix", move |_: &ProgressState, w: &mut dyn Write| {
        write!(w, "{}", prefix_owned).unwrap()
    })
}

/// Calculates the estimated file size for an upload when it cannot be determined
pub fn calculate_upload_size(file_path: &str) -> std::io::Result<u64> {
    let metadata = std::fs::metadata(file_path)?;
    Ok(metadata.len())
}

/// Helper to increment a progress bar safely (won't panic if it's None)
pub fn increment_progress(progress_bar: &Option<ProgressBar>, amount: u64) {
    if let Some(pb) = progress_bar {
        pb.inc(amount);
    }
}

/// Helper to finish a progress bar safely (won't panic if it's None)
pub fn finish_progress(progress_bar: &Option<ProgressBar>, message: &str) {
    if let Some(pb) = progress_bar {
        pb.finish_with_message(message.to_string());
    }
}

/// Helper to abandon a progress bar safely (won't panic if it's None)
pub fn abandon_progress(progress_bar: &Option<ProgressBar>, message: &str) {
    if let Some(pb) = progress_bar {
        pb.abandon_with_message(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_create_download_progress_bar_current_config() {
        // Проверяем, что функция возвращает Some или None в зависимости от текущего show_progress
        let pb = create_download_progress_bar(100, "file.txt");
        // Просто проверяем, что функция не паникует
        let _ = pb;
    }

    #[test]
    fn test_create_upload_progress_bar_current_config() {
        let pb = create_upload_progress_bar(100, "file.txt");
        let _ = pb;
    }

    #[test]
    fn test_increment_finish_abandon_progress() {
        let pb = create_download_progress_bar(100, "file.txt");
        increment_progress(&pb, 10);
        finish_progress(&pb, "done");
        let pb2 = create_upload_progress_bar(100, "file.txt");
        abandon_progress(&pb2, "abandoned");
    }

    #[test]
    fn test_calculate_upload_size_ok_and_err() {
        // Создаём временный файл
        let mut path = temp_dir();
        path.push("vkteams_test_file.tmp");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"1234567890").unwrap();
        let size = calculate_upload_size(path.to_str().unwrap()).unwrap();
        assert_eq!(size, 10);
        std::fs::remove_file(&path).unwrap();
        // Ошибка для несуществующего файла
        let err = calculate_upload_size("/nonexistent/file");
        assert!(err.is_err());
    }
}
