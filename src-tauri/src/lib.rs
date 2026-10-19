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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_secure_keystore::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Open (or create) the SQLite file in the OS app-data folder and run migrations.
            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("habitplanner.db");

            // Tauri's own async runtime is tokio, so sqlx works inside it.
            let pool = tauri::async_runtime::block_on(database::connection::init_pool(&db_path))?;

            // Start the reminder loop: it checks every minute for tasks that have
            // just come due, habits whose reminder time arrived, and the digest.
            // NOTE (mobile): Android and iOS suspend this in-process loop while the
            // app is backgrounded, so on-device notifications only fire while the
            // app is open. Server push (ntfy) is what reaches the phone when the
            // app is closed. See README "Phone reminders".
            tauri::async_runtime::spawn(services::notification::run_loop(
                app.handle().clone(),
                pool.clone(),
            ));

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
            // text-to-speech (JARVIS voice)
            commands::tts::get_tts_settings,
            commands::tts::set_tts_settings,
            commands::tts::open_speech_settings,
            commands::tts::synth_speech,
            commands::tts::list_cloud_voices,
            // phone reminders
            commands::notification::get_notification_settings,
            commands::notification::set_notification_settings,
            commands::notification::send_test_notification,
            commands::notification::send_due_reminders,
            // analytics
            commands::analytics::list_xp_over_time,
            commands::analytics::list_task_completions,
            commands::analytics::list_focus_minutes,
            commands::analytics::list_habit_checkins,
            commands::analytics::list_analytics_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Habit Planner");
}
