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
    icon: '▲',
  },
  {
    bg: 'from-blue-500 via-blue-600 to-indigo-700 shadow-blue-600/30',
    iconBg: 'bg-blue-900/40 border-blue-400/40 text-blue-200',
    icon: '◆',
  },
  {
    bg: 'from-amber-400 via-amber-500 to-orange-600 shadow-amber-500/30',
    iconBg: 'bg-amber-900/40 border-amber-300/40 text-amber-200',
    icon: '●',
  },
  {
    bg: 'from-emerald-400 via-emerald-500 to-teal-700 shadow-emerald-500/30',
    iconBg: 'bg-emerald-900/40 border-emerald-300/40 text-emerald-200',
    icon: '■',
  },
];
</script>

<template>
  <div class="max-w-5xl mx-auto space-y-6 py-4 sm:py-8 px-4">
    <!-- Header with progress, Organization Logo, & live answer count -->
    <div class="flex flex-wrap items-center justify-between gap-3 text-purple-200">
      <div class="flex items-center space-x-3">
        <span class="px-3.5 py-1 rounded-full glass-pill text-purple-200 font-extrabold text-xs sm:text-sm border border-purple-400/30">
          Question {{ gameStore.questionIndex || 1 }} of {{ gameStore.totalQuestions || 1 }}
        </span>

        <!-- Organization Logo Badge during Question -->
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt=""
          role="presentation"
          class="h-6 w-auto max-w-[100px] object-contain rounded opacity-90 hidden sm:inline"
          @error="(e: any) => e.target.style.display = 'none'"
        />
      </div>

      <div class="flex items-center space-x-2 glass-pill px-3.5 py-1.5 rounded-full text-xs sm:text-sm font-semibold">
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
    <div class="glass-panel rounded-3xl p-6 sm:p-10 text-center shadow-2xl relative overflow-hidden">
      <div class="absolute -top-24 left-1/2 -translate-x-1/2 w-80 h-32 bg-purple-600/25 blur-3xl pointer-events-none" />
      <h2 class="relative text-2xl sm:text-4xl md:text-5xl font-black text-white leading-tight tracking-tight">
        {{ gameStore.questionText || 'Loading question...' }}
      </h2>
    </div>

    <!-- 4 Options Grid (Responsive for mobile & host screen) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 sm:gap-4 pt-1">
      <div
        v-for="(opt, idx) in gameStore.options"
        :key="opt.id"
        :class="[
          'relative overflow-hidden rounded-xl sm:rounded-2xl p-3 sm:p-5 text-white font-extrabold text-sm sm:text-lg flex items-center space-x-3 sm:space-x-4 shadow-xl border border-white/20 transition-all bg-gradient-to-br',
          optionStyles[idx % 4].bg,
        ]"
      >
        <!-- Glass shine highlight -->
        <div class="absolute inset-x-0 top-0 h-1/2 bg-gradient-to-b from-white/20 to-transparent pointer-events-none rounded-t-xl sm:rounded-t-2xl" />

        <!-- Geometry symbol icon badge -->
        <div
          class="w-8 h-8 sm:w-11 sm:h-11 rounded-lg sm:rounded-xl flex items-center justify-center text-sm sm:text-xl font-black border shadow-inner shrink-0"
          :class="optionStyles[idx % 4].iconBg"
        >
          {{ optionStyles[idx % 4].icon }}
        </div>

        <span class="flex-1 break-words leading-snug drop-shadow text-xs sm:text-base">
          {{ opt.text }}
        </span>
      </div>
    </div>
  </div>
</template>
