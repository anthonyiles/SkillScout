<script setup lang="ts">
import Sidebar from './components/Sidebar.vue'
import Toast from './components/Toast.vue'
</script>

<template>
  <div class="app-layout">
    <Toast />
    <Sidebar />
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<style>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.main-content {
  flex: 1;
  background: var(--bg-base);
  position: relative;
}
</style>