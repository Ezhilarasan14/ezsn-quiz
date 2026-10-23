<script setup lang="ts">
import { computed } from 'vue';
import ProgressBar from 'primevue/progressbar';

const props = defineProps<{
  totalSecs: number;
  timeLeftSecs: number;
}>();

const percentage = computed(() => {
  if (props.totalSecs <= 0) return 0;
  return Math.max(0, Math.min(100, (props.timeLeftSecs / props.totalSecs) * 100));
});

const progressColorClass = computed(() => {
  if (props.timeLeftSecs <= 5) return 'bg-rose-500';
  if (props.timeLeftSecs <= 10) return 'bg-amber-500';
  return 'bg-indigo-500';
});
</script>

<template>
  <div class="w-full space-y-1">
    <div class="flex justify-between items-center text-sm font-semibold text-slate-400 px-1">
      <span>Time Remaining</span>
      <span
        :class="[
          'text-base font-extrabold px-2 py-0.5 rounded-full transition-colors',
          timeLeftSecs <= 5 ? 'bg-rose-500/20 text-rose-400 animate-pulse' : 'text-slate-200'
        ]"
      >
        {{ timeLeftSecs }}s
      </span>
    </div>

    <!-- PrimeVue ProgressBar -->
    <ProgressBar
      :value="percentage"
      :showValue="false"
      class="h-3 rounded-full bg-slate-800 overflow-hidden"
    >
      <div
        class="h-full transition-all duration-300 ease-linear rounded-full"
        :class="progressColorClass"
        :style="{ width: `${percentage}%` }"
      />
    </ProgressBar>
  </div>
</template>
