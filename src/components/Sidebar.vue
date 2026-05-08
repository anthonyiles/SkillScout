<script setup lang="ts">
import { useRoute } from 'vue-router'
import logoUrl from '../assets/logo.png'

const route = useRoute()
</script>

<template>
  <nav class="sidebar glass">
    <div class="logo">
      <img :src="logoUrl" alt="SkillsKit Logo" class="logo-img" />
      <h2>SkillsKit</h2>
    </div>

    <div class="links">
      <router-link to="/" class="nav-link" :class="{ active: route.path === '/' }">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        Projects
      </router-link>

      <router-link to="/skills" class="nav-link" :class="{ active: route.path === '/skills' }">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
        Skills
      </router-link>

      <router-link to="/rules" class="nav-link" :class="{ active: route.path === '/rules' }">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
        Rules
      </router-link>

      <router-link to="/unmanaged" class="nav-link" :class="{ active: route.path === '/unmanaged' }">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        Unmanaged
      </router-link>

      <router-link to="/agents" class="nav-link" :class="{ active: route.path === '/agents' }">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
        Agents
      </router-link>

      <router-link to="/settings" class="nav-link" :class="{ active: route.path === '/settings' }">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
        Settings
      </router-link>
    </div>
  </nav>
</template>

<style scoped>
.sidebar {
  width: 220px;
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 1rem;
  border-right: 1px solid var(--border-color);
  z-index: 10;
}

.logo {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 2rem;
  padding: 0 0.5rem;
}

.logo-img {
  width: 48px;
  height: 48px;
  object-fit: contain;
}

.logo h2 {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--accent-primary);
  letter-spacing: -0.5px;
}

.links {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.6rem 0.75rem;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 0.95rem;
  font-weight: 500;
  transition: all var(--transition-fast);
}

.nav-link:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.nav-link.active {
  background: var(--accent-primary);
  color: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.nav-link svg {
  width: 18px;
  height: 18px;
  opacity: 0.8;
}

.nav-link.active svg {
  opacity: 1;
}
</style>
