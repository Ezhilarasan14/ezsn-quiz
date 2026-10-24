<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { MAIN_BRAND_LOGO } from '@/stores/branding';
import Card from 'primevue/card';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Message from 'primevue/message';

const route = useRoute();
const router = useRouter();

const pin = ref('');
const nickname = ref(localStorage.getItem('ezsn_player_name') || '');
const email = ref(localStorage.getItem('ezsn_player_email') || '');
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
  const cleanEmail = email.value.trim();

  if (!cleanPin || cleanPin.length < 4) {
    errorMessage.value = 'Please enter a valid 6-digit Game PIN';
    return;
  }
  if (!cleanNick) {
    errorMessage.value = 'Please choose a nickname to join';
    return;
  }

  // Persist locally for recurring quizzes
  localStorage.setItem('ezsn_player_name', cleanNick);
  if (cleanEmail) {
    localStorage.setItem('ezsn_player_email', cleanEmail);
  }

  router.push({
    path: '/play',
    query: {
      pin: cleanPin,
      nickname: cleanNick,
      ...(cleanEmail ? { email: cleanEmail } : {}),
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
            <div class="absolute -inset-1 rounded-2xl bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 opacity-30 blur-lg group-hover:opacity-60 transition duration-500" />
            <img
              :src="MAIN_BRAND_LOGO"
              alt="EZSN Quiz Main Logo"
              class="relative h-16 sm:h-20 w-auto object-contain filter drop-shadow-2xl group-hover:scale-105 transition-transform duration-300"
            />
          </div>
        </div>

        <div class="inline-flex items-center space-x-2 px-3.5 py-1 rounded-full glass-pill text-indigo-300 text-xs font-bold uppercase tracking-wider">
          <i class="pi pi-bolt text-indigo-400" />
          <span>Real-time Live Multiplayer</span>
        </div>

        <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white tracking-tight">
          Enter The Arena
        </h1>
        <p class="text-slate-400 text-xs sm:text-sm font-medium max-w-xs mx-auto">
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
            <label for="pin" class="block text-xs font-bold uppercase tracking-wider text-slate-300">
              Game PIN
            </label>
            <div class="relative">
              <i class="pi pi-hashtag absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
              <InputText
                id="pin"
                v-model="pin"
                placeholder="e.g. 826896"
                class="glass-input w-full pl-12 py-3.5 text-center font-mono font-black text-2xl tracking-widest rounded-xl placeholder:text-slate-600 placeholder:text-base placeholder:tracking-normal"
                maxlength="6"
                autofocus
              />
            </div>
          </div>

          <!-- Nickname Input -->
          <div class="space-y-1.5">
            <label for="nickname" class="block text-xs font-bold uppercase tracking-wider text-slate-300">
              Your Nickname <span class="text-rose-400">*</span>
            </label>
            <div class="relative">
              <i class="pi pi-user absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
              <InputText
                id="nickname"
                v-model="nickname"
                placeholder="Enter your screen name (e.g. Alex)"
                class="glass-input w-full pl-12 py-3 rounded-xl font-bold placeholder:text-slate-600"
                maxlength="20"
                required
              />
            </div>
          </div>

          <!-- Email ID (Optional, for persistent history) -->
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <label for="email" class="block text-xs font-bold uppercase tracking-wider text-slate-300">
                Email Address <span class="text-[10px] text-indigo-400 font-semibold normal-case">(Saves stats)</span>
              </label>
            </div>
            <div class="relative">
              <i class="pi pi-envelope absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
              <InputText
                id="email"
                v-model="email"
                type="email"
                placeholder="alex@example.com (optional)"
                class="glass-input w-full pl-12 py-2.5 rounded-xl text-sm placeholder:text-slate-600"
              />
            </div>
            <p class="text-[11px] text-slate-500 px-1 leading-relaxed">
              Track your cumulative points, badges, and accuracy across all quizzes.
            </p>
          </div>

          <!-- Submit Button -->
          <Button
            type="submit"
            label="Join Competition"
            icon="pi pi-arrow-right"
            iconPos="right"
            size="large"
            class="w-full bg-gradient-to-r from-indigo-600 via-indigo-500 to-pink-600 hover:from-indigo-500 hover:to-pink-500 border border-white/20 font-black text-white py-3.5 rounded-xl shadow-xl shadow-indigo-600/30 transition-all duration-200 active:scale-98 mt-2"
          />
        </form>
      </div>

      <!-- Feature Badges Showcase (Responsive Glass Cards) -->
      <div class="grid grid-cols-3 gap-2.5 text-center">
        <div class="glass-card rounded-2xl p-3 space-y-1">
          <div class="w-8 h-8 mx-auto rounded-lg bg-indigo-500/20 text-indigo-400 flex items-center justify-center text-sm font-bold">
            <i class="pi pi-bolt" />
          </div>
          <span class="text-[11px] font-bold text-white block">Sub-100ms</span>
          <span class="text-[10px] text-slate-400 block">Instant WS</span>
        </div>

        <div class="glass-card rounded-2xl p-3 space-y-1">
          <div class="w-8 h-8 mx-auto rounded-lg bg-emerald-500/20 text-emerald-400 flex items-center justify-center text-sm font-bold">
            <i class="pi pi-qrcode" />
          </div>
          <span class="text-[11px] font-bold text-white block">QR Direct</span>
          <span class="text-[10px] text-slate-400 block">Camera Join</span>
        </div>

        <div class="glass-card rounded-2xl p-3 space-y-1">
          <div class="w-8 h-8 mx-auto rounded-lg bg-amber-500/20 text-amber-400 flex items-center justify-center text-sm font-bold">
            <i class="pi pi-trophy" />
          </div>
          <span class="text-[11px] font-bold text-white block">Podiums</span>
          <span class="text-[10px] text-slate-400 block">Live Results</span>
        </div>
      </div>

      <!-- Quick Nav Links -->
      <div class="flex items-center justify-center space-x-6 text-xs font-semibold text-slate-400 pt-2">
        <router-link to="/admin" class="hover:text-indigo-400 flex items-center space-x-1.5 transition-colors">
          <i class="pi pi-shield text-indigo-400" />
          <span>Host Console / Admin</span>
        </router-link>
        <span class="text-slate-700">•</span>
        <router-link to="/history" class="hover:text-indigo-400 flex items-center space-x-1.5 transition-colors">
          <i class="pi pi-chart-line text-indigo-400" />
          <span>Player Rankings</span>
        </router-link>
      </div>
    </div>
  </div>
</template>
