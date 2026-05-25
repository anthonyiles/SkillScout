
# Vue: Route All Tauri Calls Through the API Layer

All `invoke()` calls must be wrapped in typed functions in `src/api.ts`. Views and composables must never call `invoke()` directly.

**Why:** The `src/api.ts` layer is the single source of truth for the IPC contract between the frontend and Rust backend. When a backend command signature changes, there is exactly one file to update. Direct `invoke()` calls in views scatter the contract across the codebase, cause type drift, and make refactoring error-prone.

**Wrong:**
```ts
// Inside UnmanagedView.vue
const result = await invoke<{ url: string; branch: string }>('promote_item', {
    repoUrl,
    itemType: item.type,
    itemName: item.name,
    projectPath: project.path,
})
```

**Right — define in src/api.ts:**
```ts
export async function promoteItem(
    repoUrl: string,
    payload: { itemType: string; itemName: string; projectPath: string; subFolders: string[]; updateMode?: boolean }
): Promise<{ url: string; branch: string }> {
    return invoke('promote_item', { repoUrl, ...payload })
}
```

**Then use in the view:**
```ts
import { promoteItem } from '@/api'
const result = await promoteItem(repoUrl, { itemType: item.type, ... })
```

Rules for `src/api.ts` wrappers:
- Return type must exactly match the Rust command's serialized output
- Parameters must match the Rust command's `#[tauri::command]` argument names
- One exported function per Tauri command
