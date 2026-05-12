<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import logoUrl from '../assets/logo.png'
import GitHubLoginModal from './GitHubLoginModal.vue'
import ConfirmModal from './ConfirmModal.vue'

const route = useRoute()
const isGitHubModalOpen = ref(false)
const isDisconnectModalOpen = ref(false)
const isAuthenticated = ref(false)

onMounted(async () => {
  try {
    isAuthenticated.value = await invoke('check_github_auth')
  } catch (e) {
    console.error('Failed to verify GitHub auth:', e)
    isAuthenticated.value = false
  }
})

function handleAuthenticated() {
  isAuthenticated.value = true
}

async function disconnectGitHub() {
  try {
    await invoke('logout_github')
    isAuthenticated.value = false
    isDisconnectModalOpen.value = false
  } catch (e) {
    console.error('Failed to log out:', e)
  }
}
</script>

<template>
  <nav class="sidebar glass">
    <div class="logo">
      <img :src="logoUrl" alt="SkillScout Logo" class="logo-img" />
      <h2>SkillScout</h2>
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

    <div class="mt-auto">
      <button 
        v-if="!isAuthenticated" 
        class="github-btn" 
        @click="isGitHubModalOpen = true"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>
        Connect GitHub
      </button>
      <button v-else class="github-connected" @click="isDisconnectModalOpen = true" title="Disconnect GitHub">
        <div class="connected-content">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#4ade80" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
          <span>GitHub Connected</span>
        </div>
        <div class="disconnect-hint">
          Disconnect
        </div>
      </button>
    </div>
    
    <GitHubLoginModal 
      :is-open="isGitHubModalOpen" 
      @close="isGitHubModalOpen = false"
      @authenticated="handleAuthenticated"
    />

    <ConfirmModal 
      :isOpen="isDisconnectModalOpen" 
      title="Disconnect GitHub" 
      message="Are you sure you want to disconnect your GitHub account?" 
      confirmText="Disconnect" 
      danger
      @confirm="disconnectGitHub" 
      @cancel="isDisconnectModalOpen = false" 
    />
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
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 2rem;
  padding: 0;
}

.logo-img {
  width: 44px;
  height: 44px;
  object-fit: contain;
}

.logo h2 {
  font-size: 1.4rem;
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

.mt-auto {
  margin-top: auto;
}

.github-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.75rem;
  background: var(--bg-surface-hover);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.github-btn:hover {
  background: var(--bg-base);
  border-color: var(--accent-primary);
}

.github-connected {
  position: relative;
  width: 100%;
  padding: 0.75rem;
  background: rgba(74, 222, 128, 0.1);
  border: 1px solid rgba(74, 222, 128, 0.2);
  border-radius: var(--radius-md);
  color: #4ade80;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  overflow: hidden;
  transition: all var(--transition-fast);
}

.connected-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  transition: opacity var(--transition-fast);
}

.disconnect-hint {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 85, 85, 0.15);
  color: #ff5555;
  font-weight: 600;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.github-connected:hover {
  border-color: rgba(255, 85, 85, 0.3);
  background: transparent;
}

.github-connected:hover .connected-content {
  opacity: 0;
}

.github-connected:hover .disconnect-hint {
  opacity: 1;
}
</style>
