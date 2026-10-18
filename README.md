# Habit Planner

Desktop habit and task planner with XP / levels, also building for Android. Tauri 2 + Rust + SQLite + React + TypeScript + Tailwind v4.

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
  ai/                     JARVIS: Anthropic + OpenAI-compatible providers, tools, threads
src-tauri/migrations/     SQL migrations (embedded into the binary at compile time)
```

Data file: `%APPDATA%\com.habitplanner.app\habitplanner.db`. Delete it to start fresh.

## UI

The React side uses [shadcn/ui](https://ui.shadcn.com) components
(`components/ui/`), styled with the light/dark theme defined in
`src/index.css`. Screens live in `src/components/`:

```
src/App.tsx             app shell: sidebar, main, assistant panel, toasts
src/components/
  TodayPage.tsx         dashboard: XP ring, today's tasks and habits
  TasksPage.tsx         task list, filters of state, create / complete / delete
  HabitsPage.tsx        habit list, per-week check-in strip, streaks
  QuestsPage.tsx        daily/weekly quests and unlocked achievements
  GoalsPage.tsx         goals with milestones and progress
  CalendarPage.tsx      scheduled blocks on a week grid
  FocusPage.tsx         focus timer, session stats and optional focus music
  AnalyticsPage.tsx     XP, task, focus and habit-history charts
  JarvisPanel.tsx       assistant panel: chat threads and API-key settings
  ReminderSettings.tsx  phone-reminder setup dialog (ntfy topic / server)
```

Data file: `%APPDATA%\com.habitplanner.app\habitplanner.db`. Delete it to start fresh.

## Focus music

The Focus screen shows a Spotify-style now-playing bar. Its play/pause button is
the same control as the focus timer, so one press starts or pauses the session
and the audio together — it pauses when you pause, stops when the session ends,
and loops so it covers a session of any length. A mute button silences it
without stopping the timer.

Two sources are available, chosen with the music-note and folder buttons:

- **Built-in binaural beats** — a 40Hz gamma study track synthesised in the Web
  Audio API (two detuned sine waves, one per ear). It needs no network and works
  offline and on mobile.
- **Your own music or sound** — pick any audio file from your device with the
  folder button. It is stored locally (in the app's IndexedDB) so the choice
  survives a restart, and never leaves the machine.

## Phone reminders

The app can push a notification to your phone when a task is due, using
[ntfy.sh](https://ntfy.sh) — free, no account, no server to run. Open
**Reminders** in the sidebar to set it up:

1. Install the **ntfy** app (Android / iOS) and subscribe to a topic, e.g.
   `my-habit-reminders`.
2. Put the same topic name below, tick **Enable phone reminders**, save, then
   **Send test** — the phone should buzz.

Four things are delivered to that topic:

- **An alert the moment a task comes due** (checked every minute while the app
  runs, once per task, at most 5 per tick).
- **A habit reminder** at the time set on each habit (optional time field on the
  Habits screen), sent once a day per habit.
- **One digest a day** listing everything due today or overdue.
- **Anything JARVIS sends you** — say *"remind me to call Mum at 8"* and the
  `send_notification` tool pushes it.

The topic is effectively a secret: anyone who knows it can subscribe and read
your messages, so pick something unguessable. The **server** field is optional
(default `https://ntfy.sh`) if you'd rather use your own ntfy instance.
Reminders only fire while the app is open.

## Assistant (JARVIS)

