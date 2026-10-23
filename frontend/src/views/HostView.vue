<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useGameStore } from '@/stores/game';
import LobbyView from '@/components/game/LobbyView.vue';
import QuestionView from '@/components/game/QuestionView.vue';
import AnswerRevealView from '@/components/game/AnswerRevealView.vue';
import PodiumView from '@/components/game/PodiumView.vue';

const route = useRoute();
const router = useRouter();
const gameStore = useGameStore();

onMounted(() => {
  const pin = (route.query.pin as string) || '';
  const token = (route.query.token as string) || '';

  if (!pin || !token) {
    router.replace('/admin');
    return;
  }

  gameStore.connectAsHost(pin, token);
});

onUnmounted(() => {
  gameStore.disconnect();
});
</script>

<template>
  <div class="min-h-[calc(100vh-4rem)] flex flex-col justify-between">
    <main class="flex-1">
      <LobbyView v-if="gameStore.gameState === 'LOBBY'" />
      <QuestionView v-else-if="gameStore.gameState === 'QUESTION'" />
      <AnswerRevealView v-else-if="gameStore.gameState === 'REVEAL'" />
      <PodiumView v-else-if="gameStore.gameState === 'PODIUM'" />
    </main>

    <!-- Host control footer -->
    <footer
      v-if="gameStore.gameState !== 'PODIUM'"
      class="bg-slate-900/90 border-t border-slate-800 py-3 px-6 flex items-center justify-between text-xs font-semibold text-slate-400"
    >
      <div class="flex items-center space-x-3">
        <span class="w-2.5 h-2.5 rounded-full" :class="gameStore.isConnected ? 'bg-emerald-400' : 'bg-rose-400'" />
        <span>Host Console • Room {{ gameStore.roomPin }}</span>
      </div>

      <div class="flex items-center space-x-4">
        <span>{{ gameStore.playerCount }} Players Connected</span>
        <button
          @click="gameStore.disconnect(); router.push('/admin');"
          class="text-rose-400 hover:text-rose-300 font-bold"
        >
          Exit Session
        </button>
      </div>
    </footer>
  </div>
</template>
