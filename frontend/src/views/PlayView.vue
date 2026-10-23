<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useGameStore } from '@/stores/game';
import LobbyView from '@/components/game/LobbyView.vue';
import PlayerController from '@/components/game/PlayerController.vue';
import AnswerRevealView from '@/components/game/AnswerRevealView.vue';
import PodiumView from '@/components/game/PodiumView.vue';

const route = useRoute();
const router = useRouter();
const gameStore = useGameStore();

onMounted(() => {
  const pin = (route.query.pin as string) || '';
  const nickname = (route.query.nickname as string) || '';
  const email = (route.query.email as string) || '';

  if (!pin || !nickname) {
    router.replace('/');
    return;
  }

  gameStore.joinAsPlayer(pin, nickname, email);
});

onUnmounted(() => {
  gameStore.disconnect();
});
</script>

<template>
  <div class="min-h-[calc(100vh-4rem)] flex flex-col justify-between">
    <!-- Active view based on game state -->
    <main class="flex-1">
      <LobbyView v-if="gameStore.gameState === 'LOBBY'" />
      <PlayerController v-else-if="gameStore.gameState === 'QUESTION'" />
      <AnswerRevealView v-else-if="gameStore.gameState === 'REVEAL'" />
      <PodiumView v-else-if="gameStore.gameState === 'PODIUM'" />
    </main>

    <!-- Bottom Player Status Bar (during active question) -->
    <footer
      v-if="gameStore.gameState !== 'PODIUM'"
      class="bg-slate-900/90 border-t border-slate-800 py-3 px-4 flex items-center justify-between text-xs font-bold text-slate-300 max-w-4xl mx-auto w-full"
    >
      <div class="flex items-center space-x-2">
        <div class="w-2.5 h-2.5 rounded-full" :class="gameStore.isConnected ? 'bg-emerald-400' : 'bg-rose-400 animate-ping'" />
        <span>{{ gameStore.nickname }}</span>
      </div>

      <div class="flex items-center space-x-4">
        <span>PIN: {{ gameStore.roomPin }}</span>
        <span class="text-indigo-400 font-extrabold">{{ gameStore.score }} pts</span>
      </div>
    </footer>
  </div>
</template>
