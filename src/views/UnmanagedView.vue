<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useToast } from '../composables/useToast'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import CardItem from '../components/CardItem.vue'
import EmptyState from '../components/EmptyState.vue'

interface Project {
  id: number
  path: string
  agentIds: string[]
}

interface Agent {
  id: string
  name: string
  skillsPath: string
  rulesPath: string
}

interface UnmanagedItem {
  name: string
  path: string
  type: 'skill' | 'rule'
}

interface ModifiedItem {
  name: string
  path: string
  type: 'skill' | 'rule'
  repositoryItemId: string
}

interface FileHash {
  name: string
  sha: string
}

interface ProjectWithLocalItems extends Project {
  unmanaged: UnmanagedItem[]
  modified: ModifiedItem[]
}

interface PromotedItem {
  id?: number
  name: string
  path: string
  itemType: string
  repository_item_id?: string
  url: string
  branch: string
}

const projects = ref<ProjectWithLocalItems[]>([])
const availableAgents = ref<Agent[]>([])
// name -> { id, sha } for repo items
const knownSkills = ref<Map<string, { id: string; sha: string }>>(new Map())
const knownRules = ref<Map<string, { id: string; sha: string }>>(new Map())
const scanning = ref(false)
const loading = ref(true)

// Mapping from `${project.path}-${item.name}` to PromotedItem
const promotedItems = ref<Record<string, PromotedItem>>({})
const checkingPrs = ref<Set<string>>(new Set())
const mergedPrs = ref<Set<string>>(new Set())

const { success, error } = useToast()

function getProjectName(path: string) {
  if (!path) return 'New Project'
  const parts = path.split(/[/\\]/).filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : 'New Project'
}

async function scanLocal() {
  scanning.value = true

  try {
    const agents = await invoke<Agent[]>('get_agents')
    if (agents) availableAgents.value = agents

    const skills = await invoke<any[]>('get_repository_items', { folder: 'skills' })
    if (skills) {
      knownSkills.value = new Map(skills.map((s: any) => [s.name, { id: s.id, sha: s.sha }]))
    }

    const rules = await invoke<any[]>('get_repository_items', { folder: 'rules' })
    if (rules) {
      knownRules.value = new Map(rules.map((r: any) => [r.name, { id: r.id, sha: r.sha }]))
    }

    const rawProjects = await invoke<Project[]>('get_projects')
    if (rawProjects) {
      const enriched: ProjectWithLocalItems[] = []

      for (const p of rawProjects) {
        if (!p.path || !p.agentIds) {
          enriched.push({ ...p, unmanaged: [], modified: [] })
          continue
        }

        const skillPaths = new Set<string>()
        const rulePaths = new Set<string>()

        for (const agentId of p.agentIds) {
          const agent = availableAgents.value.find(a => a.id === agentId)
          if (agent) {
            if (agent.skillsPath) skillPaths.add(agent.skillsPath)
            if (agent.rulesPath) rulePaths.add(agent.rulesPath)
          }
        }

        const unmanaged: UnmanagedItem[] = []
        const modified: ModifiedItem[] = []

        if (skillPaths.size > 0) {
          try {
            const hashes: FileHash[] = await invoke('get_project_file_hashes', {
              projectPath: p.path,
              subFolders: Array.from(skillPaths)
            })
            for (const f of hashes) {
              const known = knownSkills.value.get(f.name)
              if (!known) {
                unmanaged.push({ name: f.name, path: p.path, type: 'skill' })
              } else if (known.sha !== f.sha) {
                modified.push({ name: f.name, path: p.path, type: 'skill', repositoryItemId: known.id })
              }
            }
          } catch (e) {
            console.error(`Failed to scan skills in ${p.path}:`, e)
          }
        }

        if (rulePaths.size > 0) {
          try {
            const hashes: FileHash[] = await invoke('get_project_file_hashes', {
              projectPath: p.path,
              subFolders: Array.from(rulePaths)
            })
            for (const f of hashes) {
              const known = knownRules.value.get(f.name)
              if (!known) {
                unmanaged.push({ name: f.name, path: p.path, type: 'rule' })
              } else if (known.sha !== f.sha) {
                modified.push({ name: f.name, path: p.path, type: 'rule', repositoryItemId: known.id })
              }
            }
          } catch (e) {
            console.error(`Failed to scan rules in ${p.path}:`, e)
          }
        }

        enriched.push({ ...p, unmanaged, modified })
      }
      projects.value = enriched
    }
  } catch (e) {
    console.error('Failed to load data for scan:', e)
  } finally {
    scanning.value = false
    loading.value = false
  }
}

