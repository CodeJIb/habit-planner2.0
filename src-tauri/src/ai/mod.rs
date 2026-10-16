//! Phase 5: JARVIS-style assistant. All model calls will go through this module so
//! API keys never reach the frontend.
pub mod context;
pub mod conversation;
pub mod provider;
pub mod tools;
pub mod validation;
