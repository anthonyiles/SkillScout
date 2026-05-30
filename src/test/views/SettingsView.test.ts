import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import * as api from '../../api'
import SettingsView from '../../views/SettingsView.vue'

vi.mock('../../api', () => ({
  getSetting: vi.fn(),
  setSetting: vi.fn(),
}))

const mockCheckForUpdate = vi.fn()
const mockSetBetaTester = vi.fn()
const mockIsBetaTester = ref(false)

vi.mock('../../composables/useUpdater', () => ({
  useUpdater: () => ({
    updateAvailable: ref(null),
    checking: ref(false),
    installing: ref(false),
    installPercent: ref(null),
    isBetaTester: mockIsBetaTester,
    checkForUpdate: mockCheckForUpdate,
    installUpdate: vi.fn(),
    setBetaTester: mockSetBetaTester,
  }),
}))

// Stub child components that have their own styling/behaviour we don't test here
// Actions slot rendered before default slot to match real PageLayout header layout
vi.mock('../../components/PageLayout.vue', () => ({
  default: { template: '<div><slot name="actions" /><slot /></div>' },
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
vi.mock('../../components/TickBox.vue', () => ({
  default: {
    props: ['checked', 'label'],
    emits: ['change'],
    template: '<input type="checkbox" :checked="checked" @change="$emit(\'change\', $event.target.checked)" data-testid="tickbox" />',
  },
}))

describe('SettingsView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockIsBetaTester.value = false
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(api.setSetting).mockResolvedValue(undefined)
    mockSetBetaTester.mockResolvedValue(undefined)
    mockCheckForUpdate.mockResolvedValue(undefined)
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

  it('renders the beta tester checkbox unchecked by default', async () => {
    const wrapper = mount(SettingsView)
    await flushPromises()

    const checkbox = wrapper.find('[data-testid="tickbox"]')
    expect(checkbox.exists()).toBe(true)
    expect((checkbox.element as HTMLInputElement).checked).toBe(false)
  })

  it('renders the beta tester checkbox checked when isBetaTester is true', async () => {
    mockIsBetaTester.value = true
    const wrapper = mount(SettingsView)
    await flushPromises()

    const checkbox = wrapper.find('[data-testid="tickbox"]')
    expect((checkbox.element as HTMLInputElement).checked).toBe(true)
  })

  it('calls setBetaTester and checkForUpdate when beta checkbox is toggled on', async () => {
    const wrapper = mount(SettingsView)
    await flushPromises()

    const checkbox = wrapper.find('[data-testid="tickbox"]')
    ;(checkbox.element as HTMLInputElement).checked = true
    await checkbox.trigger('change')
    await flushPromises()

    expect(mockSetBetaTester).toHaveBeenCalledWith(true)
    expect(mockCheckForUpdate).toHaveBeenCalled()
  })

  it('calls setBetaTester(false) when beta checkbox is toggled off', async () => {
    mockIsBetaTester.value = true
    const wrapper = mount(SettingsView)
    await flushPromises()

    const checkbox = wrapper.find('[data-testid="tickbox"]')
    ;(checkbox.element as HTMLInputElement).checked = false
    await checkbox.trigger('change')
    await flushPromises()

    expect(mockSetBetaTester).toHaveBeenCalledWith(false)
  })
})
