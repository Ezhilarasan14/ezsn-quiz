<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import CountdownBar from '@/components/common/CountdownBar.vue';
import Card from 'primevue/card';

const gameStore = useGameStore();

const optionStyles = [
  { bg: 'bg-rose-600', icon: '▲', border: 'border-rose-500/30' },
  { bg: 'bg-blue-600', icon: '◆', border: 'border-blue-500/30' },
  { bg: 'bg-amber-500', icon: '●', border: 'border-amber-400/30' },
  { bg: 'bg-emerald-600', icon: '■', border: 'border-emerald-500/30' },
];
</script>

<template>
  <div class="max-w-5xl mx-auto space-y-6 py-4 px-4">
    <!-- Header with progress & live answer count -->
    <div class="flex items-center justify-between text-slate-300">
      <div class="flex items-center space-x-2">
        <span class="px-3 py-1 rounded-full bg-indigo-500/20 text-indigo-400 font-extrabold text-sm border border-indigo-500/30">
          Question {{ (gameStore.questionIndex || 0) + 1 }} of {{ gameStore.totalQuestions || 1 }}
        </span>
      </div>

      <div class="flex items-center space-x-2 bg-slate-900/80 px-4 py-1.5 rounded-full border border-slate-800 text-sm font-semibold">
        <i class="pi pi-check-circle text-emerald-400" />
        <span>{{ gameStore.answeredCount }} / {{ gameStore.playerCount }} Answered</span>
      </div>
    </div>

    <!-- Countdown Bar -->
    <CountdownBar
      :totalSecs="gameStore.timeLimitSecs"
      :timeLeftSecs="gameStore.timeLeftSecs"
    />

    <!-- Question Card -->
    <Card class="bg-slate-900/90 border border-slate-800 rounded-3xl shadow-2xl backdrop-blur-xl text-center py-6 px-4">
      <template #content>
        <h2 class="text-2xl sm:text-4xl font-black text-white leading-tight">
          {{ gameStore.questionText || 'Loading question...' }}
        </h2>
      </template>
    </Card>

    <!-- 4 Options Grid (Host Display) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2">
      <div
        v-for="(opt, idx) in gameStore.options"
        :key="opt.id"
        :class="[
          'rounded-2xl p-5 text-white font-extrabold text-lg sm:text-xl flex items-center space-x-4 shadow-lg transition-transform',
          optionStyles[idx % 4].bg,
          optionStyles[idx % 4].border,
        ]"
      >
        <span class="text-2xl font-black opacity-90 w-8 text-center">
          {{ optionStyles[idx % 4].icon }}
        </span>
        <span class="flex-1 break-words">
          {{ opt.text }}
        </span>
      </div>
    </div>
  </div>
</template>
