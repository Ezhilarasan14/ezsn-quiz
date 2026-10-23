<script setup lang="ts">
import { useGameStore } from '@/stores/game';
import Button from 'primevue/button';
import Message from 'primevue/message';
import ProgressBar from 'primevue/progressbar';

const gameStore = useGameStore();

const optionStyles = [
  { icon: '▲', color: 'bg-rose-600' },
  { icon: '◆', color: 'bg-blue-600' },
  { icon: '●', color: 'bg-amber-500' },
  { icon: '■', color: 'bg-emerald-600' },
];

function isCorrect(optionId: string): boolean {
  return gameStore.correctOptionIds.includes(optionId);
}

function getCount(optionId: string): number {
  return gameStore.optionCounts[optionId] || 0;
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-6 py-6 px-4">
    <!-- Player feedback banner (if playing) -->
    <div v-if="gameStore.role === 'player' && gameStore.lastAnswerCorrect !== null" class="animate-fade-in">
      <Message
        :severity="gameStore.lastAnswerCorrect ? 'success' : 'error'"
        :icon="gameStore.lastAnswerCorrect ? 'pi pi-check' : 'pi pi-times'"
        :closable="false"
        class="rounded-2xl p-4 shadow-xl border"
      >
        <div class="flex items-center justify-between w-full">
          <div>
            <h3 class="text-xl font-black">
              {{ gameStore.lastAnswerCorrect ? 'Correct!' : 'Incorrect!' }}
            </h3>
            <p v-if="gameStore.lastAnswerCorrect" class="text-sm font-semibold opacity-90">
              +{{ gameStore.lastPointsAwarded }} points
            </p>
          </div>
          <div class="text-right">
            <span class="text-xs uppercase tracking-wider block opacity-75">Current Score</span>
            <span class="text-2xl font-black">{{ gameStore.score }}</span>
          </div>
        </div>
      </Message>
    </div>

    <!-- Question Title -->
    <div class="text-center">
      <h2 class="text-2xl sm:text-3xl font-extrabold text-white">
        {{ gameStore.questionText }}
      </h2>
    </div>

    <!-- Options Breakdown Bars -->
    <div class="space-y-3">
      <div
        v-for="(opt, idx) in gameStore.options"
        :key="opt.id"
        :class="[
          'rounded-2xl p-4 border transition-all duration-300',
          isCorrect(opt.id)
            ? 'bg-emerald-950/60 border-emerald-500 ring-2 ring-emerald-500/50'
            : 'bg-slate-900/60 border-slate-800 opacity-60'
        ]"
      >
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center space-x-3">
            <span :class="['w-7 h-7 rounded-lg text-white font-black text-xs flex items-center justify-center', optionStyles[idx % 4].color]">
              {{ optionStyles[idx % 4].icon }}
            </span>
            <span class="font-bold text-white text-base">
              {{ opt.text }}
            </span>
          </div>

          <div class="flex items-center space-x-2">
            <span v-if="isCorrect(opt.id)" class="px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 font-black text-xs">
              ✓ Correct
            </span>
            <span class="font-extrabold text-sm text-slate-300">
              {{ getCount(opt.id) }} votes
            </span>
          </div>
        </div>

        <!-- Progress visual for votes -->
        <ProgressBar
          :value="gameStore.playerCount > 0 ? (getCount(opt.id) / gameStore.playerCount) * 100 : 0"
          :showValue="false"
          class="h-2 rounded-full bg-slate-800"
        >
          <div
            class="h-full rounded-full transition-all duration-500"
            :class="isCorrect(opt.id) ? 'bg-emerald-500' : 'bg-slate-600'"
            :style="{ width: `${gameStore.playerCount > 0 ? (getCount(opt.id) / gameStore.playerCount) * 100 : 0}%` }"
          />
        </ProgressBar>
      </div>
    </div>

    <!-- Host Control: Next Question -->
    <div v-if="gameStore.role === 'host'" class="flex justify-end pt-4">
      <Button
        label="Next Question"
        icon="pi pi-arrow-right"
        iconPos="right"
        size="large"
        class="bg-indigo-600 hover:bg-indigo-500 border-none font-bold text-white px-8 py-3 rounded-xl shadow-lg shadow-indigo-600/20"
        @click="gameStore.nextQuestion"
      />
    </div>
  </div>
</template>
