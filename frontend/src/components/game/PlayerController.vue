<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import { useBrandingStore } from '@/stores/branding';
import CountdownBar from '@/components/common/CountdownBar.vue';

const gameStore = useGameStore();
const brandingStore = useBrandingStore();

const optionStyles = [
  {
    bg: 'from-rose-500 via-rose-600 to-rose-700 hover:from-rose-400 hover:to-rose-600 shadow-rose-600/30',
    border: 'border-rose-400/30',
    icon: '▲',
    label: 'Red Triangle',
  },
  {
    bg: 'from-blue-500 via-blue-600 to-indigo-700 hover:from-blue-400 hover:to-indigo-600 shadow-blue-600/30',
    border: 'border-blue-400/30',
    icon: '◆',
    label: 'Blue Diamond',
  },
  {
    bg: 'from-amber-400 via-amber-500 to-orange-600 hover:from-amber-300 hover:to-orange-500 shadow-amber-500/30',
    border: 'border-amber-300/30',
    icon: '●',
    label: 'Amber Circle',
  },
  {
    bg: 'from-emerald-400 via-emerald-500 to-teal-700 hover:from-emerald-300 hover:to-teal-600 shadow-emerald-500/30',
    border: 'border-emerald-300/30',
    icon: '■',
    label: 'Green Square',
  },
];

function selectOption(id: string) {
  if (!gameStore.answerSubmitted) {
    gameStore.submitAnswer(id);
  }
}
</script>

<template>
  <div class="max-w-md mx-auto space-y-4 py-3 px-4 min-h-[calc(100vh-8rem)] flex flex-col justify-between">
    <!-- Top Header & Timer -->
    <div class="glass-panel rounded-2xl p-3.5 space-y-2.5">
      <div class="flex items-center justify-between text-xs font-bold text-slate-300">
        <div class="flex items-center space-x-2">
          <span class="glass-pill px-2.5 py-0.5 rounded-md text-indigo-300 font-mono">
            Q {{ (gameStore.questionIndex || 0) + 1 }} / {{ gameStore.totalQuestions || 1 }}
          </span>
          <img
            v-if="brandingStore.currentTheme.logo_url"
            :src="brandingStore.currentTheme.logo_url"
            alt="Org Logo"
            class="h-4 w-auto max-w-[60px] object-contain rounded opacity-80"
          />
        </div>

        <div class="flex items-center space-x-1.5">
          <i class="pi pi-clock text-indigo-400 text-xs" />
          <span class="font-bold text-slate-300">{{ gameStore.timeLimitSecs }}s limit</span>
        </div>
      </div>

      <CountdownBar
        :totalSecs="gameStore.timeLimitSecs"
        :timeLeftSecs="gameStore.timeLeftSecs"
      />
    </div>

    <!-- Question Text Preview -->
    <div class="glass-card rounded-2xl p-4 text-center">
      <h3 class="text-base sm:text-lg font-black text-white line-clamp-3 leading-snug">
        {{ gameStore.questionText }}
      </h3>
    </div>

    <!-- Submitted / Waiting Feedback Card -->
    <div v-if="gameStore.answerSubmitted" class="py-1">
      <div class="glass-panel border-indigo-500/30 bg-indigo-950/40 rounded-2xl p-3.5 flex items-center justify-center space-x-2.5 shadow-lg shadow-indigo-950/50">
        <div class="w-5 h-5 rounded-full bg-indigo-500 flex items-center justify-center text-xs font-black text-white animate-pulse">
          ✓
        </div>
        <span class="font-black text-xs sm:text-sm text-indigo-200">
          Answer Locked In! Preparing next question...
        </span>
      </div>
    </div>

    <!-- 4 Tactile Glassmorphic Answer Buttons -->
    <div class="grid grid-cols-2 gap-3 flex-1 min-h-[260px] max-h-[380px]">
      <button
        v-for="(opt, idx) in gameStore.options"
        :key="opt.id"
        :disabled="gameStore.answerSubmitted"
        @click="selectOption(opt.id)"
        :class="[
          'relative overflow-hidden rounded-2xl flex flex-col items-center justify-center p-4 text-white font-black transition-all duration-200 border shadow-lg active:scale-95 group',
          'bg-gradient-to-br', optionStyles[idx % 4].bg, optionStyles[idx % 4].border,
          gameStore.selectedOptionId === opt.id ? 'ring-4 ring-white ring-offset-2 ring-offset-slate-950 scale-102 shadow-2xl' : '',
          gameStore.answerSubmitted && gameStore.selectedOptionId !== opt.id ? 'opacity-35 grayscale-30 cursor-not-allowed' : ''
        ]"
      >
        <!-- Top Glass Shine Reflection -->
        <div class="absolute inset-x-0 top-0 h-1/2 bg-gradient-to-b from-white/25 to-transparent pointer-events-none rounded-t-2xl" />

        <span class="text-3xl sm:text-4xl mb-1.5 drop-shadow-md group-hover:scale-110 transition-transform">
          {{ optionStyles[idx % 4].icon }}
        </span>
        <span class="text-xs sm:text-sm font-extrabold text-center line-clamp-2 drop-shadow leading-tight px-1">
          {{ opt.text }}
        </span>

        <!-- Selected Checkmark Badge -->
        <div
          v-if="gameStore.selectedOptionId === opt.id"
          class="absolute top-2 right-2 w-5 h-5 rounded-full bg-white text-slate-950 flex items-center justify-center text-[10px] font-black shadow"
        >
          ✓
        </div>
      </button>
    </div>
  </div>
</template>
