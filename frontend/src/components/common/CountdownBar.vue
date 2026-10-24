<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  totalSecs: number;
  timeLeftSecs: number;
}>();

const percentage = computed(() => {
  if (props.totalSecs <= 0) return 0;
  return Math.max(0, Math.min(100, (props.timeLeftSecs / props.totalSecs) * 100));
});

const progressGradientClass = computed(() => {
  if (props.timeLeftSecs <= 5) return 'bg-gradient-to-r from-rose-500 to-pink-600 shadow-lg shadow-rose-500/50';
  if (props.timeLeftSecs <= 10) return 'bg-gradient-to-r from-amber-400 to-orange-500 shadow-md shadow-amber-500/40';
  return 'bg-gradient-to-r from-cyan-400 via-indigo-500 to-purple-500 shadow-md shadow-indigo-500/30';
});
</script>

<template>
  <div class="w-full space-y-1.5">
    <div class="flex justify-between items-center text-xs font-bold text-slate-300 px-1">
      <span class="flex items-center space-x-1.5">
        <i class="pi pi-clock text-indigo-400 text-xs" />
        <span>Time Left</span>
      </span>
      <span
        :class="[
          'font-mono text-xs font-black px-2 py-0.5 rounded-full transition-all',
          timeLeftSecs <= 5
            ? 'bg-rose-500/20 text-rose-400 border border-rose-500/40 animate-pulse'
            : timeLeftSecs <= 10
              ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30'
              : 'glass-pill text-slate-200'
        ]"
      >
        {{ timeLeftSecs }}s
      </span>
    </div>

    <!-- Smooth Glass Progress Track -->
    <div class="w-full h-2.5 rounded-full bg-slate-950/80 p-0.5 border border-white/10 overflow-hidden shadow-inner">
      <div
        class="h-full transition-all duration-300 ease-linear rounded-full"
        :class="progressGradientClass"
        :style="{ width: `${percentage}%` }"
      />
    </div>
  </div>
</template>
