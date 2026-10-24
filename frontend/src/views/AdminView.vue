<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useQuizStore } from '@/stores/quiz';
import { useBrandingStore } from '@/stores/branding';
import { api } from '@/services/api';
import type { QuizSummary } from '@/types/quiz';
import type { AdminAllowlistEntry } from '@/types/auth';

import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Message from 'primevue/message';
import Dialog from 'primevue/dialog';
import QRCode from 'qrcode';
import QuizCard from '@/components/admin/QuizCard.vue';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const quizStore = useQuizStore();
const brandingStore = useBrandingStore();

// Local Dev Login Form
const showDevLogin = ref(false);
const username = ref('admin');
const password = ref('admin123');

// Login error from query params or auth store
const queryError = ref<string | null>(null);

// Active Tab ('quizzes' | 'branding' | 'history' | 'allowlist')
const activeTab = ref<'quizzes' | 'branding' | 'history' | 'allowlist'>('quizzes');

// Allowlist management state
const allowlist = ref<AdminAllowlistEntry[]>([]);
const isAllowlistLoading = ref(false);
const newEmail = ref('');
const newRole = ref<'admin' | 'superadmin'>('admin');
const newNotes = ref('');
const allowlistActionError = ref<string | null>(null);
const allowlistActionSuccess = ref<string | null>(null);

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

// Org branding editable state (Only dynamic fields: Logo, Name, Branch)
const orgForm = ref({
  name: '',
  branch_name: 'Main Campus',
  logo_url: null as string | null,
});
const orgSaveSuccess = ref(false);
const orgSaveError = ref('');

// History state
const historyList = ref<any[]>([]);

onMounted(async () => {
  // Check for error in query string (from Google OAuth callback)
  const errParam = route.query.error as string | undefined;
  if (errParam) {
    if (errParam === 'unauthorized_email') {
      queryError.value = 'Access Denied: Your Google account is not on the approved administrator allowlist. Please contact a superadmin.';
    } else if (errParam === 'invalid_state') {
      queryError.value = 'Security Check Failed: OAuth state was invalid or expired. Please try signing in again.';
    } else if (errParam === 'no_code') {
      queryError.value = 'Google sign-in was cancelled or authorization code was not returned.';
    } else if (errParam === 'token_exchange_failed') {
      queryError.value = 'Failed to verify authentication with Google servers. Please try again.';
    } else if (errParam === 'userinfo_failed') {
      queryError.value = 'Unable to fetch your Google profile information.';
    } else {
      queryError.value = `Authentication error: ${errParam}`;
    }
  }

  // Check server-side session cookie
  const isAuthed = await authStore.checkAuth();
  if (isAuthed || authStore.isAuthenticated) {
    await loadDashboardData();
  }
});

async function handleGoogleLogin() {
  queryError.value = null;
  await authStore.initiateGoogleLogin();
}

async function handleDevLogin() {
  queryError.value = null;
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
      branch_name: brandingStore.orgConfig.branch_name || 'Main Campus',
      logo_url: brandingStore.orgConfig.logo_url,
    };
  }

  try {
    historyList.value = await api.getHistory();
  } catch {
    // Ignore history fetch errors
  }

  if (authStore.isSuperAdmin) {
    await fetchAllowlist();
  }
}

async function fetchAllowlist() {
  isAllowlistLoading.value = true;
  allowlistActionError.value = null;
  try {
    allowlist.value = await api.getAllowlist();
  } catch (err: any) {
    console.error('Failed to load allowlist:', err);
  } finally {
    isAllowlistLoading.value = false;
  }
}

async function handleAddAllowlistEntry() {
  if (!newEmail.value || !newEmail.value.includes('@')) {
    allowlistActionError.value = 'Please provide a valid email address.';
    return;
  }
  allowlistActionError.value = null;
  allowlistActionSuccess.value = null;
  try {
    await api.addAllowlistEntry({
      email: newEmail.value.trim().toLowerCase(),
      role: newRole.value,
      notes: newNotes.value.trim() || undefined,
    });
    newEmail.value = '';
    newNotes.value = '';
    allowlistActionSuccess.value = 'Google account successfully approved for admin access!';
    await fetchAllowlist();
    setTimeout(() => (allowlistActionSuccess.value = null), 4000);
  } catch (err: any) {
    allowlistActionError.value = err.message || 'Failed to add allowlist entry';
  }
}

