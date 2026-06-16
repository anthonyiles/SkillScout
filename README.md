# SkillScout

A cross-platform desktop app for managing and distributing AI agent skills and rules across your local projects. Built with Tauri v2 + Vue 3.

SkillScout connects to a GitHub repository that acts as your team's central library of AI agent configurations — skills and rules for tools like Cursor, JetBrains AI, and Claude Code. It lets you browse, apply, and contribute to that library without ever leaving the desktop.

---

## Features

- **Browse & apply skills/rules** — View all skills and rules in your shared GitHub repo and toggle them onto any of your registered local projects with a single click.
- **Multi-agent support** — Applies files to the correct agent-specific subdirectory for Cursor, JetBrains, Claude Code, or any custom agent you configure.
- **Manage unmanaged items** — Discover local skills/rules that aren't tracked in the repo yet and promote them back via a GitHub Pull Request — directly from the app.
- **Automatic sync** — Background sync pulls the latest changes from your repo every 30 minutes. Manual sync is also available.
- **GitHub OAuth** — Authenticates via the GitHub Device Flow. Tokens are stored securely in your OS keyring (no plaintext secrets on disk).
- **Project registry** — Register multiple local project directories and manage which skills/rules are applied to each independently. On macOS, a folder picker is provided to grant the necessary filesystem permissions.
- **Update detection** — Tracks content hashes so the UI shows when an applied skill/rule has been updated upstream.
- **Auto-update** — The app checks for new releases on startup and lets you install them in-app. A beta tester toggle in Settings opts you into pre-release builds.

---

## Screenshots

> _Coming soon_

## Download

Pre-built installers for Windows, macOS, and Linux are available on the [Releases](https://github.com/anthonyiles/SkillScout/releases) page. Download the appropriate file for your platform and run the installer — no build step required.

---

## Prerequisites

> These steps are only needed if you want to **build from source** or contribute to development.

| Tool | Version |
|------|---------|
| [Node.js](https://nodejs.org/) | 18+ |
| [Rust](https://rustup.rs/) | stable (latest) |
| [Tauri CLI prerequisites](https://v2.tauri.app/start/prerequisites/) | platform-specific (see below) |

### Platform-specific Tauri dependencies

Follow the official Tauri v2 guide for your OS:

- **Linux**: https://v2.tauri.app/start/prerequisites/#linux
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Windows**: Microsoft C++ Build Tools + WebView2

---

## GitHub OAuth App

SkillScout uses the GitHub Device Flow for authentication. The app ships with the production OAuth client ID baked into the binary — contributors don't need to register their own app or set up any environment variables to build and run it.

<details>
<summary>Registering your own OAuth App (forks / self-hosted deployments only)</summary>

1. Go to **GitHub → Settings → Developer settings → OAuth Apps → New OAuth App**.
2. Set **Application name** to anything (e.g. `SkillScout`).
3. Set **Homepage URL** to `http://localhost`.
4. Set **Authorization callback URL** to `http://localhost` (Device Flow does not use a redirect URL, but the field is required).
5. Click **Register application**, then note the **Client ID**.
6. Add `GITHUB_CLIENT_ID=<your-client-id>` to your `.env` file.

</details>

---

## Installation & Setup

### 1. Clone the repository

```bash
git clone https://github.com/your-username/skillscout.git
cd skillscout
```

### 2. Install frontend dependencies

```bash
npm install
```

### 3. Run in development mode

```bash
npm run tauri dev
```

This starts the Vite dev server and the Tauri shell together.

> **Optional:** Copy `.env.example` to `.env` if you want to override the OAuth client ID (e.g. for a fork using its own GitHub OAuth App). Do **not** commit `.env`.

---

## Setting Up Your Skills Repository

SkillScout syncs from a GitHub repository with the following directory structure:

```text
your-skills-repo/
├── skills/
│   ├── my-skill/          # A skill is a directory
│   │   ├── skill.md
│   │   └── ...
│   └── another-skill/
└── rules/
    ├── my-rule/           # A rule is a directory
    │   └── rule.md
    └── ...
```

- Top-level entries inside `skills/` and `rules/` are treated as individual items.
- Each item can be a single file or a directory.

Once you have a repository set up, paste its URL into **Settings** inside SkillScout and click **Sync**.

---

## Usage

1. **Authenticate** — Open the app and sign in with GitHub via the login prompt.
2. **Configure repo** — Go to **Settings** and enter the GitHub URL of your skills repository.
3. **Sync** — Click **Sync** to pull the latest skills and rules from the repo.
4. **Register projects** — Go to **Projects** and add the local directories you want to manage.
5. **Apply skills/rules** — Go to **Skills** or **Rules** and toggle the checkboxes to apply items to your projects. Files are copied immediately into the correct agent subdirectory.
6. **Promote local items** — Go to **Unmanaged** to discover local skills/rules not yet in the repo. Click **Promote** to open a PR from within the app.
7. **Updates** — The app checks for updates on startup. You can also trigger a manual check in **Settings**. Enable **Beta Tester** in Settings to receive pre-release builds.

---

## Project Structure

```text
skillscout/
├── src/                   # Vue 3 frontend
│   ├── views/             # One component per route
│   ├── components/        # Reusable UI primitives
│   ├── composables/       # Shared state (e.g. useToast, useUpdater)
│   └── router.ts          # Vue Router config
├── src-tauri/             # Rust / Tauri backend
│   ├── src/
│   │   ├── lib.rs         # App entry point, command registration, sync loop
│   │   ├── db.rs          # SQLite init and AppState
│   │   ├── models.rs      # Shared serde structs
│   │   ├── commands/      # Tauri command handlers
│   │   ├── github/        # Low-level GitHub REST API helpers
│   │   └── utils/         # Keyring auth, filesystem helpers
│   └── tauri.conf.json    # App metadata and bundle config
├── worker/                # Cloudflare Worker — serves the Tauri update manifest
├── package.json
└── vite.config.ts
```

---

## Development Commands

| Command | Description |
|---------|-------------|
| `npm run tauri dev` | Run the full app in dev mode |
| `npm run dev` | Start Vite frontend only (port 1420) |
| `npm run build` | TypeScript check + Vite build |
| `npm test` | Run Vitest in watch mode |
| `npm run test:run` | Run Vitest once (CI mode) |
| `npm run coverage` | Run Vitest with V8 coverage report |

**Rust (run inside `src-tauri/`):**

| Command | Description |
|---------|-------------|
| `cargo build` | Compile the Rust backend |
| `cargo test` | Run Rust tests |
| `cargo check` | Type-check without building |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Vue 3 (`<script setup>`), TypeScript, Vite, Tailwind CSS v4, VueUse |
| Desktop shell | Tauri v2 |
| Backend | Rust |
| Database | SQLite via `rusqlite` (bundled) |
| Auth | GitHub OAuth Device Flow, OS keyring via `keyring` crate |
| HTTP | `reqwest` with `rustls-tls` |

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on reporting bugs, suggesting features, and submitting pull requests.

---

## License

[MIT](LICENSE)
