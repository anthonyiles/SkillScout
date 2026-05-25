import { describe, it, expect, vi, beforeEach } from 'vitest'
import { defineComponent } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import { invoke } from '@tauri-apps/api/core'
import { useItemsMatrix } from '../../composables/useItemsMatrix'

const mockInvoke = vi.mocked(invoke)

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
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_projects') return mockProjects
      if (cmd === 'get_agents') return mockAgents
      if (cmd === 'get_repository_items') return mockItems
      if (cmd === 'get_item_selections') return []
      if (cmd === 'get_project_files') return []
      if (cmd === 'toggle_item_selection') return undefined
      return null
    })
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

  it('initialises selectionMatrix from get_item_selections', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_projects') return mockProjects
      if (cmd === 'get_agents') return mockAgents
      if (cmd === 'get_repository_items') return mockItems
      if (cmd === 'get_item_selections') return [{ item_id: 'skill-1', project_id: 1, applied_sha: null }]
      if (cmd === 'get_project_files') return ['my-skill.md']
      return null
    })

    const { result } = mountMatrix()
    await flushPromises()
    expect(result.isSelected('skill-1', 1)).toBe(true)
  })

  it('toggleSelection optimistically selects an item', async () => {
    const { result } = mountMatrix()
    await flushPromises()

    mockInvoke.mockResolvedValueOnce(undefined)
    await result.toggleSelection('skill-1', 1)

    expect(result.isSelected('skill-1', 1)).toBe(true)
  })

  it('toggleSelection optimistically deselects an already-selected item', async () => {
    const { result } = mountMatrix()
    await flushPromises()

    // Select first
    mockInvoke.mockResolvedValueOnce(undefined)
    await result.toggleSelection('skill-1', 1)
    expect(result.isSelected('skill-1', 1)).toBe(true)

    // Then deselect
    mockInvoke.mockResolvedValueOnce(undefined)
    await result.toggleSelection('skill-1', 1)
    expect(result.isSelected('skill-1', 1)).toBe(false)
  })

  it('toggleSelection rolls back the optimistic update on invoke error', async () => {
    const { result } = mountMatrix()
    await flushPromises()

    // Manually pre-select via the matrix so we can test rollback of a deselect
    result.selectionMatrix.value['skill-1'] = new Set([1])

    mockInvoke.mockRejectedValueOnce(new Error('DB error'))
    await result.toggleSelection('skill-1', 1)

    // Was selected, tried to deselect, backend failed → should be re-selected
    expect(result.isSelected('skill-1', 1)).toBe(true)
  })

  it('toggleSelection calls invoke with the correct command and args', async () => {
    const { result } = mountMatrix()
    await flushPromises()
    vi.clearAllMocks()

    mockInvoke.mockResolvedValueOnce(undefined)
    await result.toggleSelection('skill-1', 1)

    expect(mockInvoke).toHaveBeenCalledWith('toggle_item_selection', {
      itemId: 'skill-1',
      projectId: 1,
    })
  })

  it('loading is true during syncRepo and false afterward', async () => {
    const { result } = mountMatrix()
    await flushPromises()

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_setting') return 'https://github.com/user/repo'
      if (cmd === 'sync_repo') return 5
      if (cmd === 'get_projects') return mockProjects
      if (cmd === 'get_agents') return mockAgents
      if (cmd === 'get_repository_items') return mockItems
      if (cmd === 'get_item_selections') return []
      if (cmd === 'get_project_files') return []
      return null
    })

    const syncPromise = result.syncRepo()
    expect(result.loading.value).toBe(true)
    await syncPromise
    expect(result.loading.value).toBe(false)
  })
})
