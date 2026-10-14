mod catalog;

#[tauri::command]
fn load_catalog() -> Result<catalog::Catalog, String> {
    catalog::load().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_catalog])
        .run(tauri::generate_context!())
        .expect("failed to start Agency Atlas");
}
