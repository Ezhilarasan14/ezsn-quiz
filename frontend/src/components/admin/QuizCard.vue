<script setup lang="ts">
import type { QuizSummary } from '@/types/quiz';
import Button from 'primevue/button';

defineProps<{
  quiz: QuizSummary;
}>();

const emit = defineEmits<{
  (e: 'launch', id: string): void;
  (e: 'edit', id: string): void;
  (e: 'theme', quiz: QuizSummary): void;
  (e: 'duplicate', id: string): void;
  (e: 'delete', id: string): void;
}>();
</script>

<template>
  <div class="glass-card glass-card-hover rounded-2xl p-5 flex flex-col justify-between space-y-4">
    <!-- Header Info -->
    <div class="space-y-2">
      <div class="flex items-start justify-between gap-2">
        <h3 class="font-black text-lg text-white line-clamp-1 group-hover:text-indigo-300 transition-colors">
          {{ quiz.title }}
        </h3>

        <!-- Custom Navbar Color Badge if set -->
        <div v-if="quiz.theme_primary_color" class="flex items-center space-x-1 shrink-0 glass-pill px-2 py-0.5 rounded-full" title="Custom theme applied">
          <span class="w-3 h-3 rounded-full border border-white/20" :style="{ backgroundColor: quiz.theme_primary_color }" />
          <span class="text-[10px] font-bold text-slate-300">Theme</span>
        </div>
      </div>

      <p class="text-xs text-slate-400 line-clamp-2 leading-relaxed">
        {{ quiz.description || 'No description provided for this quiz competition.' }}
      </p>
    </div>

    <!-- Meta metrics -->
    <div class="flex items-center space-x-4 text-xs font-semibold text-slate-400 pt-2 border-t border-white/5">
      <span class="flex items-center space-x-1.5">
        <i class="pi pi-question-circle text-indigo-400 text-xs" />
        <span class="text-slate-300">{{ quiz.question_count }} Questions</span>
      </span>
      <span class="flex items-center space-x-1.5">
        <i class="pi pi-calendar text-slate-500 text-xs" />
        <span>{{ new Date(quiz.created_at).toLocaleDateString() }}</span>
      </span>
    </div>

    <!-- Actions -->
    <div class="space-y-2 pt-1">
      <div class="grid grid-cols-2 gap-2">
        <Button
          label="Launch"
          icon="pi pi-play"
          size="small"
          class="bg-gradient-to-r from-emerald-600 to-teal-500 hover:from-emerald-500 hover:to-teal-400 border border-white/15 font-bold text-white rounded-xl shadow-lg shadow-emerald-500/20 active:scale-95 transition-all text-xs"
          @click="emit('launch', quiz.id)"
        />
        <Button
          label="Edit"
          icon="pi pi-pencil"
          size="small"
          class="glass-pill hover:bg-white/10 border border-white/10 font-bold text-slate-200 rounded-xl text-xs transition-colors"
          @click="emit('edit', quiz.id)"
        />
      </div>

      <div class="flex items-center justify-between pt-1">
        <Button
          label="Custom Theme"
          icon="pi pi-palette"
          text
          size="small"
          class="text-xs font-semibold text-slate-400 hover:text-indigo-300 p-1"
          @click="emit('theme', quiz)"
        />

        <div class="flex items-center space-x-1">
          <Button
            icon="pi pi-copy"
            text
            rounded
            size="small"
            class="text-slate-400 hover:text-white hover:bg-white/10 w-8 h-8"
            title="Duplicate Quiz"
            @click="emit('duplicate', quiz.id)"
          />
          <Button
            icon="pi pi-trash"
            text
            rounded
            size="small"
            class="text-rose-400 hover:text-rose-300 hover:bg-rose-500/10 w-8 h-8"
            title="Delete Quiz"
            @click="emit('delete', quiz.id)"
          />
        </div>
      </div>
    </div>
  </div>
</template>
