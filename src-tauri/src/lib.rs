use tauri_plugin_sql::{Migration, MigrationKind};

mod camera;

#[cfg(windows)]
mod camera_wpd;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn check_dslr_support() -> bool {
    camera::check_dslr_support_available()
}

#[tauri::command]
fn list_dslr_cameras() -> Result<Vec<camera::CameraInfo>, String> {
    camera::list_cameras().map_err(|e| e.to_string())
}

#[tauri::command]
fn capture_from_dslr(camera_index: usize) -> Result<camera::CaptureResult, String> {
    camera::capture_image(camera_index).map_err(|e| e.to_string())
}

#[tauri::command]
fn capture_and_save_dslr(camera_index: usize, save_path: String) -> Result<camera::CaptureResult, String> {
    let path = std::path::PathBuf::from(save_path);
    camera::capture_and_save(camera_index, path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Database migrations
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "
                CREATE TABLE IF NOT EXISTS events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    date TEXT NOT NULL,
                    time TEXT NOT NULL,
                    location TEXT,
                    description TEXT,
                    max_photos INTEGER DEFAULT 50,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS photos (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_id INTEGER NOT NULL,
                    file_path TEXT NOT NULL,
                    taken_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (event_id) REFERENCES events(id)
                );

                CREATE TABLE IF NOT EXISTS shares (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    photo_id INTEGER NOT NULL,
                    type TEXT NOT NULL,
                    status TEXT DEFAULT 'pending',
                    destination TEXT,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (photo_id) REFERENCES photos(id)
                );
            ",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_paper_size_and_template",
            sql: "ALTER TABLE events ADD COLUMN paper_size TEXT DEFAULT '4R'",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "add_template_image",
            sql: "ALTER TABLE events ADD COLUMN template_image TEXT",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "add_photo_boxes",
            sql: "ALTER TABLE events ADD COLUMN photo_boxes TEXT",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:snapbooth.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            greet,
            check_dslr_support,
            list_dslr_cameras,
            capture_from_dslr,
            capture_and_save_dslr
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
