//! Repositories contain SQL only. Every function takes `&mut SqliteConnection`,
//! so a service can run several of them inside one transaction
//! (pass `&mut tx`) or on a plain pooled connection (pass `&mut conn`).
pub mod achievement;
pub mod ai;
pub mod category;
pub mod focus_session;
pub mod goal;
pub mod habit;
pub mod quest;
pub mod scheduled_block;
pub mod task;
