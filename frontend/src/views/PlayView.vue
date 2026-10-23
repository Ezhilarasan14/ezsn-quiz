<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';
import { useGameStore } from '@/stores/game';
import { useBrandingStore, MAIN_BRAND_LOGO } from '@/stores/branding';
import PlayerLobbyView from '@/components/game/PlayerLobbyView.vue';
import PlayerController from '@/components/game/PlayerController.vue';
import AnswerRevealView from '@/components/game/AnswerRevealView.vue';
import PodiumView from '@/components/game/PodiumView.vue';

import Card from 'primevue/card';
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
  <div class="min-h-[calc(100vh-4rem)] flex flex-col justify-between">
    <!-- CASE 1: Player is already logged in -->
    <template v-if="isAlreadyLoggedIn">
      <!-- 1a: Connected to WebSocket: Show active game screens -->
      <template v-if="gameStore.isConnected">
        <main class="flex-1">
          <PlayerLobbyView v-if="gameStore.gameState === 'LOBBY'" />
          <PlayerController v-else-if="gameStore.gameState === 'QUESTION'" />
          <AnswerRevealView v-else-if="gameStore.gameState === 'REVEAL'" />
          <PodiumView v-else-if="gameStore.gameState === 'PODIUM'" />
        </main>

        <!-- Bottom Player Status Bar -->
        <footer
          v-if="gameStore.gameState !== 'PODIUM'"
          class="bg-slate-900/90 border-t border-slate-800 py-3 px-4 flex items-center justify-between text-xs font-bold text-slate-300 max-w-4xl mx-auto w-full"
        >
          <div class="flex items-center space-x-2">
            <div class="w-2.5 h-2.5 rounded-full" :class="gameStore.isConnected ? 'bg-emerald-400' : 'bg-rose-400 animate-ping'" />
            <span>{{ gameStore.nickname || queryNick }}</span>
          </div>

          <div class="flex items-center space-x-4">
            <span>PIN: {{ gameStore.roomPin || pin }}</span>
            <span class="text-indigo-400 font-extrabold">{{ gameStore.score }} pts</span>
          </div>
        </footer>
      </template>

      <!-- 1b: Connecting state -->
      <template v-else>
        <main class="flex-1 flex flex-col items-center justify-center py-24 space-y-4 text-center px-4">
          <div class="w-12 h-12 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
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
      <main class="flex-1 flex flex-col justify-center items-center py-12 px-4 relative overflow-hidden">
        <!-- Ambient background glow -->
        <div class="absolute -top-32 -left-32 w-80 h-80 bg-indigo-600/20 rounded-full blur-3xl pointer-events-none" />
        <div class="absolute -bottom-32 -right-32 w-80 h-80 bg-pink-600/20 rounded-full blur-3xl pointer-events-none" />

        <div class="w-full max-w-md z-10 space-y-6">
          <!-- Logo & Header -->
          <div class="text-center space-y-2">
            <div class="flex justify-center mb-2">
              <img
                v-if="brandingStore.currentTheme.logo_url"
                :src="brandingStore.currentTheme.logo_url"
                alt="Organization Logo"
                class="h-14 w-auto max-w-[180px] object-contain rounded-xl p-1 bg-slate-900/80 border border-white/10"
              />
              <img
                v-else
                :src="MAIN_BRAND_LOGO"
                alt="EZSN Quiz Logo"
                class="h-14 w-auto object-contain filter drop-shadow-lg"
              />
            </div>

            <div v-if="pin" class="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/30 text-indigo-400 text-xs font-black uppercase tracking-wider">
              <span>Room PIN: {{ pin }}</span>
            </div>

            <h1 class="text-3xl sm:text-4xl font-black text-white tracking-tight">
              Join Competition
            </h1>
            <p class="text-slate-400 text-xs sm:text-sm">
              Enter your email or username to continue.
            </p>
          </div>

          <!-- Join Form Card -->
          <Card class="bg-slate-900/90 border border-slate-800 rounded-3xl shadow-2xl backdrop-blur-xl">
            <template #content>
              <form @submit.prevent="handleQuickJoin" class="space-y-4 p-2">
                <Message v-if="errorMessage" severity="error" :closable="false" class="rounded-xl">
                  {{ errorMessage }}
                </Message>

                <!-- PIN input (only shown if not in URL) -->
                <div v-if="!pin" class="space-y-1.5">
                  <label for="roomPin" class="block text-xs font-bold uppercase tracking-wider text-slate-400">
                    Game PIN
                  </label>
                  <div class="relative">
                    <i class="pi pi-hashtag absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
                    <InputText
                      id="roomPin"
                      v-model="pin"
                      placeholder="e.g. 826896"
                      class="w-full pl-12 py-3.5 text-center font-mono font-black text-2xl tracking-widest bg-slate-950/80 border-slate-700 focus:border-indigo-500 rounded-xl text-white"
                      maxlength="6"
                      required
                    />
                  </div>
                </div>

                <!-- Single User Identifier Input: Email or Username -->
                <div class="space-y-1.5">
                  <label for="identifier" class="block text-xs font-bold uppercase tracking-wider text-slate-400">
                    Email or Username <span class="text-rose-400">*</span>
                  </label>
                  <div class="relative">
                    <i class="pi pi-user absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
                    <InputText
                      id="identifier"
                      v-model="userIdentifier"
                      placeholder="e.g. Alex or alex@example.com"
                      class="w-full pl-12 py-3.5 bg-slate-950/80 border-slate-700 focus:border-indigo-500 rounded-xl text-white font-bold text-base placeholder:text-slate-600 placeholder:text-sm"
                      maxlength="50"
                      autofocus
                      required
                    />
                  </div>
                  <p class="text-[11px] text-slate-500 leading-tight">
                    💡 Entering an email saves your lifetime points and previous quiz rankings!
                  </p>
                </div>

                <!-- Join Button -->
                <Button
                  type="submit"
                  label="Join Competition 🚀"
                  :loading="isJoining"
                  class="w-full bg-gradient-to-r from-indigo-600 to-pink-600 hover:from-indigo-500 hover:to-pink-500 border-none font-black text-white py-3.5 rounded-xl shadow-lg shadow-indigo-500/20 text-base"
                />
              </form>
            </template>
          </Card>
        </div>
      </main>
    </template>
  </div>
</template>
