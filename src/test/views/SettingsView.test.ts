import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import * as api from '../../api'
import SettingsView from '../../views/SettingsView.vue'

vi.mock('../../api', () => ({
  getSetting: vi.fn(),
  setSetting: vi.fn(),
}))

// Stub child components that have their own styling/behaviour we don't test here
vi.mock('../../components/PageLayout.vue', () => ({
  default: { template: '<div><slot /><slot name="actions" /></div>' },
}))
vi.mock('../../components/BaseButton.vue', () => ({
  default: {
    props: ['disabled'],
    emits: ['click'],
    template: '<button :disabled="disabled" @click="$emit(\'click\')"><slot /></button>',
  },
}))
vi.mock('../../components/InputField.vue', () => ({
  default: {
    props: ['modelValue', 'label', 'placeholder'],
    emits: ['update:modelValue'],
    template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
  },
}))

describe('SettingsView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(api.setSetting).mockResolvedValue(undefined)
  })

  it('loads the saved repo URL on mount', async () => {
    vi.mocked(api.getSetting).mockResolvedValue('https://github.com/org/repo')
    const wrapper = mount(SettingsView)
    await flushPromises()
    const input = wrapper.find('input')
    expect((input.element as HTMLInputElement).value).toBe('https://github.com/org/repo')
  })

  it('calls setSetting with the trimmed URL when Save is clicked', async () => {
    const wrapper = mount(SettingsView)
    await flushPromises()

    await wrapper.find('input').setValue('https://github.com/org/repo')
    await wrapper.find('button').trigger('click')
    await flushPromises()

    expect(api.setSetting).toHaveBeenCalledWith('repoUrl', 'https://github.com/org/repo')
  })

  it('does not call setSetting when the URL fails frontend validation', async () => {
    const wrapper = mount(SettingsView)
    await flushPromises()

    await wrapper.find('input').setValue('not-a-valid-url')
    await wrapper.find('button').trigger('click')
    await flushPromises()

    expect(api.setSetting).not.toHaveBeenCalled()
  })

  it('accepts an empty URL (clears the setting)', async () => {
    const wrapper = mount(SettingsView)
    await flushPromises()

    // Empty string bypasses the URL format check
    await wrapper.find('input').setValue('')
    await wrapper.find('button').trigger('click')
    await flushPromises()

    expect(api.setSetting).toHaveBeenCalledWith('repoUrl', '')
  })

  it('accepts a valid SSH URL', async () => {
    const wrapper = mount(SettingsView)
    await flushPromises()

    await wrapper.find('input').setValue('git@github.com:org/repo.git')
    await wrapper.find('button').trigger('click')
    await flushPromises()

    expect(api.setSetting).toHaveBeenCalledWith('repoUrl', 'git@github.com:org/repo.git')
  })
})
