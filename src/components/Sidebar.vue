<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useToast } from '../composables/useToast'
import { checkGithubAuth, logoutGithub } from '../api'
import logoUrl from '../assets/logo.png'
import GitHubLoginModal from './GitHubLoginModal.vue'
import ConfirmModal from './ConfirmModal.vue'

const route = useRoute()
const { error } = useToast()
const isGitHubModalOpen = ref(false)
const isDisconnectModalOpen = ref(false)
const isAuthenticated = ref(false)

onMounted(async () => {
  try {
    isAuthenticated.value = await checkGithubAuth()
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
    await logoutGithub()
    isAuthenticated.value = false
    isDisconnectModalOpen.value = false
  } catch (e: any) {
    error(typeof e === 'string' ? e : e.message ?? 'Failed to disconnect GitHub account.')
    console.error('Failed to log out:', e)
  }
}

const base = 'flex items-center gap-3 px-3 py-[0.6rem] rounded-md text-[0.95rem] font-medium transition-all'
function navLinkClass(path: string) {
  return route.path === path
    ? `${base} bg-accent text-white shadow-[0_4px_12px_rgba(0,0,0,0.2)]`
    : `${base} text-muted hover:bg-card-hover hover:text-white`
}
</script>

<template>
  <nav class="bg-card/70 backdrop-blur-md w-[220px] h-screen flex flex-col p-4 border-r border-divider z-10">
    <div class="flex items-center justify-center gap-2 mb-8">
      <img :src="logoUrl" alt="SkillScout Logo" class="w-11 h-11 object-contain" />
      <h2 class="text-[1.4rem] font-bold text-accent tracking-[-0.5px]">SkillScout</h2>
    </div>

    <div class="flex flex-col gap-2">
      <router-link to="/" :class="navLinkClass('/')">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-80"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        Projects
      </router-link>

      <router-link to="/skills" :class="navLinkClass('/skills')">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-80"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
        Skills
      </router-link>

      <router-link to="/rules" :class="navLinkClass('/rules')">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-80"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
        Rules
      </router-link>

      <router-link to="/unmanaged" :class="navLinkClass('/unmanaged')">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-80"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        Unmanaged
      </router-link>

      <router-link to="/agents" :class="navLinkClass('/agents')">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-80"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
        Agents
      </router-link>

      <router-link to="/settings" :class="navLinkClass('/settings')">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-80"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
        Settings
      </router-link>
    </div>

    <div class="mt-auto">
      <button
        v-if="!isAuthenticated"
        class="flex items-center justify-center gap-2 w-full py-3 bg-card-hover border border-divider rounded-md text-white text-[0.9rem] font-medium cursor-pointer transition-all hover:bg-page hover:border-accent"
        @click="isGitHubModalOpen = true"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>
        Connect GitHub
      </button>
      <button
        v-else
        class="group relative w-full py-3 bg-[rgba(74,222,128,0.1)] border border-[rgba(74,222,128,0.2)] rounded-md text-[#4ade80] text-[0.9rem] font-medium cursor-pointer overflow-hidden transition-all hover:border-[rgba(255,85,85,0.3)] hover:bg-transparent"
        @click="isDisconnectModalOpen = true"
        title="Disconnect GitHub"
      >
        <div class="flex items-center justify-center gap-2 transition-opacity group-hover:opacity-0">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#4ade80" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
          <span>GitHub Connected</span>
        </div>
        <div class="absolute inset-0 flex items-center justify-center bg-[rgba(255,85,85,0.15)] text-[#ff5555] font-semibold opacity-0 transition-opacity group-hover:opacity-100">
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
