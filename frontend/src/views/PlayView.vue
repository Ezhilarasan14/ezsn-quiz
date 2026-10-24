<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';
import { useGameStore } from '@/stores/game';
import { useBrandingStore, MAIN_BRAND_LOGO } from '@/stores/branding';
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
const queryEmail = computed(() => ((route.query.email as string) || '').trim());

// If nickname is in URL or already set in store, player is already logged in
const isAlreadyLoggedIn = computed(() => {
  return !!queryNick.value || !!gameStore.nickname;
});

const userIdentifier = ref(
  localStorage.getItem('ezsn_player_email') || localStorage.getItem('ezsn_player_name') || ''
);
const errorMessage = ref('');
const isJoining = ref(false);

onMounted(() => {
  const qPin = (route.query.pin as string) || '';
  if (qPin) {
    pin.value = qPin.trim().replace(/\D/g, '');
  }

  // If already logged in via query params, auto-join immediately!
  if (pin.value && queryNick.value) {
    gameStore.joinAsPlayer(pin.value, queryNick.value, queryEmail.value || undefined);
  }
});

onUnmounted(() => {
  gameStore.disconnect();
});

function handleQuickJoin() {
  errorMessage.value = '';
  const cleanPin = pin.value.trim().replace(/\D/g, '');
  const input = userIdentifier.value.trim();

  if (!cleanPin || cleanPin.length < 4) {
    errorMessage.value = 'Please provide a valid 6-digit Game PIN';
    return;
  }

  if (!input) {
    errorMessage.value = 'Please enter your email or username to continue';
    return;
  }

  let finalNickname = input;
  let finalEmail = '';

  if (input.includes('@')) {
    finalEmail = input;
    const prefix = input.split('@')[0];
    finalNickname = prefix.charAt(0).toUpperCase() + prefix.slice(1);
    localStorage.setItem('ezsn_player_email', finalEmail);
    localStorage.setItem('ezsn_player_name', finalNickname);
  } else {
    finalNickname = input;
    localStorage.setItem('ezsn_player_name', finalNickname);
  }

  isJoining.value = true;
  gameStore.joinAsPlayer(cleanPin, finalNickname, finalEmail || undefined);
}
</script>

