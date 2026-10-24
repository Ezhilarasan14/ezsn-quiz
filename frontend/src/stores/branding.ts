import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '@/services/api';
import type { OrganizationConfig, ThemeConfig } from '@/types/quiz';

export const MAIN_BRAND_LOGO = 'https://ezsnapps.in/landing_page/logo.png';
export const MAIN_BRAND_NAME = 'EZSN Quiz';

const DEFAULT_BRANDING: ThemeConfig = {
  org_name: 'EZSN Quiz',
  logo_url: null,
  primary_color: '#0F172A', // Navbar background color
  secondary_color: '#EC4899',
  background_color: '#0B0F19',
  button_color: '#4F46E5',
};

export const useBrandingStore = defineStore('branding', () => {
  const currentTheme = ref<ThemeConfig>({ ...DEFAULT_BRANDING });
  const orgConfig = ref<OrganizationConfig | null>(null);
  const isLoading = ref(false);

  // The only color that can change across the UI is the navigation bar background color
  const navbarColor = computed(() => currentTheme.value.primary_color || '#0F172A');

  function applyTheme(theme: Partial<ThemeConfig>) {
    const orgName = (theme.org_name && theme.org_name !== 'EZSN Quiz')
      ? theme.org_name
      : (orgConfig.value?.name || currentTheme.value.org_name || theme.org_name || 'EZSN Quiz');

    const logoUrl = theme.logo_url !== undefined && theme.logo_url !== null && theme.logo_url !== ''
      ? theme.logo_url
      : (orgConfig.value?.logo_url || currentTheme.value.logo_url || null);

    currentTheme.value = {
      ...currentTheme.value,
      ...theme,
      org_name: orgName,
      logo_url: logoUrl,
    };

    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      root.style.setProperty('--brand-navbar-bg', navbarColor.value);
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
    MAIN_BRAND_LOGO,
    MAIN_BRAND_NAME,
    currentTheme,
    navbarColor,
    orgConfig,
    isLoading,
    applyTheme,
    fetchOrgConfig,
    updateOrgConfig,
    resetToDefaults,
  };
});
