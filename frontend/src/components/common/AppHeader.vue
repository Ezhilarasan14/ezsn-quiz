<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useBrandingStore } from '@/stores/branding';
import { useAuthStore } from '@/stores/auth';
import { soundService } from '@/services/audio';
import Button from 'primevue/button';

const route = useRoute();
const brandingStore = useBrandingStore();
const authStore = useAuthStore();
const isMuted = ref(false);
const logoError = ref(false);

const isGameRoute = computed(() => route.path === '/play' || route.path === '/host');

watch(
  () => brandingStore.currentTheme.logo_url,
  () => {
    logoError.value = false;
  }
);

function toggleSound() {
  isMuted.value = soundService.toggleMute();
}

function handleLogoError() {
  logoError.value = true;
}
</script>

<template>
  <header
    class="border-b border-purple-500/20 sticky top-0 z-40 backdrop-blur-xl shadow-lg shadow-purple-950/40 bg-purple-950/30"
  >
    <div class="max-w-7xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between gap-2">
      <!-- Main Brand Logo (Always Permanent: ezsnapps) -->
      <router-link to="/" class="flex items-center space-x-2.5 group shrink-0">
        <span class="text-base sm:text-xl font-black tracking-tight text-white group-hover:text-purple-300 transition-colors">
          ezsnapps<span class="text-purple-400">.quiz</span>
        </span>
      </router-link>

      <!-- Center: Organization & Branch Branding (Visible during active quiz sessions) -->
      <div
        v-if="isGameRoute && (brandingStore.currentTheme.logo_url || brandingStore.currentTheme.org_name)"
        class="flex items-center space-x-2 px-3 py-1 rounded-full bg-purple-900/40 border border-purple-400/25 backdrop-blur-md shadow-md max-w-[220px] sm:max-w-[380px]"
      >
        <!-- Logo only if valid and not errored -->
        <img
          v-if="brandingStore.currentTheme.logo_url && !logoError"
          :src="brandingStore.currentTheme.logo_url"
          alt=""
          role="presentation"
          class="h-5 sm:h-6 w-auto max-w-[50px] sm:max-w-[80px] object-contain rounded bg-white/10 p-0.5 shrink-0"
          @error="handleLogoError"
        />
        <div v-else class="w-5 h-5 rounded-full bg-purple-500/20 flex items-center justify-center text-[10px] text-purple-200 shrink-0">
          <i class="pi pi-building text-[10px]" />
        </div>

        <div class="flex items-center space-x-1.5 min-w-0">
          <span class="font-bold text-white text-xs sm:text-sm truncate">
            {{ brandingStore.currentTheme.org_name || 'Live Quiz' }}
          </span>
          <span v-if="brandingStore.currentTheme.branch_name" class="hidden sm:inline text-xs text-purple-300/80 truncate">
            • {{ brandingStore.currentTheme.branch_name }}
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
          class="text-white/90 hover:text-white hover:bg-white/15 w-9 h-9"
          @click="toggleSound"
          :aria-label="isMuted ? 'Unmute' : 'Mute'"
          :title="isMuted ? 'Unmute Audio' : 'Mute Audio'"
        />

        <!-- Join Screen Link (Hidden during active quiz / game routes) -->
        <router-link v-if="!isGameRoute" to="/">
          <Button
            label="Join"
            icon="pi pi-play"
            text
            size="small"
            class="text-white/90 hover:text-white hover:bg-white/15 font-bold px-3 py-1.5 rounded-xl text-xs sm:text-sm"
          />
        </router-link>

        <!-- Player Stats / History Link (Hidden during active quiz / game routes) -->
        <router-link v-if="!isGameRoute" to="/history">
          <Button
            label="Stats"
            icon="pi pi-chart-bar"
            text
            size="small"
            class="hidden md:inline-flex text-white/90 hover:text-white hover:bg-white/15 font-bold px-3 py-1.5 rounded-xl text-xs sm:text-sm"
          />
        </router-link>

        <!-- Admin / Manage (Hidden during active quiz / game routes) -->
        <router-link v-if="!isGameRoute" to="/admin">
          <Button
            :label="authStore.isAuthenticated ? 'Dashboard' : 'Admin'"
            icon="pi pi-cog"
            size="small"
            class="bg-gradient-to-r from-purple-600 to-fuchsia-600 hover:from-purple-500 hover:to-fuchsia-500 border border-purple-400/30 font-bold text-white shadow-lg shadow-purple-600/30 px-3.5 py-1.5 rounded-xl text-xs sm:text-sm transition-all"
          />
        </router-link>
      </div>
    </div>
  </header>
</template>
