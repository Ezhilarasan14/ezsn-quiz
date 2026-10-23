<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { MAIN_BRAND_LOGO } from '@/stores/branding';
import Card from 'primevue/card';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Message from 'primevue/message';

const router = useRouter();

const pin = ref('');
const nickname = ref('');
const errorMessage = ref('');

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

  router.push({
    path: '/play',
    query: { pin: cleanPin, nickname: cleanNick },
  });
}
</script>

<template>
  <div class="min-h-[calc(100vh-4rem)] flex flex-col justify-center items-center py-12 px-4 relative overflow-hidden">
    <!-- Background glowing ambient spots -->
    <div class="absolute -top-40 -left-40 w-96 h-96 bg-indigo-600/20 rounded-full blur-3xl pointer-events-none" />
    <div class="absolute -bottom-40 -right-40 w-96 h-96 bg-pink-600/20 rounded-full blur-3xl pointer-events-none" />

    <div class="w-full max-w-md z-10 space-y-8">
      <!-- Permanent Main Brand Logo & Title on Home Page -->
      <div class="text-center space-y-3">
        <div class="flex justify-center mb-3">
          <img
            :src="MAIN_BRAND_LOGO"
            alt="EZSN Quiz Main Logo"
            class="h-16 sm:h-20 w-auto object-contain filter drop-shadow-2xl hover:scale-105 transition-transform"
          />
        </div>

        <div class="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 text-xs font-bold uppercase tracking-wider mb-1">
          <i class="pi pi-bolt" />
          <span>Real-time Multiplayer</span>
        </div>

        <h1 class="text-4xl sm:text-5xl font-black text-white tracking-tight">
          EZSN Quiz
        </h1>
        <p class="text-slate-400 text-sm font-medium">
          Enter your Game PIN and nickname to jump into the competition!
        </p>
      </div>

      <!-- Join Card -->
      <Card class="bg-slate-900/90 border border-slate-800 rounded-3xl shadow-2xl backdrop-blur-xl">
        <template #content>
          <form @submit.prevent="handleJoin" class="space-y-5 p-2">
            <Message v-if="errorMessage" severity="error" :closable="false" class="rounded-xl">
              {{ errorMessage }}
            </Message>

            <!-- Game PIN input -->
            <div class="space-y-1.5">
              <label for="pin" class="block text-xs font-bold uppercase tracking-wider text-slate-400">
                Game PIN
              </label>
              <div class="relative">
                <i class="pi pi-hashtag absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
                <InputText
                  id="pin"
                  v-model="pin"
                  placeholder="e.g. 826896"
                  class="w-full pl-12 py-3.5 text-center font-mono font-black text-2xl tracking-widest bg-slate-950/60 border-slate-800 focus:border-indigo-500 rounded-xl text-white placeholder:text-slate-600 placeholder:text-base placeholder:tracking-normal"
                  maxlength="6"
                  autofocus
                />
              </div>
            </div>

            <!-- Nickname input -->
            <div class="space-y-1.5">
              <label for="nickname" class="block text-xs font-bold uppercase tracking-wider text-slate-400">
                Nickname
              </label>
              <div class="relative">
                <i class="pi pi-user absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 text-lg" />
                <InputText
                  id="nickname"
                  v-model="nickname"
                  placeholder="Enter your game handle"
                  class="w-full pl-12 py-3 bg-slate-950/60 border-slate-800 focus:border-indigo-500 rounded-xl text-white font-bold placeholder:text-slate-600"
                  maxlength="20"
                />
              </div>
            </div>

            <!-- Submit Button -->
            <Button
              type="submit"
              label="Join Game"
              icon="pi pi-arrow-right"
              iconPos="right"
              size="large"
              class="w-full bg-gradient-to-r from-indigo-600 to-pink-600 hover:from-indigo-500 hover:to-pink-500 border-none font-black text-white py-3.5 rounded-xl shadow-lg shadow-indigo-600/30 transition-transform active:scale-98"
            />
          </form>
        </template>
      </Card>

      <!-- Host & Admin Links -->
      <div class="flex items-center justify-center space-x-6 text-xs font-semibold text-slate-400">
        <router-link to="/admin" class="hover:text-indigo-400 flex items-center space-x-1.5 transition-colors">
          <i class="pi pi-shield" />
          <span>Host a Quiz / Admin</span>
        </router-link>
        <span class="text-slate-700">•</span>
        <router-link to="/history" class="hover:text-indigo-400 flex items-center space-x-1.5 transition-colors">
          <i class="pi pi-trophy" />
          <span>Past Leaderboards</span>
        </router-link>
      </div>
    </div>
  </div>
</template>
