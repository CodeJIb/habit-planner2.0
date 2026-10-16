fn main() {
    // Re-run the build when a migration file changes (sqlx::migrate! embeds them at compile time).
    println!("cargo:rerun-if-changed=migrations");
    tauri_build::build()
}
