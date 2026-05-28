import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ConfirmModal from '../../components/ConfirmModal.vue'

const defaultProps = {
  isOpen: true,
  title: 'Delete item?',
  message: 'This cannot be undone.',
}

describe('ConfirmModal', () => {
  it('renders when isOpen is true', () => {
    const wrapper = mount(ConfirmModal, { props: defaultProps })
    expect(wrapper.find('[role="dialog"]').exists()).toBe(true)
  })

  it('does not render when isOpen is false', () => {
    const wrapper = mount(ConfirmModal, { props: { ...defaultProps, isOpen: false } })
    expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
  })

  it('displays the title and message', () => {
    const wrapper = mount(ConfirmModal, { props: defaultProps })
    expect(wrapper.text()).toContain('Delete item?')
    expect(wrapper.text()).toContain('This cannot be undone.')
  })

  it('uses default button labels when confirmText and cancelText are omitted', () => {
    const wrapper = mount(ConfirmModal, { props: defaultProps })
    expect(wrapper.text()).toContain('Confirm')
    expect(wrapper.text()).toContain('Cancel')
  })

  it('uses custom button labels when provided', () => {
    const wrapper = mount(ConfirmModal, {
      props: { ...defaultProps, confirmText: 'Delete', cancelText: 'Go back' },
    })
    expect(wrapper.text()).toContain('Delete')
    expect(wrapper.text()).toContain('Go back')
  })

  it('emits confirm when the confirm button is clicked', async () => {
    const wrapper = mount(ConfirmModal, { props: defaultProps })
    const buttons = wrapper.findAll('button')
    const confirmBtn = buttons.find(b => b.text() === 'Confirm')
    await confirmBtn!.trigger('click')
    expect(wrapper.emitted('confirm')).toBeTruthy()
  })

  it('emits cancel when the cancel button is clicked', async () => {
    const wrapper = mount(ConfirmModal, { props: defaultProps })
    const buttons = wrapper.findAll('button')
    const cancelBtn = buttons.find(b => b.text() === 'Cancel')
    await cancelBtn!.trigger('click')
    expect(wrapper.emitted('cancel')).toBeTruthy()
  })

  it('emits cancel when the backdrop is clicked', async () => {
    const wrapper = mount(ConfirmModal, { props: defaultProps })
    // The outermost div is the backdrop
    await wrapper.find('div').trigger('click')
    expect(wrapper.emitted('cancel')).toBeTruthy()
  })

  it('emits cancel when Escape is pressed while open', async () => {
    const wrapper = mount(ConfirmModal, { props: defaultProps })
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('cancel')).toBeTruthy()
  })
})
