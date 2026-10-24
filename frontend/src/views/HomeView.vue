<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import Card from 'primevue/card';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Message from 'primevue/message';

const route = useRoute();
const router = useRouter();

const pin = ref('');
const nickname = ref(localStorage.getItem('ezsn_player_name') || '');
const errorMessage = ref('');

onMounted(() => {
  const queryPin = (route.query.pin as string) || '';
  if (queryPin) {
    // Automatically route to the dedicated quiz play view
    router.replace({ path: '/play', query: { pin: queryPin.trim() } });
  }
});

function handleJoin() {
  errorMessage.value = '';
  const cleanPin = pin.value.trim().replace(/\D/g, '');
  const cleanNick = nickname.value.trim();

  if (!cleanPin || cleanPin.length < 4) {
    errorMessage.value = 'Please enter a valid 6-digit Game PIN';
    return;
  }
  if (!cleanNick) {
    errorMessage.value = 'Please choose a nickname to join';
    return;
  }

  // Persist nickname locally for recurring quizzes
  localStorage.setItem('ezsn_player_name', cleanNick);

  router.push({
    path: '/play',
    query: {
      pin: cleanPin,
      nickname: cleanNick,
    },
  });
}
</script>

<template>
  <div class="min-h-[calc(100vh-4rem)] flex flex-col justify-center items-center py-8 sm:py-16 px-4">
    <div class="w-full max-w-md space-y-6 sm:space-y-8">
      <!-- Permanent Main Brand Logo & Hero Header -->
      <div class="text-center space-y-3">
        <div class="flex justify-center mb-2">
          <div class="relative group">
            <div class="absolute -inset-1 rounded-2xl bg-gradient-to-r from-purple-500 via-fuchsia-500 to-pink-500 opacity-40 blur-xl group-hover:opacity-70 transition duration-500" />
            <div class="relative w-16 h-16 sm:w-20 sm:h-20 rounded-2xl glass-panel border border-purple-400/30 flex items-center justify-center text-3xl sm:text-4xl shadow-2xl group-hover:scale-105 transition-transform duration-300">
              ⚡
            </div>
          </div>
        </div>

        <div class="inline-flex items-center space-x-2 px-3.5 py-1 rounded-full glass-pill text-purple-200 text-xs font-bold uppercase tracking-wider">
          <i class="pi pi-bolt text-purple-300" />
          <span>Real-time Live Multiplayer</span>
        </div>

        <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white tracking-tight">
          Enter The Arena
        </h1>
        <p class="text-purple-200/80 text-xs sm:text-sm font-medium max-w-xs mx-auto">
          Type your Game PIN and screen name to jump directly into the competition.
        </p>
      </div>

      <!-- Modern Glassmorphism Join Card -->
      <div class="glass-panel rounded-3xl p-6 sm:p-8 space-y-5">
        <form @submit.prevent="handleJoin" class="space-y-4">
          <Message v-if="errorMessage" severity="error" :closable="false" class="rounded-xl">
            {{ errorMessage }}
          </Message>

          <!-- Game PIN Input -->
          <div class="space-y-1.5">
            <label for="pin" class="block text-xs font-bold uppercase tracking-wider text-purple-200">
              Game PIN
            </label>
            <div class="relative">
              <i class="pi pi-hashtag absolute left-4 top-1/2 -translate-y-1/2 text-purple-400 text-lg" />
              <InputText
                id="pin"
                v-model="pin"
                placeholder="e.g. 826896"
                class="glass-input w-full pl-12 py-3.5 text-center font-mono font-black text-2xl tracking-widest rounded-xl placeholder:text-purple-300/30 text-white"
                maxlength="6"
                autofocus
              />
            </div>
          </div>

          <!-- Nickname Input -->
          <div class="space-y-1.5">
            <label for="nickname" class="block text-xs font-bold uppercase tracking-wider text-purple-200">
              Your Nickname <span class="text-pink-400">*</span>
            </label>
            <div class="relative">
              <i class="pi pi-user absolute left-4 top-1/2 -translate-y-1/2 text-purple-400 text-lg" />
              <InputText
                id="nickname"
                v-model="nickname"
                placeholder="Enter your screen name (e.g. Alex)"
                class="glass-input w-full pl-12 py-3 rounded-xl font-bold placeholder:text-purple-300/30 text-white"
                maxlength="20"
                required
              />
            </div>
          </div>

          <!-- Submit Button -->
          <Button
            type="submit"
            label="Join Competition"
            icon="pi pi-arrow-right"
            iconPos="right"
            size="large"
            class="w-full bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-black text-white py-3.5 rounded-xl shadow-xl shadow-purple-900/40 transition-all duration-200 active:scale-98 mt-2"
          />
        </form>
      </div>

      <!-- Feature Badges Showcase (Responsive Glass Cards) -->
      <div class="grid grid-cols-3 gap-2.5 text-center">
        <div class="glass-card rounded-2xl p-3 space-y-1">
          <div class="w-8 h-8 mx-auto rounded-lg bg-purple-500/20 text-purple-300 flex items-center justify-center text-sm font-bold">
            <i class="pi pi-bolt" />
          </div>
          <span class="text-[11px] font-bold text-white block">Sub-100ms</span>
          <span class="text-[10px] text-purple-300/70 block">Instant WS</span>
        </div>

        <div class="glass-card rounded-2xl p-3 space-y-1">
          <div class="w-8 h-8 mx-auto rounded-lg bg-fuchsia-500/20 text-fuchsia-300 flex items-center justify-center text-sm font-bold">
            <i class="pi pi-qrcode" />
          </div>
          <span class="text-[11px] font-bold text-white block">QR Direct</span>
          <span class="text-[10px] text-purple-300/70 block">Camera Join</span>
        </div>

        <div class="glass-card rounded-2xl p-3 space-y-1">
          <div class="w-8 h-8 mx-auto rounded-lg bg-pink-500/20 text-pink-300 flex items-center justify-center text-sm font-bold">
            <i class="pi pi-trophy" />
          </div>
          <span class="text-[11px] font-bold text-white block">Podiums</span>
          <span class="text-[10px] text-purple-300/70 block">Live Results</span>
        </div>
      </div>

      <!-- Quick Nav Links -->
      <div class="flex items-center justify-center space-x-6 text-xs font-semibold text-purple-300/80 pt-2">
        <router-link to="/admin" class="hover:text-white flex items-center space-x-1.5 transition-colors">
          <i class="pi pi-shield text-purple-400" />
          <span>Host Console / Admin</span>
        </router-link>
        <span class="text-purple-500/40">•</span>
        <router-link to="/history" class="hover:text-white flex items-center space-x-1.5 transition-colors">
          <i class="pi pi-chart-line text-purple-400" />
          <span>Player Rankings</span>
        </router-link>
      </div>
    </div>
  </div>
</template>
