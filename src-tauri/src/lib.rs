pub mod ai;
pub mod commands;
pub mod database;
pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;
pub mod state;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Open (or create) the SQLite file in the OS app-data folder and run migrations.
            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("habitplanner.db");

            // Tauri's own async runtime is tokio, so sqlx works inside it.
            let pool = tauri::async_runtime::block_on(database::connection::init_pool(&db_path))?;

            app.manage(AppState { db: pool });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // tasks
            commands::task::create_task,
            commands::task::get_task,
            commands::task::list_tasks,
            commands::task::update_task,
            commands::task::set_task_status,
            commands::task::delete_task,
            // habits
            commands::habit::create_habit,
            commands::habit::list_habits,
            commands::habit::update_habit,
            commands::habit::delete_habit,
            commands::habit::toggle_habit_completion,
            // categories
            commands::category::list_categories,
            commands::category::create_category,
            commands::category::delete_category,
            // gamification
            commands::gamification::get_player_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Habit Planner");
}