<template>
  <div class="min-h-[calc(100vh-4rem)] flex flex-col justify-between relative pb-6">
    <!-- CASE 1: Player is already logged in -->
    <template v-if="isAlreadyLoggedIn">
      <!-- 1a: Connected to WebSocket: Show active game screens -->
      <template v-if="gameStore.isConnected">
        <!-- Persistent Organization Banner for Customer Player -->
        <header
          v-if="brandingStore.currentTheme.logo_url || (brandingStore.currentTheme.org_name && brandingStore.currentTheme.org_name !== 'EZSN Quiz')"
          class="pt-3 px-4 max-w-lg mx-auto w-full"
        >
          <div class="glass-panel flex items-center justify-between p-3 rounded-2xl border border-white/10 shadow-lg">
            <div class="flex items-center space-x-3 min-w-0">
              <img
                v-if="brandingStore.currentTheme.logo_url"
                :src="brandingStore.currentTheme.logo_url"
                alt="Organization Logo"
                class="h-9 w-auto max-w-[110px] object-contain rounded-lg p-1 bg-white/10 shrink-0"
              />
              <div class="min-w-0">
                <span class="text-[9px] font-extrabold uppercase tracking-widest text-indigo-400 block">Hosted by</span>
                <h2 class="text-xs sm:text-sm font-black text-white truncate">
                  {{ brandingStore.currentTheme.org_name }}
                </h2>
              </div>
            </div>

            <div class="text-right shrink-0">
              <span class="text-[9px] font-bold uppercase text-slate-400 block">Game PIN</span>
              <span class="text-sm sm:text-base font-mono font-black text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-pink-400">
                {{ gameStore.roomPin || pin }}
              </span>
            </div>
          </div>
        </header>

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
          <div class="glass-panel pointer-events-auto rounded-2xl py-2.5 px-4 flex items-center justify-between shadow-2xl border border-white/10">
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
              <span class="glass-pill px-2.5 py-0.5 rounded-lg text-slate-300 font-mono text-xs font-bold">
                PIN: {{ gameStore.roomPin || pin }}
              </span>
              <span class="glass-pill px-2 py-0.5 rounded-lg text-indigo-300 font-mono text-xs font-bold">
                <span class="inline-block w-1.5 h-1.5 rounded-full bg-indigo-400 animate-pulse mr-1" />
                Live
              </span>
            </div>
          </div>
        </footer>
      </template>

      <!-- 1b: Connecting state -->
      <template v-else>
        <main class="flex-1 flex flex-col items-center justify-center py-16 sm:py-24 space-y-6 text-center px-4">
          <!-- Organization Logo & Name in connecting view -->
          <div
            v-if="brandingStore.currentTheme.logo_url || (brandingStore.currentTheme.org_name && brandingStore.currentTheme.org_name !== 'EZSN Quiz')"
            class="flex flex-col items-center space-y-2 mb-2"
          >
            <img
              v-if="brandingStore.currentTheme.logo_url"
              :src="brandingStore.currentTheme.logo_url"
              alt="Organization Logo"
              class="h-14 sm:h-16 w-auto max-w-[180px] object-contain rounded-2xl p-2 glass-card shadow-2xl"
            />
            <div class="space-y-0.5">
              <span class="text-[10px] font-extrabold uppercase tracking-widest text-indigo-400 block">Hosted by</span>
              <h2 class="text-base sm:text-lg font-black text-white">
                {{ brandingStore.currentTheme.org_name }}
              </h2>
            </div>
          </div>

          <div class="w-14 h-14 rounded-2xl glass-panel flex items-center justify-center shadow-xl border border-white/15">
            <div class="w-7 h-7 border-3 border-indigo-400 border-t-transparent rounded-full animate-spin" />
          </div>
          <div class="space-y-1">
            <h2 class="text-xl font-bold text-white">Connecting to Competition...</h2>
            <p class="text-slate-400 text-sm">
              Logging in as <span class="text-indigo-400 font-semibold">{{ queryNick || gameStore.nickname }}</span> for Room {{ pin }}
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
              />
              <img
                v-else
                :src="MAIN_BRAND_LOGO"
                alt="EZSN Quiz Logo"
                class="h-14 w-auto object-contain filter drop-shadow-xl"
              />
            </div>

            <div v-if="pin" class="inline-flex items-center space-x-2 px-3.5 py-1 rounded-full glass-pill text-indigo-300 text-xs font-black uppercase tracking-wider">
              <span>Room PIN: {{ pin }}</span>
            </div>

            <h1 class="text-3xl sm:text-4xl font-black text-white tracking-tight">
              Join Competition
            </h1>
            <p class="text-slate-400 text-xs sm:text-sm">
              Enter your email or username to jump into the quiz arena!
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
                <label for="roomPin" class="block text-xs font-bold uppercase tracking-wider text-slate-300">
                  Game PIN
                </label>
                <div class="relative">
                  <i class="pi pi-hashtag absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
                  <InputText
                    id="roomPin"
                    v-model="pin"
                    placeholder="e.g. 826896"
                    class="glass-input w-full pl-12 py-3.5 text-center font-mono font-black text-2xl tracking-widest rounded-xl"
                    maxlength="6"
                    required
                  />
                </div>
              </div>

              <!-- Single User Identifier Input: Email or Username -->
              <div class="space-y-1.5">
                <label for="identifier" class="block text-xs font-bold uppercase tracking-wider text-slate-300">
                  Email or Username <span class="text-rose-400">*</span>
                </label>
                <div class="relative">
                  <i class="pi pi-user absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
                  <InputText
                    id="identifier"
                    v-model="userIdentifier"
                    placeholder="Enter name (e.g. Alex) or Email"
                    class="glass-input w-full pl-12 py-3.5 rounded-xl font-bold text-base placeholder:text-slate-600 placeholder:text-sm"
                    maxlength="50"
                    autofocus
                    required
                  />
                </div>
                <p class="text-[11px] text-slate-500 leading-tight px-1">
                  💡 Entering an email preserves your lifetime points and past quiz rankings!
                </p>
              </div>

              <!-- Join Button -->
              <Button
                type="submit"
                label="Join Competition"
                icon="pi pi-bolt"
                iconPos="right"
                :loading="isJoining"
                class="w-full bg-gradient-to-r from-indigo-600 via-indigo-500 to-pink-600 hover:from-indigo-500 hover:to-pink-500 border border-white/20 font-black text-white py-3.5 rounded-xl shadow-xl shadow-indigo-600/30 text-base active:scale-98 transition-all"
              />
            </form>
          </div>
        </div>
      </main>
    </template>
  </div>
</template>
