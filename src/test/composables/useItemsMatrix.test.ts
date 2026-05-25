import { describe, it, expect, vi, beforeEach } from 'vitest'
import { defineComponent } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import * as api from '../../api'
import { useItemsMatrix } from '../../composables/useItemsMatrix'

vi.mock('../../api', () => ({
  getProjects: vi.fn(),
  getAgents: vi.fn(),
  getRepositoryItems: vi.fn(),
  getSetting: vi.fn(),
  syncRepo: vi.fn(),
  getItemSelections: vi.fn(),
  toggleItemSelection: vi.fn(),
  getProjectFiles: vi.fn(),
  applySkills: vi.fn(),
}))

const mockProjects = [
  { id: 1, path: '/home/user/project-a', agentIds: ['cursor'] },
]

const mockAgents = [
  { id: 'cursor', name: 'Cursor', skillsPath: '.cursor/rules', rulesPath: '.cursor/rules' },
]

const mockItems = [
  {
    id: 'skill-1',
    name: 'my-skill.md',
    folder: 'skills',
    description: null,
    file_path: '/repo/skills/my-skill.md',
    content: '# My Skill',
    sha: 'abc123',
    last_synced: null,
  },
]

function mountMatrix(folder: 'skills' | 'rules' = 'skills') {
  let exposedResult: ReturnType<typeof useItemsMatrix> | undefined

  const component = defineComponent({
    setup() {
      const result = useItemsMatrix(folder)
      exposedResult = result
      return result
    },
    template: '<div />',
  })

  mount(component)

  return {
    get result() {
      if (!exposedResult) throw new Error('composable not yet initialised')
      return exposedResult
    },
  }
}

describe('useItemsMatrix', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.getProjects).mockResolvedValue(mockProjects)
    vi.mocked(api.getAgents).mockResolvedValue(mockAgents)
    vi.mocked(api.getRepositoryItems).mockResolvedValue(mockItems)
    vi.mocked(api.getItemSelections).mockResolvedValue([])
    vi.mocked(api.getProjectFiles).mockResolvedValue([])
    vi.mocked(api.toggleItemSelection).mockResolvedValue(undefined)
    vi.mocked(api.getSetting).mockResolvedValue(null)
    vi.mocked(api.syncRepo).mockResolvedValue(0)
    vi.mocked(api.applySkills).mockResolvedValue(0)
  })

  it('getProjectName extracts the last path segment', () => {
    const { result } = mountMatrix()
    expect(result.getProjectName('/home/user/my-project')).toBe('my-project')
  })

  it('getProjectName returns "New Project" for an empty string', () => {
    const { result } = mountMatrix()
    expect(result.getProjectName('')).toBe('New Project')
  })

  it('isSelected returns false before any selection is made', async () => {
    const { result } = mountMatrix()
    await flushPromises()
    expect(result.isSelected('skill-1', 1)).toBe(false)
  })

  it('populates items after mount', async () => {
    const { result } = mountMatrix()
    await flushPromises()
    expect(result.items.value).toHaveLength(1)
    expect(result.items.value[0].id).toBe('skill-1')
  })

  it('populates projects after mount', async () => {
    const { result } = mountMatrix()
    await flushPromises()
    expect(result.projects.value).toHaveLength(1)
    expect(result.projects.value[0].path).toBe('/home/user/project-a')
  })

  it('initialises selectionMatrix from getItemSelections', async () => {
    vi.mocked(api.getItemSelections).mockResolvedValue([
      { item_id: 'skill-1', project_id: 1, applied_sha: null },
    ])
    vi.mocked(api.getProjectFiles).mockResolvedValue(['my-skill.md'])

    const { result } = mountMatrix()
    await flushPromises()
    expect(result.isSelected('skill-1', 1)).toBe(true)
  })

  it('toggleSelection optimistically selects an item', async () => {
    const { result } = mountMatrix()
    await flushPromises()

    await result.toggleSelection('skill-1', 1)
    expect(result.isSelected('skill-1', 1)).toBe(true)
    expect(api.toggleItemSelection).toHaveBeenCalledWith('skill-1', 1)
  })

  it('toggleSelection optimistically deselects an already-selected item', async () => {
    const { result } = mountMatrix()
    await flushPromises()

    await result.toggleSelection('skill-1', 1)
    expect(result.isSelected('skill-1', 1)).toBe(true)

    await result.toggleSelection('skill-1', 1)
    expect(result.isSelected('skill-1', 1)).toBe(false)
  })

  it('toggleSelection rolls back the optimistic update on API error', async () => {
    const { result } = mountMatrix()
    await flushPromises()

    result.selectionMatrix.value['skill-1'] = new Set([1])
    vi.mocked(api.toggleItemSelection).mockRejectedValueOnce(new Error('DB error'))

    await result.toggleSelection('skill-1', 1)

    // Was selected, tried to deselect, backend failed → should be re-selected
    expect(result.isSelected('skill-1', 1)).toBe(true)
  })

  it('loading is true during syncRepo and false afterward', async () => {
    vi.mocked(api.getSetting).mockResolvedValue('https://github.com/user/repo')
    vi.mocked(api.syncRepo).mockResolvedValue(5)

    const { result } = mountMatrix()
    await flushPromises()

    const syncPromise = result.syncRepo()
    expect(result.loading.value).toBe(true)
    await syncPromise
    expect(result.loading.value).toBe(false)
  })

  it('syncRepo shows an error when no repo URL is configured', async () => {
    vi.mocked(api.getSetting).mockResolvedValue(null)
    const { result } = mountMatrix()
    await flushPromises()

    await result.syncRepo()
    expect(api.syncRepo).not.toHaveBeenCalled()
  })
})
