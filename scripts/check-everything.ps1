# Smoke-test the local build: run unit + integration checks from the Tauri crate
# and a quick frontend typecheck/build.
param()

$ErrorActionPreference = 'Stop'

Push-Location (Join-Path $PSScriptRoot '..')
try {
    Write-Host 'Frontend: tsc...'
    npm run build  | Out-Null
    if (-not $?) { throw 'npm run build failed' }

    Write-Host 'Rust: cargo test (unit + integration)...'
    Push-Location src-tauri
    cargo test
    if (-not $?) { throw 'cargo test failed' }
    Pop-Location

    Write-Host 'Rust: cargo build (debug)...'
    Push-Location src-tauri
    cargo build
    if (-not $?) { throw 'cargo build failed' }
    Pop-Location

    Write-Host ''
    Write-Host 'All checks passed.'
}
finally {
    Pop-Location
}
