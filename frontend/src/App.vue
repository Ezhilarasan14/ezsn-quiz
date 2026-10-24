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
  <div class="min-h-screen flex flex-col font-sans bg-[#070a13] text-slate-100 relative selection:bg-indigo-500/30 selection:text-white">
    <!-- Ambient Glass Lighting Mesh (Behind everything) -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none z-0">
      <div class="absolute -top-40 -left-40 w-[600px] h-[600px] rounded-full bg-indigo-600/15 blur-[130px] animate-glow" />
      <div class="absolute top-1/3 -right-40 w-[550px] h-[550px] rounded-full bg-pink-600/10 blur-[140px] animate-glow" style="animation-delay: -3s;" />
      <div class="absolute -bottom-40 left-1/4 w-[600px] h-[600px] rounded-full bg-cyan-600/10 blur-[150px]" />
    </div>

    <!-- Main App Navigation Bar -->
    <AppHeader class="relative z-30" />

    <!-- View Content Area -->
    <div class="flex-1 relative z-10">
      <router-view />
    </div>
  </div>
</template>
