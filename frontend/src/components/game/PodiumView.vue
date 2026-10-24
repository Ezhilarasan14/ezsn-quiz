<script setup lang="ts">
import { onMounted } from 'vue';
import { useGameStore } from '@/stores/game';
import { useBrandingStore, MAIN_BRAND_LOGO } from '@/stores/branding';
import confetti from 'canvas-confetti';
import Button from 'primevue/button';

const gameStore = useGameStore();
const brandingStore = useBrandingStore();

onMounted(() => {
  // Joyful confetti burst
  confetti({
    particleCount: 120,
    spread: 80,
    origin: { y: 0.6 },
  });

  setTimeout(() => {
    confetti({
      particleCount: 80,
      angle: 60,
      spread: 55,
      origin: { x: 0 },
    });
    confetti({
      particleCount: 80,
      angle: 120,
      spread: 55,
      origin: { x: 1 },
    });
  }, 400);
});

function getPedestal(rank: number) {
  if (rank === 1) {
    return {
      height: 'h-48 sm:h-60',
      bg: 'bg-gradient-to-t from-amber-600 via-amber-500 to-yellow-300 border-t-2 border-amber-200 shadow-2xl shadow-amber-500/30',
      textColor: 'text-slate-950',
      badge: '👑 1st',
    };
  }
  if (rank === 2) {
    return {
      height: 'h-36 sm:h-44',
      bg: 'bg-gradient-to-t from-slate-600 via-slate-500 to-slate-300 border-t-2 border-slate-200 shadow-xl shadow-slate-500/20',
      textColor: 'text-slate-950',
      badge: '🥈 2nd',
    };
  }
  return {
    height: 'h-28 sm:h-36',
    bg: 'bg-gradient-to-t from-amber-800 via-amber-700 to-orange-500 border-t-2 border-amber-300 shadow-xl shadow-orange-700/20',
    textColor: 'text-amber-100',
    badge: '🥉 3rd',
  };
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-10 py-6 sm:py-12 px-4 text-center">
    <!-- Header with Organization Logo & Title -->
    <div class="space-y-3">
      <div v-if="brandingStore.currentTheme.logo_url" class="flex justify-center mb-2">
        <img
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-14 w-auto max-w-[200px] object-contain rounded-2xl p-1.5 glass-panel shadow-2xl"
        />
      </div>

      <div class="inline-flex items-center space-x-2 px-3.5 py-1 rounded-full glass-pill text-amber-300 text-xs font-black uppercase tracking-wider">
        <span>{{ brandingStore.currentTheme.org_name ? `${brandingStore.currentTheme.org_name} • Results` : 'Final Standings' }}</span>
      </div>

      <h1 class="text-4xl sm:text-6xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-300 via-pink-400 to-indigo-400 tracking-tight">
        Quiz Champions!
      </h1>
      <p class="text-slate-400 text-sm max-w-sm mx-auto">
        Huge congratulations to all participants for an incredible competition!
      </p>
    </div>

    <!-- Top 3 Glass Pedestals Podium Visual -->
    <div v-if="gameStore.podium.length > 0" class="flex items-end justify-center space-x-3 sm:space-x-6 pt-6 pb-2">
      <!-- 2nd Place -->
      <div v-if="gameStore.podium[1]" class="flex-1 max-w-[150px] sm:max-w-[180px] flex flex-col items-center">
        <div class="mb-3 text-center">
          <div class="w-12 h-12 sm:w-14 sm:h-14 rounded-2xl glass-panel border-2 border-slate-400 flex items-center justify-center font-black text-slate-200 text-sm sm:text-base mx-auto shadow-xl ring-2 ring-slate-400/20">
            {{ gameStore.podium[1].nickname.slice(0, 2).toUpperCase() }}
          </div>
          <span class="font-extrabold text-sm text-slate-200 block truncate mt-1.5">
            {{ gameStore.podium[1].nickname }}
          </span>
          <span class="text-xs font-bold text-slate-400 font-mono">{{ gameStore.podium[1].score }} pts</span>
        </div>
        <div :class="['w-full rounded-t-3xl flex flex-col items-center justify-center p-3 font-black text-lg shadow-2xl relative overflow-hidden', getPedestal(2).bg, getPedestal(2).height, getPedestal(2).textColor]">
          <span class="text-3xl mb-1 drop-shadow">🥈</span>
          <span class="text-base font-black uppercase tracking-wider">2nd</span>
        </div>
      </div>

      <!-- 1st Place (Center & Tallest) -->
      <div v-if="gameStore.podium[0]" class="flex-1 max-w-[170px] sm:max-w-[210px] flex flex-col items-center">
        <div class="mb-3 text-center">
          <div class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl bg-amber-500/20 border-4 border-amber-400 flex items-center justify-center font-black text-amber-300 text-lg sm:text-2xl mx-auto shadow-2xl ring-4 ring-amber-400/30 animate-pulse">
            {{ gameStore.podium[0].nickname.slice(0, 2).toUpperCase() }}
          </div>
          <span class="font-black text-base sm:text-lg text-white block truncate mt-2">
            {{ gameStore.podium[0].nickname }}
          </span>
          <span class="text-xs sm:text-sm font-black text-amber-400 font-mono">{{ gameStore.podium[0].score }} pts</span>
        </div>
        <div :class="['w-full rounded-t-3xl flex flex-col items-center justify-center p-3 font-black text-xl shadow-2xl relative overflow-hidden', getPedestal(1).bg, getPedestal(1).height, getPedestal(1).textColor]">
          <span class="text-4xl mb-1 drop-shadow-md">👑</span>
          <span class="text-lg font-black uppercase tracking-wider">1st Place</span>
        </div>
      </div>

      <!-- 3rd Place -->
      <div v-if="gameStore.podium[2]" class="flex-1 max-w-[150px] sm:max-w-[180px] flex flex-col items-center">
        <div class="mb-3 text-center">
          <div class="w-12 h-12 sm:w-14 sm:h-14 rounded-2xl glass-panel border-2 border-amber-600 flex items-center justify-center font-black text-amber-300 text-sm sm:text-base mx-auto shadow-xl ring-2 ring-amber-600/20">
            {{ gameStore.podium[2].nickname.slice(0, 2).toUpperCase() }}
          </div>
          <span class="font-extrabold text-sm text-slate-200 block truncate mt-1.5">
            {{ gameStore.podium[2].nickname }}
          </span>
          <span class="text-xs font-bold text-amber-500 font-mono">{{ gameStore.podium[2].score }} pts</span>
        </div>
        <div :class="['w-full rounded-t-3xl flex flex-col items-center justify-center p-3 font-black text-lg shadow-2xl relative overflow-hidden', getPedestal(3).bg, getPedestal(3).height, getPedestal(3).textColor]">
          <span class="text-3xl mb-1 drop-shadow">🥉</span>
          <span class="text-base font-black uppercase tracking-wider">3rd</span>
        </div>
      </div>
    </div>

    <!-- Complete Results Glass Table -->
    <div class="glass-panel rounded-3xl overflow-hidden shadow-2xl text-left">
      <div class="p-5 border-b border-white/10 flex items-center justify-between">
        <div>
          <h3 class="text-lg font-black text-white">Full Leaderboard</h3>
          <p class="text-xs text-slate-400">Final scores ranked by speed and accuracy</p>
        </div>
        <span class="glass-pill px-3 py-1 rounded-full text-xs text-slate-300 font-bold">
          {{ gameStore.fullResults.length }} {{ gameStore.fullResults.length === 1 ? 'Contestant' : 'Contestants' }}
        </span>
      </div>

      <div class="divide-y divide-white/5">
        <div
          v-for="entry in gameStore.fullResults"
          :key="entry.player_id"
          :class="[
            'flex items-center justify-between p-4 sm:p-5 transition-colors',
            entry.player_id === gameStore.playerId ? 'bg-indigo-950/40 border-l-4 border-indigo-500' : 'hover:bg-white/[0.02]'
          ]"
        >
          <div class="flex items-center space-x-3.5">
            <span
              :class="[
                'w-8 h-8 rounded-xl flex items-center justify-center font-black text-xs shrink-0',
                entry.rank === 1 ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30' :
                entry.rank === 2 ? 'bg-slate-400/20 text-slate-300 border border-slate-400/30' :
                entry.rank === 3 ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30' :
                'glass-pill text-slate-400'
              ]"
            >
              #{{ entry.rank }}
            </span>
            <div>
              <span class="font-extrabold text-white text-sm sm:text-base">
                {{ entry.nickname }}
              </span>
              <span v-if="entry.player_id === gameStore.playerId" class="ml-2 text-xs text-emerald-400 font-extrabold px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/20">
                You
              </span>
            </div>
          </div>

          <div class="flex items-center space-x-3">
            <span v-if="entry.streak > 1" class="text-[11px] font-black px-2.5 py-0.5 rounded-full bg-rose-500/20 text-rose-300 border border-rose-500/30">
              🔥 {{ entry.streak }} streak
            </span>
            <span class="font-black text-white text-base sm:text-lg font-mono">
              {{ entry.score }} pts
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Actions & Return to Arena -->
    <div class="pt-4 space-y-4">
      <div>
        <router-link to="/">
          <Button
            label="Play Again / Exit to Home"
            icon="pi pi-home"
            size="large"
            class="bg-gradient-to-r from-indigo-600 to-pink-600 hover:from-indigo-500 hover:to-pink-500 border border-white/20 font-black text-white px-8 py-3.5 rounded-2xl shadow-xl shadow-indigo-600/30 active:scale-98 transition-all"
            @click="gameStore.disconnect"
          />
        </router-link>
      </div>

      <!-- Main Platform Logo Footer -->
      <div class="flex items-center justify-center space-x-2 text-xs text-slate-500 pt-2">
        <span>Powered by</span>
        <img :src="MAIN_BRAND_LOGO" alt="EZSN Quiz" class="h-4 w-auto inline opacity-80" />
        <span class="font-bold text-slate-400">EZSN Quiz</span>
      </div>
    </div>
  </div>
</template>
