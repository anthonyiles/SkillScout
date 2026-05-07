<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
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

interface ProjectWithLocalItems extends Project {
  unmanaged: UnmanagedItem[]
}

const projects = ref<ProjectWithLocalItems[]>([])
const availableAgents = ref<Agent[]>([])
const knownSkills = ref<Set<string>>(new Set())
const knownRules = ref<Set<string>>(new Set())
const scanning = ref(false)
const loading = ref(true)

const { info, success, error } = useToast()

function getProjectName(path: string) {
  if (!path) return 'New Project'
  const parts = path.split(/[/\\]/).filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : 'New Project'
}

async function scanLocal() {
  scanning.value = true

  try {
    // Load agents
    const savedAgents = localStorage.getItem('agents')
    if (savedAgents) {
      try {
        availableAgents.value = JSON.parse(savedAgents)
      } catch (e) {
        console.error('Failed to parse agents from localStorage:', e)
        availableAgents.value = []
      }
    }

    // Load known items
    const savedSkills = localStorage.getItem('skills')
    if (savedSkills) {
      try {
        const parsed = JSON.parse(savedSkills)
        knownSkills.value = new Set(parsed.map((s: any) => s.name))
      } catch (e) {
        console.error('Failed to parse skills from localStorage:', e)
        knownSkills.value = new Set()
      }
    }

    const savedRules = localStorage.getItem('rules')
    if (savedRules) {
      try {
        const parsed = JSON.parse(savedRules)
        knownRules.value = new Set(parsed.map((r: any) => r.name))
      } catch (e) {
        console.error('Failed to parse rules from localStorage:', e)
        knownRules.value = new Set()
      }
    }

    // Load projects
    const savedProjects = localStorage.getItem('projects')
    if (savedProjects) {
      try {
        const rawProjects: Project[] = JSON.parse(savedProjects)

        const enriched: ProjectWithLocalItems[] = []

        for (const p of rawProjects) {
          if (!p.path || !p.agentIds) {
            enriched.push({ ...p, unmanaged: [] })
            continue
          }

          const skillPaths = new Set<string>();
          const rulePaths = new Set<string>();

          for (const agentId of p.agentIds) {
            const agent = availableAgents.value.find(a => a.id === agentId);
            if (agent) {
              if (agent.skillsPath) skillPaths.add(agent.skillsPath);
              if (agent.rulesPath) rulePaths.add(agent.rulesPath);
            }
          }

          let unmanaged: UnmanagedItem[] = []

          // Scan skills
          if (skillPaths.size > 0) {
            try {
              const foundFiles: string[] = await invoke('get_project_files', {
                projectPath: p.path,
                subFolders: Array.from(skillPaths)
              });

              unmanaged.push(...foundFiles
                .filter(f => !knownSkills.value.has(f))
                .map(f => ({ name: f, path: p.path, type: 'skill' as const })))
            } catch (e) {
              console.error(`Failed to scan skills for project ${p.path}:`, e)
              error(`Failed to scan skills in ${p.path}: ${e}`)
            }
          }

          // Scan rules
          if (rulePaths.size > 0) {
            try {
              const foundFiles: string[] = await invoke('get_project_files', {
                projectPath: p.path,
                subFolders: Array.from(rulePaths)
              });

              unmanaged.push(...foundFiles
                .filter(f => !knownRules.value.has(f))
                .map(f => ({ name: f, path: p.path, type: 'rule' as const })))
            } catch (e) {
              console.error(`Failed to scan rules for project ${p.path}:`, e)
              error(`Failed to scan rules in ${p.path}: ${e}`)
            }
          }

          enriched.push({ ...p, unmanaged })
        }

        projects.value = enriched
      } catch (e) {
        console.error('Failed to parse projects from localStorage:', e)
        projects.value = []
      }
    }
  } finally {
    scanning.value = false
    loading.value = false
  }
}

onMounted(async () => {
  await scanLocal()
})

async function handleManualScan() {
  await scanLocal()
  success('Scan complete!')
}

function promoteItem(item: UnmanagedItem, _project: ProjectWithLocalItems) {
  info(`Promoting ${item.type} "${item.name}" is not implemented yet! (Coming soon)`)
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
                <div class="type-badge" :class="item.type">
                  {{ item.type }}
                </div>
                <div class="name-container">
                  <svg v-if="item.type === 'skill'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                  <span class="item-name">{{ item.name }}</span>
                </div>
              </div>
              <BaseButton variant="secondary" size="sm" @click="promoteItem(item, project)">
                Promote
              </BaseButton>
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

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>
