# MCP Server Feature Plan

## Overview

Add MCP (Model Context Protocol) server management to SkillScout. MCP servers are synced from the GitHub repo (like skills/rules) and applied by merging into each agent's `mcp.json` config file. Servers can be scoped globally (`~/.cursor/mcp.json`) or per-project (`.cursor/mcp.json` inside the project dir).

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

Each file:
```json
{
  "key": "context7",
  "scope": "project",
  "command": "npx",
  "args": ["-y", "@upstash/context7-mcp"],
  "env": {}
}
```

- `key` — the key used in the target `mcpServers` JSON object
- `scope` — `"global"` (writes to `~/.cursor/mcp.json`) or `"project"` (writes to `<project>/.cursor/mcp.json`)
- `command`, `args`, `env` — standard MCP server config

## Data model

No new DB tables needed — MCP servers reuse `repository_items` (with `folder = "mcp-servers"`) and `item_selections`.

### Agent table migration

Add two columns to `agents` via `ALTER TABLE` (same migration pattern as `sub_folder`):

| Column | Type | Purpose |
|--------|------|---------|
| `mcp_path` | `TEXT NOT NULL DEFAULT ''` | Project-level MCP file path (e.g. `.cursor/mcp.json`) |
| `global_mcp_path` | `TEXT NOT NULL DEFAULT ''` | Global MCP file path (e.g. `~/.cursor/mcp.json`) |

Default seed values:

| Agent | `mcp_path` | `global_mcp_path` |
|-------|-----------|------------------|
| Cursor | `.cursor/mcp.json` | `~/.cursor/mcp.json` |
| Claude Code | `.claude/mcp.json` | `~/.claude/mcp.json` |
| JetBrains | *(empty)* | *(empty)* |

## Backend changes

### `models.rs`
- Add `mcp_path` and `global_mcp_path` to `Agent` struct
- Add `McpApplyTask` struct:
  ```rust
  pub struct McpApplyTask {
      pub target_path: String,  // absolute path to the mcp.json file (~ expanded at call site)
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
      pub existing_config: String,  // JSON string of what's currently in the file
      pub incoming_config: String,  // JSON string of what we would write
  }
  ```

### `db.rs`
- Add two `ALTER TABLE agents ADD COLUMN` migrations (ignore duplicate column errors)
- Update `seed_defaults` to set `mcp_path` / `global_mcp_path` for the three default agents

### `commands/state.rs`
- Update `db_get_agents`, `db_save_agent`, `db_reset_agents` to read/write the two new columns

### `commands/sync.rs`
- Add `read_folder("mcp-servers")` after `read_folder("rules")` (line ~207)
- `.json` files are read as-is; content stored as raw JSON string in `repository_items.content`

### `commands/mcp.rs` (new file)

One command: `apply_mcp_servers(tasks: Vec<McpApplyTask>) -> Result<McpApplyResult, String>`

Logic per task:
1. Read existing `mcp.json` at `target_path` (start with `{}` if file absent)
2. Parse `mcpServers` object
3. If `remove`: delete `server_key` from object, write back, increment applied
4. If not `remove`:
   - Key absent → write entry, record `applied_sha`, increment applied
   - Key present, all fields identical → adopt (record `applied_sha` only, no file write), increment adopted
   - Key present, any field differs → append to `clashes`, do not write

Returns `McpApplyResult`. Frontend calls this once — safe work is done immediately, clashes are returned for the user to confirm. If user confirms, frontend calls again with only the clashing tasks (normal apply, since force-overwrite is implicit in the second call when the user said yes).

Uses `spawn_blocking` for all file I/O.

### `lib.rs`
- Add `mod mcp` to `commands` mod
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
  target_path: string
  server_key: string
  config: string
  remove: boolean
}

export interface McpClash {
  server_key: string
  existing_config: string
  incoming_config: string
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

Two sections:

**Global servers** (`scope === 'global'`):
- Card list of global-scoped MCP items from `repository_items`
- "Apply Globally" button — builds one `McpApplyTask` per agent with a non-empty `global_mcp_path`, expanding `~` to the home directory
- Uses `McpApplyResult.clashes` to show a confirmation modal before re-calling for clashing keys

**Project servers** (`scope === 'project'`):
- Items × projects matrix, same pattern as `SkillsView` using `useItemsMatrix`
- "Apply" button — builds tasks from selections: for each selected (server, project) pair, one task per active agent for that project (using `agent.mcpPath` joined to `project.path`)
- Same clash confirmation flow

### `router.ts`
Add `/mcp` route pointing to `McpView`.

### `components/Sidebar.vue`
Add MCP nav item.

### `views/AgentsView.vue`
Add `MCP Path` and `Global MCP Path` `InputField`s per agent card (alongside existing skills/rules path fields).

## Clash resolution flow

| State | Action |
|-------|--------|
| Key absent | Write immediately, record `applied_sha` |
| Key present, fields identical | Adopt (record `applied_sha`), no file write, no prompt |
| Key present, any field differs | Return as clash; prompt user; if confirmed, call apply again for that subset |

Once a key is owned (has `applied_sha`), subsequent applies follow the normal update flow: skip if repo SHA unchanged, update silently if SHA changed.

## Out of scope (for now)
- Promoting local MCP servers back to the repo (no "unmanaged" MCP view)
- Per-agent granularity on project-scoped servers (all active agents for a project get the server)
- Env variable secret management (stored as plain text, same as Cursor/Claude native behaviour)
