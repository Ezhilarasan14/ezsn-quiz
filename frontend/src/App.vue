<script setup lang="ts">
import { onMounted } from 'vue';
import { useBrandingStore } from '@/stores/branding';
import { useAuthStore } from '@/stores/auth';
import AppHeader from '@/components/common/AppHeader.vue';

const brandingStore = useBrandingStore();
const authStore = useAuthStore();

onMounted(async () => {
  await Promise.all([
    brandingStore.fetchOrgConfig(),
    authStore.checkAuth(),
  ]);
});
</script>

<template>
  <div class="min-h-screen flex flex-col font-sans bg-[#0a0416] text-purple-100 relative selection:bg-purple-500/40 selection:text-white">
    <!-- Ambient Glass Lighting Mesh in Deep Purple & Colorful Gradients -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none z-0">
      <div class="absolute -top-40 -left-40 w-[650px] h-[650px] rounded-full bg-purple-600/25 blur-[140px] animate-glow" />
      <div class="absolute top-1/4 -right-40 w-[600px] h-[600px] rounded-full bg-fuchsia-600/20 blur-[150px] animate-glow" style="animation-delay: -3s;" />
      <div class="absolute -bottom-40 left-1/4 w-[700px] h-[700px] rounded-full bg-indigo-600/20 blur-[160px]" />
    </div>

    <!-- Main App Navigation Bar -->
    <AppHeader class="relative z-30" />

    <!-- View Content Area -->
    <div class="flex-1 relative z-10">
      <router-view />
    </div>
  </div>
</template>
