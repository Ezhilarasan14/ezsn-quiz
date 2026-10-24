<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useGameStore } from '@/stores/game';
import { useBrandingStore, MAIN_BRAND_LOGO } from '@/stores/branding';
import QRCode from 'qrcode';
import Button from 'primevue/button';

const gameStore = useGameStore();
const brandingStore = useBrandingStore();
const copied = ref(false);
const qrCodeUrl = ref('');

onMounted(() => {
  if (gameStore.roomPin) {
    generateQr(gameStore.roomPin);
  }
});

watch(
  () => gameStore.roomPin,
  (pin) => {
    if (pin) {
      generateQr(pin);
    }
  }
);

async function generateQr(pin: string) {
  try {
    const origin = typeof window !== 'undefined' ? window.location.origin : 'https://quiz.ezsnapps.in';
    const joinUrl = `${origin}/play?pin=${pin}`;
    qrCodeUrl.value = await QRCode.toDataURL(joinUrl, {
      width: 380,
      margin: 1.5,
      color: {
        dark: '#070a13',
        light: '#ffffff',
      },
    });
  } catch (err) {
    console.error('Failed to generate QR code:', err);
  }
}

function copyPin() {
  navigator.clipboard.writeText(gameStore.roomPin);
  copied.value = true;
  setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-6 py-6 sm:py-8 px-4">
    <!-- Organization Header Banner (Shown while quiz starts) -->
    <div
      v-if="brandingStore.currentTheme.logo_url || brandingStore.currentTheme.org_name"
      class="glass-panel flex flex-col sm:flex-row items-center justify-between gap-4 p-4 rounded-2xl"
    >
      <div class="flex items-center space-x-3.5">
        <img
          v-if="brandingStore.currentTheme.logo_url"
          :src="brandingStore.currentTheme.logo_url"
          alt="Organization Logo"
          class="h-11 w-auto max-w-[160px] object-contain rounded-xl p-1.5 glass-card shadow"
        />
        <div>
          <span class="text-[10px] font-extrabold uppercase tracking-widest text-indigo-400 block">Hosted by</span>
          <h2 class="text-base sm:text-lg font-black text-white leading-tight">
            {{ brandingStore.currentTheme.org_name }}
          </h2>
        </div>
      </div>

      <!-- Main Platform Badge -->
      <div class="flex items-center space-x-2 text-xs text-slate-400 glass-pill px-3.5 py-1.5 rounded-full">
        <span class="text-[11px] font-semibold text-slate-400">Platform:</span>
        <img :src="MAIN_BRAND_LOGO" alt="EZSN Quiz" class="h-4 w-auto inline opacity-90" />
        <span class="font-bold text-slate-200">EZSN Quiz</span>
      </div>
    </div>

    <!-- Big Host Lobby Card: QR Code + Game PIN side-by-side (Host Projector View) -->
    <div class="glass-panel rounded-3xl p-6 sm:p-8 shadow-2xl relative overflow-hidden">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-8 items-center">
        <!-- Left: Big Sharp QR Code for Mobile Cameras -->
        <div class="flex flex-col items-center justify-center p-5 rounded-2xl glass-card">
          <div class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full bg-emerald-500/10 text-emerald-400 text-xs font-black uppercase tracking-wider mb-3 border border-emerald-500/25">
            <i class="pi pi-camera" />
            <span>Scan with phone camera to join</span>
          </div>

          <div class="bg-white p-3.5 rounded-2xl shadow-2xl">
            <img
              v-if="qrCodeUrl"
              :src="qrCodeUrl"
              alt="Scan to join quiz"
              class="w-48 h-48 sm:w-56 sm:h-56 object-contain rounded-lg"
            />
            <div v-else class="w-48 h-48 sm:w-56 sm:h-56 flex items-center justify-center text-slate-400 text-xs font-bold">
              Generating QR Code...
            </div>
          </div>

          <p class="text-xs text-slate-400 font-semibold mt-3 text-center">
            Point camera to open quiz arena instantly
          </p>
        </div>

        <!-- Right: Game PIN & Browser Join Option -->
        <div class="text-center md:text-left space-y-4">
          <div class="space-y-1">
            <p class="text-xs font-bold uppercase tracking-widest text-slate-400">
              Or join via web browser at:
            </p>
            <span class="text-indigo-400 font-black text-sm tracking-wide block font-mono">
              quiz.ezsnapps.in
            </span>
          </div>

          <div>
            <span class="text-xs font-extrabold text-slate-400 uppercase tracking-widest block">Room PIN</span>
            <div class="flex items-center justify-center md:justify-start space-x-3 mt-1">
              <h1 class="text-5xl sm:text-7xl font-black font-mono tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 via-pink-400 to-purple-400 drop-shadow">
                {{ gameStore.roomPin || '------' }}
              </h1>
              <Button
                :icon="copied ? 'pi pi-check' : 'pi pi-copy'"
                :class="copied ? 'p-button-success' : 'p-button-secondary'"
                rounded
                text
                size="large"
                class="glass-pill w-11 h-11"
                @click="copyPin"
                title="Copy Room PIN"
              />
            </div>
          </div>

          <h2 class="text-lg sm:text-xl font-bold text-slate-200">
            {{ gameStore.quizTitle || 'Live Multiplayer Quiz' }}
          </h2>
        </div>
      </div>
    </div>

    <!-- Players Count & Controls -->
    <div class="glass-panel flex flex-col sm:flex-row items-center justify-between gap-4 p-5 rounded-2xl">
      <div class="flex items-center space-x-3">
        <div class="w-11 h-11 rounded-xl bg-indigo-500/20 text-indigo-400 flex items-center justify-center font-bold text-lg border border-indigo-500/30">
          <i class="pi pi-users" />
        </div>
        <div>
          <h3 class="font-extrabold text-lg text-white">
            {{ gameStore.playerCount }} {{ gameStore.playerCount === 1 ? 'Contestant' : 'Contestants' }} Joined
          </h3>
          <p class="text-xs text-slate-400">
            Wait for everyone to join, then start the game
          </p>
        </div>
      </div>

      <!-- Host Action -->
      <div class="w-full sm:w-auto">
        <Button
          label="Start Competition"
          icon="pi pi-play"
          size="large"
          class="w-full sm:w-auto bg-gradient-to-r from-emerald-600 via-emerald-500 to-teal-500 hover:from-emerald-500 hover:to-teal-400 border border-white/20 font-black text-white px-8 py-3.5 rounded-xl shadow-xl shadow-emerald-500/30 active:scale-98 transition-all disabled:opacity-40 disabled:cursor-not-allowed"
          :disabled="gameStore.playerCount === 0"
          @click="gameStore.startGame"
        />
      </div>
    </div>

    <!-- Players Grid -->
    <div v-if="gameStore.players.length > 0" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
      <div
        v-for="player in gameStore.players"
        :key="player.id"
        class="glass-card p-3.5 rounded-xl flex items-center justify-between group transition-all"
      >
        <div class="flex items-center space-x-2.5 truncate">
          <div class="w-7 h-7 rounded-lg bg-gradient-to-tr from-indigo-500 to-pink-500 flex items-center justify-center font-black text-xs text-white uppercase shrink-0 shadow">
            {{ player.nickname.slice(0, 2) }}
          </div>
          <span class="font-bold text-sm text-slate-200 truncate">
            {{ player.nickname }}
          </span>
        </div>

        <!-- Host kick player button -->
        <button
          @click="gameStore.kickPlayer(player.id)"
          class="text-slate-500 hover:text-rose-400 opacity-0 group-hover:opacity-100 transition-opacity p-1"
          title="Remove player"
        >
          <i class="pi pi-times text-xs" />
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="text-center py-12 text-slate-500 glass-card rounded-2xl">
      <i class="pi pi-user-plus text-4xl mb-3 text-slate-600 animate-bounce" />
      <p class="font-medium text-base text-slate-400">Waiting for players to join with the PIN or QR above...</p>
    </div>
  </div>
</template>
