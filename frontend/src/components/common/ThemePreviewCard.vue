<script setup lang="ts">
import Card from 'primevue/card';
import Button from 'primevue/button';
import ProgressBar from 'primevue/progressbar';

defineProps<{
  theme: {
    org_name?: string;
    logo_url?: string | null;
    primary_color: string;
    secondary_color: string;
    background_color: string;
    button_color: string;
  };
}>();
</script>

<template>
  <div class="rounded-2xl p-6 border border-slate-700/60 shadow-xl transition-all duration-300"
       :style="{ backgroundColor: theme.background_color }">
    <!-- Header simulation -->
    <div class="flex items-center justify-between pb-4 border-b border-white/10 mb-6">
      <div class="flex items-center space-x-3">
        <img
          v-if="theme.logo_url"
          :src="theme.logo_url"
          alt="Preview Logo"
          class="h-7 w-auto object-contain rounded"
        />
        <div
          v-else
          class="w-7 h-7 rounded-lg flex items-center justify-center font-bold text-xs text-white"
          :style="{ backgroundColor: theme.primary_color }"
        >
          EZ
        </div>
        <span class="font-bold text-sm tracking-tight text-white">
          {{ theme.org_name || 'Organization' }}
        </span>
      </div>

      <span class="text-xs font-semibold px-2 py-0.5 rounded-full bg-white/10 text-white/80">
        PIN: 482910
      </span>
    </div>

    <!-- Question simulation card -->
    <Card class="bg-white/5 border border-white/10 backdrop-blur-md rounded-xl shadow-md text-white mb-6">
      <template #content>
        <div class="space-y-4">
          <div class="flex justify-between items-center text-xs text-white/70">
            <span>Question 3 of 10</span>
            <span class="font-mono font-bold" :style="{ color: theme.secondary_color }">15s</span>
          </div>

          <ProgressBar :value="60" :showValue="false" class="h-1.5 rounded-full bg-white/10">
            <div class="h-full rounded-full transition-all" :style="{ width: '60%', backgroundColor: theme.secondary_color }" />
          </ProgressBar>

          <p class="font-extrabold text-base leading-snug">
            Which element has the chemical symbol 'Au'?
          </p>
        </div>
      </template>
    </Card>

    <!-- 4 simulated option buttons -->
    <div class="grid grid-cols-2 gap-3 mb-6">
      <button class="py-2.5 px-3 rounded-lg text-xs font-bold bg-rose-600/90 text-white flex items-center space-x-2">
        <span>▲</span> <span>Silver</span>
      </button>
      <button class="py-2.5 px-3 rounded-lg text-xs font-bold text-white flex items-center space-x-2 shadow-lg"
              :style="{ backgroundColor: theme.button_color }">
        <span>◆</span> <span>Gold (Selected)</span>
      </button>
      <button class="py-2.5 px-3 rounded-lg text-xs font-bold bg-amber-600/90 text-white flex items-center space-x-2">
        <span>●</span> <span>Copper</span>
      </button>
      <button class="py-2.5 px-3 rounded-lg text-xs font-bold bg-emerald-600/90 text-white flex items-center space-x-2">
        <span>■</span> <span>Iron</span>
      </button>
    </div>

    <!-- Action button -->
    <div class="flex justify-end">
      <Button
        label="Primary Action Button"
        icon="pi pi-check"
        size="small"
        class="border-none text-white font-bold text-xs px-4 py-2 rounded-lg shadow-md"
        :style="{ backgroundColor: theme.button_color }"
      />
    </div>
  </div>
</template>
