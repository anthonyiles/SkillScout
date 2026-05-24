<script setup lang="ts">
import { ref } from 'vue'
import { useItemsMatrix, type RepositoryItem } from '../composables/useItemsMatrix'
import ContentModal from '../components/ContentModal.vue'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import EmptyState from '../components/EmptyState.vue'

const {
  items: skills,
  projects,
  loading,
  applying,
  scanning,
  isSelected,
  getProjectName,
  syncRepo,
  toggleSelection,
  scanLocal,
  applyToProjects,
} = useItemsMatrix('skills')

const activeSkill = ref<RepositoryItem | null>(null)
const isModalOpen = ref(false)

function openPreview(skill: RepositoryItem) {
  activeSkill.value = skill
  isModalOpen.value = true
}
</script>

<template>
  <PageLayout title="Skills">
    <template #actions>
      <BaseButton variant="secondary" @click="applyToProjects" :disabled="applying || loading || scanning">
        <svg v-if="!applying" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ applying ? 'Applying...' : 'Apply' }}
      </BaseButton>
      <BaseButton variant="secondary" @click="scanLocal(false)" :disabled="loading || applying || scanning">
        <svg v-if="!scanning" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12h3"/><path d="M19 12h3"/><path d="M12 2v3"/><path d="M12 19v3"/><path d="m4.93 4.93 2.12 2.12"/><path d="m16.95 16.95 2.12 2.12"/><path d="m4.93 19.07 2.12-2.12"/><path d="m16.95 7.05 2.12-2.12"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ scanning ? 'Scanning...' : 'Scan files' }}
      </BaseButton>
      <BaseButton variant="primary" @click="syncRepo" :disabled="loading || applying || scanning">
        <svg v-if="!loading" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 21v-5h5"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ loading ? 'Syncing...' : 'Sync' }}
      </BaseButton>
    </template>

    <EmptyState
      v-if="skills.length === 0 && !loading"
      glass
      message="No skills loaded. Click 'Sync' to fetch skills from GitHub."
    />

    <div v-else-if="skills.length > 0" class="bg-card/70 backdrop-blur-md border border-white/10 rounded-md overflow-x-auto">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr>
            <th class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap w-[40%]">Skill</th>
            <th v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              {{ getProjectName(project.path) }}
            </th>
            <th v-if="projects.length === 0" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              No projects configured
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="skill in skills" :key="skill.id" class="hover:bg-card-hover [&:last-child>td]:border-b-0">
            <td class="py-2 px-4 border-b border-divider">
              <div class="flex items-center gap-[0.4rem]">
                <span class="font-semibold text-base">{{ skill.name }}</span>
                <BaseButton variant="ghost" icon class="shrink-0" @click="openPreview(skill)" :aria-label="`Preview ${skill.name}`">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                </BaseButton>
              </div>
            </td>
            <td v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider text-center">
              <TickBox
                :checked="isSelected(skill.id, project.id)"
                @change="toggleSelection(skill.id, project.id)"
              />
            </td>
            <td v-if="projects.length === 0" class="py-2 px-4 border-b border-divider text-center text-muted text-sm">
              -
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <ContentModal
      :isOpen="isModalOpen"
      :title="activeSkill?.name || ''"
      :content="activeSkill?.content || ''"
      @close="isModalOpen = false"
    />
  </PageLayout>
</template>
