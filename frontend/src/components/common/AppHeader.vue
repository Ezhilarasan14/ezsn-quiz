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
        <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-indigo-500 via-purple-500 to-pink-500 p-0.5 shadow-md shadow-indigo-500/20 group-hover:shadow-indigo-500/40 transition-all">
          <div class="w-full h-full bg-[#070a13] rounded-[6px] flex items-center justify-center font-black text-xs text-transparent bg-clip-text bg-gradient-to-tr from-indigo-400 to-pink-400">
            EZ
          </div>
        </div>
        <span class="text-lg sm:text-xl font-black tracking-tight text-white group-hover:text-indigo-300 transition-colors">
          ezsnapps<span class="text-indigo-400">.quiz</span>
        </span>
      </router-link>

      <!-- Center: Organization Branding (Visible during active quiz sessions) -->
      <div
        v-if="isGameRoute && (brandingStore.currentTheme.logo_url || brandingStore.currentTheme.org_name)"
        class="hidden sm:flex items-center space-x-2.5 px-3.5 py-1 rounded-full bg-black/40 border border-white/15 backdrop-blur-md shadow-inner"
      >
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-5 w-auto max-w-[90px] object-contain rounded"
        />
        <div class="flex items-center space-x-1.5 text-xs">
          <span class="text-[10px] font-extrabold uppercase tracking-wider text-indigo-300">Host:</span>
          <span class="font-bold text-white truncate max-w-[140px]">
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
