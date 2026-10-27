<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';
import { useGameStore } from '@/stores/game';
import { useBrandingStore } from '@/stores/branding';
import PlayerLobbyView from '@/components/game/PlayerLobbyView.vue';
import PlayerController from '@/components/game/PlayerController.vue';
import AnswerRevealView from '@/components/game/AnswerRevealView.vue';
import PodiumView from '@/components/game/PodiumView.vue';

import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Message from 'primevue/message';

const route = useRoute();
const gameStore = useGameStore();
const brandingStore = useBrandingStore();

const pin = ref((route.query.pin as string) || '');
const queryNick = computed(() => ((route.query.nickname as string) || '').trim());

// If nickname is in URL or already set in store, player is already logged in
const isAlreadyLoggedIn = computed(() => {
  return !!queryNick.value || !!gameStore.nickname;
});

const playerNickname = ref(localStorage.getItem('ezsn_player_name') || '');
const errorMessage = ref('');
const isJoining = ref(false);

onMounted(() => {
  const qPin = (route.query.pin as string) || '';
  if (qPin) {
    pin.value = qPin.trim().replace(/\D/g, '');
  }

  // Check saved session in sessionStorage for seamless recovery after reload/tab restore
  try {
    const savedStr = sessionStorage.getItem('ezsn_session');
    if (savedStr) {
      const saved = JSON.parse(savedStr);
      if (saved.role === 'player' && (!pin.value || saved.pin === pin.value) && saved.nickname) {
        pin.value = saved.pin;
        playerNickname.value = saved.nickname;
        gameStore.joinAsPlayer(saved.pin, saved.nickname, saved.playerEmail, saved.playerId);
        return;
      }
    }
  } catch (e) {
    // ignore
  }

  // If already logged in via query params, auto-join immediately!
  if (pin.value && queryNick.value) {
    gameStore.joinAsPlayer(pin.value, queryNick.value);
  }
});

onUnmounted(() => {
  gameStore.disconnect();
});

function handleQuickJoin() {
  errorMessage.value = '';
  const cleanPin = pin.value.trim().replace(/\D/g, '');
  const cleanNick = playerNickname.value.trim();

  if (!cleanPin || cleanPin.length < 4) {
    errorMessage.value = 'Please provide a valid 6-digit Game PIN';
    return;
  }

  if (!cleanNick) {
    errorMessage.value = 'Please choose a nickname to enter the competition';
    return;
  }

  localStorage.setItem('ezsn_player_name', cleanNick);

  isJoining.value = true;
  gameStore.joinAsPlayer(cleanPin, cleanNick);
}
</script>

