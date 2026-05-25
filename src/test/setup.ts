import { vi } from 'vitest'
import { config } from '@vue/test-utils'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn(),
}))

// Teleport moves content outside the wrapper's subtree, making wrapper.find()
// return nothing in happy-dom. Stub it to render its slot inline instead.
config.global.stubs = {
  Teleport: { template: '<div><slot /></div>' },
}
