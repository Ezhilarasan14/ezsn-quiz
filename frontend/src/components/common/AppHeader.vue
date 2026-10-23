<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute } from 'vue-router';
import { useBrandingStore, MAIN_BRAND_LOGO } from '@/stores/branding';
import { useAuthStore } from '@/stores/auth';
import { soundService } from '@/services/audio';
import Button from 'primevue/button';

const route = useRoute();
const brandingStore = useBrandingStore();
const authStore = useAuthStore();
const isMuted = ref(false);

const isGameRoute = computed(() => route.path === '/play' || route.path === '/host');

function toggleSound() {
  isMuted.value = soundService.toggleMute();
}
</script>

<template>
  <header
    class="border-b border-white/10 sticky top-0 z-40 transition-colors duration-300 shadow-lg"
    :style="{ backgroundColor: brandingStore.navbarColor }"
  >
    <div class="max-w-7xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between">
      <!-- Main Brand Logo (Always Permanent: EZSN) -->
      <router-link to="/" class="flex items-center space-x-3 group">
        <span class="text-xl font-black tracking-tight text-white group-hover:text-indigo-300 transition-colors">
          ezsnapps
        </span>
      </router-link>

      <!-- Center: Organization Branding (Visible during active quiz sessions) -->
      <div
        v-if="isGameRoute && (brandingStore.currentTheme.logo_url || brandingStore.currentTheme.org_name)"
        class="hidden sm:flex items-center space-x-2.5 px-3 py-1 rounded-full bg-black/40 border border-white/15 backdrop-blur-sm shadow-sm"
      >
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-5 w-auto max-w-[90px] object-contain rounded"
        />
        <div class="flex items-center space-x-1.5 text-xs">
          <span class="text-[10px] font-extrabold uppercase text-indigo-300">Host:</span>
          <span class="font-bold text-white truncate max-w-[140px]">
            {{ brandingStore.currentTheme.org_name }}
          </span>
        </div>
      </div>

      <!-- Right Navigation Actions -->
      <div class="flex items-center space-x-2 sm:space-x-4">
        <!-- Sound Toggle -->
        <Button
          :icon="isMuted ? 'pi pi-volume-off' : 'pi pi-volume-up'"
          text
          rounded
          class="text-white/80 hover:text-white"
          @click="toggleSound"
          :aria-label="isMuted ? 'Unmute' : 'Mute'"
        />

        <!-- Home / Join -->
        <router-link to="/">
          <Button label="Join" icon="pi pi-play" text size="small" class="text-white/90 hover:text-white font-medium" />
        </router-link>

        <!-- Admin / Manage -->
        <router-link to="/admin">
          <Button
            :label="authStore.isAuthenticated ? 'Dashboard' : 'Admin'"
            icon="pi pi-cog"
            size="small"
            class="bg-indigo-600 hover:bg-indigo-500 border-none font-semibold text-white shadow-md shadow-indigo-600/30"
          />
        </router-link>
      </div>
    </div>
  </header>
</template>
