//! End-to-end tests that run the real migrations and services against a
//! temporary SQLite file, so the quest/achievement SQL is actually executed.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::ai::provider::{
    AiProvider, BoxFuture, ContentBlock, ProviderRequest, ProviderResponse, TextSink,
};
use crate::database::connection::init_pool;
use crate::errors::AppError;
use crate::models::ai::{AiSettingsInput, ChatInput};
use crate::models::focus_session::FocusSessionInput;
use crate::models::goal::{GoalInput, MilestoneInput};
use crate::models::habit::HabitInput;
use crate::models::scheduled_block::ScheduledBlockInput;
use crate::models::task::TaskInput;
use crate::services::ai::AiService;
use crate::services::focus_session::FocusSessionService;
use crate::services::goal::GoalService;
use crate::services::habit::HabitService;
use crate::services::scheduled_block::ScheduledBlockService;
use crate::services::task::TaskService;
use crate::services::{achievement, gamification, quest};

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("habitplanner_test_{nanos}.db"));
    path
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

fn task_input(title: &str) -> TaskInput {
    TaskInput {
        title: title.to_string(),
        description: None,
        priority: "medium".to_string(),
        category_id: None,
        due_at: None,
        scheduled_start_at: None,
        estimated_duration_minutes: None,
        recurrence_rule: None,
    }
}

fn habit_input(title: &str) -> HabitInput {
    HabitInput {
        title: title.to_string(),
        description: None,
        category_id: None,
        frequency_type: "daily".to_string(),
        frequency_config: None,
        target_count: None,
        reminder_time: None,
    }
}

fn goal_input(title: &str) -> GoalInput {
    GoalInput {
        title: title.to_string(),
        description: None,
        target_date: None,
    }
}

fn milestone_input(title: &str) -> MilestoneInput {
    MilestoneInput {
        title: title.to_string(),
        target_date: None,
    }
}

fn block_input(title: &str, start_at: &str, end_at: &str) -> ScheduledBlockInput {
    ScheduledBlockInput {
        title: title.to_string(),
        start_at: start_at.to_string(),
        end_at: end_at.to_string(),
        task_id: None,
        block_type: None,
    }
}

fn focus_input(task_id: Option<i64>, planned_duration_minutes: i64) -> FocusSessionInput {
    FocusSessionInput {
        task_id,
        planned_duration_minutes,
    }
}

#[test]
fn tasks_and_habits_finish_quests_and_unlock_achievements() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");

        let tasks = TaskService::new(pool.clone());
        let habits = HabitService::new(pool.clone());
        let today = chrono::Local::now().date_naive().to_string();

        // Three completed tasks should finish the daily_tasks quest (target 3).
        for i in 0..3 {
            let task = tasks.create_task(task_input(&format!("Task {i}"))).await.unwrap();
            tasks.set_status(task.id, "done".to_string()).await.unwrap();
        }

        // Two habit check-ins today should finish the daily_habits quest (target 2).
        for i in 0..2 {
            let habit = habits.create_habit(habit_input(&format!("Habit {i}"))).await.unwrap();
            habits.toggle_completion(habit.id, &today).await.unwrap();
        }

        let mut conn = pool.acquire().await.unwrap();
        let quests = quest::list(&mut conn).await.unwrap();

        let daily_tasks = quests
            .iter()
            .find(|q| q.quest.code.as_deref() == Some("daily_tasks"))
            .expect("daily_tasks quest exists");
        assert!(daily_tasks.completed, "daily_tasks should be completed");
        assert_eq!(daily_tasks.current_value, 3);

        let daily_habits = quests
            .iter()
            .find(|q| q.quest.code.as_deref() == Some("daily_habits"))
            .expect("daily_habits quest exists");
        assert!(daily_habits.completed, "daily_habits should be completed");
        assert_eq!(daily_habits.current_value, 2);

        let achievements = achievement::list(&mut conn).await.unwrap();
        let unlocked = |code: &str| {
            achievements
                .iter()
                .find(|a| a.achievement.code == code)
                .map(|a| a.unlocked)
                .unwrap_or(false)
        };
        assert!(unlocked("first_task"), "first_task should be unlocked");
        assert!(unlocked("first_habit"), "first_habit should be unlocked");

        let profile = gamification::get_profile(&mut conn).await.unwrap();
        assert!(profile.total_xp > 0, "completing work should earn XP");

        drop(conn);
        pool.close().await;
        cleanup(&path);
    });
}

