import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import BaseButton from '../../components/BaseButton.vue'

describe('BaseButton', () => {
  it('renders slot content', () => {
    const wrapper = mount(BaseButton, { slots: { default: 'Click me' } })
    expect(wrapper.text()).toBe('Click me')
  })

  it('renders a <button> element', () => {
    const wrapper = mount(BaseButton)
    expect(wrapper.element.tagName).toBe('BUTTON')
  })

  it('emits a click event when clicked', async () => {
    const onClick = vi.fn()
    const wrapper = mount(BaseButton, { props: { onClick } })
    await wrapper.trigger('click')
    expect(onClick).toHaveBeenCalledOnce()
  })

  it('is disabled when the disabled prop is true', () => {
    const wrapper = mount(BaseButton, { props: { disabled: true } })
    expect((wrapper.element as HTMLButtonElement).disabled).toBe(true)
  })

  it('is enabled by default', () => {
    const wrapper = mount(BaseButton)
    expect((wrapper.element as HTMLButtonElement).disabled).toBe(false)
  })

  it('does not emit click when disabled', async () => {
    const onClick = vi.fn()
    const wrapper = mount(BaseButton, { props: { disabled: true, onClick } })
    await wrapper.trigger('click')
    expect(onClick).not.toHaveBeenCalled()
  })

  it('applies primary variant classes', () => {
    const wrapper = mount(BaseButton, { props: { variant: 'primary' } })
    expect(wrapper.classes().some(c => c.includes('bg-accent'))).toBe(true)
  })

  it('applies danger variant classes', () => {
    const wrapper = mount(BaseButton, { props: { variant: 'danger' } })
    expect(wrapper.classes().some(c => c.includes('border-danger'))).toBe(true)
  })
})
