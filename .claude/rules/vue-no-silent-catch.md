
# Vue: No Silent catch Blocks

A `catch` block that swallows an error affecting user-visible state is a hidden bug. Users perform an action and receive no feedback; the app silently proceeds in a potentially inconsistent state.

Every `catch` block must do at least one of:
1. **Re-throw** — let it bubble to a higher error boundary
2. **Surface feedback** — show a toast or error message via `useToast`
3. **Rollback state** — restore the prior reactive state and show an error
4. **Log with context** — `console.warn` with the function name and relevant data (acceptable only when the error is truly inconsequential and a comment explains why)

**Wrong — user performs apply, nothing happens if conflict check fails:**
```ts
try {
    await checkExisting(tasks)
} catch {
    // If check fails, proceed
}
await runApply(tasks)
```

**Wrong — toggleSelection fails silently, UI is desynced:**
```ts
} catch (e) {
    console.error('Failed to toggle', e)
    // no rollback, no toast
}
```

**Right:**
```ts
try {
    await checkExisting(tasks)
} catch (e) {
    // Conflict check is safety-critical — abort and surface the error
    error(formatError(e))
    return
}
await runApply(tasks)
```

The comment `// If check fails, proceed` is not an acceptable rationale for swallowing a safety-critical error.
