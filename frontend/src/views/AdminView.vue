<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useQuizStore } from '@/stores/quiz';
import { useBrandingStore } from '@/stores/branding';
import { api } from '@/services/api';
import type { QuizSummary } from '@/types/quiz';

import Card from 'primevue/card';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Message from 'primevue/message';
import Dialog from 'primevue/dialog';
import QRCode from 'qrcode';
import QuizCard from '@/components/admin/QuizCard.vue';
import ThemePreviewCard from '@/components/common/ThemePreviewCard.vue';
import ThemeEditorModal from '@/components/admin/ThemeEditorModal.vue';

const router = useRouter();
const authStore = useAuthStore();
const quizStore = useQuizStore();
const brandingStore = useBrandingStore();

// Login Form
const username = ref('admin');
const password = ref('admin123');

// Active Tab ('quizzes' | 'branding' | 'history')
const activeTab = ref<'quizzes' | 'branding' | 'history'>('quizzes');

// Quiz Theme Customization Modal
const themeModalVisible = ref(false);
const editingQuiz = ref<QuizSummary | null>(null);

// Admin Launch Room Modal with QR Code
const launchModalVisible = ref(false);
const launchedRoom = ref<{ pin: string; token: string; title: string; qrUrl: string } | null>(null);
const launchPinCopied = ref(false);
const launchLinkCopied = ref(false);

const playerJoinUrl = computed(() => {
  if (!launchedRoom.value) return '';
  const origin = typeof window !== 'undefined' ? window.location.origin : 'https://quiz.ezsnapps.in';
  return `${origin}/play?pin=${launchedRoom.value.pin}`;
});

// Org branding editable state
const orgForm = ref({
  name: '',
  logo_url: null as string | null,
  primary_color: '#6366F1',
  secondary_color: '#EC4899',
  background_color: '#0F172A',
  button_color: '#4F46E5',
});
const orgSaveSuccess = ref(false);

// History state
const historyList = ref<any[]>([]);

onMounted(async () => {
  if (authStore.isAuthenticated) {
    await loadDashboardData();
  }
});

async function handleLogin() {
  const ok = await authStore.login(username.value, password.value);
  if (ok) {
    await loadDashboardData();
  }
}

async function loadDashboardData() {
  await Promise.all([
    quizStore.fetchQuizzes(),
    brandingStore.fetchOrgConfig(),
  ]);

  if (brandingStore.orgConfig) {
    orgForm.value = {
      name: brandingStore.orgConfig.name,
      logo_url: brandingStore.orgConfig.logo_url,
      primary_color: brandingStore.orgConfig.primary_color,
      secondary_color: brandingStore.orgConfig.secondary_color,
      background_color: brandingStore.orgConfig.background_color,
      button_color: brandingStore.orgConfig.button_color,
    };
  }

  try {
    historyList.value = await api.getHistory();
  } catch {
    // Ignore history fetch errors
  }
}

async function handleLaunch(quizId: string) {
  const quiz = quizStore.quizzes.find((q) => q.id === quizId);
  const room = await quizStore.launchRoom(quizId);
  if (room) {
    const origin = typeof window !== 'undefined' ? window.location.origin : 'https://quiz.ezsnapps.in';
    const joinUrl = `${origin}/play?pin=${room.pin}`;
    let qrUrl = '';
    try {
      qrUrl = await QRCode.toDataURL(joinUrl, {
        width: 360,
        margin: 1.5,
        color: { dark: '#0f172a', light: '#ffffff' },
      });
    } catch (e) {
      console.error('Failed to generate QR:', e);
    }

    launchedRoom.value = {
      pin: room.pin,
      token: room.host_token,
      title: quiz?.title || 'Live Quiz Competition',
      qrUrl,
    };
    launchModalVisible.value = true;
  }
}

function goToHostScreen() {
  if (!launchedRoom.value) return;
  router.push({
    path: '/host',
    query: { pin: launchedRoom.value.pin, token: launchedRoom.value.token },
  });
}

function copyLaunchPin() {
  if (!launchedRoom.value) return;
  navigator.clipboard.writeText(launchedRoom.value.pin);
  launchPinCopied.value = true;
  setTimeout(() => (launchPinCopied.value = false), 2000);
}

