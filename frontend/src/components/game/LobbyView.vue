<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import { ref } from 'vue';
import Button from 'primevue/button';
import Card from 'primevue/card';

const gameStore = useGameStore();
const copied = ref(false);

function copyPin() {
  navigator.clipboard.writeText(gameStore.roomPin);
  copied.value = true;
  setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-8 py-6 px-4">
    <!-- Big PIN Display Card -->
    <Card class="bg-slate-900/90 border border-slate-800 rounded-3xl shadow-2xl overflow-hidden backdrop-blur-xl text-center">
      <template #content>
        <div class="py-6 space-y-4">
          <p class="text-sm font-semibold uppercase tracking-wider text-slate-400">
            Join at <span class="text-indigo-400 underline font-bold">quiz.ezsnapps.in</span> with Game PIN:
          </p>

          <div class="flex items-center justify-center space-x-4">
            <h1 class="text-6xl sm:text-8xl font-black font-mono tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 via-pink-400 to-purple-400">
              {{ gameStore.roomPin || '------' }}
            </h1>
            <Button
              :icon="copied ? 'pi pi-check' : 'pi pi-copy'"
              :class="copied ? 'p-button-success' : 'p-button-secondary'"
              rounded
              text
              size="large"
              @click="copyPin"
              title="Copy PIN"
            />
          </div>

          <h2 class="text-xl sm:text-2xl font-bold text-slate-200">
            {{ gameStore.quizTitle || 'Live Multiplayer Quiz' }}
          </h2>
        </div>
      </template>
    </Card>

    <!-- Players Count & Controls -->
    <div class="flex flex-col sm:flex-row items-center justify-between gap-4 bg-slate-900/60 p-5 rounded-2xl border border-slate-800">
      <div class="flex items-center space-x-3">
        <div class="w-10 h-10 rounded-xl bg-indigo-500/20 text-indigo-400 flex items-center justify-center font-bold text-lg">
          <i class="pi pi-users" />
        </div>
        <div>
          <h3 class="font-extrabold text-lg text-white">
            {{ gameStore.playerCount }} {{ gameStore.playerCount === 1 ? 'Player' : 'Players' }} Joined
          </h3>
          <p class="text-xs text-slate-400">
            {{ gameStore.role === 'host' ? 'Wait for players, then start the game' : 'Waiting for host to start...' }}
          </p>
        </div>
      </div>

      <!-- Host Action -->
      <div v-if="gameStore.role === 'host'" class="w-full sm:w-auto">
        <Button
          label="Start Game"
          icon="pi pi-play"
          size="large"
          class="w-full sm:w-auto bg-gradient-to-r from-emerald-600 to-teal-500 hover:from-emerald-500 hover:to-teal-400 border-none font-black text-white px-8 py-3 rounded-xl shadow-lg shadow-emerald-500/20"
          :disabled="gameStore.playerCount === 0"
          @click="gameStore.startGame"
        />
      </div>

      <!-- Player Waiting State -->
      <div v-else class="flex items-center space-x-2 text-indigo-400 text-sm font-semibold">
        <i class="pi pi-spin pi-spinner" />
        <span>Get ready! Host will launch soon...</span>
      </div>
    </div>

    <!-- Players Grid -->
    <div v-if="gameStore.players.length > 0" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
      <div
        v-for="player in gameStore.players"
        :key="player.id"
        class="bg-slate-900/80 border border-slate-800/80 hover:border-indigo-500/50 p-4 rounded-xl flex items-center justify-between group transition-all"
      >
        <div class="flex items-center space-x-3 truncate">
          <div class="w-8 h-8 rounded-full bg-gradient-to-tr from-indigo-500 to-pink-500 flex items-center justify-center font-bold text-xs text-white uppercase shrink-0">
            {{ player.nickname.slice(0, 2) }}
          </div>
          <span class="font-bold text-sm text-slate-200 truncate">
            {{ player.nickname }}
          </span>
        </div>

        <!-- Host kick player button -->
        <button
          v-if="gameStore.role === 'host'"
          @click="gameStore.kickPlayer(player.id)"
          class="text-slate-500 hover:text-rose-400 opacity-0 group-hover:opacity-100 transition-opacity p-1"
          title="Kick player"
        >
          <i class="pi pi-times text-xs" />
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="text-center py-12 text-slate-500">
      <i class="pi pi-user-plus text-4xl mb-3 text-slate-600 animate-bounce" />
      <p class="font-medium text-base">Waiting for players to join with the PIN above...</p>
    </div>
  </div>
</template>
