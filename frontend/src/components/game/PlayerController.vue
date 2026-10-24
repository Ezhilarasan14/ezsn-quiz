<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import { useBrandingStore } from '@/stores/branding';
import CountdownBar from '@/components/common/CountdownBar.vue';

const gameStore = useGameStore();
const brandingStore = useBrandingStore();

const optionStyles = [
  {
    bg: 'from-rose-500 via-rose-600 to-rose-700 shadow-rose-600/30',
    iconBg: 'bg-rose-900/40 border-rose-400/40 text-rose-200',
    border: 'border-rose-400/30',
    icon: '▲',
    label: 'Red Triangle',
  },
  {
    bg: 'from-blue-500 via-blue-600 to-indigo-700 shadow-blue-600/30',
    iconBg: 'bg-blue-900/40 border-blue-400/40 text-blue-200',
    border: 'border-blue-400/30',
    icon: '◆',
    label: 'Blue Diamond',
  },
  {
    bg: 'from-amber-400 via-amber-500 to-orange-600 shadow-amber-500/30',
    iconBg: 'bg-amber-900/40 border-amber-300/40 text-amber-200',
    border: 'border-amber-300/30',
    icon: '●',
    label: 'Amber Circle',
  },
  {
    bg: 'from-emerald-400 via-emerald-500 to-teal-700 shadow-emerald-500/30',
    iconBg: 'bg-emerald-900/40 border-emerald-300/40 text-emerald-200',
    border: 'border-emerald-300/30',
    icon: '■',
    label: 'Green Square',
  },
];