function copyLaunchLink() {
  if (!launchedRoom.value) return;
  const origin = typeof window !== 'undefined' ? window.location.origin : 'https://quiz.ezsnapps.in';
  navigator.clipboard.writeText(`${origin}/play?pin=${launchedRoom.value.pin}`);
  launchLinkCopied.value = true;
  setTimeout(() => (launchLinkCopied.value = false), 2000);
}

function handleEdit(quizId: string) {
  router.push(`/edit/${quizId}`);
}

function handleCreate() {
  router.push('/edit');
}

async function handleDuplicate(quizId: string) {
  await quizStore.duplicateQuiz(quizId);
}

async function handleDelete(quizId: string) {
  if (confirm('Are you sure you want to delete this quiz?')) {
    await quizStore.deleteQuiz(quizId);
  }
}

function handleTheme(quiz: QuizSummary) {
  editingQuiz.value = quiz;
  themeModalVisible.value = true;
}

async function saveQuizTheme(theme: any) {
  if (!editingQuiz.value) return;
  await quizStore.saveQuiz({
    id: editingQuiz.value.id,
    theme_logo_url: theme.logo_url,
    theme_primary_color: theme.primary_color,
    theme_secondary_color: '#EC4899',
    theme_bg_color: '#0B0F19',
    theme_btn_color: '#4F46E5',
  });
  await quizStore.fetchQuizzes();
}

async function saveOrgBranding() {
  orgSaveSuccess.value = false;
  await brandingStore.updateOrgConfig({
    ...orgForm.value,
    secondary_color: '#EC4899',
    background_color: '#0B0F19',
    button_color: '#4F46E5',
  });
  orgSaveSuccess.value = true;
  setTimeout(() => (orgSaveSuccess.value = false), 3000);
}

