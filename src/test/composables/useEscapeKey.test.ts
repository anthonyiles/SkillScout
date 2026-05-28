import { describe, it, expect, vi, afterEach } from 'vitest'
import { defineComponent, nextTick, ref } from 'vue'
import { mount } from '@vue/test-utils'
import { useEscapeKey } from '../../composables/useEscapeKey'

function mountWithEscapeKey(initiallyActive: boolean, onEscape: () => void) {
  const isActive = ref(initiallyActive)
  const component = defineComponent({
    setup() {
      useEscapeKey(() => isActive.value, onEscape)
      return { isActive }
    },
    template: '<div />',
  })
  const wrapper = mount(component)
  return { wrapper, isActive }
}

function pressKey(key: string) {
  window.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }))
}

describe('useEscapeKey', () => {
  afterEach(() => {
    // Ensure listeners are cleaned up between tests
    vi.clearAllMocks()
  })

  it('fires onEscape when Escape is pressed and composable is active', () => {
    const onEscape = vi.fn()
    mountWithEscapeKey(true, onEscape)
    pressKey('Escape')
    expect(onEscape).toHaveBeenCalledOnce()
  })

  it('does not fire onEscape for non-Escape keys', () => {
    const onEscape = vi.fn()
    mountWithEscapeKey(true, onEscape)
    pressKey('Enter')
    pressKey('Tab')
    pressKey(' ')
    expect(onEscape).not.toHaveBeenCalled()
  })

  it('does not fire onEscape when composable is inactive', () => {
    const onEscape = vi.fn()
    mountWithEscapeKey(false, onEscape)
    pressKey('Escape')
    expect(onEscape).not.toHaveBeenCalled()
  })

  it('removes the listener and stops firing after component unmounts', () => {
    const onEscape = vi.fn()
    const { wrapper } = mountWithEscapeKey(true, onEscape)
    wrapper.unmount()
    pressKey('Escape')
    expect(onEscape).not.toHaveBeenCalled()
  })

  it('begins listening when isActive transitions from false to true', async () => {
    const onEscape = vi.fn()
    const { isActive } = mountWithEscapeKey(false, onEscape)

    pressKey('Escape')
    expect(onEscape).not.toHaveBeenCalled()

    isActive.value = true
    await nextTick()

    pressKey('Escape')
    expect(onEscape).toHaveBeenCalledOnce()
  })

  it('stops listening when isActive transitions from true to false', async () => {
    const onEscape = vi.fn()
    const { isActive } = mountWithEscapeKey(true, onEscape)

    isActive.value = false
    await nextTick()

    pressKey('Escape')
    expect(onEscape).not.toHaveBeenCalled()
  })
})
