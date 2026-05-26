import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { useToast } from '../../composables/useToast'

describe('useToast', () => {
  let toast: ReturnType<typeof useToast>

  beforeEach(() => {
    toast = useToast()
    // Clear any toasts left over from previous tests (createSharedComposable shares state)
    Array.from(toast.toasts.value).forEach(t => toast.removeToast(t.id))
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('starts with an empty toasts array', () => {
    expect(toast.toasts.value).toHaveLength(0)
  })

  it('success() adds a toast with type "success"', () => {
    toast.success('Saved!')
    expect(toast.toasts.value).toHaveLength(1)
    expect(toast.toasts.value[0]).toMatchObject({ message: 'Saved!', type: 'success' })
  })

  it('error() adds a toast with type "error"', () => {
    toast.error('Something went wrong')
    expect(toast.toasts.value).toHaveLength(1)
    expect(toast.toasts.value[0]).toMatchObject({ message: 'Something went wrong', type: 'error' })
  })

  it('info() adds a toast with type "info"', () => {
    toast.info('FYI')
    expect(toast.toasts.value).toHaveLength(1)
    expect(toast.toasts.value[0]).toMatchObject({ message: 'FYI', type: 'info' })
  })

  it('removeToast() removes only the targeted toast', () => {
    toast.success('first')
    toast.success('second')
    const firstId = toast.toasts.value[0].id
    toast.removeToast(firstId)
    expect(toast.toasts.value).toHaveLength(1)
    expect(toast.toasts.value[0].message).toBe('second')
  })

  it('each toast receives a unique id', () => {
    toast.success('a')
    toast.success('b')
    toast.success('c')
    const ids = toast.toasts.value.map(t => t.id)
    expect(new Set(ids).size).toBe(3)
  })

  it('toast auto-dismisses after the specified duration', async () => {
    vi.useFakeTimers()
    toast.showToast('Temp', 'info', 100)
    expect(toast.toasts.value).toHaveLength(1)
    await vi.advanceTimersByTimeAsync(100)
    expect(toast.toasts.value).toHaveLength(0)
  })

  it('toast does not dismiss before the duration elapses', async () => {
    vi.useFakeTimers()
    toast.showToast('Temp', 'info', 500)
    await vi.advanceTimersByTimeAsync(499)
    expect(toast.toasts.value).toHaveLength(1)
  })

  it('multiple toasts can coexist', () => {
    toast.success('one')
    toast.error('two')
    toast.info('three')
    expect(toast.toasts.value).toHaveLength(3)
  })
})