function handleOrgLogoUpload(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = async (e) => {
    const base64 = e.target?.result as string;
    try {
      const res = await api.uploadImage(base64, file.name);
      orgForm.value.logo_url = res.url;
    } catch (err) {
      console.error('Failed to upload logo:', err);
    }
  };
  reader.readAsDataURL(file);
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 py-8">
    <!-- 1. LOGIN SCREEN IF NOT AUTHENTICATED -->
    <div v-if="!authStore.isAuthenticated" class="max-w-md mx-auto py-12">
      <Card class="bg-slate-900/90 border border-slate-800 rounded-3xl shadow-2xl p-4">
        <template #title>
          <div class="text-center space-y-2 pb-4">
            <div class="w-12 h-12 rounded-2xl bg-indigo-600/20 text-indigo-400 flex items-center justify-center text-xl mx-auto">
              <i class="pi pi-lock" />
            </div>
            <h2 class="text-2xl font-black text-white">Admin Authentication</h2>
            <p class="text-xs text-slate-400">Sign in to manage quizzes & branding</p>
          </div>
        </template>
        <template #content>
          <form @submit.prevent="handleLogin" class="space-y-4">
            <Message v-if="authStore.error" severity="error" :closable="false" class="rounded-xl">
              {{ authStore.error }}
            </Message>

            <div class="space-y-1">
              <label class="text-xs font-bold text-slate-400">Username</label>
              <InputText v-model="username" class="w-full p-inputtext-sm font-bold bg-slate-950/60 border-slate-800" required />
            </div>

            <div class="space-y-1">
              <label class="text-xs font-bold text-slate-400">Password</label>
              <InputText v-model="password" type="password" class="w-full p-inputtext-sm bg-slate-950/60 border-slate-800" required />
            </div>

            <Button
              type="submit"
              label="Sign In"
              icon="pi pi-sign-in"
              :loading="authStore.isLoading"
              class="w-full bg-indigo-600 hover:bg-indigo-500 border-none font-bold text-white py-3 rounded-xl shadow-lg shadow-indigo-600/30"
            />

            <p class="text-[11px] text-slate-500 text-center">
              Default credentials: <code class="text-slate-400 font-mono">admin / admin123</code>
            </p>
          </form>
        </template>
      </Card>
    </div>

    <!-- 2. ADMIN DASHBOARD IF AUTHENTICATED -->
    <div v-else class="space-y-8">
      <!-- Dashboard Header -->
      <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 pb-6 border-b border-slate-800">
        <div>
          <h1 class="text-3xl font-black text-white">Management Console</h1>
          <p class="text-sm text-slate-400">Manage your interactive quizzes, live rooms, and organization identity</p>
        </div>

        <div class="flex items-center space-x-3">
          <Button
            label="New Quiz"
            icon="pi pi-plus"
            class="bg-indigo-600 hover:bg-indigo-500 border-none font-bold text-white rounded-xl shadow-md"
            @click="handleCreate"
          />
          <Button
            label="Logout"
            icon="pi pi-sign-out"
            text
            severity="secondary"
            class="text-slate-400 hover:text-white"
            @click="authStore.logout"
          />
        </div>
      </div>

      <!-- Navigation Tabs -->
      <div class="flex items-center space-x-2 border-b border-slate-800">
        <button
          @click="activeTab = 'quizzes'"
          :class="[
            'px-5 py-3 font-bold text-sm border-b-2 transition-colors flex items-center space-x-2',
            activeTab === 'quizzes' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'
          ]"
        >
          <i class="pi pi-list" />
          <span>Quizzes ({{ quizStore.quizzes.length }})</span>
        </button>

        <button
          @click="activeTab = 'branding'"
          :class="[
            'px-5 py-3 font-bold text-sm border-b-2 transition-colors flex items-center space-x-2',
            activeTab === 'branding' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'
          ]"
        >
          <i class="pi pi-palette" />
          <span>Organization Branding</span>
        </button>

        <button
          @click="activeTab = 'history'"
          :class="[
            'px-5 py-3 font-bold text-sm border-b-2 transition-colors flex items-center space-x-2',
            activeTab === 'history' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'
          ]"
        >
          <i class="pi pi-history" />
          <span>Game History</span>
        </button>
      </div>

      <!-- TAB 1: QUIZZES -->
      <div v-if="activeTab === 'quizzes'" class="space-y-6">
        <div v-if="quizStore.quizzes.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <QuizCard
            v-for="quiz in quizStore.quizzes"
            :key="quiz.id"
            :quiz="quiz"
            @launch="handleLaunch"
            @edit="handleEdit"
            @theme="handleTheme"
            @duplicate="handleDuplicate"
            @delete="handleDelete"
          />
        </div>

        <div v-else class="text-center py-16 bg-slate-900/40 rounded-3xl border border-slate-800 space-y-4">
          <i class="pi pi-folder-open text-4xl text-slate-600" />
          <h3 class="text-lg font-bold text-slate-300">No quizzes found</h3>
          <p class="text-xs text-slate-500">Create your first quiz to get started hosting live multiplayer games.</p>
          <Button label="Create Quiz" icon="pi pi-plus" class="bg-indigo-600 hover:bg-indigo-500 font-bold border-none" @click="handleCreate" />
        </div>
      </div>

      <!-- TAB 2: BRANDING -->
      <div v-else-if="activeTab === 'branding'" class="grid grid-cols-1 lg:grid-cols-2 gap-8 items-start">
        <!-- Settings Form -->
        <Card class="bg-slate-900/80 border border-slate-800 rounded-2xl shadow-xl">
          <template #title>
            <div class="space-y-1">
              <h3 class="text-lg font-bold text-white">Organization Quiz Branding</h3>
              <p class="text-xs text-slate-400">Customize the identity displayed when players join and participate in live quizzes.</p>
            </div>
          </template>
          <template #content>
            <form @submit.prevent="saveOrgBranding" class="space-y-5">
              <Message v-if="orgSaveSuccess" severity="success" :closable="false" class="rounded-xl">
                Organization branding saved and applied!
              </Message>

              <!-- Platform Brand Notice -->
              <div class="p-3 rounded-xl bg-slate-950/80 border border-slate-800 flex items-center justify-between">
                <div class="flex items-center space-x-3">
                  <img src="https://ezsnapps.in/landing_page/logo.png" alt="EZSN Platform Brand" class="h-8 w-auto object-contain" />
                  <div>
                    <span class="text-xs font-bold text-white block">Main Platform Logo</span>
                    <span class="text-[11px] text-slate-400">Permanently displayed on homepage and main navigation</span>
                  </div>
                </div>
                <span class="text-[10px] font-extrabold uppercase px-2 py-0.5 rounded bg-indigo-500/20 text-indigo-400">Fixed</span>
              </div>

              <div class="space-y-1">
                <label class="text-xs font-bold text-slate-400">Organization / Host Name</label>
                <InputText v-model="orgForm.name" placeholder="e.g. Acme Academy" class="w-full bg-slate-950/60 border-slate-800 font-bold" />
              </div>

              <div class="space-y-1">
                <label class="text-xs font-bold text-slate-400">Organization Logo (Displayed During Live Quizzes)</label>
                <div class="flex items-center space-x-3">
                  <input
                    type="file"
                    accept="image/*"
                    @change="handleOrgLogoUpload"
                    class="text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-indigo-600 file:text-white hover:file:bg-indigo-500 cursor-pointer block"
                  />
                  <img
                    v-if="orgForm.logo_url"
                    :src="orgForm.logo_url"
                    alt="Current Logo"
                    class="h-8 w-auto max-w-[80px] object-contain rounded p-0.5 bg-slate-950 border border-slate-800"
                  />
                </div>
              </div>

              <!-- Navbar Color Picker Only -->
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <label class="text-xs font-bold text-slate-300">Navbar Color</label>
                  <span class="text-[11px] text-slate-500">Only changes top navigation bar</span>
                </div>
                <div class="flex items-center space-x-3">
                  <input type="color" v-model="orgForm.primary_color" class="w-10 h-10 rounded-lg border border-slate-700 bg-transparent cursor-pointer" />
                  <InputText v-model="orgForm.primary_color" placeholder="#0F172A" class="p-inputtext-sm font-mono text-xs w-36 bg-slate-950 text-white border-slate-700" />
                </div>
                <!-- Presets -->
                <div class="flex items-center space-x-2 pt-1">
                  <span class="text-[11px] text-slate-500">Presets:</span>
                  <button type="button" @click="orgForm.primary_color = '#0F172A'" class="w-5 h-5 rounded-full border border-white/20 bg-[#0F172A]" title="Slate Dark"></button>
                  <button type="button" @click="orgForm.primary_color = '#1E1B4B'" class="w-5 h-5 rounded-full border border-white/20 bg-[#1E1B4B]" title="Deep Indigo"></button>
                  <button type="button" @click="orgForm.primary_color = '#0F2240'" class="w-5 h-5 rounded-full border border-white/20 bg-[#0F2240]" title="Navy Blue"></button>
                  <button type="button" @click="orgForm.primary_color = '#062E24'" class="w-5 h-5 rounded-full border border-white/20 bg-[#062E24]" title="Emerald Dark"></button>
                  <button type="button" @click="orgForm.primary_color = '#360918'" class="w-5 h-5 rounded-full border border-white/20 bg-[#360918]" title="Wine Red"></button>
                  <button type="button" @click="orgForm.primary_color = '#09090B'" class="w-5 h-5 rounded-full border border-white/20 bg-[#09090B]" title="Pitch Black"></button>
                </div>
              </div>

              <Button
                type="submit"
                label="Save Organization Theme"
                icon="pi pi-check"
                :loading="brandingStore.isLoading"
                class="w-full bg-indigo-600 hover:bg-indigo-500 font-bold text-white border-none py-3 rounded-xl"
              />
            </form>
          </template>
        </Card>

        <!-- Live Preview -->
        <div class="space-y-3">
          <h3 class="text-lg font-bold text-white">Interactive Live Preview</h3>
          <ThemePreviewCard :theme="{ ...orgForm, org_name: orgForm.name }" />
        </div>
      </div>

      <!-- TAB 3: HISTORY -->
      <div v-else-if="activeTab === 'history'" class="space-y-4">
        <Card class="bg-slate-900/80 border border-slate-800 rounded-2xl shadow-xl overflow-hidden">
          <template #title>
            <h3 class="text-lg font-bold text-white">Past Completed Quizzes</h3>
          </template>
          <template #content>
            <div v-if="historyList.length > 0" class="divide-y divide-slate-800">
              <div v-for="item in historyList" :key="item.room_id" class="p-4 flex items-center justify-between">
                <div>
                  <h4 class="font-bold text-white">{{ item.quiz_title }}</h4>
                  <p class="text-xs text-slate-400">{{ new Date(item.completed_at).toLocaleString() }}</p>
                </div>
                <span class="text-sm font-bold text-indigo-400">{{ item.participant_count }} Players</span>
              </div>
            </div>
            <div v-else class="text-center py-10 text-slate-500 text-sm">
              No completed sessions recorded yet.
            </div>
          </template>
        </Card>
      </div>
    </div>

    <!-- Quiz Specific Theme Modal -->
    <ThemeEditorModal
      v-if="editingQuiz"
      v-model:visible="themeModalVisible"
      :title="`Theme Override: ${editingQuiz.title}`"
      :initialTheme="{
        logo_url: editingQuiz.theme_logo_url,
        primary_color: editingQuiz.theme_primary_color || brandingStore.currentTheme.primary_color,
        secondary_color: editingQuiz.theme_secondary_color || brandingStore.currentTheme.secondary_color,
        background_color: editingQuiz.theme_bg_color || brandingStore.currentTheme.background_color,
        button_color: editingQuiz.theme_btn_color || brandingStore.currentTheme.button_color,
        org_name: editingQuiz.title,
      }"
      @save="saveQuizTheme"
    />

    <!-- Admin Launch QR Modal -->
    <Dialog
      v-model:visible="launchModalVisible"
      modal
      :header="launchedRoom?.title ? `${launchedRoom.title} Launched! 🚀` : 'Quiz Launched! 🚀'"
      :style="{ width: '92vw', maxWidth: '520px' }"
      class="p-dialog-dark"
    >
      <div v-if="launchedRoom" class="space-y-5 py-2 text-center">
        <p class="text-xs sm:text-sm text-slate-300">
          Your quiz is live! Players can scan this QR code with their mobile phone to move automatically to the quiz competition.
        </p>

        <!-- Big QR Code Display -->
        <div class="flex flex-col items-center justify-center p-5 bg-slate-950/90 rounded-2xl border border-slate-800 shadow-inner">
          <div class="bg-white p-3 rounded-2xl shadow-2xl mb-3">
            <img
              v-if="launchedRoom.qrUrl"
              :src="launchedRoom.qrUrl"
              alt="Scan to join quiz"
              class="w-52 h-52 sm:w-60 sm:h-60 object-contain rounded-lg"
            />
          </div>

          <span class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full bg-emerald-500/10 text-emerald-400 text-xs font-black uppercase tracking-wider mb-2 border border-emerald-500/20">
            <i class="pi pi-camera" />
            <span>Scan to join instantly on mobile</span>
          </span>

          <div class="flex items-center justify-center space-x-2 mt-1">
            <span class="text-xs text-slate-400 uppercase font-bold">Room PIN:</span>
            <span class="text-3xl font-mono font-black text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-pink-400 tracking-wider">
              {{ launchedRoom.pin }}
            </span>
            <Button
              :icon="launchPinCopied ? 'pi pi-check' : 'pi pi-copy'"
              :class="launchPinCopied ? 'p-button-success' : 'p-button-secondary'"
              text
              rounded
              size="small"
              @click="copyLaunchPin"
              title="Copy PIN"
            />
          </div>
        </div>

        <!-- Join Link Copy Bar -->
        <div class="flex items-center justify-between p-2.5 rounded-xl bg-slate-900 border border-slate-800 text-xs">
          <span class="text-slate-400 truncate max-w-[280px] font-mono text-[11px]">
            {{ playerJoinUrl }}
          </span>
          <Button
            :label="launchLinkCopied ? 'Copied!' : 'Copy Link'"
            icon="pi pi-link"
            size="small"
            text
            class="text-indigo-400 font-bold shrink-0"
            @click="copyLaunchLink"
          />
        </div>

        <!-- Action Buttons -->
        <div class="flex flex-col sm:flex-row gap-2.5 pt-2">
          <Button
            label="Open Host Screen (Projector)"
            icon="pi pi-external-link"
            class="flex-1 bg-gradient-to-r from-emerald-600 to-teal-500 hover:from-emerald-500 hover:to-teal-400 border-none font-bold text-white py-3 rounded-xl shadow-lg shadow-emerald-500/20"
            @click="goToHostScreen"
          />
          <Button
            label="Close"
            text
            severity="secondary"
            class="text-slate-400"
            @click="launchModalVisible = false"
          />
        </div>
      </div>
    </Dialog>
  </div>
</template>
