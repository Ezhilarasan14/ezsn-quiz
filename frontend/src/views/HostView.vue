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
    <main class="flex-1 pb-6">
      <LobbyView v-if="gameStore.gameState === 'LOBBY'" />
      <QuestionView v-else-if="gameStore.gameState === 'QUESTION'" />
      <AnswerRevealView v-else-if="gameStore.gameState === 'REVEAL'" />
      <PodiumView v-else-if="gameStore.gameState === 'PODIUM'" />
    </main>

    <!-- Host control footer -->
    <footer
      v-if="gameStore.gameState !== 'PODIUM'"
      class="glass-panel border-t border-purple-400/20 py-3.5 px-4 sm:px-6 flex items-center justify-between text-xs font-semibold text-purple-200 sticky bottom-0 z-20 backdrop-blur-xl"
    >
      <div class="flex items-center space-x-2.5">
        <div class="relative flex items-center justify-center">
          <span class="w-2.5 h-2.5 rounded-full" :class="gameStore.isConnected ? 'bg-emerald-400' : 'bg-rose-400 animate-ping'" />
          <span v-if="gameStore.isConnected" class="absolute -inset-0.5 rounded-full bg-emerald-400/40 animate-ping" />
        </div>
        <span class="font-bold text-white">Host Projector • Room {{ gameStore.roomPin }}</span>
      </div>

      <div class="flex items-center space-x-3 sm:space-x-5">
        <span class="glass-pill px-3 py-1 rounded-full text-purple-200 font-bold hidden sm:inline-block">
          {{ gameStore.playerCount }} {{ gameStore.playerCount === 1 ? 'Contestant' : 'Contestants' }} Connected
        </span>
        <button
          v-if="gameStore.gameState === 'QUESTION'"
          @click="gameStore.nextQuestion()"
          class="bg-purple-600/70 hover:bg-purple-500 text-white font-bold transition-all py-1.5 px-3 rounded-xl border border-purple-400/30 flex items-center space-x-1.5 text-xs shadow"
          title="End Question & Show Results"
        >
          <span>End Question Early</span>
          <i class="pi pi-forward text-xs" />
        </button>
        <button
          @click="gameStore.disconnect(); router.push('/admin');"
          class="text-rose-400 hover:text-rose-300 font-bold transition-colors py-1 px-2 rounded-lg hover:bg-rose-500/10"
        >
          Exit Session
        </button>
      </div>
    </footer>
  </div>
</template>
