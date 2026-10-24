<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import Button from 'primevue/button';

const gameStore = useGameStore();

const optionStyles = [
  { icon: '▲', color: 'from-rose-500 to-rose-600' },
  { icon: '◆', color: 'from-blue-500 to-indigo-600' },
  { icon: '●', color: 'from-amber-400 to-orange-500' },
  { icon: '■', color: 'from-emerald-400 to-teal-600' },
];

function isCorrect(optionId: string): boolean {
  return gameStore.correctOptionIds.includes(optionId);
}

function getCount(optionId: string): number {
  return gameStore.optionCounts[optionId] || 0;
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-6 py-4 sm:py-8 px-4">
    <!-- Player feedback banner (Shown only for mobile players) -->
    <div v-if="gameStore.role === 'player' && gameStore.lastAnswerCorrect !== null">
      <div
        class="glass-panel rounded-3xl p-5 sm:p-6 shadow-2xl border transition-all"
        :class="gameStore.lastAnswerCorrect
          ? 'border-emerald-500/40 bg-gradient-to-r from-emerald-950/70 via-slate-900/80 to-slate-900/90 shadow-emerald-950/50'
          : 'border-rose-500/40 bg-gradient-to-r from-rose-950/70 via-slate-900/80 to-slate-900/90 shadow-rose-950/50'"
      >
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center space-x-3.5">
            <div
              class="w-12 h-12 rounded-2xl flex items-center justify-center text-2xl font-black shadow-lg shrink-0"
              :class="gameStore.lastAnswerCorrect ? 'bg-emerald-500 text-slate-950 shadow-emerald-500/40' : 'bg-rose-500 text-white shadow-rose-500/40'"
            >
              {{ gameStore.lastAnswerCorrect ? '✓' : '✕' }}
            </div>

            <div>
              <h3 class="text-xl sm:text-2xl font-black text-white">
                {{ gameStore.lastAnswerCorrect ? 'Spot On! Correct' : 'Not Quite Right' }}
              </h3>
              <p v-if="gameStore.lastAnswerCorrect" class="text-xs sm:text-sm font-bold text-emerald-300 flex items-center space-x-1.5 mt-0.5">
                <span>+{{ gameStore.lastPointsAwarded }} pts</span>
                <span v-if="gameStore.streak > 1" class="glass-pill px-2 py-0.5 rounded-full text-amber-300 text-[11px] font-black">
                  🔥 {{ gameStore.streak }} Streak!
                </span>
              </p>
            </div>
          </div>

          <div class="text-right shrink-0">
            <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400 block">Total Score</span>
            <span class="text-xl sm:text-3xl font-mono font-black text-white">
              {{ gameStore.score }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Question Title Header -->
    <div class="text-center space-y-2">
      <span class="glass-pill px-3.5 py-1 rounded-full text-xs font-bold text-slate-300 uppercase tracking-wider">
        Answer Breakdown
      </span>
      <h2 class="text-xl sm:text-3xl font-black text-white tracking-tight">
        {{ gameStore.questionText }}
      </h2>
    </div>

    <!-- Options Breakdown Bars -->
    <div class="space-y-3 pt-2">
      <div
        v-for="(opt, idx) in gameStore.options"
        :key="opt.id"
        :class="[
          'glass-panel rounded-2xl p-4 sm:p-5 transition-all duration-300',
          isCorrect(opt.id)
            ? 'border-emerald-500/60 bg-emerald-950/40 ring-2 ring-emerald-500/30 shadow-lg shadow-emerald-950/40'
            : 'border-white/5 bg-slate-950/40 opacity-70'
        ]"
      >
        <div class="flex items-center justify-between mb-2.5">
          <div class="flex items-center space-x-3">
            <span
              class="w-7 h-7 sm:w-8 sm:h-8 rounded-lg text-white font-black text-xs sm:text-sm flex items-center justify-center bg-gradient-to-br shadow shrink-0"
              :class="optionStyles[idx % 4].color"
            >
              {{ optionStyles[idx % 4].icon }}
            </span>
            <span class="font-extrabold text-white text-sm sm:text-base">
              {{ opt.text }}
            </span>
          </div>

          <div class="flex items-center space-x-2 shrink-0">
            <span
              v-if="isCorrect(opt.id)"
              class="px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 font-black text-xs"
            >
              ✓ Correct
            </span>
            <span class="font-black text-xs sm:text-sm text-slate-300 font-mono">
              {{ getCount(opt.id) }} {{ getCount(opt.id) === 1 ? 'vote' : 'votes' }}
            </span>
          </div>
        </div>

        <!-- Visual Bar Track -->
        <div class="w-full h-2 rounded-full bg-slate-950/80 p-0.5 border border-white/5 overflow-hidden">
          <div
            class="h-full rounded-full transition-all duration-500"
            :class="isCorrect(opt.id) ? 'bg-gradient-to-r from-emerald-500 to-teal-400 shadow-sm shadow-emerald-500/50' : 'bg-slate-700'"
            :style="{ width: `${gameStore.playerCount > 0 ? (getCount(opt.id) / gameStore.playerCount) * 100 : 0}%` }"
          />
        </div>
      </div>
    </div>

    <!-- Host Control: Next Question Action -->
    <div v-if="gameStore.role === 'host'" class="flex justify-end pt-4">
      <Button
        label="Next Question"
        icon="pi pi-arrow-right"
        iconPos="right"
        size="large"
        class="bg-gradient-to-r from-indigo-600 to-pink-600 hover:from-indigo-500 hover:to-pink-500 border border-white/20 font-black text-white px-8 py-3.5 rounded-2xl shadow-xl shadow-indigo-600/30 active:scale-98 transition-all"
        @click="gameStore.nextQuestion"
      />
    </div>
  </div>
</template>
