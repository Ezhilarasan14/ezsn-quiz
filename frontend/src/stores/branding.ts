import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from '@/services/api';
import type { OrganizationConfig, ThemeConfig } from '@/types/quiz';

export const MAIN_BRAND_NAME = 'quiz ezsnapps';

const DEFAULT_BRANDING: ThemeConfig = {
  org_name: 'quiz ezsnapps',
  host_name: 'Ezhil Arasan',
  branch_name: 'Main Campus',
  logo_url: null,
  primary_color: '#4F46E5',
  secondary_color: '#EC4899',
  background_color: '#0a0416',
  button_color: '#9333EA',
};

export const useBrandingStore = defineStore('branding', () => {
  const currentTheme = ref<ThemeConfig>({ ...DEFAULT_BRANDING });
  const orgConfig = ref<OrganizationConfig | null>(null);
  const isLoading = ref(false);

  function applyTheme(theme: Partial<ThemeConfig>) {
    const orgName = theme.org_name !== undefined && theme.org_name !== null && theme.org_name.trim() !== ''
      ? theme.org_name.trim()
      : (orgConfig.value?.name || currentTheme.value.org_name || 'quiz ezsnapps');

    const hostName = theme.host_name !== undefined && theme.host_name !== null && theme.host_name.trim() !== ''
      ? theme.host_name.trim()
      : (currentTheme.value.host_name || 'Ezhil Arasan');

    const branchName = theme.branch_name !== undefined && theme.branch_name !== null
      ? theme.branch_name.trim()
      : (orgConfig.value?.branch_name || currentTheme.value.branch_name || 'Main Campus');

    const logoUrl = theme.logo_url !== undefined
      ? (theme.logo_url && theme.logo_url.trim() !== '' ? theme.logo_url.trim() : null)
      : (orgConfig.value?.logo_url || currentTheme.value.logo_url || null);

    currentTheme.value = {
      ...currentTheme.value,
      org_name: orgName,
      host_name: hostName,
      branch_name: branchName,
      logo_url: logoUrl,
    };
  }

  async function fetchOrgConfig() {
    isLoading.value = true;
    try {
      const config = await api.getOrganization();
      orgConfig.value = config;
      applyTheme({
        org_name: config.name,
        branch_name: config.branch_name,
        logo_url: config.logo_url,
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
        branch_name: updated.branch_name,
        logo_url: updated.logo_url,
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
    MAIN_BRAND_NAME,
    currentTheme,
    orgConfig,
    isLoading,
    applyTheme,
    fetchOrgConfig,
    updateOrgConfig,
    resetToDefaults,
  };
});
