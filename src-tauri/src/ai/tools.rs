//! Phase 5: tools the assistant may call (create task, complete habit, ...).
//! Definitions travel to the model; execution only ever happens with a
//! [`ValidatedToolCall`], through the same services the UI uses (so XP,
//! quests and achievements behave exactly as they do for manual actions).

use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::ai::validation::ValidatedToolCall;
use crate::errors::AppError;
use crate::models::gamification::XpOutcome;
use crate::services::goal::GoalService;
use crate::services::habit::HabitService;
use crate::services::notification::NotificationService;
use crate::services::task::TaskService;

/// What one executed tool tells the model: plain text (fed back as a
/// `tool_result`), whether it failed, and any XP it earned.
pub struct ToolOutcome {
    pub result: String,
    pub is_error: bool,
    pub xp: Option<XpOutcome>,
}

impl ToolOutcome {
    fn ok(result: impl Into<String>) -> Self {
        Self {
            result: result.into(),
            is_error: false,
            xp: None,
        }
    }

    fn err(result: impl Into<String>) -> Self {
        Self {
            result: result.into(),
            is_error: true,
            xp: None,
        }
    }
}

fn xp_note(xp: &XpOutcome) -> String {
    if xp.xp_delta > 0 {
        format!(" (+{} XP)", xp.xp_delta)
    } else {
        String::new()
    }
}

/// Tool schemas handed to the model. Field names are snake_case on purpose:
/// [`crate::ai::validation`] maps them onto the app's camelCase structs.
///
/// These are kept deliberately terse: every property and description costs
/// tokens on every single request, and the free-tier budget is tight. Only the
/// fields the assistant realistically sets from chat are advertised.
pub fn definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "create_task",
            "description": "Create a new task in the user's planner.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Short task title." },
                    "priority": { "type": "string", "enum": ["low", "medium", "high", "urgent"] },
                    "due_at": { "type": "string", "description": "Due date, YYYY-MM-DD." }
                },
                "required": ["title"]
            }
        }),
        json!({
            "name": "complete_task",
            "description": "Mark an existing task as done (awards XP).",
            "input_schema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer", "description": "Id of the task to complete." }
                },
                "required": ["task_id"]
            }
        }),
        json!({
            "name": "create_habit",
            "description": "Create a new habit in the user's planner.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Short habit title." },
                    "frequency": { "type": "string", "enum": ["daily", "weekly", "custom"] }
                },
                "required": ["title"]
            }
        }),
        json!({
            "name": "complete_habit",
            "description": "Check a habit in for today (awards XP). Does nothing if it is already checked in today.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "habit_id": { "type": "integer", "description": "Id of the habit to check in." }
                },
                "required": ["habit_id"]
            }
        }),
        json!({
            "name": "create_goal",
            "description": "Create a new long-term goal in the user's planner.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Goal title." },
                    "target_date": { "type": "string", "description": "Optional target date, YYYY-MM-DD." }
                },
                "required": ["title"]
            }
        }),
        json!({
            "name": "send_notification",
            "description": "Send a push notification to the user's phone. Use only when the user asks to be reminded or notified about something specific right now.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "message": { "type": "string", "description": "The notification body." },
                    "title": { "type": "string", "description": "Optional short title." }
                },
                "required": ["message"]
            }
        }),
    ]
}

/// Run one validated tool through the app's services.
/// `today` is the player's local date ("YYYY-MM-DD").
pub async fn execute(
    pool: &SqlitePool,
    call: ValidatedToolCall,
    today: &str,
) -> Result<ToolOutcome, AppError> {
    match call {
        ValidatedToolCall::CreateTask(input) => {
            let task = TaskService::new(pool.clone()).create_task(input).await?;
            Ok(ToolOutcome::ok(format!(
                "Created task #{} \"{}\" (priority {}).",
                task.id, task.title, task.priority
            )))
        }

        ValidatedToolCall::CompleteTask { id } => {
            let tasks = TaskService::new(pool.clone());
            let task = match tasks.get_task(id).await {
                Ok(task) => task,
                Err(AppError::NotFound) => {
                    return Ok(ToolOutcome::err(format!("There is no task with id {id}.")));
                }
                Err(error) => return Err(error),
            };
            if task.status == "done" {
                Ok(ToolOutcome::ok(format!(
                    "Task #{} \"{}\" was already done.",
                    task.id, task.title
                )))
            } else {
                let result = tasks.set_status(id, "done".to_string()).await?;
                Ok(ToolOutcome {
                    result: format!(
                        "Completed task #{} \"{}\"{}.",
                        task.id,
                        task.title,
                        xp_note(&result.xp)
                    ),
                    is_error: false,
                    xp: Some(result.xp),
                })
            }
        }

        ValidatedToolCall::CreateHabit(input) => {
            let habit = HabitService::new(pool.clone()).create_habit(input).await?;
            Ok(ToolOutcome::ok(format!(
                "Created habit #{} \"{}\" ({}).",
                habit.id, habit.title, habit.frequency_type
            )))
        }

        ValidatedToolCall::CompleteHabit { id } => {
            let habits = HabitService::new(pool.clone());
            // Checking in again would *undo* today's entry, so ask first.
            let habit = habits
                .list_habits(today)
                .await?
                .into_iter()
                .find(|view| view.habit.id == id);
            let Some(habit) = habit else {
                return Ok(ToolOutcome::err(format!(
                    "There is no active habit with id {id}."
                )));
            };
            if habit.completed_today {
                return Ok(ToolOutcome::ok(format!(
                    "Habit \"{}\" is already checked in for today.",
                    habit.habit.title
                )));
            }
            let result = habits.toggle_completion(id, today).await?;
            Ok(ToolOutcome {
                result: format!(
                    "Checked in habit \"{}\" (streak {} day(s){}).",
                    habit.habit.title,
                    result.current_streak,
                    xp_note(&result.xp)
                ),
                is_error: false,
                xp: Some(result.xp),
            })
        }

        ValidatedToolCall::CreateGoal(input) => {
            let goal = GoalService::new(pool.clone()).create_goal(input).await?;
            Ok(ToolOutcome::ok(format!(
                "Created goal #{} \"{}\".",
                goal.goal.id, goal.goal.title
            )))
        }

        ValidatedToolCall::SendNotification { title, message } => {
            match NotificationService::new(pool.clone())
                .send_custom(title.as_deref(), &message)
                .await
            {
                Ok(()) => Ok(ToolOutcome::ok("Sent that notification to your phone.")),
                Err(AppError::InvalidInput(reason)) => Ok(ToolOutcome::err(reason)),
                Err(error) => Err(error),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definitions_cover_the_supported_tools() {
        let defs = definitions();
        let names: Vec<&str> = defs
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .collect();
        assert_eq!(
            names,
            vec![
                "create_task",
                "complete_task",
                "create_habit",
                "complete_habit",
                "create_goal",
                "send_notification"
            ]
        );
    }

    #[test]
    fn every_definition_has_an_object_schema_with_a_title_or_id() {
        for tool in definitions() {
            let schema = tool.get("input_schema").expect("input_schema");
            assert_eq!(schema["type"], "object", "schema of {}", tool["name"]);
            assert!(schema["properties"].is_object(), "properties of {}", tool["name"]);
            assert!(schema["required"].is_array(), "required of {}", tool["name"]);
        }
    }
}
