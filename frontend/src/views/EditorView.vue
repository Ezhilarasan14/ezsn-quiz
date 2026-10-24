<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuizStore } from '@/stores/quiz';
import type { Question } from '@/types/quiz';

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

const optionIcons = ['▲', '◆', '●', '■'];
const optionColors = [
  'text-rose-400 bg-rose-500/10 border-rose-500/20',
  'text-blue-400 bg-blue-500/10 border-blue-500/20',
  'text-amber-400 bg-amber-500/10 border-amber-500/20',
  'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
];

onMounted(async () => {
  if (quizId.value) {
    const data = await quizStore.fetchQuiz(quizId.value);
    if (data) {
      title.value = data.title;
      description.value = data.description || '';
      questions.value = data.questions.map((q: any) => ({
        id: q.id,
        question_text: q.question_text,
        time_limit_secs: q.time_limit_sec ?? q.time_limit_secs ?? 20,
        points: q.points ?? 1000,
        order_num: q.order_index ?? q.order_num ?? 0,
        options: (q.options || []).map((o: any) => ({
          id: o.id,
          text: o.option_text ?? o.text ?? '',
          is_correct: !!o.is_correct,
        })),
      }));
    }
  } else {
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
  <div class="max-w-4xl mx-auto px-4 py-6 sm:py-10 space-y-8">
    <!-- Header with Actions -->
    <div class="glass-panel rounded-2xl p-4 sm:p-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight">
          {{ quizId ? 'Edit Quiz Competition' : 'Create New Quiz' }}
        </h1>
        <p class="text-xs sm:text-sm text-purple-300/80 mt-0.5">Configure question prompts, correct answers, and countdown limits</p>
      </div>

      <div class="flex items-center space-x-3 w-full sm:w-auto">
        <Button
          label="Cancel"
          text
          severity="secondary"
          class="glass-pill text-purple-200 hover:text-white px-4 py-2"
          @click="router.push('/admin')"
        />
        <Button
          label="Save Quiz"
          icon="pi pi-check"
          :loading="isSaving"
          class="flex-1 sm:flex-none bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-black text-white px-6 py-2.5 rounded-xl shadow-lg shadow-purple-900/40"
          @click="handleSave"
        />
      </div>
    </div>

    <Message v-if="errorMessage" severity="error" :closable="false" class="rounded-xl">
      {{ errorMessage }}
    </Message>

    <!-- General Quiz Info Card -->
    <div class="glass-panel rounded-3xl p-6 sm:p-8 space-y-4 shadow-xl">
      <div class="space-y-1.5">
        <label class="text-xs font-bold uppercase tracking-wider text-purple-200">Quiz Title <span class="text-pink-400">*</span></label>
        <InputText v-model="title" placeholder="e.g. Science & Technology Championship" class="glass-input w-full font-bold text-base py-3 rounded-xl placeholder:text-purple-300/30 text-white" />
      </div>

      <div class="space-y-1.5">
        <label class="text-xs font-bold uppercase tracking-wider text-purple-200">Description</label>
        <InputText v-model="description" placeholder="Brief summary for players..." class="glass-input w-full text-sm py-2.5 rounded-xl placeholder:text-purple-300/30 text-purple-100" />
      </div>
    </div>

    <!-- Questions Editor -->
    <div class="space-y-6">
      <div class="flex items-center justify-between px-1">
        <div class="flex items-center space-x-2">
          <h2 class="text-xl font-black text-white tracking-tight">Questions</h2>
          <span class="glass-pill px-2.5 py-0.5 rounded-full text-xs font-bold text-purple-200">
            {{ questions.length }}
          </span>
        </div>

        <Button
          label="Add Question"
          icon="pi pi-plus"
          size="small"
          class="glass-panel hover:bg-purple-500/15 text-purple-200 border border-purple-400/30 font-bold rounded-xl px-4 py-2 text-xs"
          @click="addQuestion"
        />
      </div>

      <div
        v-for="(q, qIdx) in questions"
        :key="q.id"
        class="glass-panel rounded-3xl p-5 sm:p-7 space-y-5 shadow-xl transition-all"
      >
        <div class="flex items-center justify-between">
          <span class="text-xs sm:text-sm font-extrabold text-purple-200 glass-pill px-3 py-1 rounded-full border border-purple-400/30">
            Question {{ qIdx + 1 }}
          </span>

          <div class="flex items-center space-x-3">
            <select v-model.number="q.time_limit_secs" class="glass-input text-xs font-bold text-purple-200 rounded-xl px-3 py-1.5 cursor-pointer">
              <option :value="10">⏱️ 10 seconds</option>
              <option :value="20">⏱️ 20 seconds</option>
              <option :value="30">⏱️ 30 seconds</option>
              <option :value="60">⏱️ 60 seconds</option>
            </select>

            <Button
              icon="pi pi-trash"
              text
              rounded
              size="small"
              class="text-rose-400 hover:text-rose-300 hover:bg-rose-500/10 w-8 h-8"
              title="Remove question"
              @click="removeQuestion(qIdx)"
            />
          </div>
        </div>

        <!-- Question Text Prompt -->
        <div class="space-y-1.5">
          <label class="text-xs font-bold uppercase tracking-wider text-purple-300/80">Prompt / Question Text</label>
          <InputText
            v-model="q.question_text"
            placeholder="Type your question prompt here..."
            class="glass-input w-full font-black text-base sm:text-lg text-white py-3 rounded-xl placeholder:text-purple-300/30"
          />
        </div>

        <!-- Options (4 Answer Tiles) -->
        <div class="space-y-2 pt-1">
          <span class="text-xs font-bold uppercase tracking-wider text-purple-300/80 block">
            Options (Select correct radio button)
          </span>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div
              v-for="(opt, optIdx) in q.options"
              :key="opt.id"
              :class="[
                'flex items-center space-x-3 p-3 rounded-2xl border transition-all glass-card',
                opt.is_correct
                  ? 'border-emerald-500/60 bg-emerald-950/40 ring-2 ring-emerald-500/30'
                  : 'border-purple-400/15 hover:border-purple-400/30'
              ]"
            >
              <!-- Geometric icon symbol badge -->
              <span
                class="w-7 h-7 rounded-lg text-xs font-black flex items-center justify-center border shrink-0"
                :class="optionColors[optIdx % 4]"
              >
                {{ optionIcons[optIdx % 4] }}
              </span>

              <input
                type="radio"
                :name="`correct_${q.id}`"
                :checked="opt.is_correct"
                @change="setCorrectOption(q, optIdx)"
                class="w-4 h-4 text-emerald-500 focus:ring-emerald-500 cursor-pointer accent-emerald-500"
                title="Mark as correct answer"
              />

              <InputText
                v-model="opt.text"
                :placeholder="`Option ${optIdx + 1}`"
                class="w-full p-inputtext-sm bg-transparent border-none text-white text-sm font-bold focus:ring-0 placeholder:text-purple-300/30"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
