pub fn extract_file_name_from_path(opened_file: &Option<std::path::PathBuf>) -> String {
    let path_text: String = opened_file.as_ref().map_or_else(
        || String::default(),
        |path: &std::path::PathBuf| path.to_string_lossy().to_string(),
    );
    let path_parts: Vec<&str> = path_text.split('/').collect();
    let file_name: String = path_parts.last().unwrap().to_string();
    file_name
}
