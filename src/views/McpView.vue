<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useToast } from '../composables/useToast'
import { formatError } from '../utils/formatError'
import {
  getProjects,
  getAgents,
  getRepositoryItems,
  getSetting,
  syncRepo as apiSyncRepo,
  getMcpSelections,
  setMcpSelection,
  applyMcpServers,
  type Project,
  type RepositoryItem,
  type McpApplyTask,
  type McpClash,
} from '../api'
import type { Agent } from '../types'
import ContentModal from '../components/ContentModal.vue'
import ConfirmModal from '../components/ConfirmModal.vue'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import EmptyState from '../components/EmptyState.vue'

// 0 is the sentinel column key for the "Global" scope; project ids start at 1.
const GLOBAL = 0

const { error, success } = useToast()

const items = ref<RepositoryItem[]>([])
const projects = ref<Project[]>([])
const agents = ref<Agent[]>([])
const selectionMatrix = ref<Record<string, Set<number>>>({})
const loading = ref(false)
const applying = ref(false)

const activeItem = ref<RepositoryItem | null>(null)
const isModalOpen = ref(false)

const clashes = ref<McpClash[]>([])
const isClashModalOpen = ref(false)

let unlistenSync: (() => void) | undefined

function getProjectName(path: string): string {
  if (!path) return 'New Project'
  const parts = path.split(/[/\\]/).filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : 'New Project'
}

function isSelected(itemId: string, key: number): boolean {
  return selectionMatrix.value[itemId]?.has(key) ?? false
}

function openPreview(item: RepositoryItem) {
  activeItem.value = item
  isModalOpen.value = true
}

/**
 * Parse a repo MCP file into one or more (serverKey, config) pairs.
 * Supports three shapes:
 *  - { "mcpServers": { "<key>": {...} } }  → one entry per key
 *  - { "key": "<name>", ...config }        → a single named server
 *  - { ...config }                         → single server keyed by file name
 * Throws if the content is not a JSON object.
 */
function parseServers(item: RepositoryItem): { key: string; config: string }[] {
  const obj = JSON.parse(item.content)
  if (obj === null || typeof obj !== 'object' || Array.isArray(obj)) {
    throw new Error('not a JSON object')
  }
  if (obj.mcpServers && typeof obj.mcpServers === 'object' && !Array.isArray(obj.mcpServers)) {
    return Object.entries(obj.mcpServers).map(([key, config]) => ({ key, config: JSON.stringify(config) }))
  }
  if (typeof obj.key === 'string') {
    const { key, ...config } = obj
    return [{ key, config: JSON.stringify(config) }]
  }
  const key = item.name.replace(/\.json$/i, '')
  return [{ key, config: JSON.stringify(obj) }]
}

async function initializeMatrix() {
  const nextMatrix: Record<string, Set<number>> = {}
  items.value.forEach(item => { nextMatrix[item.id] = new Set() })

  try {
    const selections = await getMcpSelections()
    for (const sel of selections ?? []) {
      const key = sel.scope === 'global' ? GLOBAL : sel.projectId
      if (nextMatrix[sel.itemId]) nextMatrix[sel.itemId].add(key)
    }
  } catch (err) {
    console.error('Failed to load MCP selections:', err)
  }

  selectionMatrix.value = nextMatrix
}

async function loadData() {
  try {
    const fetched = await getProjects()
    if (fetched) projects.value = fetched
  } catch (err) {
    console.error('Failed to load projects:', err)
  }

  try {
    const fetched = await getAgents()
    if (fetched) agents.value = fetched
  } catch (err) {
    console.error('Failed to load agents:', err)
  }

  try {
    const fetched = await getRepositoryItems('mcp-servers')
    if (Array.isArray(fetched)) {
      items.value = fetched
      await initializeMatrix()
    }
  } catch (err) {
    console.error('Failed to load MCP servers:', err)
  }
}

async function syncRepo() {
  loading.value = true
  try {
    const repoUrl = await getSetting('repoUrl')
    if (!repoUrl) {
      error('Please configure a repository URL in Settings first.')
      return
    }
    const count = await apiSyncRepo(repoUrl)
    success(`Successfully synced repository! (${count} items processed)`)
    await loadData()
  } catch (err: unknown) {
    error(formatError(err, 'Failed to sync repository. Please try again.'))
  } finally {
    loading.value = false
  }
}

async function toggleSelection(itemId: string, key: number) {
  if (!selectionMatrix.value[itemId]) selectionMatrix.value[itemId] = new Set()
  const set = selectionMatrix.value[itemId]
  const wasSelected = set.has(key)
  // Optimistic update with rollback on failure.
  if (wasSelected) set.delete(key)
  else set.add(key)

  const scope = key === GLOBAL ? 'global' : 'project'
  try {
    await setMcpSelection(itemId, scope, key, !wasSelected)
  } catch (err) {
    if (wasSelected) set.add(key)
    else set.delete(key)
    error('Failed to save selection. Please try again.')
    console.error('Failed to persist MCP selection change', err)
  }
}

/**
 * Build one apply task per (server, target mcp.json). Selected cells produce
 * write tasks; unselected cells produce remove tasks so that unticking a server
 * and applying removes it from the corresponding config.
 */
