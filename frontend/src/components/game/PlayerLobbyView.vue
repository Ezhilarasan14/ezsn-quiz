<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import { useBrandingStore, MAIN_BRAND_LOGO } from '@/stores/branding';
import Card from 'primevue/card';

const gameStore = useGameStore();
const brandingStore = useBrandingStore();
</script>

<template>
  <div class="max-w-2xl mx-auto space-y-6 py-6 px-4">
    <!-- Organization Header Banner -->
    <div
      v-if="brandingStore.currentTheme.logo_url || brandingStore.currentTheme.org_name"
      class="flex items-center justify-between gap-4 p-4 rounded-2xl bg-slate-900/60 border border-slate-800/80 backdrop-blur-md"
    >
      <div class="flex items-center space-x-3.5">
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-10 w-auto max-w-[140px] object-contain rounded-xl p-1 bg-slate-950/80 border border-white/10"
        />
        <div>
          <span class="text-[10px] font-extrabold uppercase tracking-widest text-indigo-400 block">Hosted by</span>
          <h2 class="text-base sm:text-lg font-black text-white leading-tight">
            {{ brandingStore.currentTheme.org_name }}
          </h2>
        </div>
      </div>

      <div class="text-right">
        <span class="text-[10px] font-bold uppercase text-slate-500 block">Room PIN</span>
        <span class="text-lg font-mono font-black text-indigo-400">{{ gameStore.roomPin }}</span>
      </div>
    </div>

    <!-- Player Ready Card (No QR Code - already in game!) -->
    <Card class="bg-slate-900/90 border border-slate-800 rounded-3xl shadow-2xl overflow-hidden backdrop-blur-xl text-center py-8 px-4 sm:px-6">
      <template #content>
        <div class="max-w-md mx-auto space-y-6">
          <!-- Animated Checkmark Avatar -->
          <div class="relative inline-block">
            <div class="w-20 h-20 mx-auto rounded-full bg-gradient-to-tr from-emerald-500 to-teal-400 flex items-center justify-center text-3xl font-black text-slate-950 shadow-xl shadow-emerald-500/20 animate-bounce">
              ✓
            </div>
            <div class="absolute -bottom-1 -right-1 w-6 h-6 rounded-full bg-emerald-500 border-2 border-slate-900 flex items-center justify-center text-[10px] text-white font-bold">
              ●
            </div>
          </div>

          <div class="space-y-2">
            <span class="text-xs font-black uppercase tracking-widest text-emerald-400 block">
              Connected & Logged In
            </span>
            <h1 class="text-3xl font-black text-white">
              You're in the Competition!
            </h1>
            <p class="text-slate-300 text-sm font-semibold">
              Playing as <span class="text-indigo-400 font-extrabold text-base">{{ gameStore.nickname }}</span>
              <span v-if="gameStore.playerEmail" class="text-xs text-slate-400 block font-mono mt-0.5">
                ({{ gameStore.playerEmail }})
              </span>
            </p>
          </div>

          <!-- Waiting for Host status box -->
          <div class="p-4 rounded-2xl bg-slate-950/70 border border-slate-800 space-y-2.5">
            <div class="flex items-center justify-center space-x-2 text-indigo-400 font-bold text-sm">
              <i class="pi pi-spin pi-spinner text-base" />
              <span>Waiting for host to start the quiz...</span>
            </div>
            <p class="text-xs text-slate-400 leading-relaxed">
              Keep your screen on! Four color-coded answer buttons will appear here the moment the first question begins.
            </p>
          </div>

          <!-- Players joined count -->
          <div class="flex items-center justify-center space-x-2 text-xs font-semibold text-slate-400 pt-2 border-t border-slate-800/60">
            <i class="pi pi-users text-indigo-400" />
            <span>{{ gameStore.playerCount }} {{ gameStore.playerCount === 1 ? 'player' : 'players' }} in room</span>
          </div>
        </div>
      </template>
    </Card>

    <!-- Other Players in the Room (Read-only list) -->
    <div v-if="gameStore.players.length > 1" class="space-y-2">
      <span class="text-xs font-bold uppercase tracking-wider text-slate-400 block px-1">Other Contestants</span>
      <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
        <div
          v-for="player in gameStore.players"
          :key="player.id"
          class="bg-slate-900/60 border border-slate-800 p-2.5 rounded-xl flex items-center space-x-2 truncate"
          :class="{ 'border-emerald-500/50 bg-emerald-950/20': player.nickname === gameStore.nickname }"
        >
          <div class="w-6 h-6 rounded-full bg-gradient-to-tr from-indigo-500 to-pink-500 flex items-center justify-center font-bold text-[10px] text-white uppercase shrink-0">
            {{ player.nickname.slice(0, 2) }}
          </div>
          <span class="font-bold text-xs truncate" :class="player.nickname === gameStore.nickname ? 'text-emerald-400' : 'text-slate-300'">
            {{ player.nickname }} {{ player.nickname === gameStore.nickname ? '(You)' : '' }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