onMounted(async () => {
  try {
    const promoted = await invoke<PromotedItem[]>('get_promoted_items')
    if (promoted) {
      const pmap: Record<string, PromotedItem> = {}
      for (const item of promoted) {
        pmap[`${item.path}-${item.name}`] = item
      }
      promotedItems.value = pmap
    }
  } catch (e) {
    console.error('Failed to load promoted items:', e)
  }

  await scanLocal()

  let syncNeeded = false

  await Promise.all(Object.values(promotedItems.value).map(async (item) => {
    const key = `${item.path}-${item.name}`
    checkingPrs.value.add(key)
    try {
      const res = await invoke<any>('check_pr_status', { prUrl: item.url })
      if (res.state !== 'open') {
        if (item.id) {
          await invoke('remove_promoted_item', { id: item.id })
        }
        delete promotedItems.value[key]
        if (res.merged) {
          mergedPrs.value.add(key)
          syncNeeded = true
        }
      }
    } catch (e) {
      // Keep on error
    } finally {
      checkingPrs.value.delete(key)
    }
  }))

  if (syncNeeded) {
    const repoUrl = await invoke<string | null>('get_setting', { key: 'repoUrl' })
    if (repoUrl) {
      try {
        await invoke('sync_repo', { repoUrl })
        await scanLocal()
        success('Background sync complete. Managed items updated.')
      } catch (e) {
        console.error('Failed background sync:', e)
      }
    }
  }
})

async function handleManualScan() {
  await scanLocal()
  success('Scan complete!')
}

const promotingItem = ref<string | null>(null)

async function getSubFolders(project: ProjectWithLocalItems, type: 'skill' | 'rule'): Promise<string[]> {
  const subFolders = new Set<string>()
  for (const agentId of project.agentIds) {
    const agent = availableAgents.value.find(a => a.id === agentId)
    if (agent) {
      if (type === 'skill' && agent.skillsPath) subFolders.add(agent.skillsPath)
      if (type === 'rule' && agent.rulesPath) subFolders.add(agent.rulesPath)
    }
  }
  return Array.from(subFolders)
}

async function promoteItem(item: UnmanagedItem, project: ProjectWithLocalItems) {
  const repoUrl = await invoke<string | null>('get_setting', { key: 'repoUrl' })
  if (!repoUrl) {
    error('Please configure a repository URL in Settings first.')
    return
  }

  const key = `${project.path}-${item.name}`
  promotingItem.value = key
  try {
    const result = await invoke<{ url: string; branch: string }>('promote_item', {
      repoUrl,
      itemType: item.type,
      itemName: item.name,
      projectPath: project.path,
      subFolders: await getSubFolders(project, item.type),
    })

    const pItem: PromotedItem = {
      name: item.name,
      path: project.path,
      itemType: item.type,
      url: result.url,
      branch: result.branch
    }
    const saved = await invoke<PromotedItem>('add_promoted_item', { item: pItem })

    success(`Successfully created PR!`)
    promotedItems.value[key] = saved
  } catch (e: any) {
    console.error('Failed to promote item:', e)
    error(typeof e === 'string' ? e : 'Failed to promote item. Please try again.')
  } finally {
    promotingItem.value = null
  }
}

async function promoteUpdate(item: ModifiedItem, project: ProjectWithLocalItems) {
  const repoUrl = await invoke<string | null>('get_setting', { key: 'repoUrl' })
  if (!repoUrl) {
    error('Please configure a repository URL in Settings first.')
    return
  }

  const key = `${project.path}-${item.name}`
  promotingItem.value = key
  try {
    const result = await invoke<{ url: string; branch: string }>('promote_item', {
      repoUrl,
      itemType: item.type,
      itemName: item.name,
      projectPath: project.path,
      subFolders: await getSubFolders(project, item.type),
      updateMode: true,
    })

    const pItem: PromotedItem = {
      name: item.name,
      path: project.path,
      itemType: item.type,
      repository_item_id: item.repositoryItemId,
      url: result.url,
      branch: result.branch
    }
    const saved = await invoke<PromotedItem>('add_promoted_item', { item: pItem })

    success(`Successfully created update PR!`)
    promotedItems.value[key] = saved
  } catch (e: any) {
    console.error('Failed to promote update:', e)
    error(typeof e === 'string' ? e : 'Failed to create update PR. Please try again.')
  } finally {
    promotingItem.value = null
  }
}
</script>

