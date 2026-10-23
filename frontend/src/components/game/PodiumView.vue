<script setup lang="ts">
import { onMounted } from 'vue';
import { useGameStore } from '@/stores/game';
import confetti from 'canvas-confetti';
import Card from 'primevue/card';
import Button from 'primevue/button';

const gameStore = useGameStore();

onMounted(() => {
  // Confetti burst
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
  if (rank === 1) return { height: 'h-48 sm:h-56', bg: 'bg-amber-500', text: 'text-amber-400', medal: '🥇 1st' };
  if (rank === 2) return { height: 'h-36 sm:h-44', bg: 'bg-slate-400', text: 'text-slate-300', medal: '🥈 2nd' };
  return { height: 'h-28 sm:h-36', bg: 'bg-amber-700', text: 'text-amber-600', medal: '🥉 3rd' };
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-10 py-8 px-4 text-center">
    <!-- Header Title -->
    <div class="space-y-2">
      <span class="text-xs uppercase tracking-widest text-indigo-400 font-extrabold">Final Standings</span>
      <h1 class="text-4xl sm:text-6xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-300 via-pink-400 to-indigo-400">
        Quiz Champions!
      </h1>
      <p class="text-slate-400 text-sm">Congratulations to the top performers!</p>
    </div>

    <!-- Top 3 Podium Visual -->
    <div v-if="gameStore.podium.length > 0" class="flex items-end justify-center space-x-3 sm:space-x-6 pt-10 pb-4">
      <!-- 2nd Place -->
      <div v-if="gameStore.podium[1]" class="flex-1 max-w-[160px] flex flex-col items-center">
        <div class="mb-2 text-center">
          <div class="w-12 h-12 rounded-full bg-slate-700 border-2 border-slate-400 flex items-center justify-center font-bold text-slate-200 text-sm mx-auto shadow-md">
            {{ gameStore.podium[1].nickname.slice(0, 2).toUpperCase() }}
          </div>
          <span class="font-extrabold text-sm text-slate-200 block truncate mt-1">
            {{ gameStore.podium[1].nickname }}
          </span>
          <span class="text-xs font-bold text-slate-400">{{ gameStore.podium[1].score }} pts</span>
        </div>
        <div :class="['w-full rounded-t-2xl flex flex-col items-center justify-center p-2 text-slate-900 font-black text-lg shadow-xl', getPedestal(2).bg, getPedestal(2).height]">
          <span class="text-2xl">🥈</span>
          <span>2nd</span>
        </div>
      </div>

      <!-- 1st Place (Center & Tallest) -->
      <div v-if="gameStore.podium[0]" class="flex-1 max-w-[180px] flex flex-col items-center">
        <div class="mb-2 text-center">
          <div class="w-16 h-16 rounded-full bg-amber-500/20 border-4 border-amber-400 flex items-center justify-center font-black text-amber-300 text-lg mx-auto shadow-xl ring-4 ring-amber-400/20">
            {{ gameStore.podium[0].nickname.slice(0, 2).toUpperCase() }}
          </div>
          <span class="font-black text-base text-white block truncate mt-1">
            {{ gameStore.podium[0].nickname }}
          </span>
          <span class="text-sm font-extrabold text-amber-400">{{ gameStore.podium[0].score }} pts</span>
        </div>
        <div :class="['w-full rounded-t-2xl flex flex-col items-center justify-center p-2 text-slate-950 font-black text-xl shadow-2xl', getPedestal(1).bg, getPedestal(1).height]">
          <span class="text-3xl">👑</span>
          <span>1st</span>
        </div>
      </div>

      <!-- 3rd Place -->
      <div v-if="gameStore.podium[2]" class="flex-1 max-w-[160px] flex flex-col items-center">
        <div class="mb-2 text-center">
          <div class="w-12 h-12 rounded-full bg-amber-900/60 border-2 border-amber-700 flex items-center justify-center font-bold text-amber-200 text-sm mx-auto shadow-md">
            {{ gameStore.podium[2].nickname.slice(0, 2).toUpperCase() }}
          </div>
          <span class="font-extrabold text-sm text-slate-200 block truncate mt-1">
            {{ gameStore.podium[2].nickname }}
          </span>
          <span class="text-xs font-bold text-amber-500">{{ gameStore.podium[2].score }} pts</span>
        </div>
        <div :class="['w-full rounded-t-2xl flex flex-col items-center justify-center p-2 text-amber-100 font-black text-lg shadow-xl', getPedestal(3).bg, getPedestal(3).height]">
          <span class="text-2xl">🥉</span>
          <span>3rd</span>
        </div>
      </div>
    </div>

    <!-- Complete Results Table -->
    <Card class="bg-slate-900/90 border border-slate-800 rounded-3xl shadow-xl text-left overflow-hidden">
      <template #title>
        <div class="p-4 border-b border-slate-800 flex items-center justify-between">
          <h3 class="text-lg font-bold text-white">Full Leaderboard</h3>
          <span class="text-xs text-slate-400 font-semibold">{{ gameStore.fullResults.length }} participants</span>
        </div>
      </template>
      <template #content>
        <div class="divide-y divide-slate-800/80">
          <div
            v-for="entry in gameStore.fullResults"
            :key="entry.player_id"
            :class="[
              'flex items-center justify-between p-4 transition-colors',
              entry.player_id === gameStore.playerId ? 'bg-indigo-900/30' : 'hover:bg-slate-800/40'
            ]"
          >
            <div class="flex items-center space-x-4">
              <span class="w-8 font-black text-sm text-slate-400">
                #{{ entry.rank }}
              </span>
              <span class="font-bold text-slate-100">
                {{ entry.nickname }}
                <span v-if="entry.player_id === gameStore.playerId" class="ml-2 text-xs text-indigo-400 font-extrabold">(You)</span>
              </span>
            </div>

            <div class="flex items-center space-x-3">
              <span v-if="entry.streak > 1" class="text-xs font-bold px-2 py-0.5 rounded-full bg-rose-500/20 text-rose-400">
                🔥 {{ entry.streak }} streak
              </span>
              <span class="font-extrabold text-white text-base">
                {{ entry.score }} pts
              </span>
            </div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Return button -->
    <div class="pt-4">
      <router-link to="/">
        <Button
          label="Play Again / Return Home"
          icon="pi pi-home"
          size="large"
          class="bg-indigo-600 hover:bg-indigo-500 border-none font-bold text-white px-8 py-3 rounded-xl shadow-lg shadow-indigo-600/20"
          @click="gameStore.disconnect"
        />
      </router-link>
    </div>
  </div>
</template>
