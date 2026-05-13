<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useToast } from '../composables/useToast'
import { useProjects } from '../composables/useProjects'
import { useAgents } from '../composables/useAgents'
import { syncRepo, promoteItem, checkPrStatus, getProjectFiles } from '../api'
import type { Project } from '../types'
import { getProjectName } from '../utils/formatters'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import CardItem from '../components/CardItem.vue'
import EmptyState from '../components/EmptyState.vue'

interface UnmanagedItem {
  name: string
  path: string
  type: 'skill' | 'rule'
}

interface ProjectWithLocalItems extends Project {
  unmanaged: UnmanagedItem[]
}

const { projects: rawProjects, load: loadProjects } = useProjects()
const { agents, load: loadAgents } = useAgents()
const { success, error } = useToast()

const projects = ref<ProjectWithLocalItems[]>([])
const knownSkills = ref<Set<string>>(new Set())
const knownRules = ref<Set<string>>(new Set())
const scanning = ref(false)
const loading = ref(true)
const promotedItems = ref<Record<string, string>>({})
const checkingPrs = ref<Set<string>>(new Set())
const mergedPrs = ref<Set<string>>(new Set())

async function scanLocal() {
  scanning.value = true
  try {
    const savedSkills = localStorage.getItem('skills')
    if (savedSkills) {
      try { knownSkills.value = new Set(JSON.parse(savedSkills).map((s: any) => s.name)) }
      catch { knownSkills.value = new Set() }
    }
    const savedRules = localStorage.getItem('rules')
    if (savedRules) {
      try { knownRules.value = new Set(JSON.parse(savedRules).map((r: any) => r.name)) }
      catch { knownRules.value = new Set() }
    }

    const enriched: ProjectWithLocalItems[] = []
    for (const p of rawProjects.value) {
      if (!p.path || !p.agentIds) { enriched.push({ ...p, unmanaged: [] }); continue }

      const skillPaths = new Set<string>()
      const rulePaths = new Set<string>()
      for (const agentId of p.agentIds) {
        const agent = agents.value.find(a => a.id === agentId)
        if (agent) {
          if (agent.skillsPath) skillPaths.add(agent.skillsPath)
          if (agent.rulesPath) rulePaths.add(agent.rulesPath)
        }
      }

      const unmanaged: UnmanagedItem[] = []

      if (skillPaths.size > 0) {
        try {
          const found = await getProjectFiles(p.path, Array.from(skillPaths))
          unmanaged.push(...found.filter(f => !knownSkills.value.has(f)).map(f => ({ name: f, path: p.path, type: 'skill' as const })))
        } catch (e) {
          console.error(`Failed to scan skills for ${p.path}:`, e)
          error(`Failed to scan skills in ${p.path}: ${e}`)
        }
      }

      if (rulePaths.size > 0) {
        try {
          const found = await getProjectFiles(p.path, Array.from(rulePaths))
          unmanaged.push(...found.filter(f => !knownRules.value.has(f)).map(f => ({ name: f, path: p.path, type: 'rule' as const })))
        } catch (e) {
          console.error(`Failed to scan rules for ${p.path}:`, e)
          error(`Failed to scan rules in ${p.path}: ${e}`)
        }
      }

      enriched.push({ ...p, unmanaged })
    }
    projects.value = enriched
  } finally {
    scanning.value = false
    loading.value = false
  }
}

onMounted(async () => {
  loadProjects()
  loadAgents()

  const savedPromoted = localStorage.getItem('promotedItems')
  if (savedPromoted) {
    try { promotedItems.value = JSON.parse(savedPromoted) }
    catch { console.error('Failed to parse promoted items') }
  }

  await scanLocal()

  const repoUrl = localStorage.getItem('repoUrl')
  let syncNeeded = false
  let changed = false

  await Promise.all(Object.entries(promotedItems.value).map(async ([key, url]) => {
    checkingPrs.value.add(key)
    try {
      const res = await checkPrStatus(url)
      if (res.state !== 'open') {
        delete promotedItems.value[key]
        changed = true
        if (res.merged) { mergedPrs.value.add(key); syncNeeded = true }
      }
    } catch {
      // Keep entry on error — PR status unknown
    } finally {
      checkingPrs.value.delete(key)
    }
  }))

  if (changed) localStorage.setItem('promotedItems', JSON.stringify(promotedItems.value))

  if (syncNeeded && repoUrl) {
    try {
      const fetched = await syncRepo(repoUrl)
      const filteredSkills = fetched.filter(s => s.folder === 'skills')
      const filteredRules = fetched.filter(s => s.folder === 'rules')
      localStorage.setItem('all_repo_data', JSON.stringify(fetched))
      localStorage.setItem('skills', JSON.stringify(filteredSkills))
      localStorage.setItem('rules', JSON.stringify(filteredRules))
      knownSkills.value = new Set(filteredSkills.map(s => s.name))
      knownRules.value = new Set(filteredRules.map(s => s.name))
      await scanLocal()
      success('Background sync complete. Managed items updated.')
    } catch (e) {
      console.error('Failed background sync:', e)
    }
  }
})

