<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import CountdownBar from '@/components/common/CountdownBar.vue';
import Message from 'primevue/message';

const gameStore = useGameStore();

const optionStyles = [
  { bg: 'bg-rose-600 hover:bg-rose-500 active:scale-95', icon: '▲' },
  { bg: 'bg-blue-600 hover:bg-blue-500 active:scale-95', icon: '◆' },
  { bg: 'bg-amber-500 hover:bg-amber-400 active:scale-95', icon: '●' },
  { bg: 'bg-emerald-600 hover:bg-emerald-500 active:scale-95', icon: '■' },
];

function selectOption(id: string) {
  if (!gameStore.answerSubmitted) {
    gameStore.submitAnswer(id);
  }
}
</script>

<template>
  <div class="max-w-md mx-auto space-y-5 py-4 px-4 h-full flex flex-col justify-between">
    <!-- Top Status -->
    <div class="space-y-3">
      <div class="flex items-center justify-between text-xs font-bold text-slate-400">
        <span>Q {{ (gameStore.questionIndex || 0) + 1 }} / {{ gameStore.totalQuestions || 1 }}</span>
        <span>Score: {{ gameStore.score }}</span>
      </div>

      <CountdownBar
        :totalSecs="gameStore.timeLimitSecs"
        :timeLeftSecs="gameStore.timeLeftSecs"
      />
    </div>

    <!-- Question Text Preview -->
    <div class="text-center py-2">
      <h3 class="text-lg font-black text-white line-clamp-3">
        {{ gameStore.questionText }}
      </h3>
    </div>

    <!-- Submitted / Waiting Message -->
    <div v-if="gameStore.answerSubmitted" class="py-4">
      <Message severity="info" icon="pi pi-check-circle" :closable="false" class="rounded-xl shadow-lg">
        <span class="font-bold text-sm">
          Answer submitted! Waiting for timer to end...
        </span>
      </Message>
    </div>

    <!-- 4 Big Kahoot-style buttons for player -->
    <div class="grid grid-cols-2 gap-3 flex-1 min-h-[260px]">
      <button
        v-for="(opt, idx) in gameStore.options"
        :key="opt.id"
        :disabled="gameStore.answerSubmitted"
        @click="selectOption(opt.id)"
        :class="[
          'rounded-2xl flex flex-col items-center justify-center p-4 text-white font-black text-xl transition-all shadow-xl',
          optionStyles[idx % 4].bg,
          gameStore.selectedOptionId === opt.id ? 'ring-4 ring-white ring-offset-2 ring-offset-slate-900 scale-102' : '',
          gameStore.answerSubmitted && gameStore.selectedOptionId !== opt.id ? 'opacity-40 grayscale-30' : ''
        ]"
      >
        <span class="text-4xl mb-2">{{ optionStyles[idx % 4].icon }}</span>
        <span class="text-sm font-extrabold text-center line-clamp-2">{{ opt.text }}</span>
      </button>
    </div>
  </div>
</template>
