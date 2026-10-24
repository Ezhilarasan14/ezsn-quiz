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
    class="border-b border-white/10 sticky top-0 z-40 transition-colors duration-300 backdrop-blur-xl shadow-xl shadow-black/20"
    :style="{ backgroundColor: brandingStore.navbarColor }"
  >
    <div class="max-w-7xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between gap-2">
      <!-- Main Brand Logo (Always Permanent: ezsnapps) -->
      <router-link to="/" class="flex items-center space-x-2.5 group shrink-0">
        <img
          :src="MAIN_BRAND_LOGO"
          alt="ezsnapps"
          class="h-7 sm:h-8 w-auto object-contain transition-transform group-hover:scale-105"
        />
        <span class="text-base sm:text-xl font-black tracking-tight text-white group-hover:text-indigo-300 transition-colors">
          ezsnapps<span class="text-indigo-400">.quiz</span>
        </span>
      </router-link>

      <!-- Center: Organization Branding (Visible during active quiz sessions) -->
      <div
        v-if="isGameRoute && (brandingStore.currentTheme.logo_url || brandingStore.currentTheme.org_name)"
        class="flex items-center space-x-2 px-2.5 sm:px-3.5 py-1 rounded-full bg-black/40 border border-white/15 backdrop-blur-md shadow-inner max-w-[170px] sm:max-w-[280px]"
      >
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-5 sm:h-6 w-auto max-w-[50px] sm:max-w-[90px] object-contain rounded bg-white/10 p-0.5 shrink-0"
        />
        <div class="flex items-center space-x-1 min-w-0">
          <span class="hidden sm:inline text-[10px] font-extrabold uppercase tracking-wider text-indigo-300">Host:</span>
          <span class="font-bold text-white text-xs sm:text-sm truncate">
            {{ brandingStore.currentTheme.org_name }}
          </span>
        </div>
      </div>

      <!-- Right Navigation Actions -->
      <div class="flex items-center space-x-1.5 sm:space-x-3">
        <!-- Sound Toggle -->
        <Button
          :icon="isMuted ? 'pi pi-volume-off' : 'pi pi-volume-up'"
          text
          rounded
          class="text-white/80 hover:text-white hover:bg-white/10 w-9 h-9"
          @click="toggleSound"
          :aria-label="isMuted ? 'Unmute' : 'Mute'"
          :title="isMuted ? 'Unmute Audio' : 'Mute Audio'"
        />

        <!-- Join Screen Link -->
        <router-link to="/">
          <Button
            label="Join"
            icon="pi pi-play"
            text
            size="small"
            class="text-slate-200 hover:text-white hover:bg-white/10 font-bold px-3 py-1.5"
          />
        </router-link>

        <!-- Player Stats / History Link -->
        <router-link to="/history">
          <Button
            label="Stats"
            icon="pi pi-chart-bar"
            text
            size="small"
            class="hidden md:inline-flex text-slate-200 hover:text-white hover:bg-white/10 font-bold px-3 py-1.5"
          />
        </router-link>

        <!-- Admin / Manage -->
        <router-link to="/admin">
          <Button
            :label="authStore.isAuthenticated ? 'Dashboard' : 'Admin'"
            icon="pi pi-cog"
            size="small"
            class="bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 border border-white/15 font-bold text-white shadow-lg shadow-indigo-600/30 px-3.5 py-1.5 rounded-xl transition-all"
          />
        </router-link>
      </div>
    </div>
  </header>
</template>