<template>
  <div class="min-h-[calc(100vh-4rem)] flex flex-col justify-between relative pb-6">
    <!-- CASE 1: Player is already logged in -->
    <template v-if="isAlreadyLoggedIn">
      <!-- 1a: Connected to WebSocket: Show active game screens -->
      <template v-if="gameStore.isConnected">
        <main class="flex-1 pb-16">
          <PlayerLobbyView v-if="gameStore.gameState === 'LOBBY'" />
          <PlayerController v-else-if="gameStore.gameState === 'QUESTION'" />
          <AnswerRevealView v-else-if="gameStore.gameState === 'REVEAL'" />
          <PodiumView v-else-if="gameStore.gameState === 'PODIUM'" />
        </main>

        <!-- Floating Glassmorphic Player Status Bar -->
        <footer
          v-if="gameStore.gameState !== 'PODIUM'"
          class="fixed bottom-4 inset-x-0 z-30 px-4 max-w-lg mx-auto w-full pointer-events-none"
        >
          <div class="glass-panel pointer-events-auto rounded-2xl py-2.5 px-4 flex items-center justify-between shadow-2xl border border-purple-400/20">
            <div class="flex items-center space-x-2.5">
              <div class="relative flex items-center justify-center">
                <div class="w-2.5 h-2.5 rounded-full" :class="gameStore.isConnected ? 'bg-emerald-400' : 'bg-rose-400 animate-ping'" />
                <div v-if="gameStore.isConnected" class="absolute -inset-0.5 rounded-full bg-emerald-400/40 animate-ping" />
              </div>
              <span class="font-bold text-sm text-white truncate max-w-[130px] sm:max-w-[180px]">
                {{ gameStore.nickname || queryNick }}
              </span>
            </div>

            <div class="flex items-center space-x-2.5">
              <span class="glass-pill px-2.5 py-0.5 rounded-lg text-purple-200 font-mono text-xs font-bold">
                PIN: {{ gameStore.roomPin || pin }}
              </span>
              <span class="glass-pill px-2 py-0.5 rounded-lg text-purple-300 font-mono text-xs font-bold">
                <span class="inline-block w-1.5 h-1.5 rounded-full bg-purple-400 animate-pulse mr-1" />
                Live
              </span>
            </div>
          </div>
        </footer>
      </template>

      <!-- 1b: Connecting state -->
      <template v-else>
        <main class="flex-1 flex flex-col items-center justify-center py-16 sm:py-24 space-y-6 text-center px-4">
          <div class="w-14 h-14 rounded-2xl glass-panel flex items-center justify-center shadow-xl border border-purple-400/25">
            <div class="w-7 h-7 border-3 border-purple-400 border-t-transparent rounded-full animate-spin" />
          </div>
          <div class="space-y-1">
            <h2 class="text-xl font-bold text-white">Connecting to Competition...</h2>
            <p class="text-purple-200/80 text-sm">
              Logging in as <span class="text-purple-300 font-semibold">{{ queryNick || gameStore.nickname }}</span> for Room {{ pin }}
            </p>
          </div>
        </main>
      </template>
    </template>

    <!-- CASE 2: Needs username/email input before joining (No QR code!) -->
    <template v-else>
      <main class="flex-1 flex flex-col justify-center items-center py-8 sm:py-16 px-4">
        <div class="w-full max-w-md space-y-6">
          <!-- Logo & Header -->
          <div class="text-center space-y-2">
            <div class="flex justify-center mb-2">
              <img
                v-if="brandingStore.currentTheme.logo_url"
                :src="brandingStore.currentTheme.logo_url"
                alt="Organization Logo"
                class="h-14 w-auto max-w-[180px] object-contain rounded-xl p-1.5 glass-card"
                @error="(e: any) => e.target.style.display = 'none'"
              />
              <div
                v-else
                class="w-14 h-14 rounded-2xl glass-panel border border-purple-400/30 flex items-center justify-center text-2xl shadow-xl"
              >
                🎮
              </div>
            </div>

            <div v-if="pin" class="inline-flex items-center space-x-2 px-3.5 py-1 rounded-full glass-pill text-purple-200 text-xs font-black uppercase tracking-wider">
              <span>Room PIN: {{ pin }}</span>
            </div>

            <h1 class="text-3xl sm:text-4xl font-black text-white tracking-tight">
              Join Competition
            </h1>
            <p class="text-purple-200/80 text-xs sm:text-sm">
              Enter your screen name to jump directly into the quiz arena!
            </p>
          </div>

          <!-- Glassmorphism Join Card -->
          <div class="glass-panel rounded-3xl p-6 sm:p-8 space-y-5">
            <form @submit.prevent="handleQuickJoin" class="space-y-4">
              <Message v-if="errorMessage" severity="error" :closable="false" class="rounded-xl">
                {{ errorMessage }}
              </Message>

              <!-- PIN input (only shown if not in URL) -->
              <div v-if="!pin" class="space-y-1.5">
                <label for="roomPin" class="block text-xs font-bold uppercase tracking-wider text-purple-200">
                  Game PIN
                </label>
                <div class="relative">
                  <i class="pi pi-hashtag absolute left-4 top-1/2 -translate-y-1/2 text-purple-400 text-lg" />
                  <InputText
                    id="roomPin"
                    v-model="pin"
                    placeholder="e.g. 826896"
                    class="glass-input w-full pl-12 py-3.5 text-center font-mono font-black text-2xl tracking-widest rounded-xl text-white placeholder:text-purple-300/30"
                    maxlength="6"
                    required
                  />
                </div>
              </div>

              <!-- Nickname Input (No Email) -->
              <div class="space-y-1.5">
                <label for="identifier" class="block text-xs font-bold uppercase tracking-wider text-purple-200">
                  Your Nickname <span class="text-pink-400">*</span>
                </label>
                <div class="relative">
                  <i class="pi pi-user absolute left-4 top-1/2 -translate-y-1/2 text-purple-400 text-lg" />
                  <InputText
                    id="identifier"
                    v-model="playerNickname"
                    placeholder="Enter your screen name (e.g. Alex)"
                    class="glass-input w-full pl-12 py-3.5 rounded-xl font-bold text-base placeholder:text-purple-300/30 text-white"
                    maxlength="20"
                    autofocus
                    required
                  />
                </div>
              </div>

              <!-- Join Button -->
              <Button
                type="submit"
                label="Join Competition"
                icon="pi pi-bolt"
                iconPos="right"
                :loading="isJoining"
                class="w-full bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-black text-white py-3.5 rounded-xl shadow-xl shadow-purple-900/40 text-base active:scale-98 transition-all"
              />
            </form>
          </div>
        </div>
      </main>
    </template>
  </div>
</template>
