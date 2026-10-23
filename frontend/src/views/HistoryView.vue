<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '@/services/api';
import Card from 'primevue/card';
import Button from 'primevue/button';

const history = ref<any[]>([]);
const isLoading = ref(true);

onMounted(async () => {
  try {
    history.value = await api.getHistory();
  } catch (e) {
    console.warn('Failed to load history:', e);
  } finally {
    isLoading.value = false;
  }
});
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-8 space-y-8">
    <div class="flex items-center justify-between pb-4 border-b border-slate-800">
      <div>
        <h1 class="text-3xl font-black text-white">Quiz Leaderboards & History</h1>
        <p class="text-xs text-slate-400">Preserved results and rankings from completed quiz sessions</p>
      </div>

      <router-link to="/">
        <Button label="Back to Game" icon="pi pi-arrow-left" text severity="secondary" />
      </router-link>
    </div>

    <Card class="bg-slate-900/80 border border-slate-800 rounded-2xl shadow-xl overflow-hidden">
      <template #content>
        <div v-if="isLoading" class="text-center py-12 text-slate-400">
          <i class="pi pi-spin pi-spinner text-3xl mb-2" />
          <p class="text-xs">Loading game history...</p>
        </div>

        <div v-else-if="history.length > 0" class="divide-y divide-slate-800">
          <div v-for="item in history" :key="item.room_id" class="p-5 flex items-center justify-between hover:bg-slate-800/30 transition-colors">
            <div class="space-y-1">
              <h3 class="font-extrabold text-white text-base">{{ item.quiz_title }}</h3>
              <div class="flex items-center space-x-3 text-xs text-slate-400">
                <span>Room {{ item.pin }}</span>
                <span>•</span>
                <span>{{ new Date(item.completed_at).toLocaleDateString() }}</span>
              </div>
            </div>

            <div class="flex items-center space-x-4">
              <div class="text-right">
                <span class="text-xs font-bold text-slate-400 block">Participants</span>
                <span class="text-base font-black text-indigo-400">{{ item.participant_count }}</span>
              </div>
            </div>
          </div>
        </div>

        <div v-else class="text-center py-16 text-slate-500 space-y-2">
          <i class="pi pi-trophy text-4xl text-slate-600 mb-2" />
          <p class="font-bold text-slate-300">No game records found yet</p>
          <p class="text-xs">Completed quizzes will record their podium and rankings here.</p>
        </div>
      </template>
    </Card>
  </div>
</template>
