
# Vue: No Optimistic State Mutation Without Rollback

Never mutate reactive state (refs, reactive objects) before a backend call confirms success. If the backend call fails, the UI will be out of sync with the database — a silent data-integrity bug that is hard to reproduce.

Choose one of two patterns:

**Pattern A — Confirm then commit (preferred for correctness):**
```ts
async function toggleSelection(itemId: string, projectId: number) {
    try {
        await toggleItemSelection(itemId, projectId)  // backend first
        // only mutate state after success
        if (selectionMatrix.value[itemId].has(projectId)) {
            selectionMatrix.value[itemId].delete(projectId)
        } else {
            selectionMatrix.value[itemId].add(projectId)
        }
    } catch (e) {
        error(formatError(e))
    }
}
```

**Pattern B — Optimistic with rollback (preferred for perceived performance):**
```ts
async function toggleSelection(itemId: string, projectId: number) {
    const wasSelected = selectionMatrix.value[itemId]?.has(projectId) ?? false
    // optimistic update
    wasSelected
        ? selectionMatrix.value[itemId].delete(projectId)
        : selectionMatrix.value[itemId].add(projectId)
    try {
        await toggleItemSelection(itemId, projectId)
    } catch (e) {
        // rollback
        wasSelected
            ? selectionMatrix.value[itemId].add(projectId)
            : selectionMatrix.value[itemId].delete(projectId)
        error(formatError(e))
    }
}
```

This applies to any toggle, save, or delete operation that mirrors local state to a backend store.
