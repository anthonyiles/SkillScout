import { ref } from 'vue'
import { createSharedComposable } from '@vueuse/core'
import type { Agent } from '../types'

const DEFAULT_AGENTS: Agent[] = [
  { id: 'cursor',    name: 'Cursor',       skillsPath: '.cursor/skills',  rulesPath: '.cursor/rules' },
  { id: 'jetbrains', name: 'JetBrains AI', skillsPath: '.agents/skills',  rulesPath: '.agents/rules' },
  { id: 'claude',    name: 'Claude Code',  skillsPath: '.claude/skills',  rulesPath: '.claude/rules' },
]

export const useAgents = createSharedComposable(() => {
  const agents = ref<Agent[]>([])

  function load() {
    const raw = localStorage.getItem('agents')
    if (!raw) {
      agents.value = [...DEFAULT_AGENTS]
      return
    }
    try {
      const parsed = JSON.parse(raw)
      if (Array.isArray(parsed) && parsed.every(a => a.id && a.name && a.skillsPath !== undefined && a.rulesPath !== undefined)) {
        agents.value = parsed
      } else {
        agents.value = [...DEFAULT_AGENTS]
      }
    } catch {
      agents.value = [...DEFAULT_AGENTS]
    }
  }

  function save() {
    localStorage.setItem('agents', JSON.stringify(agents.value))
  }

  function reset() {
    agents.value = [...DEFAULT_AGENTS]
    save()
  }

  return { agents, defaultAgents: DEFAULT_AGENTS, load, save, reset }
})
