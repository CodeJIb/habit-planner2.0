fn main() {
    // Re-run the build when a migration file changes (sqlx::migrate! embeds them at compile time).
    println!("cargo:rerun-if-changed=migrations");

    let target = std::env::var("TARGET").unwrap_or_default();
    let windows_msvc = target.contains("windows") && target.contains("msvc");

    if windows_msvc {
        embed_manifest_for_all_targets();
    } else {
        tauri_build::build();
    }
}

/// Embed the Windows application manifest through the linker instead of as a
/// bin-only resource.
///
/// `tauri_build` links its Common-Controls v6 manifest into the *application
/// binary* only, so cargo's test executables get none. Those test exes still
/// import `TaskDialogIndirect` from `comctl32` (transitively via tao/muda), a
/// symbol exported only by comctl32 **v6**. Without a manifest the loader binds
/// legacy comctl32 v5 and the process dies before running a single test with
/// `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139).
///
/// Embedding via `rustc-link-arg` covers every linked product (bins, cdylib,
/// tests), so the manifest must move out of `tauri_build` to avoid a duplicate
/// `RT_MANIFEST` resource (`LNK1123`). `new_without_app_manifest()` drops the
/// resource; `app.manifest` reproduces it 1:1 for the linker.
fn embed_manifest_for_all_targets() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest()),
    )
    .expect("failed to run tauri-build");

    let manifest = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo"),
    )
    .join("app.manifest");

    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}
