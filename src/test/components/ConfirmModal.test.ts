import { describe, it, expect, afterEach } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import ConfirmModal from '../../components/ConfirmModal.vue'

const defaultProps = {
  isOpen: true,
  title: 'Delete item?',
  message: 'This cannot be undone.',
}

describe('ConfirmModal', () => {
  const wrappers: VueWrapper[] = []
  const mountModal = (props = defaultProps) => {
    const w = mount(ConfirmModal, { props })
    wrappers.push(w)
    return w
  }

  afterEach(() => {
    wrappers.forEach(w => w.unmount())
    wrappers.length = 0
  })

  it('renders when isOpen is true', () => {
    const wrapper = mountModal()
    expect(wrapper.find('[role="dialog"]').exists()).toBe(true)
  })

  it('does not render when isOpen is false', () => {
    const wrapper = mountModal({ ...defaultProps, isOpen: false })
    expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
  })

  it('displays the title and message', () => {
    const wrapper = mountModal()
    expect(wrapper.text()).toContain('Delete item?')
    expect(wrapper.text()).toContain('This cannot be undone.')
  })

  it('uses default button labels when confirmText and cancelText are omitted', () => {
    const wrapper = mountModal()
    expect(wrapper.text()).toContain('Confirm')
    expect(wrapper.text()).toContain('Cancel')
  })

  it('uses custom button labels when provided', () => {
    const wrapper = mountModal({ ...defaultProps, confirmText: 'Delete', cancelText: 'Go back' })
    expect(wrapper.text()).toContain('Delete')
    expect(wrapper.text()).toContain('Go back')
  })

  it('emits confirm when the confirm button is clicked', async () => {
    const wrapper = mountModal()
    const buttons = wrapper.findAll('button')
    const confirmBtn = buttons.find(b => b.text() === 'Confirm')
    expect(confirmBtn, 'Confirm button should exist').toBeDefined()
    await confirmBtn!.trigger('click')
    expect(wrapper.emitted('confirm')).toBeTruthy()
  })

  it('emits cancel when the cancel button is clicked', async () => {
    const wrapper = mountModal()
    const buttons = wrapper.findAll('button')
    const cancelBtn = buttons.find(b => b.text() === 'Cancel')
    expect(cancelBtn, 'Cancel button should exist').toBeDefined()
    await cancelBtn!.trigger('click')
    expect(wrapper.emitted('cancel')).toBeTruthy()
  })

  it('emits cancel when the backdrop is clicked', async () => {
    const wrapper = mountModal()
    // The outermost div is the backdrop
    await wrapper.find('div').trigger('click')
    expect(wrapper.emitted('cancel')).toBeTruthy()
  })

  it('emits cancel when Escape is pressed while open', async () => {
    const wrapper = mountModal()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('cancel')).toBeTruthy()
  })
})
