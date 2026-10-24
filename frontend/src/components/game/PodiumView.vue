<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useGameStore } from '@/stores/game';
import { useBrandingStore } from '@/stores/branding';
import confetti from 'canvas-confetti';
import Button from 'primevue/button';

const gameStore = useGameStore();
const brandingStore = useBrandingStore();
const logoFailed = ref(false);

watch(
  () => brandingStore.currentTheme.logo_url,
  () => {
    logoFailed.value = false;
  }
);

// Default to 'review' tab for players with summary, or 'leaderboard' for host
const activeTab = ref<'review' | 'leaderboard'>(
  gameStore.playerSummary ? 'review' : 'leaderboard'
);

const summary = computed(() => gameStore.playerSummary);

onMounted(() => {
  // Joyful confetti burst for celebration
  confetti({
    particleCount: 120,
    spread: 80,
    origin: { y: 0.6 },
  });

  setTimeout(() => {
    confetti({
      particleCount: 80,
      angle: 60,
      spread: 55,
      origin: { x: 0 },
    });
    confetti({
      particleCount: 80,
      angle: 120,
      spread: 55,
      origin: { x: 1 },
    });
  }, 400);

  if (gameStore.playerSummary) {
    activeTab.value = 'review';
  } else {
    activeTab.value = 'leaderboard';
  }
});

