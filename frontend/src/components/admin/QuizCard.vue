<script setup lang="ts">
import type { QuizSummary } from '@/types/quiz';
import Button from 'primevue/button';

defineProps<{
  quiz: QuizSummary;
}>();

const emit = defineEmits<{
  (e: 'launch', id: string): void;
  (e: 'edit', id: string): void;
  (e: 'duplicate', id: string): void;
  (e: 'delete', id: string): void;
}>();
</script>

<template>
  <div class="glass-card glass-card-hover rounded-2xl p-5 flex flex-col justify-between space-y-4">
    <!-- Header Info -->
    <div class="space-y-2">
      <div class="flex items-start justify-between gap-2">
        <h3 class="font-black text-lg text-white line-clamp-1 group-hover:text-purple-200 transition-colors">
          {{ quiz.title }}
        </h3>
      </div>

      <p class="text-xs text-purple-300/80 line-clamp-2 leading-relaxed">
        {{ quiz.description || 'No description provided for this quiz competition.' }}
      </p>
    </div>

    <!-- Meta metrics -->
    <div class="flex items-center space-x-4 text-xs font-semibold text-purple-300/80 pt-2 border-t border-purple-400/15">
      <span class="flex items-center space-x-1.5">
        <i class="pi pi-question-circle text-purple-300 text-xs" />
        <span class="text-purple-200">{{ quiz.question_count }} Questions</span>
      </span>
      <span class="flex items-center space-x-1.5">
        <i class="pi pi-calendar text-purple-400 text-xs" />
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
          class="bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-bold text-white rounded-xl shadow-lg shadow-purple-900/40 active:scale-95 transition-all text-xs"
          @click="emit('launch', quiz.id)"
        />
        <Button
          label="Edit"
          icon="pi pi-pencil"
          size="small"
          class="glass-pill hover:bg-purple-500/15 border border-purple-400/25 font-bold text-purple-100 rounded-xl text-xs transition-colors"
          @click="emit('edit', quiz.id)"
        />
      </div>

      <div class="flex items-center justify-end space-x-2 pt-1 border-t border-purple-400/10">
        <Button
          label="Duplicate"
          icon="pi pi-copy"
          text
          size="small"
          class="text-xs font-semibold text-purple-300/80 hover:text-white hover:bg-purple-500/20 px-2.5 py-1 rounded-lg transition-colors"
          @click="emit('duplicate', quiz.id)"
        />
        <Button
          label="Delete"
          icon="pi pi-trash"
          text
          size="small"
          class="text-xs font-semibold text-rose-400 hover:text-rose-300 hover:bg-rose-500/15 px-2.5 py-1 rounded-lg transition-colors"
          @click="emit('delete', quiz.id)"
        />
      </div>
    </div>
  </div>
</template>
