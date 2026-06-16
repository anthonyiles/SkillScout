# MCP Server Feature Plan

## Overview

Add MCP (Model Context Protocol) server management to SkillScout. MCP servers are synced from the GitHub repo (like skills/rules) and applied by merging into each agent's `mcp.json` config file. Each server can be applied globally (`~/.cursor/mcp.json`) or per-project (`.cursor/mcp.json` inside the project dir) — **this is a user decision at apply time, not something fixed in the server definition**.

## Repo format

MCP servers live in an `mcp-servers/` folder in the user's GitHub repo alongside `skills/` and `rules/`. Each server is a single `.json` file:

```
repo/
  skills/
  rules/
  mcp-servers/
    context7.json
    github.json
```

Each file contains only the MCP server definition — no `scope` field:
```json
{
  "key": "context7",
  "command": "npx",
  "args": ["-y", "@upstash/context7-mcp"],
  "env": {}
}
```

- `key` — the key used in the target `mcpServers` JSON object
- `command`, `args`, `env` — standard MCP server config fields; `args` and `env` are optional

Where a server gets applied (globally or per-project) is controlled entirely by the user in the UI. The same server can be applied both globally and per-project independently.

## Data model

No new DB tables needed — MCP servers reuse `repository_items` (with `folder = "mcp-servers"`) and `item_selections`.

`item_selections` records an additional `scope` column to distinguish a global selection from a project-level one for the same server:

| Column addition | Type | Purpose |
|----------------|------|---------|
| `scope` | `TEXT NOT NULL DEFAULT 'project'` | `'global'` or `'project'` |

The primary key becomes `(item_id, project_id, scope)` — a server can be independently selected for global apply and project apply. For global selections, `project_id` is `0` (a sentinel; no project row exists for it).

### Agent table migration

Add two columns to `agents` via `ALTER TABLE` (same migration pattern as `sub_folder`):

| Column | Type | Purpose |
|--------|------|---------|
| `mcp_path` | `TEXT NOT NULL DEFAULT ''` | Project-level MCP file path (e.g. `.cursor/mcp.json`) |
| `global_mcp_path` | `TEXT NOT NULL DEFAULT ''` | Global MCP file path (e.g. `~/.cursor/mcp.json`) |

Default seed values:

| Agent | `mcp_path` | `global_mcp_path` |
|-------|-----------|------------------|
| Windsurf | `.windsurf/mcp.json` | `~/.windsurf/mcp.json` |
| Claude Code | `.claude/mcp.json` | `~/.claude/mcp.json` |
| JetBrains | *(empty)* | *(empty)* |

`~` is expanded at apply time by the Rust backend using `dirs::home_dir()` — never by the frontend — so it works correctly on Windows too.

## Backend changes

### `models.rs`
- Add `mcp_path` and `global_mcp_path` to `Agent` struct
- Add `McpApplyTask` struct:
  ```rust
  pub struct McpApplyTask {
      pub target_path: String,  // absolute path to mcp.json, ~ already expanded by backend
      pub server_key: String,
      pub config: String,       // JSON string: {command, args?, env?}
      pub remove: bool,
  }
  ```
- Add `McpApplyResult` struct:
  ```rust
  pub struct McpApplyResult {
      pub applied: usize,
      pub adopted: usize,
      pub clashes: Vec<McpClash>,
  }

  pub struct McpClash {
      pub server_key: String,
      pub target_path: String,
      pub existing_config: String,  // JSON string of what's currently in the file
      pub incoming_config: String,  // JSON string of what we would write
  }
  ```

### `db.rs`
- Add two `ALTER TABLE agents ADD COLUMN` migrations (ignore duplicate column errors)
- Add `ALTER TABLE item_selections ADD COLUMN scope TEXT NOT NULL DEFAULT 'project'` migration
- Update primary key handling for `item_selections` (SQLite can't alter PKs, so new inserts use `(item_id, project_id, scope)` uniqueness via `ON CONFLICT`)
- Update `seed_defaults` / `db_reset_agents` to set `mcp_path` / `global_mcp_path` for the three default agents

### `commands/state.rs`
- Update `db_get_agents`, `db_save_agent`, `db_reset_agents` to read/write the two new columns

### `commands/sync.rs`
- Add `read_folder("mcp-servers")` after `read_folder("rules")`
- `.json` files are read as-is; content stored as raw JSON string in `repository_items.content`

### `commands/mcp.rs` (new file)

One command: `apply_mcp_servers(tasks: Vec<McpApplyTask>) -> Result<McpApplyResult, String>`

The frontend builds tasks; the backend resolves `~` using `dirs::home_dir()` before any file I/O.

Logic per task:
1. Read existing `mcp.json` at `target_path` (start with `{"mcpServers":{}}` if file absent)
2. Parse `mcpServers` object
3. If `remove`: delete `server_key` from object, write back, increment `applied`
4. If not `remove`:
   - Key absent → write entry, increment `applied`
   - Key present, all fields identical → adopt (no file write), increment `adopted`
   - Key present, any field differs → append to `clashes`, do not write

Returns `McpApplyResult`. Frontend calls this once — safe work (absent keys + identical keys) is done immediately, clashes are returned for the user to confirm. If user confirms, frontend calls again with only the clashing tasks (force-overwrite is implicit on the second call).

Uses `spawn_blocking` for all file I/O.

### `lib.rs`
- Register `apply_mcp_servers` in `tauri::Builder`

## Frontend changes

### `types.ts`
Add to `Agent`:
```ts
mcpPath: string
globalMcpPath: string
```

### `api.ts`
```ts
export interface McpApplyTask {
  targetPath: string
  serverKey: string
  config: string
  remove: boolean
}

export interface McpClash {
  serverKey: string
  targetPath: string
  existingConfig: string
  incomingConfig: string
}

export interface McpApplyResult {
  applied: number
  adopted: number
  clashes: McpClash[]
}

export function applyMcpServers(tasks: McpApplyTask[]): Promise<McpApplyResult> {
  return invoke('apply_mcp_servers', { tasks })
}
```

### `views/McpView.vue` (new)

Two sections, both showing the same list of servers from `repository_items` where `folder = 'mcp-servers'`:

**Global** — toggle row per server. Toggling applies/removes the server from every active agent's `global_mcp_path` (agents with an empty `global_mcp_path` are skipped). One `McpApplyTask` is built per agent per server.

**Per-project** — servers × projects matrix, same pattern as `SkillsView` using `useItemsMatrix`. Toggling a cell applies/removes the server from that project's active agents' `mcp_path` files.

Both sections share the same clash confirmation flow: show a `ConfirmModal` listing the clashing keys and their diff, then re-call `applyMcpServers` with only those tasks on confirmation.

### `router.ts`
Add `/mcp` route pointing to `McpView`.

### `components/Sidebar.vue`
Add MCP nav item.

### `views/AgentsView.vue`
Add `MCP Path` and `Global MCP Path` `InputField`s per agent card (alongside existing skills/rules path fields).

## Clash resolution flow

| State | Action |
|-------|--------|
| Key absent | Write immediately |
| Key present, config identical | Adopt silently (no file write, no prompt) |
| Key present, config differs | Return as clash; prompt user; if confirmed, overwrite |

## Out of scope (for now)
- Promoting local MCP servers back to the repo (no "unmanaged" MCP view)
- Per-agent granularity on apply (all active agents for a project/globally get the server)
- Env variable secret management (stored as plain text, same as native agent behaviour)
- `applied_sha` tracking for MCP servers (update detection not implemented in first pass)
