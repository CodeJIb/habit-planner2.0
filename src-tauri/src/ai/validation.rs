//! Phase 5: validate every tool call from the model before it touches the
//! database. Nothing here trusts the model: names, types, lengths and enums
//! are all re-checked even if the schema said they were fine.

use serde_json::Value;

use crate::models::goal::GoalInput;
use crate::models::habit::HabitInput;
use crate::models::task::TaskInput;

const PRIORITIES: [&str; 4] = ["low", "medium", "high", "urgent"];
const FREQUENCIES: [&str; 3] = ["daily", "weekly", "custom"];

const MAX_TITLE: usize = 200;
const MAX_DESCRIPTION: usize = 2000;
const MAX_DATE: usize = 40;

/// A tool call exactly as the model sent it (untrusted).
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: Value,
}

/// A tool call that survived validation; the only shape services may execute.
#[derive(Debug, Clone)]
pub enum ValidatedToolCall {
    CreateTask(TaskInput),
    CompleteTask { id: i64 },
    CreateHabit(HabitInput),
    CompleteHabit { id: i64 },
    CreateGoal(GoalInput),
    SendNotification { title: Option<String>, message: String },
}

pub fn validate(call: &ToolCall) -> Result<ValidatedToolCall, String> {
    let input = call
        .input
        .as_object()
        .ok_or_else(|| format!("Tool '{}' input must be a JSON object", call.name))?;

    match call.name.as_str() {
        "create_task" => {
            let priority = optional_string(input, "priority", 20)?
                .map(|p| p.to_lowercase())
                .unwrap_or_else(|| "medium".to_string());
            if !PRIORITIES.contains(&priority.as_str()) {
                return Err(format!(
                    "'{priority}' is not a known priority (use {})",
                    PRIORITIES.join(", ")
                ));
            }
            Ok(ValidatedToolCall::CreateTask(TaskInput {
                title: required_string(input, "title", MAX_TITLE, "create_task")?,
                description: optional_string(input, "description", MAX_DESCRIPTION)?,
                priority,
                category_id: None,
                due_at: optional_string(input, "due_at", MAX_DATE)?,
                scheduled_start_at: None,
                estimated_duration_minutes: optional_positive_int(
                    input,
                    "estimated_duration_minutes",
                )?,
                recurrence_rule: None,
            }))
        }
        "complete_task" => Ok(ValidatedToolCall::CompleteTask {
            id: required_id(input, "task_id", "complete_task")?,
        }),
        "create_habit" => {
            let frequency = optional_string(input, "frequency", 20)?
                .map(|f| f.to_lowercase())
                .unwrap_or_else(|| "daily".to_string());
            if !FREQUENCIES.contains(&frequency.as_str()) {
                return Err(format!(
                    "'{frequency}' is not a known frequency (use {})",
                    FREQUENCIES.join(", ")
                ));
            }
            Ok(ValidatedToolCall::CreateHabit(HabitInput {
                title: required_string(input, "title", MAX_TITLE, "create_habit")?,
                description: optional_string(input, "description", MAX_DESCRIPTION)?,
                category_id: None,
                frequency_type: frequency,
                frequency_config: None,
                target_count: None,
                reminder_time: None,
            }))
        }
        "complete_habit" => Ok(ValidatedToolCall::CompleteHabit {
            id: required_id(input, "habit_id", "complete_habit")?,
        }),
        "create_goal" => Ok(ValidatedToolCall::CreateGoal(GoalInput {
            title: required_string(input, "title", MAX_TITLE, "create_goal")?,
            description: optional_string(input, "description", MAX_DESCRIPTION)?,
            target_date: optional_string(input, "target_date", MAX_DATE)?,
        })),
        "send_notification" => {
            let message = required_string(input, "message", 1000, "send_notification")?;
            let title = optional_string(input, "title", 120)?;
            Ok(ValidatedToolCall::SendNotification { title, message })
        }
        other => Err(format!("Unknown tool '{other}'")),
    }
}

