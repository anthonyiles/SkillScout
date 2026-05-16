<script setup lang="ts">
import { ref } from 'vue'
import { useItemsMatrix, type RepositoryItem } from '../composables/useItemsMatrix'
import ContentModal from '../components/ContentModal.vue'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import EmptyState from '../components/EmptyState.vue'

const {
  items: rules,
  projects,
  loading,
  applying,
  isSelected,
  getProjectName,
  syncRepo,
  toggleSelection,
  applyToProjects,
} = useItemsMatrix('rules')

const activeRule = ref<RepositoryItem | null>(null)
const isModalOpen = ref(false)

function openPreview(rule: RepositoryItem) {
  activeRule.value = rule
  isModalOpen.value = true
}
</script>

<template>
  <PageLayout title="Rules">
    <template #actions>
      <BaseButton variant="secondary" @click="applyToProjects" :disabled="applying || loading">
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
      v-if="rules.length === 0 && !loading"
      glass
      message="No rules loaded. Click 'Sync' to fetch rules from GitHub."
    />

    <div v-else-if="rules.length > 0" class="bg-card/70 backdrop-blur-md border border-white/10 rounded-md overflow-x-auto">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr>
            <th class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap w-[40%]">Rule</th>
            <th v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              {{ getProjectName(project.path) }}
            </th>
            <th v-if="projects.length === 0" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              No projects configured
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="rule in rules" :key="rule.id" class="hover:bg-card-hover [&:last-child>td]:border-b-0">
            <td class="py-2 px-4 border-b border-divider">
              <div class="flex items-center gap-[0.4rem]">
                <span class="font-semibold text-base">{{ rule.name }}</span>
                <button
                  type="button"
                  class="bg-transparent border-0 p-0 cursor-pointer text-muted flex items-center transition-colors shrink-0 hover:text-accent"
                  @click="openPreview(rule)"
                  :aria-label="`Preview ${rule.name}`"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                </button>
              </div>
            </td>
            <td v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider text-center">
              <TickBox
                :checked="isSelected(rule.id, project.id)"
                @change="toggleSelection(rule.id, project.id)"
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
      :title="activeRule?.name || ''"
      :content="activeRule?.content || ''"
      @close="isModalOpen = false"
    />
  </PageLayout>
</template>