async function handleRemoveAllowlistEntry(email: string) {
  if (email.toLowerCase() === authStore.user?.email?.toLowerCase()) {
    if (!confirm('Warning: You are about to remove your own email from the allowlist. Are you sure?')) {
      return;
    }
  } else if (!confirm(`Revoke admin access for ${email}?`)) {
    return;
  }

  allowlistActionError.value = null;
  allowlistActionSuccess.value = null;
  try {
    await api.removeAllowlistEntry(email);
    allowlistActionSuccess.value = `Access revoked for ${email}`;
    await fetchAllowlist();
    setTimeout(() => (allowlistActionSuccess.value = null), 4000);
  } catch (err: any) {
    allowlistActionError.value = err.message || 'Failed to remove allowlist entry';
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
        color: { dark: '#070a13', light: '#ffffff' },
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
  navigator.clipboard.writeText(playerJoinUrl.value);
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

async function saveOrgBranding() {
  orgSaveSuccess.value = false;
  orgSaveError.value = '';
  try {
    await brandingStore.updateOrgConfig({
      name: orgForm.value.name,
      branch_name: orgForm.value.branch_name,
      logo_url: orgForm.value.logo_url,
    });
    orgSaveSuccess.value = true;
    setTimeout(() => (orgSaveSuccess.value = false), 3500);
  } catch (err: any) {
    orgSaveError.value = err?.message || 'Failed to save organization details. Please check connection and permissions.';
  }
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
  <div class="max-w-7xl mx-auto px-4 sm:px-6 py-6 sm:py-10">
    <!-- 1. LOGIN SCREEN IF NOT AUTHENTICATED -->
    <div v-if="!authStore.isAuthenticated" class="max-w-md mx-auto py-10 sm:py-16">
      <div class="glass-panel rounded-3xl p-6 sm:p-9 shadow-2xl space-y-6 border border-purple-400/20">
        <!-- Logo & Header -->
        <div class="text-center space-y-2 pb-1">
          <div class="w-16 h-16 rounded-2xl bg-gradient-to-tr from-purple-600/30 via-fuchsia-600/20 to-pink-600/30 text-purple-200 flex items-center justify-center text-2xl mx-auto border border-purple-400/30 shadow-xl shadow-purple-950/50">
            <i class="pi pi-shield" />
          </div>
          <h2 class="text-2xl sm:text-3xl font-black text-white tracking-tight">Admin Portal</h2>
          <p class="text-xs text-purple-300/80">Sign in with an approved Google administrator account</p>
        </div>

        <!-- Error Messages -->
        <div v-if="queryError || authStore.error" class="p-3.5 rounded-2xl bg-rose-500/15 border border-rose-500/30 text-rose-200 text-xs flex items-start space-x-2.5 shadow-lg">
          <i class="pi pi-exclamation-triangle text-rose-400 text-sm mt-0.5 shrink-0" />
          <div class="space-y-0.5">
            <span class="font-bold block">Access Restricted</span>
            <span class="text-[11px] leading-relaxed text-rose-300">{{ queryError || authStore.error }}</span>
          </div>
        </div>

        <!-- Google OAuth Sign-in Button -->
        <div class="space-y-3 pt-1">
          <button
            type="button"
            :disabled="authStore.isLoading"
            @click="handleGoogleLogin"
            class="w-full flex items-center justify-center space-x-3.5 bg-white hover:bg-slate-50 text-slate-900 font-bold py-3.5 px-5 rounded-2xl shadow-xl hover:shadow-2xl hover:scale-[1.01] active:scale-[0.99] transition-all disabled:opacity-50 disabled:cursor-not-allowed border border-white/60"
          >
            <!-- Google Official "G" Multi-color Icon -->
            <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24">
              <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
              <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
              <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
              <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
            </svg>
            <span class="text-sm tracking-wide">
              {{ authStore.isLoading ? 'Redirecting to Google...' : 'Sign in with Google' }}
            </span>
          </button>

          <p class="text-[11px] text-center text-purple-300/60">
            Access is restricted to pre-approved Google accounts via server-side allowlist.
          </p>
        </div>

        <!-- Collapsible Developer Credentials Fallback -->
        <div class="pt-4 border-t border-purple-400/15 space-y-3">
          <button
            type="button"
            @click="showDevLogin = !showDevLogin"
            class="w-full text-center text-xs font-semibold text-purple-300/70 hover:text-purple-200 transition-colors flex items-center justify-center space-x-1.5 py-1"
          >
            <span>Local Developer Sign-in</span>
            <i :class="['pi text-[10px] transition-transform duration-200', showDevLogin ? 'pi-chevron-up' : 'pi-chevron-down']" />
          </button>

          <form v-if="showDevLogin" @submit.prevent="handleDevLogin" class="space-y-3 pt-2">
            <div class="space-y-1">
              <label class="text-[11px] font-bold uppercase tracking-wider text-purple-200">Username</label>
              <div class="relative">
                <i class="pi pi-user absolute left-3.5 top-1/2 -translate-y-1/2 text-purple-400 text-xs" />
                <InputText v-model="username" class="glass-input w-full pl-9 py-2 rounded-xl text-xs font-bold text-white placeholder:text-purple-300/30" required />
              </div>
            </div>

            <div class="space-y-1">
              <label class="text-[11px] font-bold uppercase tracking-wider text-purple-200">Password</label>
              <div class="relative">
                <i class="pi pi-key absolute left-3.5 top-1/2 -translate-y-1/2 text-purple-400 text-xs" />
                <InputText v-model="password" type="password" class="glass-input w-full pl-9 py-2 rounded-xl text-xs font-bold text-white" required />
              </div>
            </div>

            <Button
              type="submit"
              label="Sign In with Credentials"
              icon="pi pi-lock-open"
              :loading="authStore.isLoading"
              class="w-full bg-purple-700/60 hover:bg-purple-600/80 border border-purple-400/30 font-bold text-xs text-white py-2.5 rounded-xl transition-all"
            />

            <p class="text-[10px] text-purple-400/80 text-center font-mono">
              Dev default: admin / admin123
            </p>
          </form>
        </div>
      </div>
    </div>

    <!-- 2. ADMIN DASHBOARD IF AUTHENTICATED -->
    <div v-else class="space-y-8">
      <!-- Dashboard Top Header with Admin Profile & Role Badge -->
      <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 pb-6 border-b border-purple-400/20">
        <div>
          <h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight">Management Console</h1>
          <p class="text-xs sm:text-sm text-purple-300/80 mt-0.5">Manage live quiz rooms, questions, allowlist, and organization brand identity</p>
        </div>

        <div class="flex flex-wrap items-center gap-3 w-full md:w-auto justify-between md:justify-end">
          <!-- Admin User Profile Chip -->
          <div class="glass-pill px-3 py-1.5 rounded-2xl flex items-center space-x-2.5 border border-purple-400/30">
            <img
              v-if="authStore.user?.avatar_url"
              :src="authStore.user.avatar_url"
              alt="Avatar"
              class="w-7 h-7 rounded-full object-cover border border-purple-300/40"
            />
            <div
              v-else
              class="w-7 h-7 rounded-full bg-gradient-to-tr from-purple-500 to-pink-500 flex items-center justify-center text-xs font-black text-white"
            >
              {{ (authStore.user?.name || authStore.user?.username || 'A')[0].toUpperCase() }}
            </div>

            <div class="flex flex-col text-left">
              <span class="text-xs font-bold text-white truncate max-w-[140px]">
                {{ authStore.user?.name || authStore.user?.username }}
              </span>
              <span v-if="authStore.user?.email" class="text-[10px] text-purple-300/70 truncate max-w-[140px]">
                {{ authStore.user.email }}
              </span>
            </div>

            <span
              :class="[
                'text-[10px] font-black uppercase px-2 py-0.5 rounded-full border',
                authStore.isSuperAdmin
                  ? 'bg-amber-500/20 text-amber-300 border-amber-400/40'
                  : 'bg-purple-500/20 text-purple-200 border-purple-400/30'
              ]"
            >
              {{ authStore.user?.role || 'admin' }}
            </span>
          </div>

          <div class="flex items-center space-x-2">
            <Button
              label="New Quiz"
              icon="pi pi-plus"
              class="bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-bold text-xs sm:text-sm text-white px-4 py-2.5 rounded-xl shadow-lg shadow-purple-900/40"
              @click="handleCreate"
            />
            <Button
              label="Logout"
              icon="pi pi-sign-out"
              text
              severity="secondary"
              class="glass-pill text-purple-200 hover:text-white px-3 py-2.5 rounded-xl text-xs sm:text-sm"
              @click="authStore.logout"
            />
          </div>
        </div>
      </div>

      <!-- Navigation Tabs (Modern Glass Pills) -->
      <div class="flex items-center space-x-2 border-b border-purple-400/20 pb-1 overflow-x-auto">
        <button
          @click="activeTab = 'quizzes'"
          :class="[
            'px-5 py-2.5 font-bold text-xs sm:text-sm rounded-xl transition-all flex items-center space-x-2 shrink-0',
            activeTab === 'quizzes'
              ? 'glass-panel text-white border border-purple-400/40 shadow-lg shadow-purple-900/30'
              : 'text-purple-300/70 hover:text-white hover:bg-purple-500/10'
          ]"
        >
          <i class="pi pi-list text-purple-300" />
          <span>Quizzes ({{ quizStore.quizzes.length }})</span>
        </button>

        <button
          @click="activeTab = 'branding'"
          :class="[
            'px-5 py-2.5 font-bold text-xs sm:text-sm rounded-xl transition-all flex items-center space-x-2 shrink-0',
            activeTab === 'branding'
              ? 'glass-panel text-white border border-purple-400/40 shadow-lg shadow-purple-900/30'
              : 'text-purple-300/70 hover:text-white hover:bg-purple-500/10'
          ]"
        >
          <i class="pi pi-palette text-purple-300" />
          <span>Organization Branding</span>
        </button>

        <button
          @click="activeTab = 'history'"
          :class="[
            'px-5 py-2.5 font-bold text-xs sm:text-sm rounded-xl transition-all flex items-center space-x-2 shrink-0',
            activeTab === 'history'
              ? 'glass-panel text-white border border-purple-400/40 shadow-lg shadow-purple-900/30'
              : 'text-purple-300/70 hover:text-white hover:bg-purple-500/10'
          ]"
        >
          <i class="pi pi-history text-purple-300" />
          <span>Game History</span>
        </button>

        <!-- Access Allowlist Tab -->
        <button
          @click="activeTab = 'allowlist'"
          :class="[
            'px-5 py-2.5 font-bold text-xs sm:text-sm rounded-xl transition-all flex items-center space-x-2 shrink-0',
            activeTab === 'allowlist'
              ? 'glass-panel text-white border border-purple-400/40 shadow-lg shadow-purple-900/30'
              : 'text-purple-300/70 hover:text-white hover:bg-purple-500/10'
          ]"
        >
          <i class="pi pi-shield text-purple-300" />
          <span>Access Allowlist</span>
          <span v-if="allowlist.length > 0" class="text-[10px] px-1.5 py-0.5 rounded-full bg-purple-500/30 text-purple-200">
            {{ allowlist.length }}
          </span>
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
            @duplicate="handleDuplicate"
            @delete="handleDelete"
          />
        </div>

        <div v-else class="text-center py-16 glass-card rounded-3xl space-y-4">
          <div class="w-16 h-16 mx-auto rounded-2xl glass-panel flex items-center justify-center text-3xl text-purple-300/70">
            <i class="pi pi-folder-open" />
          </div>
          <h3 class="text-lg font-bold text-slate-200">No quizzes found</h3>
          <p class="text-xs text-purple-300/70 max-w-sm mx-auto">Create your first quiz competition to start hosting live multiplayer arenas.</p>
          <Button label="Create Quiz" icon="pi pi-plus" class="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 font-bold border-none" @click="handleCreate" />
        </div>
      </div>

      <!-- TAB 2: BRANDING -->
      <div v-else-if="activeTab === 'branding'" class="grid grid-cols-1 lg:grid-cols-2 gap-8 items-start">
        <!-- Settings Form -->
        <div class="glass-panel rounded-3xl p-6 sm:p-8 space-y-6 shadow-2xl">
          <div class="space-y-1">
            <h3 class="text-lg font-black text-white">Organization Quiz Branding</h3>
            <p class="text-xs text-purple-300/70">Customize the identity displayed when players join and participate in live quizzes.</p>
          </div>

          <form @submit.prevent="saveOrgBranding" class="space-y-5">
            <Message v-if="orgSaveSuccess" severity="success" :closable="false" class="rounded-xl">
              Organization details saved and applied!
            </Message>

            <Message v-if="orgSaveError" severity="error" :closable="false" class="rounded-xl">
              {{ orgSaveError }}
            </Message>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label class="text-xs font-bold text-purple-200">Organization Name</label>
                <InputText v-model="orgForm.name" placeholder="e.g. Acme Academy" class="glass-input w-full font-bold py-2.5 rounded-xl" />
              </div>

              <div class="space-y-1.5">
                <label class="text-xs font-bold text-purple-200">Branch Name</label>
                <InputText v-model="orgForm.branch_name" placeholder="e.g. Main Campus / Chennai Branch" class="glass-input w-full font-bold py-2.5 rounded-xl" />
              </div>
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-bold text-purple-200">Organization Logo (Displayed in Live Quizzes)</label>
              <div class="flex items-center space-x-3">
                <input
                  type="file"
                  accept="image/*"
                  @change="handleOrgLogoUpload"
                  class="text-xs text-purple-300 file:mr-3 file:py-2 file:px-4 file:rounded-xl file:border-0 file:text-xs file:font-semibold file:bg-purple-600 file:text-white hover:file:bg-purple-500 cursor-pointer block"
                />
                <img
                  v-if="orgForm.logo_url"
                  :src="orgForm.logo_url"
                  alt="Current Logo"
                  class="h-9 w-auto max-w-[80px] object-contain rounded-xl p-1 glass-card"
                  @error="(e: any) => e.target.style.display = 'none'"
                />
              </div>
            </div>

            <Button
              type="submit"
              label="Save Organization Details"
              icon="pi pi-check"
              :loading="brandingStore.isLoading"
              class="w-full bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-bold text-white py-3 rounded-xl shadow-lg shadow-purple-900/40 active:scale-98 transition-all"
            />
          </form>
        </div>

        <!-- Live Preview Card (Unified Glassmorphic Appearance) -->
        <div class="space-y-4">
          <div class="space-y-1">
            <h3 class="text-base font-black text-white px-1">Live Quiz Branding Preview</h3>
            <p class="text-xs text-purple-300/70 px-1">How players and hosts see your organization branding during quizzes.</p>
          </div>

          <!-- Preview 1: Top Navigation Bar Header Capsule -->
          <div class="glass-panel p-4 rounded-3xl space-y-3">
            <span class="text-[10px] uppercase font-bold text-purple-300 tracking-wider">Top Navigation Bar Capsule</span>
            <div class="p-3 rounded-2xl bg-purple-950/50 border border-purple-500/20 flex items-center justify-between">
              <span class="text-sm font-black text-white">ezsnapps<span class="text-purple-400">.quiz</span></span>
              <div class="flex items-center space-x-2 px-3 py-1 rounded-full bg-purple-900/40 border border-purple-400/25">
                <img
                  v-if="orgForm.logo_url"
                  :src="orgForm.logo_url"
                  alt=""
                  class="h-5 w-auto max-w-[60px] object-contain rounded bg-white/10 p-0.5"
                  @error="(e: any) => e.target.style.display = 'none'"
                />
                <div v-else class="w-5 h-5 rounded-full bg-purple-500/20 flex items-center justify-center text-[10px] text-purple-200">
                  <i class="pi pi-building text-[10px]" />
                </div>
                <div class="flex items-center space-x-1.5">
                  <span class="font-bold text-white text-xs">{{ orgForm.name || 'Organization Name' }}</span>
                  <span v-if="orgForm.branch_name" class="text-xs text-purple-300/80">• {{ orgForm.branch_name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Preview 2: Results Page Strip -->
          <div class="glass-panel p-4 rounded-3xl space-y-3">
            <span class="text-[10px] uppercase font-bold text-purple-300 tracking-wider">Results Page Strip</span>
            <div class="p-3 rounded-2xl bg-purple-950/50 border border-purple-500/20 flex flex-wrap items-center justify-center gap-3 text-xs">
              <span class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full bg-purple-900/30 border border-purple-400/20 text-purple-100 font-bold">
                <i class="pi pi-building text-purple-300 text-xs" />
                <span>Org: {{ orgForm.name || 'Organization Name' }}</span>
              </span>
              <span class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full bg-purple-900/30 border border-purple-400/20 text-purple-100 font-bold">
                <i class="pi pi-map-marker text-fuchsia-300 text-xs" />
                <span>Branch: {{ orgForm.branch_name || 'Main Campus' }}</span>
              </span>
              <span class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full bg-purple-900/30 border border-purple-400/20 text-purple-100 font-bold">
                <i class="pi pi-user text-amber-300 text-xs" />
                <span>Host: Ezhil Arasan</span>
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- TAB 3: HISTORY -->
      <div v-else-if="activeTab === 'history'" class="space-y-4">
        <div class="glass-panel rounded-3xl overflow-hidden shadow-2xl border border-purple-400/20">
          <div class="p-5 border-b border-purple-400/20 flex items-center justify-between">
            <h3 class="text-lg font-black text-white">Past Completed Quizzes</h3>
            <span class="glass-pill px-3 py-1 rounded-full text-xs text-purple-200 font-bold">
              {{ historyList.length }} Sessions
            </span>
          </div>

          <div v-if="historyList.length > 0" class="divide-y divide-purple-400/10">
            <div
              v-for="item in historyList"
              :key="item.room_id"
              class="p-4 sm:p-5 flex items-center justify-between hover:bg-purple-500/10 transition-colors"
            >
              <div>
                <h4 class="font-extrabold text-white text-base">{{ item.quiz_title }}</h4>
                <p class="text-xs text-purple-300/70 mt-0.5">{{ new Date(item.completed_at).toLocaleString() }}</p>
              </div>
              <span class="glass-pill px-3 py-1 rounded-full text-xs font-bold text-purple-200 border border-purple-400/30">
                {{ item.participant_count }} Contestants
              </span>
            </div>
          </div>
          <div v-else class="text-center py-12 text-purple-300/60">
            No completed sessions recorded yet.
          </div>
        </div>
      </div>

      <!-- TAB 4: ACCESS ALLOWLIST (RBAC & GOOGLE OAUTH SECURITY) -->
      <div v-else-if="activeTab === 'allowlist'" class="space-y-6">
        <!-- Superadmin Add Account Form -->
        <div class="glass-panel rounded-3xl p-6 sm:p-8 space-y-6 shadow-2xl border border-purple-400/20">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div>
              <h3 class="text-lg font-black text-white flex items-center space-x-2">
                <i class="pi pi-shield text-purple-300" />
                <span>Authorized Google Accounts</span>
              </h3>
              <p class="text-xs text-purple-300/80 mt-1 max-w-2xl">
                Pre-approved Google email addresses allowed to sign in to the management dashboard. Only <code class="text-amber-300 font-mono">superadmin</code> accounts can modify access.
              </p>
            </div>
            <span class="glass-pill px-3 py-1 rounded-full text-xs text-purple-200 font-bold self-start">
              {{ allowlist.length }} Active Admins
            </span>
          </div>

          <!-- Notification feedback -->
          <div v-if="allowlistActionSuccess" class="p-3.5 rounded-2xl bg-emerald-500/15 border border-emerald-500/30 text-emerald-200 text-xs flex items-center space-x-2">
            <i class="pi pi-check-circle text-emerald-400 text-base shrink-0" />
            <span>{{ allowlistActionSuccess }}</span>
          </div>
          <div v-if="allowlistActionError" class="p-3.5 rounded-2xl bg-rose-500/15 border border-rose-500/30 text-rose-200 text-xs flex items-center space-x-2">
            <i class="pi pi-exclamation-triangle text-rose-400 text-base shrink-0" />
            <span>{{ allowlistActionError }}</span>
          </div>

          <!-- Add Account Form (visible to superadmin) -->
          <div v-if="authStore.isSuperAdmin" class="p-5 rounded-2xl glass-card border border-purple-400/20 space-y-4">
            <h4 class="text-xs font-black uppercase tracking-wider text-purple-200">Authorize New Google Account</h4>
            <div class="grid grid-cols-1 sm:grid-cols-12 gap-3">
              <div class="sm:col-span-5 space-y-1">
                <label class="text-[11px] font-bold text-purple-300">Google Email</label>
                <InputText
                  v-model="newEmail"
                  placeholder="admin@example.com"
                  type="email"
                  class="glass-input w-full py-2 px-3 rounded-xl text-xs font-semibold text-white placeholder:text-purple-300/40"
                />
              </div>

              <div class="sm:col-span-3 space-y-1">
                <label class="text-[11px] font-bold text-purple-300">Role</label>
                <select
                  v-model="newRole"
                  class="glass-input w-full py-2 px-3 rounded-xl text-xs font-bold text-white bg-slate-900 border border-purple-400/30"
                >
                  <option value="admin">admin</option>
                  <option value="superadmin">superadmin</option>
                </select>
              </div>

              <div class="sm:col-span-4 space-y-1">
                <label class="text-[11px] font-bold text-purple-300">Notes (Optional)</label>
                <InputText
                  v-model="newNotes"
                  placeholder="e.g. Lead Quiz Host"
                  class="glass-input w-full py-2 px-3 rounded-xl text-xs font-semibold text-white placeholder:text-purple-300/40"
                />
              </div>
            </div>

            <div class="flex justify-end pt-1">
              <Button
                label="Authorize Account"
                icon="pi pi-user-plus"
                @click="handleAddAllowlistEntry"
                class="bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-bold text-xs text-white px-5 py-2 rounded-xl shadow-lg shadow-purple-900/30"
              />
            </div>
          </div>
          <div v-else class="p-3.5 rounded-2xl glass-card border border-purple-400/20 text-xs text-purple-300">
            <i class="pi pi-info-circle mr-1 text-purple-400" />
            You are signed in as an <code class="text-purple-200 font-mono">admin</code>. Superadmin privileges are required to add or delete allowlisted accounts.
          </div>

          <!-- Allowlist Table -->
          <div class="rounded-2xl overflow-hidden border border-purple-400/20">
            <div class="overflow-x-auto">
              <table class="w-full text-left text-xs">
                <thead class="bg-purple-950/40 text-purple-200 uppercase font-black tracking-wider text-[10px] border-b border-purple-400/20">
                  <tr>
                    <th class="py-3 px-4">Account / Email</th>
                    <th class="py-3 px-4">Role</th>
                    <th class="py-3 px-4">Notes</th>
                    <th class="py-3 px-4">Authorized Date</th>
                    <th v-if="authStore.isSuperAdmin" class="py-3 px-4 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-purple-400/10">
                  <tr
                    v-for="entry in allowlist"
                    :key="entry.email"
                    class="hover:bg-purple-500/10 transition-colors"
                  >
                    <td class="py-3.5 px-4">
                      <div class="flex items-center space-x-2.5">
                        <div class="w-7 h-7 rounded-full bg-purple-600/30 border border-purple-400/30 flex items-center justify-center text-purple-200">
                          <i class="pi pi-user text-xs" />
                        </div>
                        <div class="flex flex-col">
                          <span class="font-bold text-white">{{ entry.email }}</span>
                          <span v-if="entry.email.toLowerCase() === authStore.user?.email?.toLowerCase()" class="text-[10px] text-amber-300 font-semibold">
                            (Current User)
                          </span>
                        </div>
                      </div>
                    </td>

                    <td class="py-3.5 px-4">
                      <span
                        :class="[
                          'text-[10px] font-black uppercase px-2.5 py-0.5 rounded-full border',
                          entry.role === 'superadmin'
                            ? 'bg-amber-500/20 text-amber-300 border-amber-400/40'
                            : 'bg-purple-500/20 text-purple-200 border-purple-400/30'
                        ]"
                      >
                        {{ entry.role }}
                      </span>
                    </td>

                    <td class="py-3.5 px-4 text-purple-300/80">
                      {{ entry.notes || '—' }}
                    </td>

                    <td class="py-3.5 px-4 text-purple-300/70 font-mono text-[11px]">
                      {{ new Date(entry.created_at).toLocaleDateString() }}
                    </td>

                    <td v-if="authStore.isSuperAdmin" class="py-3.5 px-4 text-right">
                      <Button
                        icon="pi pi-trash"
                        severity="danger"
                        text
                        size="small"
                        class="text-rose-400 hover:text-rose-200 p-1"
                        @click="handleRemoveAllowlistEntry(entry.email)"
                        title="Revoke Access"
                      />
                    </td>
                  </tr>

                  <tr v-if="allowlist.length === 0">
                    <td :colspan="authStore.isSuperAdmin ? 5 : 4" class="text-center py-8 text-purple-300/60">
                      {{ isAllowlistLoading ? 'Loading authorized accounts...' : 'No allowlist entries found.' }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Admin Launch QR Modal (Modern Frosted Glass Dialog) -->
    <Dialog
      v-model:visible="launchModalVisible"
      modal
      :header="launchedRoom?.title ? `${launchedRoom.title} Launched! 🚀` : 'Quiz Launched! 🚀'"
      :style="{ width: '92vw', maxWidth: '520px' }"
    >
      <div v-if="launchedRoom" class="space-y-5 py-2 text-center">
        <p class="text-xs sm:text-sm text-purple-200">
          Your quiz room is live! Players can scan this QR code with their mobile camera to move directly into the competition.
        </p>

        <!-- Big QR Code Display -->
        <div class="flex flex-col items-center justify-center p-5 rounded-2xl glass-card">
          <div class="bg-white p-3.5 rounded-2xl shadow-2xl mb-3">
            <img
              v-if="launchedRoom.qrUrl"
              :src="launchedRoom.qrUrl"
              alt="Scan to join quiz"
              class="w-52 h-52 sm:w-60 sm:h-60 object-contain rounded-lg"
            />
          </div>

          <span class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full bg-emerald-500/15 text-emerald-300 text-xs font-black uppercase tracking-wider mb-2 border border-emerald-400/30">
            <i class="pi pi-camera" />
            <span>Scan to join instantly on mobile</span>
          </span>

          <div class="flex items-center justify-center space-x-2 mt-1">
            <span class="text-xs text-purple-300/80 uppercase font-bold">Room PIN:</span>
            <span class="text-3xl font-mono font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-300 via-fuchsia-300 to-pink-300 tracking-wider">
              {{ launchedRoom.pin }}
            </span>
            <Button
              :icon="launchPinCopied ? 'pi pi-check' : 'pi pi-copy'"
              :class="launchPinCopied ? 'p-button-success' : 'p-button-secondary'"
              text
              rounded
              size="small"
              class="glass-pill w-8 h-8"
              @click="copyLaunchPin"
              title="Copy PIN"
            />
          </div>
        </div>

        <!-- Join Link Copy Bar -->
        <div class="flex items-center justify-between p-2.5 rounded-xl glass-card text-xs">
          <span class="text-purple-200 truncate max-w-[280px] font-mono text-[11px] px-2">
            {{ playerJoinUrl }}
          </span>
          <Button
            :label="launchLinkCopied ? 'Copied!' : 'Copy Link'"
            icon="pi pi-link"
            size="small"
            text
            class="text-purple-300 font-bold shrink-0 glass-pill px-3 py-1 hover:text-white"
            @click="copyLaunchLink"
          />
        </div>

        <!-- Action Buttons -->
        <div class="flex flex-col sm:flex-row gap-2.5 pt-2">
          <Button
            label="Open Host Screen (Projector)"
            icon="pi pi-external-link"
            class="flex-1 bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-bold text-white py-3 rounded-xl shadow-lg shadow-purple-900/40 active:scale-95 transition-all"
            @click="goToHostScreen"
          />
          <Button
            label="Close"
            text
            severity="secondary"
            class="glass-pill text-purple-200 hover:text-white"
            @click="launchModalVisible = false"
          />
        </div>
      </div>
    </Dialog>
  </div>
</template>