/// A string field that must exist, be a string and be within `max` characters.
fn required_string(
    input: &serde_json::Map<String, Value>,
    field: &str,
    max: usize,
    tool: &str,
) -> Result<String, String> {
    let value = input
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .ok_or_else(|| format!("Tool '{tool}' needs a text '{field}'"))?;
    if value.is_empty() {
        return Err(format!("Tool '{tool}' needs a non-empty '{field}'"));
    }
    if value.chars().count() > max {
        return Err(format!("'{field}' must be {max} characters or fewer"));
    }
    Ok(value.to_string())
}

/// A string field that may be absent or null; blank becomes `None`.
fn optional_string(
    input: &serde_json::Map<String, Value>,
    field: &str,
    max: usize,
) -> Result<Option<String>, String> {
    match input.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            let text = value
                .as_str()
                .ok_or_else(|| format!("'{field}' must be text"))?
                .trim();
            if text.is_empty() {
                return Ok(None);
            }
            if text.chars().count() > max {
                return Err(format!("'{field}' must be {max} characters or fewer"));
            }
            Ok(Some(text.to_string()))
        }
    }
}

/// A numeric id that must be present and a positive whole number.
fn required_id(
    input: &serde_json::Map<String, Value>,
    field: &str,
    tool: &str,
) -> Result<i64, String> {
    let value = input
        .get(field)
        .ok_or_else(|| format!("Tool '{tool}' needs a '{field}'"))?;
    let id = match value {
        Value::Number(n) if n.is_i64() => n.as_i64().unwrap_or(0),
        // The model may write 3.0; accept it only when it is a whole number.
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.fract() == 0.0 && f > 0.0 && f <= i64::MAX as f64 {
                f as i64
            } else {
                return Err(format!("'{field}' must be a whole number"));
            }
        }
        _ => return Err(format!("'{field}' must be a number")),
    };
    if id <= 0 {
        return Err(format!("'{field}' must be greater than zero"));
    }
    Ok(id)
}

