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
  <div class="min-h-screen flex flex-col font-sans bg-slate-950 text-slate-100">
    <AppHeader />
    <div class="flex-1">
      <router-view />
    </div>
  </div>
</template>
