<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '@/services/api';
import type { PlayerStats } from '@/types/game';
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
  <div class="max-w-4xl mx-auto px-4 py-6 sm:py-10 space-y-8">
    <!-- Header -->
    <div class="glass-panel rounded-2xl p-4 sm:p-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight">Records & Player Analytics</h1>
        <p class="text-xs sm:text-sm text-purple-300/80 mt-0.5">Track your historical quiz performance, trophies, and past session podiums</p>
      </div>

      <router-link to="/">
        <Button label="Back to Arena" icon="pi pi-arrow-left" text class="glass-pill text-purple-200 hover:text-white px-4 py-2" />
      </router-link>
    </div>

    <!-- Tab Selector -->
    <div class="flex items-center space-x-2 border-b border-purple-400/20 pb-1">
      <button
        @click="activeTab = 'player'"
        :class="[
          'px-5 py-2.5 font-bold text-xs sm:text-sm rounded-xl transition-all flex items-center space-x-2',
          activeTab === 'player'
            ? 'glass-panel text-white border border-purple-400/40 shadow-lg shadow-purple-900/30'
            : 'text-purple-300/70 hover:text-white hover:bg-purple-500/10'
        ]"
      >
        <i class="pi pi-user text-purple-300" />
        <span>My Player Profile & Stats</span>
      </button>

      <button
        @click="activeTab = 'rooms'"
        :class="[
          'px-5 py-2.5 font-bold text-xs sm:text-sm rounded-xl transition-all flex items-center space-x-2',
          activeTab === 'rooms'
            ? 'glass-panel text-white border border-purple-400/40 shadow-lg shadow-purple-900/30'
            : 'text-purple-300/70 hover:text-white hover:bg-purple-500/10'
        ]"
      >
        <i class="pi pi-history text-purple-300" />
        <span>Completed Quiz Rooms</span>
      </button>
    </div>

    <!-- TAB 1: PLAYER PROFILE & STATS -->
    <div v-if="activeTab === 'player'" class="space-y-6">
      <!-- Search Input Card -->
      <div class="glass-panel rounded-3xl p-6 sm:p-7 shadow-xl">
        <form @submit.prevent="lookupPlayerStats" class="space-y-3">
          <label class="text-xs font-bold uppercase tracking-wider text-purple-200 block">
            Look Up Lifetime Stats by Email
          </label>
          <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
            <div class="relative flex-1">
              <i class="pi pi-envelope absolute left-4 top-1/2 -translate-y-1/2 text-purple-400 text-base" />
              <InputText
                v-model="searchEmail"
                type="email"
                placeholder="Enter email used during quiz (e.g. alex@example.com)"
                class="glass-input w-full pl-12 py-3 text-white font-bold text-sm rounded-xl placeholder:text-purple-300/30"
                required
              />
            </div>
            <Button
              type="submit"
              label="Look Up"
              icon="pi pi-search"
              :loading="isSearchingPlayer"
              class="bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-black text-white px-6 py-3 rounded-xl shadow-lg shadow-purple-900/40"
            />
          </div>
          <p class="text-[11px] text-purple-300/70">
            Matches the email you entered when scanning the room QR code or joining a game.
          </p>
        </form>
      </div>

      <!-- Player Search Error -->
      <Message v-if="playerSearchError" severity="info" :closable="false" class="rounded-xl">
        {{ playerSearchError }}
      </Message>

      <!-- Player Stats Dashboard -->
      <div v-if="playerStats" class="space-y-6">
        <!-- Overview Header Card -->
        <div class="glass-panel rounded-3xl p-6 sm:p-8 shadow-2xl border border-purple-400/30 flex flex-col sm:flex-row items-center justify-between gap-6 relative overflow-hidden">
          <div class="absolute -top-20 -left-20 w-60 h-60 bg-purple-600/20 rounded-full blur-3xl pointer-events-none" />

          <div class="flex items-center space-x-4 relative">
            <div class="w-16 h-16 sm:w-18 sm:h-18 rounded-2xl bg-gradient-to-tr from-purple-600 to-pink-500 flex items-center justify-center text-white font-black text-2xl shadow-xl shadow-purple-900/40 shrink-0">
              {{ playerStats.nickname.slice(0, 2).toUpperCase() }}
            </div>
            <div>
              <div class="flex items-center space-x-2">
                <h2 class="text-2xl sm:text-3xl font-black text-white tracking-tight">{{ playerStats.nickname }}</h2>
                <span class="px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 font-bold text-xs border border-emerald-500/30">
                  Verified
                </span>
              </div>
              <p class="text-xs text-purple-300/80 font-mono mt-0.5">{{ playerStats.email }}</p>
            </div>
          </div>

          <div class="text-center sm:text-right relative">
            <span class="text-xs uppercase font-extrabold text-purple-300/80 block tracking-widest">Total Lifetime Points</span>
            <span class="text-3xl sm:text-4xl font-mono font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-300 via-fuchsia-300 to-pink-300">
              {{ playerStats.total_score }} pts
            </span>
          </div>
        </div>

        <!-- 4 Metric Glass Cards -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div class="glass-card rounded-2xl p-4 sm:p-5 text-center space-y-1.5">
            <span class="text-2xl block">🎮</span>
            <span class="text-2xl sm:text-3xl font-black text-white font-mono block">{{ playerStats.quizzes_played }}</span>
            <span class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-purple-300/80 block">Quizzes Played</span>
          </div>

          <div class="glass-card rounded-2xl p-4 sm:p-5 text-center space-y-1.5">
            <span class="text-2xl block">🥇</span>
            <span class="text-2xl sm:text-3xl font-black text-amber-400 font-mono block">{{ playerStats.wins_count }}</span>
            <span class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-purple-300/80 block">1st Place Wins</span>
          </div>

          <div class="glass-card rounded-2xl p-4 sm:p-5 text-center space-y-1.5">
            <span class="text-2xl block">🏆</span>
            <span class="text-2xl sm:text-3xl font-black text-purple-300 font-mono block">{{ playerStats.podium_count }}</span>
            <span class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-purple-300/80 block">Podium Finishes</span>
          </div>

          <div class="glass-card rounded-2xl p-4 sm:p-5 text-center space-y-1.5">
            <span class="text-2xl block">🎯</span>
            <span class="text-2xl sm:text-3xl font-black text-emerald-400 font-mono block">{{ playerStats.avg_accuracy }}%</span>
            <span class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-purple-300/80 block">Accuracy Rate</span>
          </div>
        </div>

        <!-- History Itemized Table -->
        <div class="glass-panel rounded-3xl overflow-hidden shadow-2xl">
          <div class="p-5 border-b border-white/10 flex items-center justify-between">
            <h3 class="text-base font-black text-white">Quiz Participation Matches</h3>
            <span class="glass-pill px-3 py-1 rounded-full text-xs text-slate-300 font-bold">
              {{ playerStats.history.length }} Matches
            </span>
          </div>

          <div class="divide-y divide-white/5">
            <div
              v-for="item in playerStats.history"
              :key="item.game_id"
              class="p-4 sm:p-5 flex items-center justify-between hover:bg-white/[0.02] transition-colors"
            >
              <div class="space-y-1">
                <h4 class="font-extrabold text-white text-sm sm:text-base">{{ item.quiz_title }}</h4>
                <p class="text-xs text-slate-400 font-mono">
                  {{ new Date(item.played_at).toLocaleDateString() }} • Correct Answers: {{ item.correct_count }}/{{ item.total_answered }}
                </p>
              </div>

              <div class="flex items-center space-x-3 text-right">
                <span
                  :class="[
                    'px-2.5 py-0.5 rounded-full text-xs font-black',
                    item.rank === 1 ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30' :
                    item.rank === 2 ? 'bg-slate-400/20 text-slate-300 border border-slate-400/30' :
                    item.rank === 3 ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30' :
                    'glass-pill text-slate-400'
                  ]"
                >
                  Rank #{{ item.rank }}
                </span>
                <span class="text-base sm:text-lg font-black text-white font-mono">
                  {{ item.score }} pts
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 2: ALL COMPLETED ROOMS -->
    <div v-else-if="activeTab === 'rooms'" class="space-y-4">
      <div class="glass-panel rounded-3xl overflow-hidden shadow-2xl">
        <div class="p-5 border-b border-white/10 flex items-center justify-between">
          <h3 class="text-base font-black text-white">All Completed Competitions</h3>
          <span class="glass-pill px-3 py-1 rounded-full text-xs text-slate-300 font-bold">
            {{ history.length }} Completed Sessions
          </span>
        </div>

        <div v-if="history.length > 0" class="divide-y divide-white/5">
          <div
            v-for="session in history"
            :key="session.room_id"
            class="p-4 sm:p-5 flex items-center justify-between hover:bg-white/[0.02] transition-colors"
          >
            <div>
              <h4 class="font-extrabold text-white text-base">{{ session.quiz_title }}</h4>
              <p class="text-xs text-slate-400 mt-0.5">
                {{ new Date(session.completed_at).toLocaleString() }}
              </p>
            </div>

            <span class="glass-pill px-3.5 py-1 rounded-full text-xs font-bold text-indigo-300 border border-indigo-500/30">
              {{ session.participant_count }} Contestants
            </span>
          </div>
        </div>
        <div v-else class="text-center py-12 text-slate-500">
          No completed sessions found.
        </div>
      </div>
    </div>
  </div>
</template>
