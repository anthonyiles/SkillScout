import { describe, it, expect, vi, beforeEach } from 'vitest'
import { defineComponent } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import * as updaterPlugin from '@tauri-apps/plugin-updater'
import * as api from '../../api'

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn(),
}))

vi.mock('../../api', () => ({
  getSetting: vi.fn(),
  setSetting: vi.fn(),
}))

vi.mock('../../composables/useToast', () => ({
  useToast: () => ({ error: vi.fn(), success: vi.fn() }),
}))

// Bypass createSharedComposable memoisation so each test gets a fresh instance
vi.mock('@vueuse/core', () => ({
  createSharedComposable: (fn: () => unknown) => fn,
}))

// Re-import fresh module each test to avoid shared state from the factory
async function freshUpdater() {
  vi.resetModules()
  const { useUpdater } = await import('../../composables/useUpdater')
  let instance: ReturnType<typeof useUpdater>
  const Wrapper = defineComponent({
    setup() { instance = useUpdater(); return {} },
    template: '<div />',
  })
  mount(Wrapper)
  return instance!
}

describe('useUpdater', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(api.setSetting).mockResolvedValue(undefined)
    vi.mocked(updaterPlugin.check).mockResolvedValue(null)
  })

  it('isBetaTester starts false', async () => {
    const updater = await freshUpdater()
    expect(updater.isBetaTester.value).toBe(false)
  })

  it('checkForUpdate loads betaTester setting and passes stable header by default', async () => {
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(updaterPlugin.check).mockResolvedValue(null)

    const updater = await freshUpdater()
    await updater.checkForUpdate()
    await flushPromises()

    expect(api.getSetting).toHaveBeenCalledWith('betaTester')
    expect(updaterPlugin.check).toHaveBeenCalledWith(
      expect.objectContaining({ headers: { 'X-Channel': 'stable' } })
    )
  })

  it('checkForUpdate passes beta header when betaTester setting is "true"', async () => {
    vi.mocked(api.getSetting).mockResolvedValue('true')
    vi.mocked(updaterPlugin.check).mockResolvedValue(null)

    const updater = await freshUpdater()
    await updater.checkForUpdate()
    await flushPromises()

    expect(updaterPlugin.check).toHaveBeenCalledWith(
      expect.objectContaining({ headers: { 'X-Channel': 'beta' } })
    )
    expect(updater.isBetaTester.value).toBe(true)
  })

  it('checkForUpdate sets updateAvailable when an update is found', async () => {
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(updaterPlugin.check).mockResolvedValue({
      available: true,
      version: '1.2.0',
      body: 'New features',
      downloadAndInstall: vi.fn(),
    } as any)

    const updater = await freshUpdater()
    await updater.checkForUpdate()
    await flushPromises()

    expect(updater.updateAvailable.value).toEqual({ version: '1.2.0', notes: 'New features' })
  })

  it('checkForUpdate clears updateAvailable when no update is found', async () => {
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(updaterPlugin.check).mockResolvedValue(null)

    const updater = await freshUpdater()
    updater.updateAvailable.value = { version: '1.0.0', notes: null }
    await updater.checkForUpdate()
    await flushPromises()

    expect(updater.updateAvailable.value).toBeNull()
  })

  it('checkForUpdate does not throw when check() fails — best-effort', async () => {
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(updaterPlugin.check).mockRejectedValue(new Error('Network error'))

    const updater = await freshUpdater()
    await expect(updater.checkForUpdate()).resolves.toBeUndefined()
    expect(updater.checking.value).toBe(false)
  })

  it('setBetaTester updates isBetaTester and persists via setSetting', async () => {
    const updater = await freshUpdater()
    await updater.setBetaTester(true)

    expect(updater.isBetaTester.value).toBe(true)
    expect(api.setSetting).toHaveBeenCalledWith('betaTester', 'true')
  })

  it('setBetaTester(false) persists "false"', async () => {
    const updater = await freshUpdater()
    updater.isBetaTester.value = true
    await updater.setBetaTester(false)

    expect(updater.isBetaTester.value).toBe(false)
    expect(api.setSetting).toHaveBeenCalledWith('betaTester', 'false')
  })

  it('installUpdate passes the correct channel header', async () => {
    vi.mocked(api.getSetting).mockResolvedValue('true')
    const mockDownloadAndInstall = vi.fn().mockResolvedValue(undefined)
    vi.mocked(updaterPlugin.check).mockResolvedValue({
      available: true,
      version: '1.2.0',
      body: null,
      downloadAndInstall: mockDownloadAndInstall,
    } as any)

    const updater = await freshUpdater()
    updater.isBetaTester.value = true

    // installUpdate calls check() — verify it uses the current isBetaTester state
    await updater.installUpdate()
    await flushPromises()

    expect(updaterPlugin.check).toHaveBeenCalledWith(
      expect.objectContaining({ headers: { 'X-Channel': 'beta' } })
    )
  })

  it('installUpdate clears updateAvailable when check returns no update', async () => {
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(updaterPlugin.check).mockResolvedValue(null)

    const updater = await freshUpdater()
    updater.updateAvailable.value = { version: '1.0.0', notes: null }
    await updater.installUpdate()
    await flushPromises()

    expect(updater.updateAvailable.value).toBeNull()
    expect(updater.installing.value).toBe(false)
  })
})
