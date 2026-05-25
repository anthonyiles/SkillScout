
# Vue: Use Collision-Safe ID Generation

Never use `Date.now()` alone as a unique identifier for reactive list items, Vue `:key` values, or anything used to locate and remove a specific item. Two calls within the same millisecond produce the same ID, causing wrong items to be deleted and duplicate Vue `:key` warnings.

**Wrong:**
```ts
agents.value.push({ id: `custom-${Date.now()}`, name: 'New Agent' })
toasts.value.push({ id: Date.now(), message })
```

**Right — with random suffix:**
```ts
const id = `custom-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
agents.value.push({ id, name: 'New Agent' })
```

**Right — with crypto.randomUUID() (preferred when available):**
```ts
const id = crypto.randomUUID()
toasts.value.push({ id, message })
```

**Right — with a module-scoped counter:**
```ts
let _nextId = 0
const nextId = () => ++_nextId
```

Use `crypto.randomUUID()` as the default choice. Fall back to the `Date.now() + random suffix` pattern only when UUID generation is unavailable.