function selectOption(id: string) {
  if (!gameStore.answerSubmitted && gameStore.timeLeftSecs > 0) {
    gameStore.submitAnswer(id);
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-4 py-3 sm:py-6 px-4 min-h-[calc(100vh-5rem)] flex flex-col justify-between">
    <!-- Top Header & Timer -->
    <div class="glass-panel rounded-2xl p-3 sm:p-4 space-y-2.5 shadow-lg border border-purple-400/20">
      <div class="flex items-center justify-between text-xs sm:text-sm font-bold text-purple-200">
        <div class="flex items-center space-x-2.5">
          <span class="glass-pill px-3 py-1 rounded-full text-purple-200 font-mono font-bold text-xs sm:text-sm border border-purple-400/30">
            Question {{ gameStore.questionIndex || 1 }} of {{ gameStore.totalQuestions || 1 }}
          </span>
          <img
            v-if="brandingStore.currentTheme.logo_url"
            :src="brandingStore.currentTheme.logo_url"
            alt="Org Logo"
            class="h-4 sm:h-5 w-auto max-w-[60px] object-contain rounded opacity-80"
          />
        </div>

        <div class="flex items-center space-x-1.5 text-xs sm:text-sm">
          <i class="pi pi-clock text-purple-300" />
          <span class="font-bold text-purple-200">{{ gameStore.timeLimitSecs }}s limit</span>
        </div>
      </div>

      <CountdownBar
        :totalSecs="gameStore.timeLimitSecs"
        :timeLeftSecs="gameStore.timeLeftSecs"
      />
    </div>

    <!-- Question Text Card (Readable, matching Host style) -->
    <div class="glass-panel rounded-2xl sm:rounded-3xl p-5 sm:p-8 text-center shadow-xl relative overflow-hidden border border-purple-400/20">
      <div class="absolute -top-16 left-1/2 -translate-x-1/2 w-64 h-24 bg-purple-600/20 blur-2xl pointer-events-none" />
      <h2 class="relative text-base sm:text-2xl md:text-3xl font-black text-white leading-snug tracking-tight">
        {{ gameStore.questionText || 'Loading question...' }}
      </h2>
    </div>

    <!-- Submitted / Waiting Feedback Card with Dynamic Response Time -->
    <div v-if="gameStore.answerSubmitted" class="py-0.5">
      <div class="glass-panel border-purple-400/30 bg-purple-950/60 rounded-xl sm:rounded-2xl p-3 flex items-center justify-between shadow-lg shadow-purple-950/50">
        <div class="flex items-center space-x-2">
          <div class="w-6 h-6 rounded-full bg-emerald-500 text-slate-950 flex items-center justify-center text-xs font-black">
            ✓
          </div>
          <span class="font-extrabold text-xs sm:text-sm text-purple-100">
            Answer Locked In!
          </span>
        </div>

        <!-- Dynamic Response Time -->
        <div class="flex items-center space-x-1.5 px-3 py-1 rounded-full bg-purple-500/20 border border-purple-400/30 font-mono font-bold text-xs text-purple-200">
          <i class="pi pi-clock text-cyan-300 text-xs" />
          <span>{{ gameStore.currentQuestionResponseTimeMs ? `${(gameStore.currentQuestionResponseTimeMs / 1000).toFixed(1)}s` : 'Fast!' }}</span>
        </div>
      </div>
    </div>

    <!-- Time Expired Feedback Card -->
    <div v-else-if="gameStore.timeLeftSecs <= 0" class="py-0.5">
      <div class="glass-panel border-rose-500/40 bg-rose-950/50 rounded-xl sm:rounded-2xl p-3 flex items-center justify-center space-x-2.5 shadow-lg shadow-rose-950/50">
        <div class="w-6 h-6 rounded-full bg-rose-600 flex items-center justify-center text-xs font-black text-white">
          <i class="pi pi-lock text-xs" />
        </div>
        <span class="font-black text-xs sm:text-sm text-rose-200">
          Time's Up! Answers locked.
        </span>
      </div>
    </div>

    <!-- 4 Tactile Glassmorphic Answer Buttons (Matching Host Proportions & Geometry Badges) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4 flex-1 items-stretch">
      <button
        v-for="(opt, idx) in gameStore.options"
        :key="opt.id"
        :disabled="gameStore.answerSubmitted || gameStore.timeLeftSecs <= 0"
        @click="selectOption(opt.id)"
        :class="[
          'relative overflow-hidden rounded-xl sm:rounded-2xl p-3.5 sm:p-5 text-white font-extrabold text-sm sm:text-lg flex items-center space-x-3 sm:space-x-4 shadow-xl border border-white/20 transition-all duration-150 group text-left min-h-[64px] sm:min-h-[76px]',
          (gameStore.answerSubmitted || gameStore.timeLeftSecs <= 0) ? 'cursor-not-allowed' : 'active:scale-98 hover:brightness-105 cursor-pointer',
          'bg-gradient-to-br', optionStyles[idx % 4].bg,
          gameStore.selectedOptionId === opt.id ? 'ring-4 ring-white ring-offset-2 ring-offset-slate-950 scale-[1.01] shadow-2xl z-10' : '',
          gameStore.answerSubmitted && gameStore.selectedOptionId !== opt.id ? 'opacity-35 grayscale-40' : '',
          gameStore.timeLeftSecs <= 0 && !gameStore.answerSubmitted ? 'opacity-35 grayscale-50' : ''
        ]"
      >
        <!-- Top Glass Shine Reflection -->
        <div class="absolute inset-x-0 top-0 h-1/2 bg-gradient-to-b from-white/20 to-transparent pointer-events-none rounded-t-xl sm:rounded-t-2xl" />

        <!-- Geometry Shape Icon Badge -->
        <div
          class="w-9 h-9 sm:w-11 sm:h-11 rounded-lg sm:rounded-xl flex items-center justify-center text-sm sm:text-xl font-black border shadow-inner shrink-0"
          :class="optionStyles[idx % 4].iconBg"
        >
          {{ optionStyles[idx % 4].icon }}
        </div>

        <!-- Option Text -->
        <span class="flex-1 break-words leading-snug drop-shadow text-xs sm:text-base font-extrabold">
          {{ opt.text }}
        </span>

        <!-- Dynamic Selected Time Badge -->
        <div
          v-if="gameStore.selectedOptionId === opt.id"
          class="flex items-center space-x-1 px-2.5 py-1 rounded-full bg-white text-slate-950 font-black shadow-lg text-xs shrink-0"
        >
          <span class="text-emerald-700">✓</span>
          <span v-if="gameStore.currentQuestionResponseTimeMs">
            {{ (gameStore.currentQuestionResponseTimeMs / 1000).toFixed(1) }}s
          </span>
        </div>
      </button>
    </div>
  </div>
</template>
