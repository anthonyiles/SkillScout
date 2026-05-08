import { createRouter, createWebHistory } from 'vue-router'
import ProjectsView from './views/ProjectsView.vue'
import SkillsView from './views/SkillsView.vue'
import RulesView from './views/RulesView.vue'
import AgentsView from './views/AgentsView.vue'
import SettingsView from './views/SettingsView.vue'
import UnmanagedView from './views/UnmanagedView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'projects',
      component: ProjectsView
    },
    {
      path: '/skills',
      name: 'skills',
      component: SkillsView
    },
    {
      path: '/rules',
      name: 'rules',
      component: RulesView
    },
    {
      path: '/unmanaged',
      name: 'unmanaged',
      component: UnmanagedView
    },
    {
      path: '/agents',
      name: 'agents',
      component: AgentsView
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView
    }
  ]
})

export default router
