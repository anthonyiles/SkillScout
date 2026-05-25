import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import * as api from '../../api'
import ProjectsView from '../../views/ProjectsView.vue'

vi.mock('../../api', () => ({
  getProjects: vi.fn(),
  saveProject: vi.fn(),
  deleteProject: vi.fn(),
  getAgents: vi.fn(),
}))

vi.mock('../../components/PageLayout.vue', () => ({
  default: { template: '<div><slot /><slot name="actions" /></div>' },
}))
vi.mock('../../components/CardItem.vue', () => ({
  default: {
    props: ['title'],
    template: '<div><slot name="actions" /><slot /></div>',
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
    props: ['modelValue', 'label', 'placeholder'],
    emits: ['update:modelValue'],
    template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
  },
}))
vi.mock('../../components/TickBox.vue', () => ({
  default: {
    props: ['checked', 'label'],
    emits: ['change'],
    template: '<label><input type="checkbox" :checked="checked" @change="$emit(\'change\', !checked)" />{{ label }}</label>',
  },
}))
vi.mock('../../components/EmptyState.vue', () => ({
  default: { props: ['message'], template: '<div class="empty">{{ message }}</div>' },
}))

const mockProjects = [
  { id: 1, path: '/home/user/alpha', agentIds: ['cursor'] },
]
const mockAgents = [
  { id: 'cursor', name: 'Cursor', skillsPath: '.cursor/skills', rulesPath: '.cursor/rules' },
]

describe('ProjectsView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.getProjects).mockResolvedValue(mockProjects)
    vi.mocked(api.getAgents).mockResolvedValue(mockAgents)
    vi.mocked(api.saveProject).mockResolvedValue({ id: 1, path: '/home/user/alpha', agentIds: ['cursor'] })
    vi.mocked(api.deleteProject).mockResolvedValue(undefined)
  })

  it('loads projects and agents on mount', async () => {
    mount(ProjectsView)
    await flushPromises()
    expect(api.getProjects).toHaveBeenCalledOnce()
    expect(api.getAgents).toHaveBeenCalledOnce()
  })

  it('renders an input with the project path', async () => {
    const wrapper = mount(ProjectsView)
    await flushPromises()
    const inputs = wrapper.findAll('input[type="text"], input:not([type="checkbox"])')
    const pathInput = inputs.find(i => (i.element as HTMLInputElement).value === '/home/user/alpha')
    expect(pathInput).toBeDefined()
  })

  it('renders agent checkboxes', async () => {
    const wrapper = mount(ProjectsView)
    await flushPromises()
    const checkboxes = wrapper.findAll('input[type="checkbox"]')
    expect(checkboxes.length).toBeGreaterThanOrEqual(1)
  })

  it('addProject appends a blank project entry', async () => {
    const wrapper = mount(ProjectsView)
    await flushPromises()

    const inputsBefore = wrapper.findAll('input').length
    const newBtn = wrapper.findAll('button').find(b => b.text() === 'New')
    await newBtn!.trigger('click')

    expect(wrapper.findAll('input').length).toBeGreaterThan(inputsBefore)
  })

  it('saveConfig calls saveProject for each project with a path', async () => {
    const wrapper = mount(ProjectsView)
    await flushPromises()

    const saveBtn = wrapper.findAll('button').find(b => b.text() === 'Save')
    await saveBtn!.trigger('click')
    await flushPromises()

    expect(api.saveProject).toHaveBeenCalledOnce()
    expect(api.saveProject).toHaveBeenCalledWith(
      expect.objectContaining({ path: '/home/user/alpha' })
    )
  })

  it('removeProject calls deleteProject for a saved project', async () => {
    const wrapper = mount(ProjectsView)
    await flushPromises()

    const removeBtn = wrapper.findAll('button')[0]
    await removeBtn.trigger('click')
    await flushPromises()

    expect(api.deleteProject).toHaveBeenCalledWith(1)
  })

  it('adds an empty project when no projects are returned', async () => {
    vi.mocked(api.getProjects).mockResolvedValue([])
    const wrapper = mount(ProjectsView)
    await flushPromises()
    // Should still show an input for the blank project added by addProject()
    expect(wrapper.findAll('input').length).toBeGreaterThan(0)
  })
})
