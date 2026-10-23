<script setup lang="ts">
import { ref, onMounted } from 'vue';
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
  const room = await quizStore.launchRoom(quizId);
  if (room) {
    router.push({
      path: '/host',
      query: { pin: room.pin, token: room.host_token },
    });
  }
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
    theme_secondary_color: theme.secondary_color,
    theme_bg_color: theme.background_color,
    theme_btn_color: theme.button_color,
  });
  await quizStore.fetchQuizzes();
}

async function saveOrgBranding() {
  orgSaveSuccess.value = false;
  await brandingStore.updateOrgConfig(orgForm.value);
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

              <div class="grid grid-cols-2 gap-4">
                <div class="space-y-1">
                  <label class="text-xs font-bold text-slate-400">Primary Color</label>
                  <div class="flex items-center space-x-2">
                    <input type="color" v-model="orgForm.primary_color" class="w-8 h-8 rounded border-none bg-transparent cursor-pointer" />
                    <InputText v-model="orgForm.primary_color" class="p-inputtext-sm font-mono text-xs w-full bg-slate-950/60" />
                  </div>
                </div>

                <div class="space-y-1">
                  <label class="text-xs font-bold text-slate-400">Secondary Color</label>
                  <div class="flex items-center space-x-2">
                    <input type="color" v-model="orgForm.secondary_color" class="w-8 h-8 rounded border-none bg-transparent cursor-pointer" />
                    <InputText v-model="orgForm.secondary_color" class="p-inputtext-sm font-mono text-xs w-full bg-slate-950/60" />
                  </div>
                </div>

                <div class="space-y-1">
                  <label class="text-xs font-bold text-slate-400">Background Color</label>
                  <div class="flex items-center space-x-2">
                    <input type="color" v-model="orgForm.background_color" class="w-8 h-8 rounded border-none bg-transparent cursor-pointer" />
                    <InputText v-model="orgForm.background_color" class="p-inputtext-sm font-mono text-xs w-full bg-slate-950/60" />
                  </div>
                </div>

                <div class="space-y-1">
                  <label class="text-xs font-bold text-slate-400">Button Color</label>
                  <div class="flex items-center space-x-2">
                    <input type="color" v-model="orgForm.button_color" class="w-8 h-8 rounded border-none bg-transparent cursor-pointer" />
                    <InputText v-model="orgForm.button_color" class="p-inputtext-sm font-mono text-xs w-full bg-slate-950/60" />
                  </div>
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
  </div>
</template>