<template>
  <PageLayout title="Unmanaged" description="Skills and rules found in your local projects that are not managed by your central repository.">
    <template #actions>
      <BaseButton variant="secondary" @click="handleManualScan" :disabled="loading || scanning">
        <svg v-if="!scanning" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12h3"/><path d="M19 12h3"/><path d="M12 2v3"/><path d="M12 19v3"/><path d="m4.93 4.93 2.12 2.12"/><path d="m16.95 16.95 2.12 2.12"/><path d="m4.93 19.07 2.12-2.12"/><path d="m16.95 7.05 2.12-2.12"/></svg>
        <span v-else class="loader"></span>
        {{ scanning ? 'Scanning...' : 'Scan files' }}
      </BaseButton>
    </template>

    <EmptyState
      v-if="loading"
      message="Loading unmanaged items..."
    />
    <div v-else class="projects-list">

      <!-- Unmanaged items (no repo match) -->
      <template v-for="project in projects" :key="`unmanaged-${project.id}`">
        <CardItem
          v-if="project.unmanaged.length > 0"
          glass
          :title="getProjectName(project.path)"
          :subtitle="`${project.unmanaged.length} item${project.unmanaged.length === 1 ? '' : 's'} to promote`"
        >
          <div class="items-list">
            <div v-for="item in project.unmanaged" :key="`${item.type}-${item.name}`" class="item-row">
              <div class="item-info">
                <div class="type-badge" :class="item.type">
                  {{ item.type }}
                </div>
                <div class="name-container">
                  <svg v-if="item.type === 'skill'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                  <span class="item-name">{{ item.name }}</span>
                </div>
              </div>
              <template v-if="checkingPrs.has(`${project.path}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled>
                  <span class="loader small-loader"></span>
                </BaseButton>
              </template>
              <template v-else-if="mergedPrs.has(`${project.path}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled class="success-btn">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#4ade80" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                  Merged!
                </BaseButton>
              </template>
              <template v-else-if="promotedItems[`${project.path}-${item.name}`]">
                <BaseButton
                  variant="primary"
                  size="sm"
                  @click="openUrl(promotedItems[`${project.path}-${item.name}`].url).catch(() => error('Failed to open PR URL'))"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
                  View PR
                </BaseButton>
              </template>
              <template v-else>
                <BaseButton
                  variant="secondary"
                  size="sm"
                  :disabled="promotingItem === `${project.path}-${item.name}`"
                  @click="promoteItem(item, project)"
                >
                  <span v-if="promotingItem === `${project.path}-${item.name}`" class="loader small-loader"></span>
                  <span v-else>Promote</span>
                </BaseButton>
              </template>
            </div>
          </div>
        </CardItem>
      </template>

      <!-- Modified managed items (local changes differ from repo) -->
      <template v-for="project in projects" :key="`modified-${project.id}`">
        <CardItem
          v-if="project.modified.length > 0"
          glass
          :title="getProjectName(project.path)"
          :subtitle="`${project.modified.length} locally modified item${project.modified.length === 1 ? '' : 's'}`"
        >
          <template #actions>
            <span class="section-badge modified">Modified</span>
          </template>
          <div class="items-list">
            <div v-for="item in project.modified" :key="`${item.type}-${item.name}`" class="item-row">
              <div class="item-info">
                <div class="type-badge modified">
                  modified
                </div>
                <div class="name-container">
                  <svg v-if="item.type === 'skill'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                  <span class="item-name">{{ item.name }}</span>
                </div>
              </div>
              <template v-if="checkingPrs.has(`${project.path}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled>
                  <span class="loader small-loader"></span>
                </BaseButton>
              </template>
              <template v-else-if="mergedPrs.has(`${project.path}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled class="success-btn">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#4ade80" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                  Merged!
                </BaseButton>
              </template>
              <template v-else-if="promotedItems[`${project.path}-${item.name}`]">
                <BaseButton
                  variant="primary"
                  size="sm"
                  @click="openUrl(promotedItems[`${project.path}-${item.name}`].url).catch(() => error('Failed to open PR URL'))"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
                  View PR
                </BaseButton>
              </template>
              <template v-else>
                <BaseButton
                  variant="secondary"
                  size="sm"
                  :disabled="promotingItem === `${project.path}-${item.name}`"
                  @click="promoteUpdate(item, project)"
                >
                  <span v-if="promotingItem === `${project.path}-${item.name}`" class="loader small-loader"></span>
                  <span v-else>Promote update</span>
                </BaseButton>
              </template>
            </div>
          </div>
        </CardItem>
      </template>

      <EmptyState
        v-if="projects.filter(p => p.unmanaged.length > 0 || p.modified.length > 0).length === 0"
        glass
        message="No unmanaged or locally modified items found. All your local files are synced with the central repository!"
      />
    </div>
  </PageLayout>
</template>

<style scoped>
.projects-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.item-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background: rgba(0, 0, 0, 0.15);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
}

.item-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.type-badge {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  min-width: 60px;
  text-align: center;
}

.type-badge.skill {
  background: rgba(52, 152, 219, 0.2);
  color: #3498db;
  border: 1px solid rgba(52, 152, 219, 0.3);
}

.type-badge.rule {
  background: rgba(155, 89, 182, 0.2);
  color: #9b59b2;
  border: 1px solid rgba(155, 89, 182, 0.3);
}

.type-badge.modified {
  background: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.section-badge {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
}

.section-badge.modified {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.25);
}

.name-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.item-name {
  font-family: monospace;
  font-size: 0.9rem;
}

.text-secondary {
  color: var(--text-secondary);
}

.loader {
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top: 2px solid white;
  border-radius: 50%;
  width: 14px;
  height: 14px;
  animation: spin 1s linear infinite;
}

.small-loader {
  width: 12px;
  height: 12px;
  border-width: 2px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.success-btn {
  color: #4ade80;
  border-color: rgba(74, 222, 128, 0.3);
  background: rgba(74, 222, 128, 0.1);
  opacity: 1 !important;
}
</style>
