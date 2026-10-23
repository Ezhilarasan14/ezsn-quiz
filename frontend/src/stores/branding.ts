import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from '@/services/api';
import type { OrganizationConfig, ThemeConfig } from '@/types/quiz';

const DEFAULT_BRANDING: ThemeConfig = {
  org_name: 'EZSN Quiz',
  logo_url: null,
  primary_color: '#6366F1',
  secondary_color: '#EC4899',
  background_color: '#0F172A',
  button_color: '#4F46E5',
};

export const useBrandingStore = defineStore('branding', () => {
  const currentTheme = ref<ThemeConfig>({ ...DEFAULT_BRANDING });
  const orgConfig = ref<OrganizationConfig | null>(null);
  const isLoading = ref(false);

  function applyTheme(theme: Partial<ThemeConfig>) {
    currentTheme.value = {
      ...currentTheme.value,
      ...theme,
    };

    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      root.style.setProperty('--brand-primary', currentTheme.value.primary_color);
      root.style.setProperty('--brand-secondary', currentTheme.value.secondary_color);
      root.style.setProperty('--brand-bg', currentTheme.value.background_color);
      root.style.setProperty('--brand-btn', currentTheme.value.button_color);

      // Map to PrimeVue primary color variables
      root.style.setProperty('--p-primary-500', currentTheme.value.primary_color);
      root.style.setProperty('--p-primary-color', currentTheme.value.primary_color);
    }
  }

  async function fetchOrgConfig() {
    isLoading.value = true;
    try {
      const config = await api.getOrganization();
      orgConfig.value = config;
      applyTheme({
        org_name: config.name,
        logo_url: config.logo_url,
        primary_color: config.primary_color,
        secondary_color: config.secondary_color,
        background_color: config.background_color,
        button_color: config.button_color,
      });
    } catch (e) {
      console.warn('[Branding] Could not fetch organization config:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function updateOrgConfig(data: Partial<OrganizationConfig>) {
    isLoading.value = true;
    try {
      const updated = await api.updateOrganization(data);
      orgConfig.value = updated;
      applyTheme({
        org_name: updated.name,
        logo_url: updated.logo_url,
        primary_color: updated.primary_color,
        secondary_color: updated.secondary_color,
        background_color: updated.background_color,
        button_color: updated.button_color,
      });
      return updated;
    } finally {
      isLoading.value = false;
    }
  }

  function resetToDefaults() {
    applyTheme(DEFAULT_BRANDING);
  }

  return {
    currentTheme,
    orgConfig,
    isLoading,
    applyTheme,
    fetchOrgConfig,
    updateOrgConfig,
    resetToDefaults,
  };
});
