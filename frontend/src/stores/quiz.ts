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

  /**
   * Transform frontend Question[] shape into the backend CreateQuizRequest shape.
   * Frontend uses: options[].text, time_limit_secs, order_num
   * Backend expects: options[].option_text, time_limit_sec (no trailing 's')
   */
  function toApiPayload(quiz: Partial<QuizDetail>) {
    return {
      title: quiz.title ?? '',
      description: quiz.description ?? null,
      organization_id: (quiz as any).organization_id ?? null,
      theme_logo_url: quiz.theme_logo_url ?? null,
      theme_primary_color: quiz.theme_primary_color ?? null,
      theme_secondary_color: quiz.theme_secondary_color ?? null,
      theme_bg_color: quiz.theme_bg_color ?? null,
      theme_btn_color: quiz.theme_btn_color ?? null,
      host_name: (quiz as any).host_name ?? null,
      branch_name: (quiz as any).branch_name ?? null,
      questions: (quiz.questions ?? []).map((q: any) => ({
        question_text: q.question_text ?? q.text ?? '',
        text: q.question_text ?? q.text ?? '',
        time_limit_sec: q.time_limit_secs ?? q.time_limit_sec ?? 20,
        time_limit_secs: q.time_limit_secs ?? q.time_limit_sec ?? 20,
        points: q.points ?? 1000,
        options: (q.options ?? [])
          .filter((o: any) => (o.text ?? o.option_text ?? '').trim().length > 0)
          .map((o: any) => ({
            option_text: o.option_text ?? o.text ?? '',
            text: o.text ?? o.option_text ?? '',
            is_correct: !!o.is_correct,
          })),
      })),
    };
  }

  async function saveQuiz(quiz: Partial<QuizDetail>): Promise<string | null> {
    isLoading.value = true;
    error.value = null;
    try {
      const payload = toApiPayload(quiz);
      if (quiz.id) {
        const updated = await api.updateQuiz(quiz.id, payload);
        currentQuiz.value = updated;
        await fetchQuizzes();
        return updated.id;
      } else {
        const created = await api.createQuiz(payload);
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
