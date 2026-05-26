import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import * as api from '../../api'
import AgentsView from '../../views/AgentsView.vue'

vi.mock('../../api', () => ({
  getAgents: vi.fn(),
  saveAgent: vi.fn(),
  deleteAgent: vi.fn(),
  resetAgentsToDefaults: vi.fn(),
}))

vi.mock('../../components/PageLayout.vue', () => ({
  default: { template: '<div><slot /><slot name="actions" /></div>' },
}))
vi.mock('../../components/CardItem.vue', () => ({
  default: { template: '<div><slot name="title" /><slot name="actions" /><slot /></div>' },
}))
vi.mock('../../components/ConfirmModal.vue', () => ({
  default: {
    props: ['isOpen', 'title', 'message', 'confirmText', 'danger'],
    emits: ['confirm', 'cancel'],
    template: `<div v-if="isOpen">
      <button class="confirm-btn" @click="$emit('confirm')">Confirm</button>
      <button class="cancel-btn" @click="$emit('cancel')">Cancel</button>
    </div>`,
  },
}))
vi.mock('../../components/BaseButton.vue', () => ({
  default: {
    props: ['variant', 'disabled', 'icon'],
    emits: ['click'],
    template: '<button :disabled="disabled" @click="$emit(\'click\')"><slot /></button>',
  },
}))
vi.mock('../../components/InputField.vue', () => ({
  default: {
    props: ['modelValue', 'label'],
    emits: ['update:modelValue'],
    template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
  },
}))

const mockAgents = [
  { id: 'cursor', name: 'Cursor', skillsPath: '.cursor/skills', rulesPath: '.cursor/rules' },
]

describe('AgentsView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.getAgents).mockResolvedValue(mockAgents)
    vi.mocked(api.saveAgent).mockResolvedValue(undefined)
    vi.mocked(api.deleteAgent).mockResolvedValue(undefined)
    vi.mocked(api.resetAgentsToDefaults).mockResolvedValue(undefined)
  })

  it('loads agents on mount', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()
    expect(api.getAgents).toHaveBeenCalledOnce()
    // The agent name is rendered in an <input v-model="agent.name"> — check its value
    const inputs = wrapper.findAll('input')
    const nameInput = inputs.find(i => (i.element as HTMLInputElement).value === 'Cursor')
    expect(nameInput).toBeDefined()
  })

  it('addAgent appends a new agent to the list', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()

    const addBtn = wrapper.findAll('button').find(b => b.text() === 'Add Custom Agent')
    await addBtn!.trigger('click')

    // After adding, there should be more inputs (2 original + 2 new agent paths)
    const inputs = wrapper.findAll('input')
    expect(inputs.length).toBeGreaterThan(2)
  })

  it('removeAgent calls deleteAgent and removes the agent', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()

    // The first ×-button in the template is the remove button for the first agent
    const removeBtn = wrapper.findAll('button')[0]
    await removeBtn.trigger('click')
    await flushPromises()

    expect(api.deleteAgent).toHaveBeenCalledWith('cursor')
  })

  it('saveConfig calls saveAgent for each agent', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()

    const saveBtn = wrapper.findAll('button').find(b => b.text() === 'Save')
    await saveBtn!.trigger('click')
    await flushPromises()

    expect(api.saveAgent).toHaveBeenCalledWith(mockAgents[0])
  })

  it('resetToDefaults opens the confirmation modal', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()

    const resetBtn = wrapper.findAll('button').find(b => b.text() === 'Reset to Defaults')
    await resetBtn!.trigger('click')

    expect(wrapper.find('.confirm-btn').exists()).toBe(true)
  })

  it('executeReset calls resetAgentsToDefaults and reloads agents', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()
    vi.mocked(api.getAgents).mockResolvedValue(mockAgents)

    // Open modal then confirm
    const resetBtn = wrapper.findAll('button').find(b => b.text() === 'Reset to Defaults')
    await resetBtn!.trigger('click')
    await wrapper.find('.confirm-btn').trigger('click')
    await flushPromises()

    expect(api.resetAgentsToDefaults).toHaveBeenCalledOnce()
    expect(api.getAgents).toHaveBeenCalledTimes(2) // once on mount, once after reset
  })
})