function getPedestal(rank: number) {
  if (rank === 1) {
    return {
      height: 'h-40 sm:h-52',
      bg: 'bg-gradient-to-t from-amber-600 via-amber-500 to-yellow-300 border-t-2 border-amber-200 shadow-2xl shadow-amber-500/30',
      textColor: 'text-slate-950',
      badge: '👑 1st',
    };
  }
  if (rank === 2) {
    return {
      height: 'h-32 sm:h-40',
      bg: 'bg-gradient-to-t from-slate-600 via-slate-500 to-slate-300 border-t-2 border-slate-200 shadow-xl shadow-slate-500/20',
      textColor: 'text-slate-950',
      badge: '🥈 2nd',
    };
  }
  return {
    height: 'h-24 sm:h-32',
    bg: 'bg-gradient-to-t from-amber-800 via-amber-700 to-orange-500 border-t-2 border-amber-300 shadow-xl shadow-orange-700/20',
    textColor: 'text-amber-100',
    badge: '🥉 3rd',
  };
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-8 py-4 sm:py-8 px-4 text-center">
    <!-- Simplified, Compact Results Header -->
    <header class="space-y-3 pb-1">
      <h1 class="text-3xl sm:text-5xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-300 via-fuchsia-300 to-pink-300 tracking-tight">
        Final Quiz Results
      </h1>

      <!-- Host attribution badge (Organization & Campus are already pinned in top header) -->
      <div v-if="brandingStore.currentTheme.host_name" class="flex items-center justify-center">
        <div class="inline-flex items-center space-x-2 px-4 py-1.5 rounded-full glass-pill border border-amber-400/30 text-xs sm:text-sm text-purple-200 shadow-sm">
          <div class="w-4 h-4 rounded-full bg-amber-500/25 border border-amber-400/30 flex items-center justify-center text-amber-300 text-[10px]">
            <i class="pi pi-user text-[10px]" />
          </div>
          <span>Host: <strong class="text-amber-200 font-extrabold">{{ brandingStore.currentTheme.host_name }}</strong></span>
        </div>
      </div>

      <p class="text-purple-200/75 text-xs sm:text-sm max-w-md mx-auto pt-0.5">
        Review your question-by-question performance, response times, and final standings below!
      </p>
    </header>

    <!-- PLAYER PERSONAL SUMMARY STATS CARDS -->
    <div v-if="summary" class="space-y-4">
      <!-- Big Hero Result Banner -->
      <div class="glass-panel rounded-3xl p-5 sm:p-6 shadow-2xl border border-purple-400/20 relative overflow-hidden">
        <div class="absolute -right-8 -top-8 w-36 h-36 bg-gradient-to-br from-purple-600/25 to-pink-600/25 rounded-full blur-2xl pointer-events-none" />

        <div class="flex flex-col sm:flex-row items-center justify-between gap-4 text-center sm:text-left">
          <div class="flex items-center space-x-4">
            <div
              :class="[
                'w-16 h-16 sm:w-20 sm:h-20 rounded-2xl flex items-center justify-center font-black text-2xl sm:text-3xl shadow-xl shrink-0',
                summary.final_rank === 1 ? 'bg-amber-400/20 text-amber-300 border-2 border-amber-400/40 ring-4 ring-amber-400/20' :
                summary.final_rank === 2 ? 'bg-purple-300/20 text-purple-200 border-2 border-purple-300/40 ring-4 ring-purple-300/20' :
                summary.final_rank === 3 ? 'bg-orange-500/20 text-orange-400 border-2 border-orange-500/40 ring-4 ring-orange-500/20' :
                'glass-pill text-purple-200 border border-purple-400/30'
              ]"
            >
              {{ summary.final_rank === 1 ? '👑' : summary.final_rank === 2 ? '🥈' : summary.final_rank === 3 ? '🥉' : `#${summary.final_rank}` }}
            </div>
            <div class="space-y-1">
              <span class="text-xs font-black uppercase tracking-wider text-purple-300 block">Your Final Standing</span>
              <h2 class="text-2xl sm:text-3xl font-black text-white">
                Rank #{{ summary.final_rank }} <span class="text-purple-300/70 text-base sm:text-lg font-bold">of {{ summary.total_players }}</span>
              </h2>
              <p class="text-xs text-purple-200">
                Playing as <span class="text-white font-extrabold">{{ gameStore.nickname }}</span>
              </p>
            </div>
          </div>

          <div class="text-center sm:text-right">
            <span class="text-xs font-bold uppercase text-purple-300/80 block">Total Score</span>
            <span class="text-3xl sm:text-4xl font-mono font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-300 via-fuchsia-300 to-pink-300">
              {{ summary.total_score }} <span class="text-base text-purple-300/80">pts</span>
            </span>
          </div>
        </div>
      </div>

      <!-- 4 Quick Metric Cards -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <!-- 1. Accuracy -->
        <div class="glass-card rounded-2xl p-4 text-center space-y-1 border border-purple-400/20">
          <span class="text-[10px] font-black uppercase tracking-wider text-purple-300/80 block">Accuracy</span>
          <span class="text-2xl sm:text-3xl font-mono font-black text-emerald-400 block">
            {{ summary.accuracy_percentage }}%
          </span>
          <span class="text-[11px] text-purple-300/80 font-bold block">
            {{ summary.correct_count }} / {{ summary.total_questions }} Correct
          </span>
        </div>

        <!-- 2. Incorrect Answers -->
        <div class="glass-card rounded-2xl p-4 text-center space-y-1 border border-purple-400/20">
          <span class="text-[10px] font-black uppercase tracking-wider text-purple-300/80 block">Incorrect</span>
          <span class="text-2xl sm:text-3xl font-mono font-black text-rose-400 block">
            {{ summary.incorrect_count }}
          </span>
          <span class="text-[11px] text-purple-300/80 font-bold block">
            {{ summary.incorrect_count === 0 ? 'Flawless!' : 'Missed Questions' }}
          </span>
        </div>

        <!-- 3. Total Time -->
        <div class="glass-card rounded-2xl p-4 text-center space-y-1 border border-purple-400/20">
          <span class="text-[10px] font-black uppercase tracking-wider text-purple-300/80 block">Total Time</span>
          <span class="text-2xl sm:text-3xl font-mono font-black text-cyan-300 block">
            {{ (summary.total_time_ms / 1000).toFixed(1) }}s
          </span>
          <span class="text-[11px] text-purple-300/80 font-bold block">
            Quiz Completion Time
          </span>
        </div>

        <!-- 4. Speed / Avg per Question -->
        <div class="glass-card rounded-2xl p-4 text-center space-y-1 border border-purple-400/20">
          <span class="text-[10px] font-black uppercase tracking-wider text-purple-300/80 block">Avg Response</span>
          <span class="text-2xl sm:text-3xl font-mono font-black text-amber-400 block">
            {{ summary.total_questions > 0 ? (summary.total_time_ms / (summary.total_questions * 1000)).toFixed(1) : 0 }}s
          </span>
          <span class="text-[11px] text-purple-300/80 font-bold block">
            Seconds per Question
          </span>
        </div>
      </div>
    </div>

    <!-- TABS NAVIGATION: Answer Review vs Leaderboard -->
    <div class="flex items-center justify-center space-x-2 pt-2">
      <button
        v-if="summary"
        @click="activeTab = 'review'"
        :class="[
          'px-5 py-2.5 rounded-2xl text-xs sm:text-sm font-black transition-all flex items-center space-x-2 shadow-lg',
          activeTab === 'review'
            ? 'bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 text-white shadow-purple-900/40 ring-2 ring-purple-300/30'
            : 'glass-panel text-purple-300/80 hover:text-white'
        ]"
      >
        <i class="pi pi-check-circle" />
        <span>Answer Review ({{ summary.reviews.length }})</span>
      </button>

      <button
        @click="activeTab = 'leaderboard'"
        :class="[
          'px-5 py-2.5 rounded-2xl text-xs sm:text-sm font-black transition-all flex items-center space-x-2 shadow-lg',
          activeTab === 'leaderboard'
            ? 'bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 text-white shadow-purple-900/40 ring-2 ring-purple-300/30'
            : 'glass-panel text-purple-300/80 hover:text-white'
        ]"
      >
        <i class="pi pi-trophy" />
        <span>Final Leaderboard ({{ gameStore.fullResults.length }})</span>
      </button>
    </div>

    <!-- TAB 1: QUESTION-BY-QUESTION FINAL ANSWER REVIEW -->
    <div v-if="activeTab === 'review' && summary" class="space-y-4 text-left">
      <div class="flex items-center justify-between px-2">
        <h3 class="text-base sm:text-lg font-black text-white">
          Question-by-Question Review
        </h3>
        <span class="text-xs text-purple-300/80 font-bold">
          Detailed response times & correct solutions
        </span>
      </div>

      <div class="space-y-3.5">
        <div
          v-for="item in summary.reviews"
          :key="item.question_id"
          class="glass-panel rounded-3xl p-4 sm:p-6 shadow-xl border border-purple-400/20 space-y-3"
        >
          <!-- Top Row: Question # + Accuracy Status + Response Time -->
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <div class="flex items-center space-x-2">
              <span class="glass-pill px-2.5 py-0.5 rounded-lg text-purple-200 font-mono text-xs font-bold">
                Question {{ item.question_index }}
              </span>

              <!-- Correct / Incorrect indicator -->
              <span
                v-if="item.is_correct"
                class="inline-flex items-center space-x-1.5 px-3 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 text-xs font-black"
              >
                <span>✓ Correct</span>
                <span class="text-[10px] text-emerald-400 font-mono">(+{{ item.points_awarded }} pts)</span>
              </span>
              <span
                v-else
                class="inline-flex items-center space-x-1.5 px-3 py-0.5 rounded-full bg-rose-500/20 text-rose-300 border border-rose-500/30 text-xs font-black"
              >
                <span>✕ Incorrect</span>
                <span class="text-[10px] text-rose-400 font-mono">(0 pts)</span>
              </span>
            </div>

            <!-- Time Taken by Player -->
            <div class="flex items-center space-x-1.5 glass-pill px-3 py-1 rounded-xl text-xs font-mono font-black text-cyan-300 border border-cyan-400/30 bg-cyan-950/40 shadow-sm">
              <i class="pi pi-clock text-cyan-400 text-xs" />
              <span>Time: {{ item.selected_option_id ? `${(item.response_time_ms / 1000).toFixed(1)}s` : `${(item.time_limit_sec || 5).toFixed(1)}s (Timed Out)` }}</span>
            </div>
          </div>

          <!-- Question Text -->
          <h4 class="text-base sm:text-lg font-black text-white leading-snug">
            {{ item.question_text }}
          </h4>

          <!-- Answer Comparison Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 pt-1">
            <!-- Player's Selected Answer -->
            <div
              :class="[
                'p-3.5 rounded-2xl border transition-all text-sm font-bold flex flex-col justify-between space-y-1',
                item.is_correct
                  ? 'bg-emerald-950/40 border-emerald-500/30 text-emerald-200'
                  : 'bg-rose-950/40 border-rose-500/30 text-rose-200'
              ]"
            >
              <div class="flex items-center justify-between text-[10px] font-black uppercase tracking-wider">
                <span :class="item.is_correct ? 'text-emerald-400' : 'text-rose-400'">Your Selected Answer</span>
                <span class="font-mono text-cyan-300 font-extrabold">⏱ {{ (item.response_time_ms / 1000).toFixed(1) }}s</span>
              </div>
              <span class="text-sm font-extrabold text-white">
                {{ item.selected_option_text || 'No Answer (Timed Out)' }}
              </span>
            </div>

            <!-- Correct Answer -->
            <div class="p-3.5 rounded-2xl bg-emerald-950/20 border border-emerald-500/30 text-emerald-200 text-sm font-bold flex flex-col justify-between space-y-1">
              <div class="flex items-center justify-between text-[10px] font-black uppercase tracking-wider text-emerald-400">
                <span>Correct Answer</span>
                <span>★ Solution</span>
              </div>
              <span class="text-sm font-extrabold text-emerald-100">
                {{ item.correct_option_text }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 2: FINAL LEADERBOARD & PODIUM -->
    <div v-show="activeTab === 'leaderboard'" class="space-y-8">
      <!-- Top 3 Glass Pedestals Podium Visual -->
      <div v-if="gameStore.podium.length > 0" class="flex items-end justify-center space-x-3 sm:space-x-6 pt-4 pb-2">
        <!-- 2nd Place -->
        <div v-if="gameStore.podium[1]" class="flex-1 max-w-[140px] sm:max-w-[170px] flex flex-col items-center">
          <div class="mb-2 text-center">
            <div class="w-12 h-12 sm:w-14 sm:h-14 rounded-2xl glass-panel border-2 border-slate-400 flex items-center justify-center font-black text-slate-200 text-sm sm:text-base mx-auto shadow-xl ring-2 ring-slate-400/20">
              {{ gameStore.podium[1].nickname.slice(0, 2).toUpperCase() }}
            </div>
            <span class="font-extrabold text-xs sm:text-sm text-slate-200 block truncate mt-1">
              {{ gameStore.podium[1].nickname }}
            </span>
            <span class="text-xs font-bold text-slate-400 font-mono">{{ gameStore.podium[1].score }} pts</span>
          </div>
          <div :class="['w-full rounded-t-3xl flex flex-col items-center justify-center p-3 font-black text-lg shadow-2xl relative overflow-hidden', getPedestal(2).bg, getPedestal(2).height, getPedestal(2).textColor]">
            <span class="text-2xl sm:text-3xl mb-1 drop-shadow">🥈</span>
            <span class="text-sm font-black uppercase tracking-wider">2nd</span>
          </div>
        </div>

        <!-- 1st Place (Center & Tallest) -->
        <div v-if="gameStore.podium[0]" class="flex-1 max-w-[160px] sm:max-w-[200px] flex flex-col items-center">
          <div class="mb-2 text-center">
            <div class="w-14 h-14 sm:w-18 sm:h-18 rounded-2xl bg-amber-500/20 border-4 border-amber-400 flex items-center justify-center font-black text-amber-300 text-lg sm:text-2xl mx-auto shadow-2xl ring-4 ring-amber-400/30 animate-pulse">
              {{ gameStore.podium[0].nickname.slice(0, 2).toUpperCase() }}
            </div>
            <span class="font-black text-sm sm:text-base text-white block truncate mt-1.5">
              {{ gameStore.podium[0].nickname }}
            </span>
            <span class="text-xs sm:text-sm font-black text-amber-400 font-mono">{{ gameStore.podium[0].score }} pts</span>
          </div>
          <div :class="['w-full rounded-t-3xl flex flex-col items-center justify-center p-3 font-black text-xl shadow-2xl relative overflow-hidden', getPedestal(1).bg, getPedestal(1).height, getPedestal(1).textColor]">
            <span class="text-3xl sm:text-4xl mb-1 drop-shadow-md">👑</span>
            <span class="text-base font-black uppercase tracking-wider">1st Place</span>
          </div>
        </div>

        <!-- 3rd Place -->
        <div v-if="gameStore.podium[2]" class="flex-1 max-w-[140px] sm:max-w-[170px] flex flex-col items-center">
          <div class="mb-2 text-center">
            <div class="w-12 h-12 sm:w-14 sm:h-14 rounded-2xl glass-panel border-2 border-amber-600 flex items-center justify-center font-black text-amber-300 text-sm sm:text-base mx-auto shadow-xl ring-2 ring-amber-600/20">
              {{ gameStore.podium[2].nickname.slice(0, 2).toUpperCase() }}
            </div>
            <span class="font-extrabold text-xs sm:text-sm text-slate-200 block truncate mt-1">
              {{ gameStore.podium[2].nickname }}
            </span>
            <span class="text-xs font-bold text-amber-500 font-mono">{{ gameStore.podium[2].score }} pts</span>
          </div>
          <div :class="['w-full rounded-t-3xl flex flex-col items-center justify-center p-3 font-black text-lg shadow-2xl relative overflow-hidden', getPedestal(3).bg, getPedestal(3).height, getPedestal(3).textColor]">
            <span class="text-2xl sm:text-3xl mb-1 drop-shadow">🥉</span>
            <span class="text-sm font-black uppercase tracking-wider">3rd</span>
          </div>
        </div>
      </div>

      <!-- Complete Results Glass Table -->
      <div class="glass-panel rounded-3xl overflow-hidden shadow-2xl text-left">
        <div class="p-5 border-b border-purple-400/20 flex items-center justify-between">
          <div>
            <h3 class="text-lg font-black text-white">Full Leaderboard</h3>
            <p class="text-xs text-purple-300/80">Final ranking based on speed, score and accuracy</p>
          </div>
          <span class="glass-pill px-3 py-1 rounded-full text-xs text-purple-200 font-bold">
            {{ gameStore.fullResults.length }} Contestants
          </span>
        </div>

        <div class="divide-y divide-purple-400/10">
          <div
            v-for="entry in gameStore.fullResults"
            :key="entry.player_id"
            :class="[
              'flex items-center justify-between p-4 sm:p-5 transition-colors',
              entry.player_id === gameStore.playerId ? 'bg-purple-950/60 border-l-4 border-purple-400' : 'hover:bg-purple-500/5'
            ]"
          >
            <div class="flex items-center space-x-3.5">
              <span
                :class="[
                  'w-8 h-8 rounded-xl flex items-center justify-center font-black text-xs shrink-0',
                  entry.rank === 1 ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30' :
                  entry.rank === 2 ? 'bg-purple-300/20 text-purple-200 border border-purple-300/30' :
                  entry.rank === 3 ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30' :
                  'glass-pill text-purple-300/80'
                ]"
              >
                #{{ entry.rank }}
              </span>
              <div>
                <span class="font-extrabold text-white text-sm sm:text-base">
                  {{ entry.nickname }}
                </span>
                <span v-if="entry.player_id === gameStore.playerId" class="ml-2 text-xs text-emerald-400 font-extrabold px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/20">
                  You
                </span>
              </div>
            </div>

            <div class="flex items-center space-x-3">
              <span v-if="entry.total_time_ms" class="text-xs font-mono text-cyan-300 glass-pill px-2 py-0.5 rounded-lg">
                ⏱ {{ (entry.total_time_ms / 1000).toFixed(1) }}s
              </span>
              <span class="font-black text-white text-base sm:text-lg font-mono">
                {{ entry.score }} pts
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Actions & Return to Arena -->
    <div class="pt-4 space-y-4">
      <div class="flex flex-wrap items-center justify-center gap-3">
        <router-link to="/history">
          <Button
            label="View Match History"
            icon="pi pi-history"
            class="glass-panel hover:bg-purple-500/15 text-white font-bold px-6 py-3 rounded-2xl border border-purple-400/30 transition-all"
          />
        </router-link>

        <router-link to="/">
          <Button
            label="Play Again / Exit"
            icon="pi pi-home"
            class="bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-black text-white px-7 py-3 rounded-2xl shadow-xl shadow-purple-900/40 active:scale-98 transition-all"
            @click="gameStore.disconnect"
          />
        </router-link>
      </div>

      <!-- Platform Footer -->
      <div class="flex items-center justify-center space-x-1.5 text-xs text-purple-400/80 pt-2">
        <span>Powered by</span>
        <span class="font-bold text-white">ezsnapps<span class="text-purple-400">.quiz</span></span>
      </div>
    </div>
  </div>
</template>
