import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import * as api from '../../api'
import McpView from '../../views/McpView.vue'

vi.mock('../../api', () => ({
  getProjects: vi.fn(),
  getAgents: vi.fn(),
  getRepositoryItems: vi.fn(),
  getSetting: vi.fn(),
  syncRepo: vi.fn(),
  getMcpSelections: vi.fn(),
  setMcpSelection: vi.fn(),
  applyMcpServers: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

const toastError = vi.fn()
const toastSuccess = vi.fn()
vi.mock('../../composables/useToast', () => ({
  useToast: () => ({ error: toastError, success: toastSuccess }),
}))

vi.mock('../../components/PageLayout.vue', () => ({
  default: { template: '<div><slot /><slot name="actions" /></div>' },
}))
vi.mock('../../components/ContentModal.vue', () => ({
  default: { props: ['isOpen', 'title', 'content'], template: '<div />' },
}))
vi.mock('../../components/EmptyState.vue', () => ({
  default: { props: ['message', 'glass'], template: '<div class="empty">{{ message }}</div>' },
}))
vi.mock('../../components/BaseButton.vue', () => ({
  default: {
    props: ['variant', 'disabled', 'icon'],
    emits: ['click'],
    template: '<button :disabled="disabled" @click="$emit(\'click\')"><slot /></button>',
  },
}))
vi.mock('../../components/TickBox.vue', () => ({
  default: {
    props: ['checked'],
    emits: ['change'],
    template: '<input type="checkbox" :checked="checked" @change="$emit(\'change\')" />',
  },
}))
vi.mock('../../components/ConfirmModal.vue', () => ({
  default: {
    props: ['isOpen', 'title', 'message', 'confirmText', 'danger'],
    emits: ['confirm', 'cancel'],
    template: `<div v-if="isOpen" class="confirm-modal">
      <button class="confirm-btn" @click="$emit('confirm')">Confirm</button>
      <button class="cancel-btn" @click="$emit('cancel')">Cancel</button>
    </div>`,
  },
}))

const mockAgents = [
  { id: 'windsurf', name: 'Windsurf', skillsPath: '.windsurf/skills', rulesPath: '.windsurf/rules', mcpPath: '.windsurf/mcp.json', globalMcpPath: '~/.codeium/windsurf/mcp_config.json' },
]

const mockProjects = [
  { id: 1, path: '/home/user/proj', agentIds: ['windsurf'] },
]

const mockItems = [
  {
    id: 'mcp-servers-context7.json',
    name: 'context7.json',
    folder: 'mcp-servers',
    description: 'File from mcp-servers folder',
    file_path: '/repo/mcp-servers/context7.json',
    content: JSON.stringify({ key: 'context7', command: 'npx', args: ['-y', 'ctx7'] }),
    sha: 'abc',
    last_synced: null,
  },
]

function findButton(wrapper: any, text: string) {
  return wrapper.findAll('button').find((b: any) => b.text().includes(text))
}

describe('McpView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.getProjects).mockResolvedValue(mockProjects as any)
    vi.mocked(api.getAgents).mockResolvedValue(mockAgents as any)
    vi.mocked(api.getRepositoryItems).mockResolvedValue(mockItems as any)
    vi.mocked(api.getMcpSelections).mockResolvedValue([])
    vi.mocked(api.setMcpSelection).mockResolvedValue(undefined)
    vi.mocked(api.applyMcpServers).mockResolvedValue({ applied: 0, adopted: 0, removed: 0, clashes: [] })
  })

  it('loads MCP servers and renders a Global column plus a project column', async () => {
    const wrapper = mount(McpView)
    await flushPromises()

    expect(api.getRepositoryItems).toHaveBeenCalledWith('mcp-servers')
    const headers = wrapper.findAll('th').map(h => h.text())
    expect(headers).toContain('Global')
    expect(headers).toContain('proj')
    // One server row → 2 tickboxes (global + project).
    expect(wrapper.findAll('input[type="checkbox"]').length).toBe(2)
  })

  it('persists a global selection via setMcpSelection', async () => {
    const wrapper = mount(McpView)
    await flushPromises()

    // First checkbox is the Global column.
    await wrapper.findAll('input[type="checkbox"]')[0].trigger('change')
    await flushPromises()

    expect(api.setMcpSelection).toHaveBeenCalledWith('mcp-servers-context7.json', 'global', 0, true)
  })

  it('builds project + global apply tasks from the selection matrix', async () => {
    // Both global and the project are pre-selected.
    vi.mocked(api.getMcpSelections).mockResolvedValue([
      { itemId: 'mcp-servers-context7.json', scope: 'global', projectId: 0 },
      { itemId: 'mcp-servers-context7.json', scope: 'project', projectId: 1 },
    ])

    const wrapper = mount(McpView)
    await flushPromises()

    await findButton(wrapper, 'Apply')!.trigger('click')
    await flushPromises()

    expect(api.applyMcpServers).toHaveBeenCalledOnce()
    const tasks = vi.mocked(api.applyMcpServers).mock.calls[0][0]

    const global = tasks.find(t => t.targetPath === '~/.codeium/windsurf/mcp_config.json')
    const project = tasks.find(t => t.targetPath === '/home/user/proj/.windsurf/mcp.json')

    expect(global).toMatchObject({ serverKey: 'context7', remove: false, force: false })
    expect(project).toMatchObject({ serverKey: 'context7', remove: false, force: false })
    // The repo file's `key` is stripped from the written config.
    expect(JSON.parse(global!.config)).toEqual({ command: 'npx', args: ['-y', 'ctx7'] })
  })

  it('emits remove tasks for unselected servers', async () => {
    const wrapper = mount(McpView)
    await flushPromises()

    await findButton(wrapper, 'Apply')!.trigger('click')
    await flushPromises()

    const tasks = vi.mocked(api.applyMcpServers).mock.calls[0][0]
    expect(tasks.every(t => t.remove)).toBe(true)
  })

  it('shows the overwrite modal on a clash and re-applies with force on confirm', async () => {
    vi.mocked(api.getMcpSelections).mockResolvedValue([
      { itemId: 'mcp-servers-context7.json', scope: 'global', projectId: 0 },
    ])
    vi.mocked(api.applyMcpServers)
      .mockResolvedValueOnce({
        applied: 0, adopted: 0, removed: 0,
        clashes: [{ targetPath: '~/.codeium/windsurf/mcp_config.json', serverKey: 'context7', existingConfig: '{}', incomingConfig: '{}' }],
      })
      .mockResolvedValueOnce({ applied: 1, adopted: 0, removed: 0, clashes: [] })

    const wrapper = mount(McpView)
    await flushPromises()

    await findButton(wrapper, 'Apply')!.trigger('click')
    await flushPromises()

    // Modal appears; first apply was not forced.
    expect(wrapper.find('.confirm-modal').exists()).toBe(true)
    expect(vi.mocked(api.applyMcpServers).mock.calls[0][0].every(t => !t.force)).toBe(true)

    await wrapper.find('.confirm-btn').trigger('click')
    await flushPromises()

    expect(api.applyMcpServers).toHaveBeenCalledTimes(2)
    expect(vi.mocked(api.applyMcpServers).mock.calls[1][0].every(t => t.force)).toBe(true)
    expect(toastSuccess).toHaveBeenCalled()
  })

  it('surfaces an error when a server file is not valid JSON', async () => {
    vi.mocked(api.getRepositoryItems).mockResolvedValue([
      { ...mockItems[0], content: '{ not json' },
    ] as any)

    const wrapper = mount(McpView)
    await flushPromises()

    await findButton(wrapper, 'Apply')!.trigger('click')
    await flushPromises()

    expect(api.applyMcpServers).not.toHaveBeenCalled()
    expect(toastError).toHaveBeenCalled()
  })
})