/// An optional positive integer (durations, counts).
fn optional_positive_int(
    input: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<i64>, String> {
    match input.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            let n = value
                .as_i64()
                .or_else(|| value.as_f64().filter(|f| f.fract() == 0.0).map(|f| f as i64))
                .ok_or_else(|| format!("'{field}' must be a whole number"))?;
            if n <= 0 {
                return Err(format!("'{field}' must be positive"));
            }
            Ok(Some(n))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn call(name: &str, input: Value) -> ToolCall {
        ToolCall {
            id: "toolu_1".to_string(),
            name: name.to_string(),
            input,
        }
    }

    #[test]
    fn accepts_a_full_create_task() {
        let result = validate(&call(
            "create_task",
            json!({
                "title": "  Write the report  ",
                "description": "Quarterly numbers",
                "priority": "High",
                "due_at": "2026-10-20",
                "estimated_duration_minutes": 45
            }),
        ))
        .unwrap();
        let ValidatedToolCall::CreateTask(task) = result else {
            panic!("expected CreateTask");
        };
        assert_eq!(task.title, "Write the report"); // trimmed
        assert_eq!(task.priority, "High".to_lowercase()); // normalised
        assert_eq!(task.due_at.as_deref(), Some("2026-10-20"));
        assert_eq!(task.estimated_duration_minutes, Some(45));
    }

    #[test]
    fn create_task_defaults_to_medium_priority() {
        let ValidatedToolCall::CreateTask(task) =
            validate(&call("create_task", json!({ "title": "Water plants" }))).unwrap()
        else {
            panic!("expected CreateTask");
        };
        assert_eq!(task.priority, "medium");
        assert!(task.description.is_none());
        assert!(task.due_at.is_none());
    }

    #[test]
    fn rejects_missing_or_empty_or_long_titles() {
        assert!(validate(&call("create_task", json!({}))).is_err());
        assert!(validate(&call("create_task", json!({ "title": "   " }))).is_err());
        assert!(validate(&call("create_task", json!({ "title": 42 }))).is_err());
        let long = "x".repeat(MAX_TITLE + 1);
        assert!(validate(&call("create_task", json!({ "title": long }))).is_err());
    }

    #[test]
    fn rejects_unknown_priority_and_frequency() {
        assert!(
            validate(&call(
                "create_task",
                json!({ "title": "Task", "priority": "someday" })
            ))
            .is_err()
        );
        assert!(
            validate(&call(
                "create_habit",
                json!({ "title": "Habit", "frequency": "hourly" })
            ))
            .is_err()
        );
    }

    #[test]
    fn accepts_complete_task_ids_including_float_whole_numbers() {
        let ValidatedToolCall::CompleteTask { id } =
            validate(&call("complete_task", json!({ "task_id": 7 }))).unwrap()
        else {
            panic!("expected CompleteTask");
        };
        assert_eq!(id, 7);

        let ValidatedToolCall::CompleteTask { id } =
            validate(&call("complete_task", json!({ "task_id": 7.0 }))).unwrap()
        else {
            panic!("expected CompleteTask");
        };
        assert_eq!(id, 7);
    }

    #[test]
    fn rejects_bad_ids() {
        assert!(validate(&call("complete_task", json!({ "task_id": 0 }))).is_err());
        assert!(validate(&call("complete_task", json!({ "task_id": -3 }))).is_err());
        assert!(validate(&call("complete_task", json!({ "task_id": 2.5 }))).is_err());
        assert!(validate(&call("complete_task", json!({ "task_id": "12" }))).is_err());
        assert!(validate(&call("complete_task", json!({}))).is_err());
    }

    #[test]
    fn rejects_non_object_input_and_unknown_tools() {
        assert!(validate(&call("create_task", json!("just a string"))).is_err());
        assert!(validate(&call("delete_everything", json!({}))).is_err());
    }

    #[test]
    fn optional_fields_reject_wrong_types() {
        let wrong_due = call("create_task", json!({ "title": "T", "due_at": 5 }));
        assert!(validate(&wrong_due).is_err());

        let wrong_duration =
            call("create_task", json!({ "title": "T", "estimated_duration_minutes": -10 }));
        assert!(validate(&wrong_duration).is_err());

        let null_target = call("create_goal", json!({ "title": "Goal", "target_date": null }));
        assert!(validate(&null_target).is_ok());
    }

    #[test]
    fn create_habit_and_goal_round_trip() {
        let ValidatedToolCall::CreateHabit(habit) =
            validate(&call("create_habit", json!({ "title": "Meditate" }))).unwrap()
        else {
            panic!("expected CreateHabit");
        };
        assert_eq!(habit.frequency_type, "daily");

        let ValidatedToolCall::CreateGoal(goal) = validate(&call(
            "create_goal",
            json!({ "title": "Ship v1", "target_date": "2026-12-01" }),
        ))
        .unwrap()
        else {
            panic!("expected CreateGoal");
        };
        assert_eq!(goal.target_date.as_deref(), Some("2026-12-01"));
    }

    #[test]
    fn send_notification_needs_a_message() {
        let ValidatedToolCall::SendNotification { title, message } = validate(&call(
            "send_notification",
            json!({ "title": "Stand up", "message": "Stretch now." }),
        ))
        .unwrap()
        else {
            panic!("expected SendNotification");
        };
        assert_eq!(title.as_deref(), Some("Stand up"));
        assert_eq!(message, "Stretch now.");

        assert!(validate(&call("send_notification", json!({}))).is_err());
        assert!(validate(&call("send_notification", json!({ "message": "  " }))).is_err());
        let long = "x".repeat(1001);
        assert!(validate(&call("send_notification", json!({ "message": long }))).is_err());
    }
}