The right-hand panel is JARVIS, a personal assistant in the spirit of the one
from Iron Man: calm, quietly witty and on your side. He will plan your day and
run errands in the app, but he is also someone to think out loud with — vent
about a rough day and he listens and answers with empathy before anything else.
When voice is on, he also greets you ("Good morning, sir. What can I do for you
today?") each time the app opens. Configure him under the gear icon:

- **Anthropic (default)** — paste an API key from
  [console.anthropic.com](https://console.anthropic.com/settings/keys); model
  defaults to `claude-sonnet-5-5`.
- **Any OpenAI-compatible service (free options)** — set a **Base URL** and leave
  the key empty if the service needs none. Examples:

  | Service | Base URL | Notes |
  | --- | --- | --- |
  | Ollama (local) | `http://localhost:11434/v1` | Free, private, no key; e.g. model `llama3.1:8b` |
  | Google Gemini | `https://generativelanguage.googleapis.com/v1beta/openai` | Free API key from Google AI Studio |
  | Groq | `https://api.groq.com/openai/v1` | Free API key, fast |

Keys and the base URL are stored locally in the `app_settings` table on this device
only and are never sent back to the UI (the key is reported only as "set"). A base
URL switches the provider to the OpenAI-compatible client.

The assistant receives a live snapshot of your tasks, habits, quests, goals and
focus stats as context, and can act on your behalf through validated tools
(`create_task`, `complete_task`, `create_habit`, `complete_habit`, `create_goal`,
`send_notification`). Chat threads are persisted in `ai_conversations` /
`ai_messages`; tool traffic stays in memory and is never stored.

### Voice

JARVIS can read his replies aloud. The speaker button next to the mic toggles
speech; the gear icon opens **Voice settings**, where you pick a sound source,
choose a voice, set speed, and **Test voice**. Settings are saved in the
`app_settings` table (`tts_enabled`, `tts_source`, `tts_voice`,
`tts_cloud_voice`, `tts_rate`, `tts_pitch`) so they survive a restart.

There are two sound sources:

- **Cloud** (the default) uses Microsoft Edge's free neural voices — the same
  ones behind Edge's Read Aloud feature, with no account or API key. The
  default is **Ryan (en-GB)**, a British male voice, which is about as close to
  the film assistant as free speech gets. Sentences are synthesised to audio by
  the Rust backend and played back in order, so a streamed reply comes out
  smoothly. This is the only source that gives JARVIS a British voice, and it
  needs an internet connection.
- **System** uses the voices installed on your device through the Web Speech
  API. It works with no network, but Windows ships without a British voice, so
  the panel shows **Open voice settings** (jumps straight to Windows Speech
  settings): add **Daniel (United Kingdom)** — or a Natural voice such as Ryan
  — restart the app, and **Auto (butler voice)** picks it up.

Other behaviour worth knowing:

- If the cloud service cannot be reached (offline, or the request fails), the
  line is spoken with the system voice instead, so JARVIS keeps talking.
- Where the platform can produce no speech at all, the speaker button does not
  appear and the rest of the app is unaffected.
- System-voice replies are kept alive with a periodic pause/resume, working
  around a Chromium limitation that otherwise cuts a single utterance off after
  ~15s. (Cloud playback has no such limit.)
- Streaming replies are buffered and spoken sentence by sentence as they
  arrive.

## Phone / Android

The same app runs on Android with a phone-sized layout. Below 768px the
three-column desktop view becomes a top status bar, a bottom icon nav and the
JARVIS panel as a full-screen overlay (opened from the orb in the top bar); at
768px and wider the desktop layout is unchanged. The web fonts are self-hosted
under `public/fonts/`, so the app looks right with no network.

Phone reminders can be delivered either as a **local notification** (works with
no network) or pushed to your phone with [ntfy.sh](https://ntfy.sh); both are
configured under **Reminders**.

### Building for Android

Prerequisites, in addition to the desktop ones above:

1. **Android Studio** with the Android **SDK** and **NDK**, and **CMake**
   (installed via the SDK Manager). The NDK is required because `reqwest` pulls
   in a TLS stack that compiles native code for the phone.
2. Environment variables (adjust the versions/folders to match your install):

   ```powershell
   setx ANDROID_HOME "$env:LOCALAPPDATA\Android\Sdk"
   setx NDK_HOME    "$env:LOCALAPPDATA\Android\Sdk\ndk\27.0.12077973"
   setx JAVA_HOME   "C:\Program Files\Android\Android Studio\jbr"
   ```

3. Add the Rust Android targets and initialise the project:

   ```powershell
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   npm run tauri android init
   ```

4. Run or build:

   ```powershell
   npm run tauri android dev        # onto a connected device / emulator
   npm run tauri android build      # signed APK/AAB in src-tauri/gen/android/.../outputs
   ```

5. Regenerate the launcher icons (already checked in) after changing
   `app-icon.png`: `npm run tauri icon app-icon.png`.

Phone reminders on Android rely on the notification plugin, which is declared in
`src-tauri/capabilities/default.json`. The background reminder loop only runs
while the app is open; a suspended app is resumed by the OS, not by us.

## Adding a feature (the pattern)

1. Add SQL to a new migration file `src-tauri/migrations/<timestamp>_<name>.sql` (never edit an applied one once you have real data).
2. Add a struct in `models/`, functions in `repositories/`, rules in `services/`, a command in `commands/`.
3. Register the command in `invoke_handler![...]` in `src-tauri/src/lib.rs`.
4. Add the wrapper to `src/lib/api.ts` and the type to `src/types.ts`.
