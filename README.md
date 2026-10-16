# Habit Planner

Desktop habit and task planner with XP / levels. Tauri 2 + Rust + SQLite + React + TypeScript + Tailwind v4.

## Run it (Windows)

Prerequisites: Node 20+, Rust (`rustup`), Visual Studio Build Tools ("Desktop development with C++"), and WebView2 (already on Windows 11).

```powershell
npm install
npm run tauri dev
```

The first run compiles the Rust crates and takes a few minutes. Later runs are fast.

Other commands:

```powershell
cd src-tauri
cargo test      # unit tests: validation, XP levels, streaks
cargo clippy
cd ..
npm run lint
npm run tauri build   # installer in src-tauri/target/release/bundle
```

If you ever replace `app-icon.png`, regenerate the icon set with `npm run tauri icon app-icon.png`.

## Where things live

```
src/                      React UI (components/, lib/api.ts = the only place that calls Rust)
src-tauri/src/
  commands/               thin Tauri commands  (frontend -> Rust entry points)
  services/               business rules: validation, XP, streaks, transactions
  repositories/           SQL only
  models/                 structs shared with the UI (serialized as camelCase)
  database/connection.rs  opens SQLite, enforces foreign keys, runs migrations
  ai/                     Phase 5 stubs
src-tauri/migrations/     SQL migrations (embedded into the binary at compile time)
```

Data file: `%APPDATA%\com.habitplanner.app\habitplanner.db`. Delete it to start fresh.

## UI

The React side uses [shadcn/ui](https://ui.shadcn.com) components
(`components/ui/`), styled with the light/dark theme defined in
`src/index.css`. Screens live next to `App.tsx`:

```
src/components/
  App.tsx               app shell: sidebar, main, assistant panel, toasts
  TodayPage.tsx        dashboard: XP ring, today's tasks and habits
  TasksPage.tsx        task list, filters of state, create / complete / delete
  HabitsPage.tsx       habit list, per-week check-in strip, streaks
  PlaceholderPage.tsx  screens that arrive in later phases
  JarvisPanel.tsx      assistant panel (Phase 5; currently a placeholder)
```

Data file: `%APPDATA%\com.habitplanner.app\habitplanner.db`. Delete it to start fresh.

## Adding a feature (the pattern)

1. Add SQL to a new migration file `src-tauri/migrations/<timestamp>_<name>.sql` (never edit an applied one once you have real data).
2. Add a struct in `models/`, functions in `repositories/`, rules in `services/`, a command in `commands/`.
3. Register the command in `invoke_handler![...]` in `src-tauri/src/lib.rs`.
4. Add the wrapper to `src/lib/api.ts` and the type to `src/types.ts`.
