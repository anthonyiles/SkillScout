<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useToast } from '../composables/useToast'
import { formatError } from '../utils/formatError'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import CardItem from '../components/CardItem.vue'
import EmptyState from '../components/EmptyState.vue'
import ContentModal from '../components/ContentModal.vue'

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
  content: string
  subFolder: string
}

interface ModifiedItem {
  name: string
  path: string
  type: 'skill' | 'rule'
  repositoryItemId: string
  subFolder: string
  content: string
}

interface FileHash {
  name: string
  sha: string
  folder: string
  content: string
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
const knownSkills = ref<Map<string, { id: string; sha: string }>>(new Map())
const knownRules = ref<Map<string, { id: string; sha: string }>>(new Map())
const scanning = ref(false)
const loading = ref(true)

const promotedItems = ref<Record<string, PromotedItem>>({})
const checkingPrs = ref<Set<string>>(new Set())
const mergedPrs = ref<Set<string>>(new Set())

const { success, error } = useToast()

function getProjectName(path: string) {
  if (!path) return 'New Project'
  const parts = path.split(/[/\\]/).filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : 'New Project'
}

async function loadRepositoryIndex() {
  const agents = await invoke<Agent[]>('get_agents')
  if (agents) availableAgents.value = agents

  const skills = await invoke<any[]>('get_repository_items', { folder: 'skills' })
  if (skills) knownSkills.value = new Map(skills.map((skill: any) => [skill.name, { id: skill.id, sha: skill.sha }]))

  const rules = await invoke<any[]>('get_repository_items', { folder: 'rules' })
  if (rules) knownRules.value = new Map(rules.map((rule: any) => [rule.name, { id: rule.id, sha: rule.sha }]))
}

function collectAgentPaths(project: Project): { skillPaths: Set<string>; rulePaths: Set<string> } {
  const skillPaths = new Set<string>()
  const rulePaths = new Set<string>()
  for (const agentId of project.agentIds) {
    const agent = availableAgents.value.find(agent => agent.id === agentId)
    if (agent?.skillsPath) skillPaths.add(agent.skillsPath)
    if (agent?.rulesPath) rulePaths.add(agent.rulesPath)
  }
  return { skillPaths, rulePaths }
}

function classifyHashes(
  hashes: FileHash[],
  knownItems: Map<string, { id: string; sha: string }>,
  itemType: 'skill' | 'rule',
  projectPath: string,
): { unmanaged: UnmanagedItem[]; modified: ModifiedItem[] } {
  const unmanaged: UnmanagedItem[] = []
  const modified: ModifiedItem[] = []
  for (const fileHash of hashes) {
    const known = knownItems.get(fileHash.name)
    if (!known) {
      unmanaged.push({ name: fileHash.name, path: projectPath, type: itemType, content: fileHash.content, subFolder: fileHash.folder })
    } else if (known.sha !== fileHash.sha) {
      modified.push({ name: fileHash.name, path: projectPath, type: itemType, repositoryItemId: known.id, subFolder: fileHash.folder, content: fileHash.content })
    }
  }
  return { unmanaged, modified }
}

async function enrichProjectWithLocalItems(project: Project): Promise<ProjectWithLocalItems> {
  if (!project.path || !project.agentIds?.length) {
    return { ...project, unmanaged: [], modified: [] }
  }

  const { skillPaths, rulePaths } = collectAgentPaths(project)
  const unmanaged: UnmanagedItem[] = []
  const modified: ModifiedItem[] = []

  if (skillPaths.size > 0) {
    try {
      const hashes: FileHash[] = await invoke('get_project_file_hashes', { projectPath: project.path, subFolders: Array.from(skillPaths) })
      const classified = classifyHashes(hashes, knownSkills.value, 'skill', project.path)
      unmanaged.push(...classified.unmanaged)
      modified.push(...classified.modified)
    } catch (err) { console.error(`Failed to scan skills in ${project.path}:`, err) }
  }

  if (rulePaths.size > 0) {
    try {
      const hashes: FileHash[] = await invoke('get_project_file_hashes', { projectPath: project.path, subFolders: Array.from(rulePaths) })
      const classified = classifyHashes(hashes, knownRules.value, 'rule', project.path)
      unmanaged.push(...classified.unmanaged)
      modified.push(...classified.modified)
    } catch (err) { console.error(`Failed to scan rules in ${project.path}:`, err) }
  }

  return { ...project, unmanaged, modified }
}

async function scanLocal() {
  scanning.value = true
  try {
    await loadRepositoryIndex()
    const rawProjects = await invoke<Project[]>('get_projects')
    if (rawProjects) {
      projects.value = await Promise.all(rawProjects.map(enrichProjectWithLocalItems))
    }
  } catch (err) {
    console.error('Failed to load data for scan:', err)
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
      for (const item of promoted) pmap[`${item.path}-${item.name}`] = item
      promotedItems.value = pmap
    }
  } catch (err) { console.error('Failed to load promoted items:', err) }

  await scanLocal()