function buildTasks(force: boolean): McpApplyTask[] | null {
  const tasks: McpApplyTask[] = []

  for (const item of items.value) {
    let servers: { key: string; config: string }[]
    try {
      servers = parseServers(item)
    } catch (err) {
      error(`"${item.name}" is not a valid MCP server file: ${formatError(err, 'invalid JSON')}`)
      return null
    }

    const selected = selectionMatrix.value[item.id] ?? new Set<number>()

    // Global scope → every agent that has a global MCP path configured.
    const globalSelected = selected.has(GLOBAL)
    for (const agent of agents.value) {
      if (!agent.globalMcpPath) continue
      for (const s of servers) {
        tasks.push({ targetPath: agent.globalMcpPath, serverKey: s.key, config: s.config, remove: !globalSelected, force })
      }
    }

    // Project scope → each project's enabled agents that have a project MCP path.
    for (const project of projects.value) {
      if (!project.path || project.id === null) continue
      const projectSelected = selected.has(project.id)
      for (const agentId of project.agentIds ?? []) {
        const agent = agents.value.find(a => a.id === agentId)
        if (!agent || !agent.mcpPath) continue
        for (const s of servers) {
          tasks.push({ targetPath: `${project.path}/${agent.mcpPath}`, serverKey: s.key, config: s.config, remove: !projectSelected, force })
        }
      }
    }
  }

  return tasks
}

function summarise(applied: number, adopted: number, removed: number): string {
  const parts: string[] = []
  if (applied) parts.push(`${applied} written`)
  if (adopted) parts.push(`${adopted} already present`)
  if (removed) parts.push(`${removed} removed`)
  return parts.length ? parts.join(', ') : 'no changes'
}

async function runApply(force: boolean) {
  const tasks = buildTasks(force)
  if (tasks === null) return
  if (tasks.length === 0) {
    error('No agents with MCP paths are configured. Set MCP paths on the Agents page first.')
    return
  }

  applying.value = true
  try {
    const result = await applyMcpServers(tasks)
    if (result.clashes.length > 0 && !force) {
      clashes.value = result.clashes
      isClashModalOpen.value = true
      return
    }
    success(`MCP servers applied (${summarise(result.applied, result.adopted, result.removed)}).`)
  } catch (err: unknown) {
    error(formatError(err, 'Failed to apply MCP servers.'))
  } finally {
    applying.value = false
  }
}

function applyToTargets() {
  runApply(false)
}

async function confirmOverwrite() {
  isClashModalOpen.value = false
  clashes.value = []
  await runApply(true)
}

const clashMessage = () => {
  const names = [...new Set(clashes.value.map(c => c.serverKey))].join(', ')
  return `These servers already exist with a different configuration: ${names}. Overwrite them with the repository version?`
}

onMounted(async () => {
  await loadData()
  unlistenSync = await listen('repo_synced', () => { loadData() })
})

onUnmounted(() => { unlistenSync?.() })
</script>

<template>
  <PageLayout title="MCP Servers">
    <template #actions>
      <BaseButton variant="secondary" @click="applyToTargets" :disabled="applying || loading">
        <svg v-if="!applying" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ applying ? 'Applying...' : 'Apply' }}
      </BaseButton>
      <BaseButton variant="primary" @click="syncRepo" :disabled="loading || applying">
        <svg v-if="!loading" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 21v-5h5"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ loading ? 'Syncing...' : 'Sync' }}
      </BaseButton>
    </template>

    <EmptyState
      v-if="items.length === 0 && !loading"
      glass
      message="No MCP servers loaded. Add JSON files to the repository's mcp-servers folder, then click 'Sync'."
    />

    <div v-else-if="items.length > 0" class="bg-card/70 backdrop-blur-md border border-white/10 rounded-md overflow-x-auto">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr>
            <th class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap w-[40%]">Server</th>
            <th class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">Global</th>
            <th v-for="project in projects" :key="project.id ?? project.path" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              {{ getProjectName(project.path) }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in items" :key="item.id" class="hover:bg-card-hover [&:last-child>td]:border-b-0">
            <td class="py-2 px-4 border-b border-divider">
              <div class="flex items-center gap-[0.4rem]">
                <span class="font-semibold text-base">{{ item.name }}</span>
                <BaseButton variant="ghost" icon class="shrink-0" @click="openPreview(item)" :aria-label="`Preview ${item.name}`">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                </BaseButton>
              </div>
            </td>
            <td class="py-2 px-4 border-b border-divider text-center">
              <TickBox
                :checked="isSelected(item.id, GLOBAL)"
                @change="toggleSelection(item.id, GLOBAL)"
              />
            </td>
            <td v-for="project in projects" :key="project.id ?? project.path" class="py-2 px-4 border-b border-divider text-center">
              <TickBox
                v-if="project.id !== null"
                :checked="isSelected(item.id, project.id)"
                @change="toggleSelection(item.id, project.id)"
              />
              <span v-else class="text-muted text-sm">-</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <ContentModal
      :isOpen="isModalOpen"
      :title="activeItem?.name || ''"
      :content="activeItem?.content || ''"
      @close="isModalOpen = false"
    />

    <ConfirmModal
      :isOpen="isClashModalOpen"
      title="Overwrite existing servers?"
      :message="clashMessage()"
      confirmText="Overwrite"
      :danger="true"
      @confirm="confirmOverwrite"
      @cancel="isClashModalOpen = false; clashes = []"
    />
  </PageLayout>
</template>