async function handleManualScan() {
  await scanLocal()
  success('Scan complete!')
}

const promotingItem = ref<string | null>(null)

async function handlePromoteItem(item: UnmanagedItem, project: ProjectWithLocalItems) {
  const repoUrl = localStorage.getItem('repoUrl')
  if (!repoUrl) { error('Please configure a repository URL in Settings first.'); return }

  const subFolders = new Set<string>()
  for (const agentId of project.agentIds) {
    const agent = agents.value.find(a => a.id === agentId)
    if (agent) {
      if (item.type === 'skill' && agent.skillsPath) subFolders.add(agent.skillsPath)
      if (item.type === 'rule' && agent.rulesPath) subFolders.add(agent.rulesPath)
    }
  }

  const key = `${project.id}-${item.name}`
  promotingItem.value = key
  try {
    const prUrl = await promoteItem(repoUrl, item.type, item.name, project.path, Array.from(subFolders))
    success('Successfully created PR!')
    promotedItems.value[key] = prUrl
    localStorage.setItem('promotedItems', JSON.stringify(promotedItems.value))
  } catch (e: any) {
    error(typeof e === 'string' ? e : e.message ?? 'Failed to promote item')
    console.error(e)
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

    <EmptyState v-if="loading" message="Loading unmanaged items..." />

    <div v-else class="projects-list">
      <div v-for="project in projects" :key="project.id">
        <CardItem
          v-if="project.unmanaged.length > 0"
          glass
          :title="getProjectName(project.path)"
          :subtitle="`${project.unmanaged.length} items to promote`"
        >
          <div class="items-list">
            <div v-for="item in project.unmanaged" :key="`${item.type}-${item.name}`" class="item-row">
              <div class="item-info">
                <div class="type-badge" :class="item.type">{{ item.type }}</div>
                <div class="name-container">
                  <svg v-if="item.type === 'skill'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                  <span class="item-name">{{ item.name }}</span>
                </div>
              </div>
              <template v-if="checkingPrs.has(`${project.id}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled>
                  <span class="loader small-loader"></span>
                </BaseButton>
              </template>
              <template v-else-if="mergedPrs.has(`${project.id}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled class="success-btn">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#4ade80" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                  Merged!
                </BaseButton>
              </template>
              <template v-else-if="promotedItems[`${project.id}-${item.name}`]">
                <BaseButton
                  variant="primary"
                  size="sm"
                  @click="openUrl(promotedItems[`${project.id}-${item.name}`]).catch(() => error('Failed to open PR URL'))"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
                  View PR
                </BaseButton>
              </template>
              <template v-else>
                <BaseButton
                  variant="secondary"
                  size="sm"
                  :disabled="promotingItem === `${project.id}-${item.name}`"
                  @click="handlePromoteItem(item, project)"
                >
                  <span v-if="promotingItem === `${project.id}-${item.name}`" class="loader small-loader"></span>
                  <span v-else>Promote</span>
                </BaseButton>
              </template>
            </div>
          </div>
        </CardItem>
      </div>

      <EmptyState
        v-if="projects.filter(p => p.unmanaged.length > 0).length === 0"
        glass
        message="No unmanaged items found. All your local files are synced with the central repository!"
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
  min-width: 50px;
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

.name-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.item-name {
  font-family: monospace;
  font-size: 0.9rem;
}

.text-secondary { color: var(--text-secondary); }

.loader {
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top: 2px solid white;
  border-radius: 50%;
  width: 14px;
  height: 14px;
  animation: spin 1s linear infinite;
}

.small-loader { width: 12px; height: 12px; border-width: 2px; }

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