  let syncNeeded = false
  await Promise.all(Object.values(promotedItems.value).map(async (item) => {
    const key = `${item.path}-${item.name}`
    checkingPrs.value.add(key)
    try {
      const res = await invoke<any>('check_pr_status', { prUrl: item.url })
      if (res.state !== 'open') {
        if (item.id) await invoke('remove_promoted_item', { id: item.id })
        delete promotedItems.value[key]
        if (res.merged) { mergedPrs.value.add(key); syncNeeded = true }
      }
    } catch {
      // PR check failed — leave item in pending state
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
      } catch (err) { console.error('Failed background sync:', err) }
    }
  }
})

async function handleManualScan() {
  await scanLocal()
  success('Scan complete!')
}

const promotingItem = ref<string | null>(null)
const activePreview = ref<{ title: string; content: string } | null>(null)

function openPreview(item: { name: string; content: string }) {
  activePreview.value = { title: item.name, content: item.content }
}

async function promoteItem(item: UnmanagedItem, project: ProjectWithLocalItems) {
  const repoUrl = await invoke<string | null>('get_setting', { key: 'repoUrl' })
  if (!repoUrl) { error('Please configure a repository URL in Settings first.'); return }
  const key = `${project.path}-${item.subFolder}-${item.name}`
  promotingItem.value = key
  try {
    const result = await invoke<{ url: string; branch: string }>('promote_item', { repoUrl, itemType: item.type, itemName: item.name, projectPath: project.path, subFolders: [item.subFolder] })
    const pItem: PromotedItem = { name: item.name, path: project.path, itemType: item.type, url: result.url, branch: result.branch }
    const saved = await invoke<PromotedItem>('add_promoted_item', { item: pItem })
    success(`Successfully created PR!`)
    promotedItems.value[key] = saved
  } catch (err: unknown) {
    error(formatError(err, 'Failed to promote item. Please try again.'))
  } finally {
    promotingItem.value = null
  }
}

