<script setup lang="ts">
import Sidebar from './components/Sidebar.vue'
import Toast from './components/Toast.vue'
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden">
    <Toast />
    <Sidebar />
    <main class="flex-1 bg-page relative">
      <router-view />
    </main>
  </div>
</template>
