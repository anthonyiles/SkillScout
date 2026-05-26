import { describe, it, expect, vi, beforeEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import * as api from '../api'

const mockInvoke = vi.mocked(invoke)

describe('api', () => {
  beforeEach(() => {
    mockInvoke.mockResolvedValue(undefined)
  })

  // ── Auth ────────────────────────────────────────────────────────────────

  describe('startGithubDeviceFlow', () => {
    it('calls the correct command with no args', async () => {
      await api.startGithubDeviceFlow()
      expect(mockInvoke).toHaveBeenCalledWith('start_github_device_flow')
    })
  })

  describe('pollGithubToken', () => {
    it('passes deviceCode to the command', async () => {
      await api.pollGithubToken('abc123')
      expect(mockInvoke).toHaveBeenCalledWith('poll_github_token', { deviceCode: 'abc123' })
    })
  })

  describe('checkGithubAuth', () => {
    it('calls the correct command with no args', async () => {
      await api.checkGithubAuth()
      expect(mockInvoke).toHaveBeenCalledWith('check_github_auth')
    })
  })

  describe('logoutGithub', () => {
    it('calls the correct command with no args', async () => {
      await api.logoutGithub()
      expect(mockInvoke).toHaveBeenCalledWith('logout_github')
    })
  })

  // ── Settings ────────────────────────────────────────────────────────────

  describe('getSetting', () => {
    it('passes the key to the command', async () => {
      await api.getSetting('repoUrl')
      expect(mockInvoke).toHaveBeenCalledWith('get_setting', { key: 'repoUrl' })
    })
  })

  describe('setSetting', () => {
    it('passes key and value to the command', async () => {
      await api.setSetting('repoUrl', 'https://github.com/user/repo')
      expect(mockInvoke).toHaveBeenCalledWith('set_setting', {
        key: 'repoUrl',
        value: 'https://github.com/user/repo',
      })
    })
  })

  // ── Projects & Agents ───────────────────────────────────────────────────

  describe('getProjects', () => {
    it('calls the correct command with no args', async () => {
      await api.getProjects()
      expect(mockInvoke).toHaveBeenCalledWith('get_projects')
    })
  })

  describe('saveProject', () => {
    it('passes the project to the command', async () => {
      const project = { id: null, path: '/home/user/project', agentIds: ['cursor'] }
      await api.saveProject(project)
      expect(mockInvoke).toHaveBeenCalledWith('save_project', { project })
    })
  })

  describe('deleteProject', () => {
    it('passes the id to the command', async () => {
      await api.deleteProject(3)
      expect(mockInvoke).toHaveBeenCalledWith('delete_project', { id: 3 })
    })
  })

  describe('getAgents', () => {
    it('calls the correct command with no args', async () => {
      await api.getAgents()
      expect(mockInvoke).toHaveBeenCalledWith('get_agents')
    })
  })

  describe('saveAgent', () => {
    it('passes the agent to the command', async () => {
      const agent = { id: 'cursor', name: 'Cursor', skillsPath: '.cursor/skills', rulesPath: '.cursor/rules' }
      await api.saveAgent(agent)
      expect(mockInvoke).toHaveBeenCalledWith('save_agent', { agent })
    })
  })

  describe('deleteAgent', () => {
    it('passes the id to the command', async () => {
      await api.deleteAgent('custom-abc')
      expect(mockInvoke).toHaveBeenCalledWith('delete_agent', { id: 'custom-abc' })
    })
  })

  describe('resetAgentsToDefaults', () => {
    it('calls the correct command with no args', async () => {
      await api.resetAgentsToDefaults()
      expect(mockInvoke).toHaveBeenCalledWith('reset_agents_to_defaults')
    })
  })

  // ── Repository items ────────────────────────────────────────────────────

  describe('getRepositoryItems', () => {
    it('passes the optional folder arg', async () => {
      await api.getRepositoryItems('skills')
      expect(mockInvoke).toHaveBeenCalledWith('get_repository_items', { folder: 'skills' })
    })

    it('passes undefined folder when omitted', async () => {
      await api.getRepositoryItems()
      expect(mockInvoke).toHaveBeenCalledWith('get_repository_items', { folder: undefined })
    })
  })

  describe('syncRepo', () => {
    it('passes repoUrl to the command', async () => {
      await api.syncRepo('https://github.com/user/repo')
      expect(mockInvoke).toHaveBeenCalledWith('sync_repo', {
        repoUrl: 'https://github.com/user/repo',
      })
    })
  })

  // ── Selections ──────────────────────────────────────────────────────────

  describe('getItemSelections', () => {
    it('calls the correct command with no args', async () => {
      await api.getItemSelections()
      expect(mockInvoke).toHaveBeenCalledWith('get_item_selections')
    })
  })

  describe('toggleItemSelection', () => {
    it('passes itemId and projectId to the command', async () => {
      await api.toggleItemSelection('skill-1', 42)
      expect(mockInvoke).toHaveBeenCalledWith('toggle_item_selection', {
        itemId: 'skill-1',
        projectId: 42,
      })
    })
  })

  // ── File operations ─────────────────────────────────────────────────────

  describe('checkExisting', () => {
    it('passes the tasks array to the command', async () => {
      const tasks: api.SyncTask[] = [
        { source_file: null, target_dir: '/tmp', file_name: 'test.md', remove: false },
      ]
      await api.checkExisting(tasks)
      expect(mockInvoke).toHaveBeenCalledWith('check_existing', { tasks })
    })
  })

  describe('applySkills', () => {
    it('passes the tasks array to the command', async () => {
      const tasks: api.SyncTask[] = [
        { source_file: '/src/test.md', target_dir: '/tmp', file_name: 'test.md', remove: false },
      ]
      await api.applySkills(tasks)
      expect(mockInvoke).toHaveBeenCalledWith('apply_skills', { tasks })
    })
  })

  describe('getProjectFiles', () => {
    it('passes projectPath and subFolders to the command', async () => {
      await api.getProjectFiles('/home/user/project', ['.cursor/rules'])
      expect(mockInvoke).toHaveBeenCalledWith('get_project_files', {
        projectPath: '/home/user/project',
        subFolders: ['.cursor/rules'],
      })
    })
  })

  describe('getProjectFileHashes', () => {
    it('passes projectPath and subFolders to the command', async () => {
      await api.getProjectFileHashes('/home/user/project', ['.cursor/rules'])
      expect(mockInvoke).toHaveBeenCalledWith('get_project_file_hashes', {
        projectPath: '/home/user/project',
        subFolders: ['.cursor/rules'],
      })
    })
  })

  // ── Promoted items ──────────────────────────────────────────────────────

  describe('getPromotedItems', () => {
    it('calls the correct command with no args', async () => {
      await api.getPromotedItems()
      expect(mockInvoke).toHaveBeenCalledWith('get_promoted_items')
    })
  })

  describe('addPromotedItem', () => {
    it('passes the item to the command', async () => {
      const item: Omit<api.PromotedItem, 'id'> = {
        name: 'my-skill.md',
        path: '/path/to/skill',
        itemType: 'skills',
        branch: 'feat/my-skill',
      }
      await api.addPromotedItem(item)
      expect(mockInvoke).toHaveBeenCalledWith('add_promoted_item', { item })
    })
  })

  describe('removePromotedItem', () => {
    it('passes the id to the command', async () => {
      await api.removePromotedItem(7)
      expect(mockInvoke).toHaveBeenCalledWith('remove_promoted_item', { id: 7 })
    })
  })

  describe('promoteItem', () => {
    it('passes all positional args to the command', async () => {
      await api.promoteItem(
        'https://github.com/user/repo',
        'skills',
        'my-skill.md',
        '/home/user/project',
        ['.cursor/rules'],
      )
      expect(mockInvoke).toHaveBeenCalledWith('promote_item', {
        repoUrl: 'https://github.com/user/repo',
        itemType: 'skills',
        itemName: 'my-skill.md',
        projectPath: '/home/user/project',
        subFolders: ['.cursor/rules'],
        updateMode: undefined,
      })
    })

    it('passes updateMode when provided', async () => {
      await api.promoteItem(
        'https://github.com/user/repo',
        'skills',
        'my-skill.md',
        '/home/user/project',
        ['.cursor/rules'],
        true,
      )
      expect(mockInvoke).toHaveBeenCalledWith('promote_item', {
        repoUrl: 'https://github.com/user/repo',
        itemType: 'skills',
        itemName: 'my-skill.md',
        projectPath: '/home/user/project',
        subFolders: ['.cursor/rules'],
        updateMode: true,
      })
    })
  })

  describe('checkPrStatus', () => {
    it('passes prUrl to the command', async () => {
      await api.checkPrStatus('https://github.com/user/repo/pull/1')
      expect(mockInvoke).toHaveBeenCalledWith('check_pr_status', {
        prUrl: 'https://github.com/user/repo/pull/1',
      })
    })
  })
})
