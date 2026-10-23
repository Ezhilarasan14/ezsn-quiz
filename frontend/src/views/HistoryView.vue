<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '@/services/api';
import type { PlayerStats } from '@/types/game';
import Card from 'primevue/card';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Message from 'primevue/message';

const activeTab = ref<'player' | 'rooms'>('player');

// Player stats lookup
const searchEmail = ref(localStorage.getItem('ezsn_player_email') || '');
const playerStats = ref<PlayerStats | null>(null);
const isSearchingPlayer = ref(false);
const playerSearchError = ref('');

// Global room history
const history = ref<any[]>([]);
const isLoadingHistory = ref(true);

onMounted(async () => {
  // If user has an email stored, auto-load their stats!
  if (searchEmail.value.trim()) {
    lookupPlayerStats();
  }

  try {
    history.value = await api.getHistory();
  } catch (e) {
    console.warn('Failed to load history:', e);
  } finally {
    isLoadingHistory.value = false;
  }
});

async function lookupPlayerStats() {
  const email = searchEmail.value.trim();
  if (!email) return;

  isSearchingPlayer.value = true;
  playerSearchError.value = '';
  try {
    playerStats.value = await api.getPlayerStats(email);
  } catch (err: any) {
    playerStats.value = null;
    playerSearchError.value = 'No completed quiz history found for this email address.';
  } finally {
    isSearchingPlayer.value = false;
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-8 space-y-8">
    <!-- Header -->
    <div class="flex items-center justify-between pb-4 border-b border-slate-800">
      <div>
        <h1 class="text-3xl font-black text-white">Quiz Records & Player Analytics</h1>
        <p class="text-xs text-slate-400">Track your historical quiz performance, trophies, and past session podiums</p>
      </div>

      <router-link to="/">
        <Button label="Back to Game" icon="pi pi-arrow-left" text severity="secondary" />
      </router-link>
    </div>

    <!-- Tab Selector -->
    <div class="flex items-center space-x-2 border-b border-slate-800">
      <button
        @click="activeTab = 'player'"
        :class="[
          'px-5 py-3 font-bold text-sm border-b-2 transition-colors flex items-center space-x-2',
          activeTab === 'player' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'
        ]"
      >
        <i class="pi pi-user" />
        <span>My Player Profile & Stats</span>
      </button>

      <button
        @click="activeTab = 'rooms'"
        :class="[
          'px-5 py-3 font-bold text-sm border-b-2 transition-colors flex items-center space-x-2',
          activeTab === 'rooms' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'
        ]"
      >
        <i class="pi pi-history" />
        <span>Completed Quiz Rooms</span>
      </button>
    </div>

    <!-- TAB 1: PLAYER PROFILE & STATS -->
    <div v-if="activeTab === 'player'" class="space-y-6">
      <!-- Search Input Card -->
      <Card class="bg-slate-900/80 border border-slate-800 rounded-2xl shadow-xl">
        <template #content>
          <form @submit.prevent="lookupPlayerStats" class="space-y-3">
            <label class="text-xs font-bold uppercase tracking-wider text-slate-400 block">
              Look Up Player Stats by Email
            </label>
            <div class="flex items-center gap-3">
              <div class="relative flex-1">
                <i class="pi pi-envelope absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-base" />
                <InputText
                  v-model="searchEmail"
                  type="email"
                  placeholder="Enter email used during quiz (e.g. alex@example.com)"
                  class="w-full pl-12 py-2.5 bg-slate-950/70 border-slate-800 text-white font-medium text-sm rounded-xl focus:border-indigo-500"
                  required
                />
              </div>
              <Button
                type="submit"
                label="Search"
                icon="pi pi-search"
                :loading="isSearchingPlayer"
                class="bg-indigo-600 hover:bg-indigo-500 border-none font-bold text-white px-6 py-2.5 rounded-xl shadow-md"
              />
            </div>
            <p class="text-[11px] text-slate-500">
              Matches email provided when scanning QR code or entering a game room.
            </p>
          </form>
        </template>
      </Card>

      <!-- Player Search Error -->
      <Message v-if="playerSearchError" severity="info" :closable="false" class="rounded-xl">
        {{ playerSearchError }}
      </Message>

      <!-- Player Stats Dashboard -->
      <div v-if="playerStats" class="space-y-6">
        <!-- Overview Header Card -->
        <div class="p-6 rounded-3xl bg-gradient-to-r from-indigo-950/70 to-slate-900/90 border border-indigo-500/30 shadow-2xl flex flex-col sm:flex-row items-center justify-between gap-4">
          <div class="flex items-center space-x-4">
            <div class="w-16 h-16 rounded-2xl bg-gradient-to-tr from-indigo-500 to-pink-500 flex items-center justify-center text-white font-black text-2xl shadow-lg shadow-indigo-500/30">
              {{ playerStats.nickname.slice(0, 2).toUpperCase() }}
            </div>
            <div>
              <div class="flex items-center space-x-2">
                <h2 class="text-2xl font-black text-white">{{ playerStats.nickname }}</h2>
                <span class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 font-bold text-xs border border-emerald-500/30">
                  Verified Player
                </span>
              </div>
              <p class="text-xs text-slate-400 font-mono">{{ playerStats.email }}</p>
            </div>
          </div>

          <div class="text-center sm:text-right">
            <span class="text-xs uppercase font-extrabold text-slate-400 block tracking-widest">Total Lifetime Points</span>
            <span class="text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-300 to-pink-400">
              {{ playerStats.total_score }} pts
            </span>
          </div>
        </div>

        <!-- 4 Metric Cards -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div class="p-4 rounded-2xl bg-slate-900/80 border border-slate-800 text-center space-y-1">
            <span class="text-2xl">🎮</span>
            <span class="text-2xl font-black text-white block">{{ playerStats.quizzes_played }}</span>
            <span class="text-[11px] font-bold uppercase tracking-wider text-slate-400 block">Quizzes Played</span>
          </div>

          <div class="p-4 rounded-2xl bg-slate-900/80 border border-slate-800 text-center space-y-1">
            <span class="text-2xl">🥇</span>
            <span class="text-2xl font-black text-amber-400 block">{{ playerStats.wins_count }}</span>
            <span class="text-[11px] font-bold uppercase tracking-wider text-slate-400 block">1st Place Wins</span>
          </div>

          <div class="p-4 rounded-2xl bg-slate-900/80 border border-slate-800 text-center space-y-1">
            <span class="text-2xl">🏆</span>
            <span class="text-2xl font-black text-indigo-400 block">{{ playerStats.podium_count }}</span>
            <span class="text-[11px] font-bold uppercase tracking-wider text-slate-400 block">Podium Finishes</span>
          </div>

          <div class="p-4 rounded-2xl bg-slate-900/80 border border-slate-800 text-center space-y-1">
            <span class="text-2xl">🎯</span>
            <span class="text-2xl font-black text-emerald-400 block">{{ playerStats.avg_accuracy }}%</span>
            <span class="text-[11px] font-bold uppercase tracking-wider text-slate-400 block">Accuracy Rate</span>
          </div>
        </div>

        <!-- History Table -->
        <Card class="bg-slate-900/80 border border-slate-800 rounded-2xl shadow-xl overflow-hidden">
          <template #title>
            <div class="p-4 border-b border-slate-800 flex items-center justify-between">
              <h3 class="text-base font-bold text-white">Quiz Participation History</h3>
              <span class="text-xs text-slate-400 font-semibold">{{ playerStats.history.length }} games</span>
            </div>
          </template>
          <template #content>
            <div class="divide-y divide-slate-800/80">
              <div
                v-for="item in playerStats.history"
                :key="item.game_id"
                class="p-4 flex items-center justify-between hover:bg-slate-800/30 transition-colors"
              >
                <div class="space-y-1">
                  <div class="flex items-center space-x-2">
                    <span
                      :class="[
                        'text-xs font-black px-2 py-0.5 rounded-full',
                        item.rank === 1 ? 'bg-amber-500/20 text-amber-400 border border-amber-400/30' :
                        item.rank <= 3 ? 'bg-indigo-500/20 text-indigo-400' : 'bg-slate-800 text-slate-400'
                      ]"
                    >
                      #{{ item.rank }} of {{ item.total_players }}
                    </span>
                    <h4 class="font-extrabold text-white text-sm">{{ item.quiz_title }}</h4>
                  </div>
                  <p class="text-[11px] text-slate-400 font-mono">
                    Room {{ item.room_pin }} • {{ new Date(item.played_at).toLocaleDateString() }}
                  </p>
                </div>

                <div class="text-right">
                  <span class="font-black text-white text-base block">{{ item.score }} pts</span>
                  <span class="text-[10px] text-emerald-400 font-bold">
                    {{ item.correct_count }}/{{ item.total_answered }} correct
                  </span>
                </div>
              </div>
            </div>
          </template>
        </Card>
      </div>
    </div>

    <!-- TAB 2: GLOBAL ROOM HISTORY -->
    <div v-else-if="activeTab === 'rooms'" class="space-y-4">
      <Card class="bg-slate-900/80 border border-slate-800 rounded-2xl shadow-xl overflow-hidden">
        <template #content>
          <div v-if="isLoadingHistory" class="text-center py-12 text-slate-400">
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
                  <span v-if="item.winner_nickname" class="text-amber-400 font-bold">
                    🏆 Winner: {{ item.winner_nickname }} ({{ item.winner_score }} pts)
                  </span>
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
  </div>
</template>
