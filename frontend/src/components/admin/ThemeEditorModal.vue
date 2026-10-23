<script setup lang="ts">
import { ref, watch } from 'vue';
import { api } from '@/services/api';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import ThemePreviewCard from '@/components/common/ThemePreviewCard.vue';

const props = defineProps<{
  visible: boolean;
  title: string;
  initialTheme: {
    logo_url?: string | null;
    primary_color: string;
    secondary_color: string;
    background_color: string;
    button_color: string;
    org_name?: string;
  };
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'save', theme: typeof props.initialTheme): void;
}>();

const themeData = ref({ ...props.initialTheme });
const isUploading = ref(false);

watch(
  () => props.initialTheme,
  (newVal) => {
    themeData.value = { ...newVal };
  },
  { deep: true }
);

function handleFileUpload(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = async (e) => {
    const base64 = e.target?.result as string;
    isUploading.value = true;
    try {
      const res = await api.uploadImage(base64, file.name);
      themeData.value.logo_url = res.url;
    } catch (err) {
      console.error('Failed to upload image:', err);
    } finally {
      isUploading.value = false;
    }
  };
  reader.readAsDataURL(file);
}

function removeLogo() {
  themeData.value.logo_url = null;
}

function save() {
  emit('save', { ...themeData.value });
  emit('update:visible', false);
}
</script>

<template>
  <Dialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    modal
    :header="title"
    :style="{ width: '90vw', maxWidth: '750px' }"
    class="p-dialog-dark"
  >
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 py-2">
      <!-- Form Controls -->
      <div class="space-y-4">
        <!-- Logo Upload -->
        <div class="space-y-1.5">
          <label class="block text-xs font-bold uppercase text-slate-400">Logo Image</label>
          <div class="flex items-center space-x-3">
            <input
              type="file"
              accept="image/*"
              @change="handleFileUpload"
              class="text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-indigo-600 file:text-white hover:file:bg-indigo-500 cursor-pointer"
            />
            <Button
              v-if="themeData.logo_url"
              icon="pi pi-trash"
              text
              rounded
              severity="danger"
              size="small"
              @click="removeLogo"
              title="Remove logo"
            />
          </div>
        </div>

        <!-- Color pickers -->
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <label class="block text-xs font-bold text-slate-400">Primary Color</label>
            <div class="flex items-center space-x-2">
              <input type="color" v-model="themeData.primary_color" class="w-8 h-8 rounded border-none cursor-pointer bg-transparent" />
              <InputText v-model="themeData.primary_color" class="p-inputtext-sm font-mono text-xs w-full" />
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="block text-xs font-bold text-slate-400">Secondary Color</label>
            <div class="flex items-center space-x-2">
              <input type="color" v-model="themeData.secondary_color" class="w-8 h-8 rounded border-none cursor-pointer bg-transparent" />
              <InputText v-model="themeData.secondary_color" class="p-inputtext-sm font-mono text-xs w-full" />
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="block text-xs font-bold text-slate-400">Background Color</label>
            <div class="flex items-center space-x-2">
              <input type="color" v-model="themeData.background_color" class="w-8 h-8 rounded border-none cursor-pointer bg-transparent" />
              <InputText v-model="themeData.background_color" class="p-inputtext-sm font-mono text-xs w-full" />
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="block text-xs font-bold text-slate-400">Button Color</label>
            <div class="flex items-center space-x-2">
              <input type="color" v-model="themeData.button_color" class="w-8 h-8 rounded border-none cursor-pointer bg-transparent" />
              <InputText v-model="themeData.button_color" class="p-inputtext-sm font-mono text-xs w-full" />
            </div>
          </div>
        </div>
      </div>

      <!-- Live Preview -->
      <div class="space-y-1.5">
        <label class="block text-xs font-bold uppercase text-slate-400">Live Preview</label>
        <ThemePreviewCard :theme="themeData" />
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end space-x-2 pt-4">
        <Button label="Cancel" text severity="secondary" @click="emit('update:visible', false)" />
        <Button label="Save Theme" icon="pi pi-check" class="bg-indigo-600 hover:bg-indigo-500 font-bold border-none" @click="save" />
      </div>
    </template>
  </Dialog>
</template>