#[test]
fn reopening_a_task_does_not_double_count_quest_progress() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");
        let tasks = TaskService::new(pool.clone());

        let task = tasks.create_task(task_input("One task")).await.unwrap();
        tasks.set_status(task.id, "done".to_string()).await.unwrap();

        let mut conn = pool.acquire().await.unwrap();
        let after_done = quest::list(&mut conn).await.unwrap();
        let daily = after_done
            .iter()
            .find(|q| q.quest.code.as_deref() == Some("daily_tasks"))
            .unwrap();
        assert_eq!(daily.current_value, 1);

        tasks.set_status(task.id, "todo".to_string()).await.unwrap();
        let after_reopen = quest::list(&mut conn).await.unwrap();
        let daily = after_reopen
            .iter()
            .find(|q| q.quest.code.as_deref() == Some("daily_tasks"))
            .unwrap();
        assert_eq!(daily.current_value, 0, "progress is derived from tasks, not counted up");
        assert!(!daily.completed, "only one task was ever done, so the quest never completed");

        drop(conn);
        pool.close().await;
        cleanup(&path);
    });
}

#[test]
fn completing_a_goal_awards_xp_once_and_reopening_refunds_it() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");
        let goals = GoalService::new(pool.clone());

        let created = goals.create_goal(goal_input("Learn Rust")).await.unwrap();
        assert_eq!(created.goal.status, "active");
        assert!(created.milestones.is_empty());

        let with_milestone = goals
            .add_milestone(created.goal.id, milestone_input("Read the book"))
            .await
            .unwrap();
        assert_eq!(with_milestone.milestones.len(), 1);
        let milestone_id = with_milestone.milestones[0].id;

        let toggled = goals
            .set_milestone_status(milestone_id, "completed".to_string())
            .await
            .unwrap();
        assert_eq!(toggled.milestones[0].status, "completed");

        let mut conn = pool.acquire().await.unwrap();
        let before = gamification::get_profile(&mut conn).await.unwrap().total_xp;

        let done = goals
            .set_status(created.goal.id, "completed".to_string())
            .await
            .unwrap();
        assert_eq!(done.goal.goal.status, "completed");
        assert_eq!(done.xp.xp_delta, 100, "a goal completion awards 100 XP");

        let reopened = goals
            .set_status(created.goal.id, "active".to_string())
            .await
            .unwrap();
        assert_eq!(reopened.goal.goal.status, "active");
        let after = gamification::get_profile(&mut conn).await.unwrap().total_xp;
        assert_eq!(after, before, "reopening a goal refunds its XP");

        let after_delete = goals.delete_milestone(milestone_id).await.unwrap();
        assert!(after_delete.milestones.is_empty());
        goals.delete_goal(created.goal.id).await.unwrap();
        assert_eq!(goals.list_goals(None).await.unwrap().len(), 0);

        drop(conn);
        pool.close().await;
        cleanup(&path);
    });
}

#[test]
fn blocks_filter_by_date_range_and_track_status() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");
        let blocks = ScheduledBlockService::new(pool.clone());
        let tasks = TaskService::new(pool.clone());

        let task = tasks.create_task(task_input("Write report")).await.unwrap();

        let mut deep = block_input("Deep work", "2026-10-16T09:00", "2026-10-16T10:30");
        deep.task_id = Some(task.id);
        deep.block_type = Some("task".to_string());
        let deep = blocks.create(deep).await.unwrap();
        assert_eq!(deep.block_type, "task");
        assert_eq!(deep.task_id, Some(task.id));
        assert_eq!(deep.status, "scheduled");

        let mut lunch = block_input("Lunch", "2026-10-16T12:00", "2026-10-16T12:30");
        lunch.block_type = Some("break".to_string());
        let lunch = blocks.create(lunch).await.unwrap();
        assert_eq!(lunch.block_type, "break");

        let planning = blocks
            .create(block_input("Planning", "2026-10-17T09:00", "2026-10-17T09:30"))
            .await
            .unwrap();

        // No bounds returns everything.
        assert_eq!(blocks.list(None, None).await.unwrap().len(), 3);

        // A single day returns only its own blocks.
        let day = blocks
            .list(Some("2026-10-16".into()), Some("2026-10-16".into()))
            .await
            .unwrap();
        assert_eq!(day.len(), 2);
        assert!(day.iter().all(|b| b.start_at.starts_with("2026-10-16")));

        // A block spanning midnight into the day still counts for that day.
        let overnight = blocks
            .create(block_input("On call", "2026-10-15T23:00", "2026-10-16T01:00"))
            .await
            .unwrap();
        let day = blocks
            .list(Some("2026-10-16".into()), Some("2026-10-16".into()))
            .await
            .unwrap();
        assert_eq!(day.len(), 3, "an overnight block overlaps the next morning");

        // Status changes and deletion.
        let running = blocks
            .set_status(overnight.id, "running".to_string())
            .await
            .unwrap();
        assert_eq!(running.status, "running");
        blocks.delete(planning.id).await.unwrap();
        assert_eq!(blocks.list(None, None).await.unwrap().len(), 3);

        pool.close().await;
        cleanup(&path);
    });
}

