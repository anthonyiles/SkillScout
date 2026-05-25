import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import TickBox from '../../components/TickBox.vue'

describe('TickBox', () => {
  it('renders a checkbox input', () => {
    const wrapper = mount(TickBox, { props: { checked: false } })
    expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true)
  })

  it('reflects the checked prop on the input', () => {
    const wrapper = mount(TickBox, { props: { checked: true } })
    const input = wrapper.find('input[type="checkbox"]').element as HTMLInputElement
    expect(input.checked).toBe(true)
  })

  it('renders the label when provided', () => {
    const wrapper = mount(TickBox, { props: { checked: false, label: 'Enable feature' } })
    expect(wrapper.text()).toContain('Enable feature')
  })

  it('does not render a text label when no label prop is given', () => {
    const wrapper = mount(TickBox, { props: { checked: false } })
    // The checkbox indicator span is always present; the text label span is conditional
    expect(wrapper.findAll('span')).toHaveLength(1)
  })

  it('emits update:checked with true when checkbox is checked', async () => {
    const wrapper = mount(TickBox, { props: { checked: false } })
    const input = wrapper.find('input[type="checkbox"]')
    await input.setValue(true)
    expect(wrapper.emitted('update:checked')).toBeTruthy()
    expect(wrapper.emitted('update:checked')![0]).toEqual([true])
  })

  it('emits change with the new value', async () => {
    const wrapper = mount(TickBox, { props: { checked: false } })
    const input = wrapper.find('input[type="checkbox"]')
    await input.setValue(true)
    expect(wrapper.emitted('change')![0]).toEqual([true])
  })

  it('emits update:checked with false when unchecked', async () => {
    const wrapper = mount(TickBox, { props: { checked: true } })
    const input = wrapper.find('input[type="checkbox"]')
    await input.setValue(false)
    expect(wrapper.emitted('update:checked')![0]).toEqual([false])
  })
})
