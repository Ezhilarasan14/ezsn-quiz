<script setup lang="ts">
import { useBrandingStore } from '@/stores/branding';
import { useAuthStore } from '@/stores/auth';
import { soundService } from '@/services/audio';
import { ref } from 'vue';
import Button from 'primevue/button';

const brandingStore = useBrandingStore();
const authStore = useAuthStore();
const isMuted = ref(false);

function toggleSound() {
  isMuted.value = soundService.toggleMute();
}
</script>

<template>
  <header class="bg-slate-900/80 backdrop-blur-md border-b border-slate-800 sticky top-0 z-40 transition-colors">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between">
      <!-- Logo & Organization Name -->
      <router-link to="/" class="flex items-center space-x-3 group">
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-9 w-auto max-w-[140px] object-contain rounded"
        />
        <div
          v-else
          class="w-9 h-9 rounded-xl bg-gradient-to-tr from-indigo-600 to-pink-500 flex items-center justify-center font-black text-white shadow-lg shadow-indigo-500/20"
        >
          EZ
        </div>
        <span class="text-xl font-black tracking-tight text-white group-hover:text-indigo-400 transition-colors">
          {{ brandingStore.currentTheme.org_name || 'EZSN Quiz' }}
        </span>
      </router-link>

      <!-- Right Navigation Actions -->
      <div class="flex items-center space-x-2 sm:space-x-4">
        <!-- Sound Toggle -->
        <Button
          :icon="isMuted ? 'pi pi-volume-off' : 'pi pi-volume-up'"
          text
          rounded
          severity="secondary"
          @click="toggleSound"
          :aria-label="isMuted ? 'Unmute' : 'Mute'"
        />

        <!-- Home / Join -->
        <router-link to="/">
          <Button label="Join" icon="pi pi-play" text size="small" class="text-slate-300" />
        </router-link>

        <!-- Admin / Manage -->
        <router-link to="/admin">
          <Button
            :label="authStore.isAuthenticated ? 'Dashboard' : 'Admin'"
            icon="pi pi-cog"
            size="small"
            class="bg-indigo-600 hover:bg-indigo-500 border-none font-semibold text-white shadow-md shadow-indigo-600/20"
          />
        </router-link>
      </div>
    </div>
  </header>
</template>
