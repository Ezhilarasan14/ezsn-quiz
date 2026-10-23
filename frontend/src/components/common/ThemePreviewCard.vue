<script setup lang="ts">
defineProps<{
  theme: {
    org_name?: string;
    logo_url?: string | null;
    primary_color: string;
    secondary_color?: string;
    background_color?: string;
    button_color?: string;
  };
}>();
</script>

<template>
  <div class="rounded-2xl overflow-hidden border border-slate-800 bg-[#0B0F19] shadow-2xl">
    <!-- Navbar Live Preview -->
    <div
      class="px-4 py-3 border-b border-white/10 flex items-center justify-between transition-colors duration-200"
      :style="{ backgroundColor: theme.primary_color || '#0F172A' }"
    >
      <!-- Left Brand -->
      <div class="flex items-center space-x-2">
        <span class="text-sm font-black tracking-tight text-white">ezsnapps</span>
      </div>

      <!-- Center Org Badge -->
      <div class="flex items-center space-x-2 px-2.5 py-1 rounded-full bg-black/40 border border-white/15">
        <img
          v-if="theme.logo_url"
          :src="theme.logo_url"
          alt="Org Logo"
          class="h-4 w-auto max-w-[60px] object-contain rounded"
        />
        <div v-else class="w-3.5 h-3.5 rounded bg-indigo-500 flex items-center justify-center text-[9px] font-bold text-white">
          ★
        </div>
        <span class="text-xs font-bold text-white truncate max-w-[120px]">
          {{ theme.org_name || 'Organization Name' }}
        </span>
      </div>

      <!-- Right Action Dummy Buttons -->
      <div class="flex items-center space-x-2">
        <span class="text-[11px] font-semibold text-white/80 px-2 py-0.5 rounded bg-white/10">Join</span>
        <span class="text-[11px] font-bold text-white px-2.5 py-0.5 rounded bg-indigo-600">Admin</span>
      </div>
    </div>

    <!-- Page Body (Always Clean High-Contrast Dark) -->
    <div class="p-5 space-y-4">
      <div class="flex items-center justify-between text-xs text-slate-400">
        <span class="font-medium text-slate-300">Live Quiz View Simulation</span>
        <span class="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 font-semibold text-[10px] border border-emerald-500/20">
          Page Background: Clean High-Contrast
        </span>
      </div>

      <div class="rounded-xl p-4 bg-slate-900/90 border border-slate-800 space-y-2.5">
        <div class="flex justify-between items-center text-xs">
          <span class="text-indigo-400 font-bold">Question 1 of 5</span>
          <span class="font-mono text-slate-400">⏱️ 15s</span>
        </div>
        <p class="font-bold text-sm text-white">
          Which planet in our solar system is known as the Red Planet?
        </p>
      </div>

      <div class="grid grid-cols-2 gap-2 text-xs font-semibold text-white">
        <div class="p-2.5 rounded-lg bg-rose-600/90 flex items-center space-x-2">
          <span>▲</span> <span>Mars</span>
        </div>
        <div class="p-2.5 rounded-lg bg-blue-600/90 flex items-center space-x-2">
          <span>◆</span> <span>Jupiter</span>
        </div>
      </div>

      <p class="text-[11px] text-slate-500 text-center italic">
        Only the top Navbar adopts your custom color. Backgrounds and quiz typography stay dark and crisp.
      </p>
    </div>
  </div>
</template>