async function promoteUpdate(item: ModifiedItem, project: ProjectWithLocalItems) {
  const repoUrl = await invoke<string | null>('get_setting', { key: 'repoUrl' })
  if (!repoUrl) { error('Please configure a repository URL in Settings first.'); return }
  const key = `${project.path}-${item.subFolder}-${item.name}`
  promotingItem.value = key
  try {
    const result = await invoke<{ url: string; branch: string }>('promote_item', { repoUrl, itemType: item.type, itemName: item.name, projectPath: project.path, subFolders: [item.subFolder], updateMode: true })
    const pItem: PromotedItem = { name: item.name, path: project.path, itemType: item.type, repository_item_id: item.repositoryItemId, url: result.url, branch: result.branch }
    const saved = await invoke<PromotedItem>('add_promoted_item', { item: pItem })
    success(`Successfully created update PR!`)
    promotedItems.value[key] = saved
  } catch (err: unknown) {
    error(formatError(err, 'Failed to create update PR. Please try again.'))
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
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ scanning ? 'Scanning...' : 'Scan files' }}
      </BaseButton>
    </template>

    <EmptyState v-if="loading" message="Loading unmanaged items..." />

    <div v-else class="flex flex-col gap-4">

      <!-- Unmanaged items (no repo match) -->
      <template v-for="project in projects" :key="`unmanaged-${project.id}`">
        <CardItem
          v-if="project.unmanaged.length > 0"
          glass
          :title="getProjectName(project.path)"
          :subtitle="`${project.unmanaged.length} item${project.unmanaged.length === 1 ? '' : 's'} to promote`"
        >
          <div class="flex flex-col gap-2">
            <div
              v-for="item in project.unmanaged"
              :key="`${item.type}-${item.subFolder}-${item.name}`"
              class="flex justify-between items-center p-3 bg-black/15 rounded-sm border border-divider"
            >
              <div class="flex items-center gap-4">
                <!-- type badge -->
                <span
                  class="text-[0.7rem] font-bold uppercase py-[0.2rem] px-2 rounded-[4px] min-w-[60px] text-center border"
                  :class="item.type === 'skill'
                    ? 'bg-[rgba(52,152,219,0.2)] text-[#3498db] border-[rgba(52,152,219,0.3)]'
                    : 'bg-[rgba(155,89,182,0.2)] text-[#9b59b2] border-[rgba(155,89,182,0.3)]'"
                >{{ item.type }}</span>
                <div class="flex items-center gap-2">
                  <svg v-if="item.type === 'skill'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                  <div class="flex flex-col gap-[0.1rem]">
                    <div class="flex items-center gap-[0.4rem]">
                      <span class="font-mono text-[0.9rem]">{{ item.name }}</span>
                      <button type="button" class="bg-transparent border-0 p-0 cursor-pointer text-muted flex items-center transition-colors shrink-0 hover:text-accent" @click="openPreview(item)" :aria-label="`Preview ${item.name}`">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                      </button>
                    </div>
                    <span class="font-mono text-[0.7rem] text-muted opacity-70">{{ item.subFolder }}</span>
                  </div>
                </div>
              </div>
              <!-- action buttons -->
              <template v-if="checkingPrs.has(`${project.path}-${item.subFolder}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled>
                  <span class="inline-block w-3 h-3 rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
                </BaseButton>
              </template>
              <template v-else-if="mergedPrs.has(`${project.path}-${item.subFolder}-${item.name}`)">
                <BaseButton variant="secondary" size="sm" disabled class="!text-[#4ade80] !border-[rgba(74,222,128,0.3)] !bg-[rgba(74,222,128,0.1)] !opacity-100">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#4ade80" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                  Merged!
                </BaseButton>
              </template>
              <template v-else-if="promotedItems[`${project.path}-${item.subFolder}-${item.name}`]">
                <BaseButton variant="primary" size="sm" @click="openUrl(promotedItems[`${project.path}-${item.subFolder}-${item.name}`].url).catch(() => error('Failed to open PR URL'))">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
                  View PR
                </BaseButton>
              </template>
              <template v-else>
                <BaseButton variant="secondary" size="sm" :disabled="promotingItem === `${project.path}-${item.subFolder}-${item.name}`" @click="promoteItem(item, project)">
                  <span v-if="promotingItem === `${project.path}-${item.subFolder}-${item.name}`" class="inline-block w-3 h-3 rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
                  <span v-else>Promote</span>
                </BaseButton>
              </template>
            </div>
          </div>
        </CardItem>
      </template>

      <!-- Modified managed items -->
      <template v-for="project in projects" :key="`modified-${project.id}`">
        <CardItem
          v-if="project.modified.length > 0"
          glass
          :title="getProjectName(project.path)"
          :subtitle="`${project.modified.length} locally modified item${project.modified.length === 1 ? '' : 's'}`"
        >
          <template #actions>
            <span class="text-[0.65rem] font-bold uppercase py-[0.15rem] px-[0.4rem] rounded-[4px] bg-[rgba(245,158,11,0.15)] text-[#f59e0b] border border-[rgba(245,158,11,0.25)]">Modified</span>
          </template>
          <div class="flex flex-col gap-2">
            <div
              v-for="item in project.modified"
              :key="`${item.type}-${item.subFolder}-${item.name}`"
              class="flex justify-between items-center p-3 bg-black/15 rounded-sm border border-divider"
            >
              <div class="flex items-center gap-4">
                <!-- modified badge -->
                <span class="text-[0.7rem] font-bold uppercase py-[0.2rem] px-2 rounded-[4px] min-w-[60px] text-center border bg-[rgba(245,158,11,0.2)] text-[#f59e0b] border-[rgba(245,158,11,0.3)]">modified</span>
                <div class="flex items-center gap-2">
                  <svg v-if="item.type === 'skill'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                  <div class="flex flex-col gap-[0.1rem]">
                    <div class="flex items-center gap-[0.4rem]">
                      <span class="font-mono text-[0.9rem]">{{ item.name }}</span>
                      <button type="button" class="bg-transparent border-0 p-0 cursor-pointer text-muted flex items-center transition-colors shrink-0 hover:text-accent" @click="openPreview(item)" :aria-label="`Preview ${item.name}`">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                      </button>
                    </div>
                    <span class="font-mono text-[0.7rem] text-muted opacity-70">{{ item.subFolder }}</span>
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-[0.4rem]">
                <template v-if="checkingPrs.has(`${project.path}-${item.subFolder}-${item.name}`)">
                  <BaseButton variant="secondary" size="sm" disabled>
                    <span class="inline-block w-3 h-3 rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
                  </BaseButton>
                </template>
                <template v-else-if="mergedPrs.has(`${project.path}-${item.subFolder}-${item.name}`)">
                  <BaseButton variant="secondary" size="sm" disabled class="!text-[#4ade80] !border-[rgba(74,222,128,0.3)] !bg-[rgba(74,222,128,0.1)] !opacity-100">
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#4ade80" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                    Merged!
                  </BaseButton>
                </template>
                <template v-else-if="promotedItems[`${project.path}-${item.subFolder}-${item.name}`]">
                  <BaseButton variant="primary" size="sm" @click="openUrl(promotedItems[`${project.path}-${item.subFolder}-${item.name}`].url).catch(() => error('Failed to open PR URL'))">
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
                    View PR
                  </BaseButton>
                </template>
                <template v-else>
                  <BaseButton variant="secondary" size="sm" :disabled="promotingItem === `${project.path}-${item.subFolder}-${item.name}`" @click="promoteUpdate(item, project)">
                    <span v-if="promotingItem === `${project.path}-${item.subFolder}-${item.name}`" class="inline-block w-3 h-3 rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
                    <span v-else>Promote update</span>
                  </BaseButton>
                </template>
              </div>
            </div>
          </div>
        </CardItem>
      </template>

      <EmptyState
        v-if="projects.filter(project => project.unmanaged.length > 0 || project.modified.length > 0).length === 0"
        glass
        message="No unmanaged or locally modified items found. All your local files are synced with the central repository!"
      />
    </div>
  </PageLayout>

  <ContentModal
    :is-open="activePreview !== null"
    :title="activePreview?.title ?? ''"
    :content="activePreview?.content ?? ''"
    @close="activePreview = null"
  />
</template>