#[test]
fn focus_sessions_award_xp_and_refund_it_on_delete() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");
        let focus = FocusSessionService::new(pool.clone());
        let tasks = TaskService::new(pool.clone());

        let task = tasks.create_task(task_input("Write report")).await.unwrap();

        // A 25-minute session, focused the full 25 minutes, earns 25 XP (1 per minute).
        let session = focus.start(focus_input(Some(task.id), 25)).await.unwrap();
        assert_eq!(session.status, "active");
        assert_eq!(session.task_id, Some(task.id));
        assert!(session.ended_at.is_none());

        let done = focus.complete(session.id, 25 * 60).await.unwrap();
        assert_eq!(done.session.status, "completed");
        assert_eq!(done.session.actual_duration_seconds, Some(25 * 60));
        assert!(done.session.ended_at.is_some());
        assert_eq!(done.xp.xp_delta, 25);

        // Completing again is idempotent: no double XP.
        let again = focus.complete(session.id, 25 * 60).await.unwrap();
        assert_eq!(again.xp.xp_delta, 0);

        // A second completed session without a task link.
        let short = focus.start(focus_input(None, 10)).await.unwrap();
        let short = focus.complete(short.id, 10 * 60).await.unwrap();
        assert_eq!(short.xp.xp_delta, 10);

        // Cancelling an active session ends it without XP.
        let aborted = focus.start(focus_input(None, 25)).await.unwrap();
        let cancelled = focus.cancel(aborted.id).await.unwrap();
        assert_eq!(cancelled.session.status, "cancelled");
        assert_eq!(cancelled.xp.xp_delta, 0);

        // Stats only count completed sessions: 25 + 10 focused minutes today.
        let stats = focus.stats().await.unwrap();
        assert_eq!(stats.today_sessions, 2);
        assert_eq!(stats.today_minutes, 35);
        assert_eq!(stats.week_sessions, 2);

        // Deleting a completed session takes its XP back.
        let before = {
            let mut conn = pool.acquire().await.unwrap();
            gamification::get_profile(&mut conn).await.unwrap().total_xp
        };
        focus.delete(session.id).await.unwrap();
        let after = {
            let mut conn = pool.acquire().await.unwrap();
            gamification::get_profile(&mut conn).await.unwrap().total_xp
        };
        assert_eq!(after, before - 25);

        // The task-linked session is gone, but the other two remain.
        assert!(focus.list(Some(task.id), None).await.unwrap().is_empty());
        assert_eq!(focus.list(None, None).await.unwrap().len(), 2);

        pool.close().await;
        cleanup(&path);
    });
}

/// A provider that answers without a network: first round asks for a task,
/// every later round just talks.
struct FakeProvider {
    calls: AtomicUsize,
}

impl AiProvider for FakeProvider {
    fn complete<'a>(
        &'a self,
        request: ProviderRequest,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>> {
        Box::pin(async move {
            // The model always receives the live context and the tool schemas.
            assert!(request.system.contains("LIVE DATA"));
            assert!(!request.tools.is_empty());

            if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
                Ok(ProviderResponse {
                    content: vec![
                        // A thinking block the provider must pass back verbatim.
                        ContentBlock::Raw(json!({
                            "type": "thinking",
                            "thinking": "The user wants a task.",
                            "signature": "sig"
                        })),
                        ContentBlock::text("Adding that now."),
                        ContentBlock::ToolUse {
                            id: "toolu_1".into(),
                            name: "create_task".into(),
                            input: json!({ "title": "From JARVIS", "priority": "high" }),
                        },
                    ],
                    stop_reason: Some("tool_use".into()),
                })
            } else {
                // Whenever the history holds a tool_use turn, the matching
                // tool_result must have been sent back with it.
                let expects_tool_result = request.messages.iter().any(|message| {
                    message
                        .content
                        .iter()
                        .any(|block| matches!(block, ContentBlock::ToolUse { .. }))
                });
                if expects_tool_result {
                    let has_tool_result = request.messages.iter().any(|message| {
                        message
                            .content
                            .iter()
                            .any(|block| matches!(block, ContentBlock::ToolResult { .. }))
                    });
                    assert!(has_tool_result, "tool results should be sent back");
                }
                Ok(ProviderResponse {
                    content: vec![ContentBlock::text("Created the task for you.")],
                    stop_reason: Some("end_turn".into()),
                })
            }
        })
    }

    fn complete_streaming<'a>(
        &'a self,
        request: ProviderRequest,
        _sink: TextSink<'a>,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>> {
        self.complete(request)
    }
}

