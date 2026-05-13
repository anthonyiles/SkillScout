import { ref } from 'vue'
import { createSharedComposable } from '@vueuse/core'
import type { Project } from '../types'

export const useProjects = createSharedComposable(() => {
  const projects = ref<Project[]>([])

  function load() {
    const raw = localStorage.getItem('projects')
    if (!raw) return
    try {
      const parsed = JSON.parse(raw)
      projects.value = parsed.map((p: any) => ({ ...p, agentIds: p.agentIds ?? [] }))
    } catch {
      projects.value = []
    }
  }

  function save() {
    localStorage.setItem('projects', JSON.stringify(projects.value))
  }

  function add() {
    projects.value.push({ id: crypto.randomUUID(), path: '', agentIds: [] })
  }

  function remove(id: string) {
    projects.value = projects.value.filter(p => p.id !== id)
  }

  return { projects, load, save, add, remove }
})
