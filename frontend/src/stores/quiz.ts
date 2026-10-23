import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from '@/services/api';
import type { QuizSummary, QuizDetail } from '@/types/quiz';

export const useQuizStore = defineStore('quiz', () => {
  const quizzes = ref<QuizSummary[]>([]);
  const currentQuiz = ref<QuizDetail | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchQuizzes() {
    isLoading.value = true;
    error.value = null;
    try {
      quizzes.value = await api.getQuizzes();
    } catch (e: any) {
      error.value = e.message || 'Failed to fetch quizzes';
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchQuiz(id: string): Promise<QuizDetail | null> {
    isLoading.value = true;
    error.value = null;
    try {
      const q = await api.getQuiz(id);
      currentQuiz.value = q;
      return q;
    } catch (e: any) {
      error.value = e.message || 'Failed to fetch quiz';
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveQuiz(quiz: Partial<QuizDetail>): Promise<string | null> {
    isLoading.value = true;
    error.value = null;
    try {
      if (quiz.id) {
        const updated = await api.updateQuiz(quiz.id, quiz);
        currentQuiz.value = updated;
        await fetchQuizzes();
        return updated.id;
      } else {
        const created = await api.createQuiz(quiz);
        await fetchQuizzes();
        return created.id;
      }
    } catch (e: any) {
      error.value = e.message || 'Failed to save quiz';
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  async function deleteQuiz(id: string): Promise<boolean> {
    try {
      await api.deleteQuiz(id);
      quizzes.value = quizzes.value.filter((q) => q.id !== id);
      return true;
    } catch (e: any) {
      error.value = e.message || 'Failed to delete quiz';
      return false;
    }
  }

  async function duplicateQuiz(id: string): Promise<string | null> {
    try {
      const res = await api.duplicateQuiz(id);
      await fetchQuizzes();
      return res.id;
    } catch (e: any) {
      error.value = e.message || 'Failed to duplicate quiz';
      return null;
    }
  }

  async function launchRoom(quizId: string): Promise<{ pin: string; host_token: string } | null> {
    try {
      return await api.launchRoom(quizId);
    } catch (e: any) {
      error.value = e.message || 'Failed to launch room';
      return null;
    }
  }

  return {
    quizzes,
    currentQuiz,
    isLoading,
    error,
    fetchQuizzes,
    fetchQuiz,
    saveQuiz,
    deleteQuiz,
    duplicateQuiz,
    launchRoom,
  };
});