#[test]
fn assistant_chat_runs_tools_and_persists_the_thread() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");
        let assistant = AiService::new(pool.clone());
        let provider = FakeProvider {
            calls: AtomicUsize::new(0),
        };

        let first = assistant
            .chat_with(
                &ChatInput {
                    conversation_id: None,
                    message: "Add a task to call mum".into(),
                },
                &provider,
            )
            .await
            .unwrap();

        // Only the user line and the assistant's final words are stored.
        assert_eq!(first.messages.len(), 2);
        assert_eq!(first.messages[0].role, "user");
        assert_eq!(first.messages[1].role, "assistant");
        assert_eq!(first.messages[1].content, "Created the task for you.");
        assert_eq!(
            first.conversation.title.as_deref(),
            Some("Add a task to call mum")
        );

        // The tool ran through the normal task service, so the task exists.
        let tasks = TaskService::new(pool.clone())
            .list_tasks(None)
            .await
            .unwrap();
        assert!(tasks
            .iter()
            .any(|task| task.title == "From JARVIS" && task.priority == "high"));

        // Continuing the thread keeps the earlier turns.
        let second = assistant
            .chat_with(
                &ChatInput {
                    conversation_id: Some(first.conversation.id),
                    message: "Thanks".into(),
                },
                &provider,
            )
            .await
            .unwrap();
        assert_eq!(second.conversation.id, first.conversation.id);

        let history = assistant.get_conversation(first.conversation.id).await.unwrap();
        assert_eq!(history.len(), 4);
        assert_eq!(history[2].content, "Thanks");

        // Threads are listed, openable and deletable (messages cascade).
        assert_eq!(assistant.list_conversations().await.unwrap().len(), 1);
        assistant.delete_conversation(first.conversation.id).await.unwrap();
        assert!(assistant.list_conversations().await.unwrap().is_empty());
        assert!(assistant.get_conversation(first.conversation.id).await.is_err());

        pool.close().await;
        cleanup(&path);
    });
}

#[test]
fn assistant_without_a_key_refuses_to_chat() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");
        let assistant = AiService::new(pool.clone());

        let error = assistant
            .chat(ChatInput {
                conversation_id: None,
                message: "Hello".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, AppError::InvalidInput(_)), "got {error}");
        assert!(assistant.list_conversations().await.unwrap().is_empty());

        pool.close().await;
        cleanup(&path);
    });
}

#[test]
fn assistant_settings_never_leak_the_key_and_validate_input() {
    tauri::async_runtime::block_on(async {
        let path = temp_db_path();
        let pool = init_pool(&path).await.expect("init pool");
        let assistant = AiService::new(pool.clone());

        let initial = assistant.get_settings().await.unwrap();
        assert!(!initial.api_key_set);
        assert!(initial.base_url.is_empty());
        assert!(!initial.model.is_empty());

        let saved = assistant
            .set_settings(AiSettingsInput {
                api_key: Some("sk-ant-test-key".into()),
                base_url: None,
                model: Some("claude-haiku-5-5".into()),
            })
            .await
            .unwrap();
        assert!(saved.api_key_set);
        assert_eq!(saved.model, "claude-haiku-5-5");

        // The view carries a boolean, never the key itself.
        let serialised = serde_json::to_value(&saved).unwrap();
        assert_eq!(serialised["apiKeySet"], true);
        assert!(!serialised.to_string().contains("sk-ant"));

        // A base URL switches to the OpenAI-compatible provider and is reported back.
        let local = assistant
            .set_settings(AiSettingsInput {
                api_key: None,
                base_url: Some("http://localhost:11434/v1".into()),
                model: Some("llama3.1:8b".into()),
            })
            .await
            .unwrap();
        assert_eq!(local.base_url, "http://localhost:11434/v1");
        assert_eq!(local.model, "llama3.1:8b", "colons and slashes are valid in model ids");

        // Empty strings clear the key, base URL and model.
        let cleared = assistant
            .set_settings(AiSettingsInput {
                api_key: Some(String::new()),
                base_url: Some(String::new()),
                model: Some(String::new()),
            })
            .await
            .unwrap();
        assert!(!cleared.api_key_set);
        assert!(cleared.base_url.is_empty());
        assert_ne!(cleared.model, "claude-haiku-5-5");

        // Nonsense keys, base URLs and model ids are rejected.
        assert!(assistant
            .set_settings(AiSettingsInput {
                api_key: Some("spaces in key".into()),
                base_url: None,
                model: None,
            })
            .await
            .is_err());
        assert!(assistant
            .set_settings(AiSettingsInput {
                api_key: None,
                base_url: Some("localhost:11434".into()),
                model: None,
            })
            .await
            .is_err());
        assert!(assistant
            .set_settings(AiSettingsInput {
                api_key: None,
                base_url: None,
                model: Some("not a model!".into()),
            })
            .await
            .is_err());

        pool.close().await;
        cleanup(&path);
    });
}
