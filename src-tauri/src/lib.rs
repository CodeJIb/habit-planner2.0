pub mod ai;
pub mod commands;
pub mod database;
pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;
pub mod state;

#[cfg(test)]
mod integration_tests;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_secure_keystore::init())
        .setup(|app| {
            // Open (or create) the SQLite file in the OS app-data folder and run migrations.
            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("habitplanner.db");

            // Tauri's own async runtime is tokio, so sqlx works inside it.
            let pool = tauri::async_runtime::block_on(database::connection::init_pool(&db_path))?;

            app.manage(AppState {
                db: pool,
                app: app.handle().clone(),
            });
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
            // goals + milestones
            commands::goal::list_goals,
            commands::goal::create_goal,
            commands::goal::update_goal,
            commands::goal::set_goal_status,
            commands::goal::delete_goal,
            commands::goal::add_goal_milestone,
            commands::goal::set_goal_milestone_status,
            commands::goal::delete_goal_milestone,
            // quests + achievements
            commands::quest::list_quests,
            commands::achievement::list_achievements,
            // calendar (scheduled blocks)
            commands::scheduled_block::list_blocks,
            commands::scheduled_block::create_block,
            commands::scheduled_block::update_block,
            commands::scheduled_block::set_block_status,
            commands::scheduled_block::delete_block,
            // focus sessions
            commands::focus_session::list_focus_sessions,
            commands::focus_session::start_focus_session,
            commands::focus_session::complete_focus_session,
            commands::focus_session::cancel_focus_session,
            commands::focus_session::delete_focus_session,
            commands::focus_session::focus_stats,
            // assistant (JARVIS)
            commands::ai::ai_chat,
            commands::ai::ai_chat_stream,
            commands::ai::list_conversations,
            commands::ai::get_conversation,
            commands::ai::delete_conversation,
            commands::ai::get_ai_settings,
            commands::ai::set_ai_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Habit Planner");
}
