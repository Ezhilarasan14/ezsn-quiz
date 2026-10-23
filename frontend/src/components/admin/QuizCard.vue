<script setup lang="ts">
import type { QuizSummary } from '@/types/quiz';
import Card from 'primevue/card';
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
  <Card class="bg-slate-900/80 border border-slate-800 hover:border-slate-700 rounded-2xl shadow-xl transition-all duration-200 overflow-hidden flex flex-col justify-between">
    <template #content>
      <div class="space-y-4">
        <!-- Header Info -->
        <div class="flex items-start justify-between gap-2">
          <div class="space-y-1">
            <h3 class="font-extrabold text-lg text-white line-clamp-1">
              {{ quiz.title }}
            </h3>
            <p class="text-xs text-slate-400 line-clamp-2">
              {{ quiz.description || 'No description provided' }}
            </p>
          </div>

          <!-- Custom Theme Badge if set -->
          <div v-if="quiz.theme_primary_color" class="flex items-center space-x-1 shrink-0" title="Custom theme applied">
            <span class="w-3.5 h-3.5 rounded-full border border-white/20" :style="{ backgroundColor: quiz.theme_primary_color }" />
            <span class="text-[10px] font-bold text-slate-400">Custom</span>
          </div>
        </div>

        <!-- Meta metrics -->
        <div class="flex items-center space-x-4 text-xs font-semibold text-slate-400 pt-2 border-t border-slate-800/60">
          <span class="flex items-center space-x-1.5">
            <i class="pi pi-question-circle text-indigo-400" />
            <span>{{ quiz.question_count }} Questions</span>
          </span>
          <span class="flex items-center space-x-1.5">
            <i class="pi pi-calendar text-slate-500" />
            <span>{{ new Date(quiz.created_at).toLocaleDateString() }}</span>
          </span>
        </div>

        <!-- Actions -->
        <div class="pt-3 grid grid-cols-2 gap-2">
          <Button
            label="Launch"
            icon="pi pi-play"
            size="small"
            class="bg-indigo-600 hover:bg-indigo-500 border-none font-bold text-white rounded-xl shadow-md"
            @click="emit('launch', quiz.id)"
          />
          <Button
            label="Edit"
            icon="pi pi-pencil"
            size="small"
            severity="secondary"
            class="rounded-xl font-bold"
            @click="emit('edit', quiz.id)"
          />
        </div>

        <div class="flex items-center justify-between pt-1">
          <Button
            label="Theme"
            icon="pi pi-palette"
            text
            size="small"
            class="text-xs font-semibold text-slate-400 hover:text-white"
            @click="emit('theme', quiz)"
          />

          <div class="flex items-center space-x-1">
            <Button
              icon="pi pi-copy"
              text
              rounded
              size="small"
              class="text-slate-400 hover:text-white"
              title="Duplicate"
              @click="emit('duplicate', quiz.id)"
            />
            <Button
              icon="pi pi-trash"
              text
              rounded
              size="small"
              severity="danger"
              title="Delete"
              @click="emit('delete', quiz.id)"
            />
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>
