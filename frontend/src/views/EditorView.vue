<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuizStore } from '@/stores/quiz';
import type { Question, Option } from '@/types/quiz';

import Card from 'primevue/card';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Message from 'primevue/message';

const route = useRoute();
const router = useRouter();
const quizStore = useQuizStore();

const quizId = ref<string | null>((route.params.id as string) || null);
const title = ref('');
const description = ref('');
const questions = ref<Question[]>([]);
const errorMessage = ref('');
const isSaving = ref(false);

onMounted(async () => {
  if (quizId.value) {
    const data = await quizStore.fetchQuiz(quizId.value);
    if (data) {
      title.value = data.title;
      description.value = data.description || '';
      questions.value = data.questions.map((q) => ({
        ...q,
        options: q.options || [],
      }));
    }
  } else {
    // Start with 1 blank question
    addQuestion();
  }
});

function addQuestion() {
  const newQ: Question = {
    id: `temp_${Date.now()}_${Math.random()}`,
    question_text: '',
    time_limit_secs: 20,
    points: 1000,
    order_num: questions.value.length + 1,
    options: [
      { id: `opt_1_${Date.now()}`, text: '', is_correct: true },
      { id: `opt_2_${Date.now()}`, text: '', is_correct: false },
      { id: `opt_3_${Date.now()}`, text: '', is_correct: false },
      { id: `opt_4_${Date.now()}`, text: '', is_correct: false },
    ],
  };
  questions.value.push(newQ);
}

function removeQuestion(index: number) {
  if (questions.value.length <= 1) {
    alert('A quiz must have at least one question.');
    return;
  }
  questions.value.splice(index, 1);
}

function setCorrectOption(q: Question, optionIndex: number) {
  q.options.forEach((opt, idx) => {
    opt.is_correct = idx === optionIndex;
  });
}

async function handleSave() {
  errorMessage.value = '';
  if (!title.value.trim()) {
    errorMessage.value = 'Quiz title is required.';
    return;
  }

  for (let i = 0; i < questions.value.length; i++) {
    const q = questions.value[i];
    if (!q.question_text.trim()) {
      errorMessage.value = `Question ${i + 1} is missing question text.`;
      return;
    }
    const hasCorrect = q.options.some((o) => o.is_correct);
    if (!hasCorrect) {
      errorMessage.value = `Question ${i + 1} must have a correct option selected.`;
      return;
    }
    const hasValidOption = q.options.filter((o) => o.text.trim().length > 0).length >= 2;
    if (!hasValidOption) {
      errorMessage.value = `Question ${i + 1} must have at least 2 filled options.`;
      return;
    }
  }

  isSaving.value = true;
  try {
    await quizStore.saveQuiz({
      id: quizId.value || undefined,
      title: title.value,
      description: description.value,
      questions: questions.value,
    });
    router.push('/admin');
  } catch (e: any) {
    errorMessage.value = e.message || 'Failed to save quiz';
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-8 space-y-8">
    <!-- Header -->
    <div class="flex items-center justify-between pb-4 border-b border-slate-800">
      <div>
        <h1 class="text-3xl font-black text-white">
          {{ quizId ? 'Edit Quiz' : 'Create New Quiz' }}
        </h1>
        <p class="text-xs text-slate-400">Design questions, configure time limits, and set answers</p>
      </div>

      <div class="flex items-center space-x-3">
        <Button label="Cancel" text severity="secondary" @click="router.push('/admin')" />
        <Button
          label="Save Quiz"
          icon="pi pi-check"
          :loading="isSaving"
          class="bg-indigo-600 hover:bg-indigo-500 font-bold text-white border-none rounded-xl"
          @click="handleSave"
        />
      </div>
    </div>

    <Message v-if="errorMessage" severity="error" :closable="false" class="rounded-xl">
      {{ errorMessage }}
    </Message>

    <!-- General Quiz Info Card -->
    <Card class="bg-slate-900/80 border border-slate-800 rounded-2xl shadow-xl">
      <template #content>
        <div class="space-y-4">
          <div class="space-y-1">
            <label class="text-xs font-bold text-slate-400 uppercase">Quiz Title</label>
            <InputText v-model="title" placeholder="e.g. Science & Nature Trivia" class="w-full bg-slate-950/60 border-slate-800 font-bold" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-bold text-slate-400 uppercase">Description</label>
            <InputText v-model="description" placeholder="Brief summary for players..." class="w-full bg-slate-950/60 border-slate-800 text-sm" />
          </div>
        </div>
      </template>
    </Card>

    <!-- Questions Editor -->
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h2 class="text-xl font-bold text-white">Questions ({{ questions.length }})</h2>
        <Button
          label="Add Question"
          icon="pi pi-plus"
          size="small"
          class="bg-slate-800 hover:bg-slate-700 text-indigo-400 border border-indigo-500/30 font-bold rounded-xl"
          @click="addQuestion"
        />
      </div>

      <div v-for="(q, qIdx) in questions" :key="q.id" class="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-xl space-y-5">
        <div class="flex items-center justify-between">
          <span class="text-sm font-extrabold text-indigo-400 bg-indigo-500/10 px-3 py-1 rounded-full border border-indigo-500/20">
            Question {{ qIdx + 1 }}
          </span>

          <div class="flex items-center space-x-3">
            <select v-model.number="q.time_limit_secs" class="bg-slate-950 border border-slate-800 text-xs font-bold text-slate-300 rounded-lg px-2 py-1">
              <option :value="10">10 seconds</option>
              <option :value="20">20 seconds</option>
              <option :value="30">30 seconds</option>
              <option :value="60">60 seconds</option>
            </select>

            <Button
              icon="pi pi-trash"
              text
              rounded
              severity="danger"
              size="small"
              title="Remove question"
              @click="removeQuestion(qIdx)"
            />
          </div>
        </div>

        <!-- Question Text -->
        <InputText
          v-model="q.question_text"
          placeholder="Type your question here..."
          class="w-full bg-slate-950/80 border-slate-800 font-extrabold text-base text-white"
        />

        <!-- Options (4 tiles) -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
          <div
            v-for="(opt, optIdx) in q.options"
            :key="opt.id"
            :class="[
              'flex items-center space-x-3 p-3 rounded-xl border transition-all',
              opt.is_correct ? 'bg-emerald-950/40 border-emerald-500 ring-1 ring-emerald-500/30' : 'bg-slate-950/60 border-slate-800'
            ]"
          >
            <input
              type="radio"
              :name="`correct_${q.id}`"
              :checked="opt.is_correct"
              @change="setCorrectOption(q, optIdx)"
              class="w-4 h-4 text-emerald-500 focus:ring-emerald-500 cursor-pointer"
              title="Mark as correct answer"
            />

            <InputText
              v-model="opt.text"
              :placeholder="`Option ${optIdx + 1}`"
              class="w-full p-inputtext-sm bg-transparent border-none text-white text-sm font-semibold focus:ring-0"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
