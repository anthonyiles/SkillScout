# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Tech Stack

- **Frontend**: Vue 3 (`<script setup>` SFCs), TypeScript, Vite, Vue Router, VueUse
- **Backend**: Tauri v2 (Rust), SQLite via `rusqlite` (bundled)
- **Auth**: GitHub OAuth Device Flow, tokens stored in OS keyring via the `keyring` crate
- **HTTP**: `reqwest` with `rustls-tls` for GitHub API calls

## Commands

### Development
```bash
npm run tauri dev      # Run the full desktop app in dev mode (starts Vite + Tauri)
npm run dev            # Start Vite frontend only (port 1420, no Tauri shell)
```

### Building
```bash
npm run build          # TypeScript check + Vite build
npm run tauri build    # Build production desktop bundle
```

### Rust (run inside `src-tauri/`)
```bash
cargo build            # Compile Rust backend
cargo test             # Run Rust tests
cargo check            # Type-check without building
```

## Architecture

### IPC Boundary

The frontend communicates with the Rust backend exclusively via `invoke()` from `@tauri-apps/api/core`. All Tauri commands are registered in `src-tauri/src/lib.rs` and implemented in `src-tauri/src/commands/`. The backend emits events (e.g. `repo_synced`) back to the frontend via `app.emit()`.

### Rust Backend (`src-tauri/src/`)

- **`lib.rs`** — App entry point: initializes SQLite, registers all Tauri commands, starts the 30-minute background sync loop.
- **`db.rs`** — Opens/creates the SQLite database in the Tauri app data dir (`app_state.db`), creates tables, seeds default agents. `AppState { db: Mutex<Connection> }` is stored as managed Tauri state.
- **`models.rs`** — Shared `Serialize`/`Deserialize` structs: `Agent`, `Project`, `RepositoryItem`, `ItemSelection`, `PromotedItem`, etc.
- **`commands/`** — One file per concern:
  - `auth.rs` — GitHub Device Flow OAuth (`start_github_device_flow`, `poll_github_token`, `check_github_auth`, `logout_github`)
  - `github.rs` — `promote_item` (creates a PR from a local skill/rule) and `check_pr_status`
  - `sync.rs` — `sync_repo` (git clone/pull into app data dir, parse `skills/` and `rules/` folders, upsert into DB), `apply_skills` (copy files to project directories), `check_existing`, `get_project_files`
  - `state.rs` — CRUD for `settings`, `agents`, and `projects` tables
  - `items.rs` — CRUD for `repository_items`, `item_selections`, and `promoted_items` tables
- **`github/`** — Low-level GitHub REST API helpers (blob creation, tree, commit, branch, PR)
- **`utils/`** — `auth.rs` (keyring token load/save), `filesystem.rs` (`copy_dir_all`)

### SQLite Schema

| Table | Purpose |
|-------|---------|
| `settings` | Key/value app config (e.g. `repoUrl`) |
| `agents` | AI agent definitions with `skills_path` / `rules_path` (cursor, jetbrains, claude seeded by default) |
| `projects` | Registered local project directories |
| `project_agents` | Many-to-many: which agents are active for a project |
| `repository_items` | Skills/rules synced from the remote GitHub repo |
| `item_selections` | Which repository items are applied to which projects (tracks `applied_sha` for update detection) |
| `promoted_items` | Local items pushed back to the remote repo via PR |

### Frontend (`src/`)

- **`router.ts`** — Six routes: `/` (Projects), `/skills`, `/rules`, `/unmanaged`, `/agents`, `/settings`
- **`views/`** — One Vue component per route. `SkillsView` and `RulesView` render a skills×projects selection matrix; `UnmanagedView` discovers local skills/rules not tracked in the DB; `ProjectsView` manages registered project paths.
- **`composables/useToast.ts`** — Shared toast notification state used across all views.
- **`components/`** — Reusable UI primitives (`BaseButton`, `CardItem`, `TickBox`, `ContentModal`, `ConfirmModal`, `InputField`, `PageLayout`, `EmptyState`, `Sidebar`, `Toast`, `GitHubLoginModal`).

### Core Data Flow

1. User configures a GitHub repo URL in Settings → `set_setting("repoUrl", ...)` saved to DB.
2. `sync_repo` clones/pulls the repo into the Tauri app data dir, reads `skills/` and `rules/` top-level entries, hashes content, upserts into `repository_items`.
3. `SkillsView`/`RulesView` display a matrix of items × projects; toggling a cell calls `toggle_item_selection` then `apply_skills` to copy files into the project's agent-specific subdirectory.
4. `UnmanagedView` scans project directories for items not in `repository_items` and can `promote_item` them back to the repo via a GitHub PR (creates branch → blob → tree → commit → PR).
5. Background loop in `lib.rs` re-runs `sync_repo` every 30 minutes.
