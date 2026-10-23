import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '@/services/api';
import type { AdminUser } from '@/types/auth';

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('ezsn_admin_token'));
  const user = ref<AdminUser | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const isAuthenticated = computed(() => !!token.value);

  async function login(username: string, password: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;
    try {
      const res = await api.login({ username, password });
      token.value = res.token;
      user.value = res.admin;
      localStorage.setItem('ezsn_admin_token', res.token);
      return true;
    } catch (e: any) {
      error.value = e.message || 'Login failed';
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  async function checkAuth(): Promise<boolean> {
    if (!token.value) return false;
    try {
      const res = await api.getMe();
      user.value = res.admin;
      return true;
    } catch {
      logout();
      return false;
    }
  }

  async function logout() {
    try {
      await api.logout();
    } catch {
      // Ignore network errors on logout
    } finally {
      token.value = null;
      user.value = null;
      localStorage.removeItem('ezsn_admin_token');
    }
  }

  return {
    token,
    user,
    isLoading,
    error,
    isAuthenticated,
    login,
    checkAuth,
    logout,
  };
});
