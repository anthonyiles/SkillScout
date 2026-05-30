<script setup lang="ts">
import { onMounted } from 'vue'
import Sidebar from './components/Sidebar.vue'
import Toast from './components/Toast.vue'
import { useUpdater } from './composables/useUpdater'

const { checkForUpdate } = useUpdater()
onMounted(() => { checkForUpdate() })
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden antialiased">
    <Toast />
    <Sidebar />
    <main class="flex-1 bg-page relative">
      <router-view />
    </main>
  </div>
</template>
