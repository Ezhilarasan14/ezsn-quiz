<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import { useBrandingStore } from '@/stores/branding';

const gameStore = useGameStore();
const brandingStore = useBrandingStore();
</script>

<template>
  <div class="max-w-2xl mx-auto space-y-6 py-6 sm:py-10 px-4">
    <!-- Organization Header Banner -->
    <div
      v-if="brandingStore.currentTheme.logo_url || brandingStore.currentTheme.org_name"
      class="glass-panel flex items-center justify-between gap-4 p-4 rounded-2xl"
    >
      <div class="flex items-center space-x-3.5">
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-10 w-auto max-w-[130px] object-contain rounded-xl p-1 glass-card"
        />
        <div>
          <span class="text-[10px] font-extrabold uppercase tracking-widest text-indigo-400 block">Hosted by</span>
          <h2 class="text-sm sm:text-base font-black text-white leading-tight">
            {{ brandingStore.currentTheme.org_name }}
          </h2>
        </div>
      </div>

      <div class="text-right">
        <span class="text-[10px] font-bold uppercase text-slate-400 block">Room PIN</span>
        <span class="text-lg font-mono font-black text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-pink-400">
          {{ gameStore.roomPin }}
        </span>
      </div>
    </div>

    <!-- Player Ready Card (No QR Code - already connected) -->
    <div class="glass-panel rounded-3xl p-6 sm:p-10 text-center space-y-6 shadow-2xl">
      <!-- Animated Avatar -->
      <div class="relative inline-block">
        <div class="w-20 h-20 mx-auto rounded-full bg-gradient-to-tr from-emerald-500 via-teal-400 to-cyan-400 flex items-center justify-center text-3xl font-black text-slate-950 shadow-xl shadow-emerald-500/30 animate-bounce">
          ✓
        </div>
        <div class="absolute -bottom-1 -right-1 w-6 h-6 rounded-full bg-emerald-500 border-2 border-slate-950 flex items-center justify-center text-[10px] text-white font-bold animate-pulse">
          ●
        </div>
      </div>

      <div class="space-y-1.5">
        <span class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full glass-pill text-emerald-400 text-xs font-black uppercase tracking-wider">
          <span>Connected & Ready</span>
        </span>
        <h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight">
          You're in the Competition!
        </h1>
        <p class="text-slate-300 text-sm font-semibold">
          Playing as <span class="text-indigo-400 font-extrabold text-base">{{ gameStore.nickname }}</span>
          <span v-if="gameStore.playerEmail" class="text-xs text-slate-400 block font-mono mt-0.5">
            {{ gameStore.playerEmail }}
          </span>
        </p>
      </div>

      <!-- Waiting Status Box -->
      <div class="glass-card rounded-2xl p-4 sm:p-5 space-y-2 border border-white/10">
        <div class="flex items-center justify-center space-x-2 text-indigo-400 font-bold text-sm">
          <i class="pi pi-spin pi-spinner text-base" />
          <span>Waiting for host to start the quiz...</span>
        </div>
        <p class="text-xs text-slate-400 leading-relaxed max-w-sm mx-auto">
          Keep your phone awake! Four color-coded answer buttons will appear here the moment the first question begins.
        </p>
      </div>

      <!-- Live Contestants Count -->
      <div class="flex items-center justify-center space-x-2 text-xs font-semibold text-slate-400 pt-2 border-t border-white/10">
        <i class="pi pi-users text-indigo-400" />
        <span>{{ gameStore.playerCount }} {{ gameStore.playerCount === 1 ? 'contestant' : 'contestants' }} in arena</span>
      </div>
    </div>

    <!-- Contestants Arena Chips (Read-only list) -->
    <div v-if="gameStore.players.length > 1" class="space-y-2.5">
      <span class="text-xs font-bold uppercase tracking-wider text-slate-400 block px-1">Other Contestants</span>
      <div class="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
        <div
          v-for="player in gameStore.players"
          :key="player.id"
          class="glass-card p-3 rounded-xl flex items-center space-x-2.5 truncate"
          :class="{ 'border-emerald-500/50 bg-emerald-950/30 ring-1 ring-emerald-500/30': player.nickname === gameStore.nickname }"
        >
          <div class="w-7 h-7 rounded-lg bg-gradient-to-tr from-indigo-500 to-pink-500 flex items-center justify-center font-black text-xs text-white uppercase shrink-0 shadow">
            {{ player.nickname.slice(0, 2) }}
          </div>
          <span class="font-bold text-xs truncate" :class="player.nickname === gameStore.nickname ? 'text-emerald-400' : 'text-slate-200'">
            {{ player.nickname }} {{ player.nickname === gameStore.nickname ? '(You)' : '' }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
