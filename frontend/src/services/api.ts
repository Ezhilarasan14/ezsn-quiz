import type { QuizSummary, QuizDetail, OrganizationConfig } from '@/types/quiz';
import type { AdminAllowlistEntry, AdminUser, GoogleAuthUrlResponse, LoginResponse } from '@/types/auth';

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

function getAuthHeader(): Record<string, string> {
  const token = localStorage.getItem('ezsn_admin_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const headers = {
    'Content-Type': 'application/json',
    ...getAuthHeader(),
    ...options.headers,
  };

  const response = await fetch(endpoint, {
    credentials: 'include',
    ...options,
    headers,
  });
  if (!response.ok) {
    let errMsg = `Request failed: ${response.statusText}`;
    try {
      const errJson = await response.json();
      errMsg = errJson.message || errJson.error || errMsg;
    } catch {
      // Use status text fallback
    }
    throw new ApiError(response.status, errMsg);
  }

  // Handle 204 No Content
  if (response.status === 204) {
    return {} as T;
  }

  return response.json();
}

export const api = {
  // Auth & Google Single Sign-On
  getGoogleAuthUrl: () => request<GoogleAuthUrlResponse>('/api/auth/google/url'),
  login: (credentials: { username: string; password: string }) =>
    request<LoginResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    }),
  getMe: () => request<{ admin: AdminUser; organization?: any }>('/api/auth/me'),
  logout: () =>
    request<{ success: boolean }>('/api/auth/logout', {
      method: 'POST',
    }),

  // Admin Allowlist (RBAC)
  getAllowlist: () => request<AdminAllowlistEntry[]>('/api/admin/allowlist'),
  addAllowlistEntry: (data: { email: string; role?: string; notes?: string }) =>
    request<AdminAllowlistEntry>('/api/admin/allowlist', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  removeAllowlistEntry: (email: string) =>
    request<{ status: string }>(`/api/admin/allowlist/${encodeURIComponent(email)}`, {
      method: 'DELETE',
    }),

  // Organization Branding
  getOrganization: () => request<OrganizationConfig>('/api/organization'),
  updateOrganization: (data: Partial<OrganizationConfig>) =>
    request<OrganizationConfig>('/api/organization', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // Quizzes
  getQuizzes: () => request<QuizSummary[]>('/api/quizzes'),
  getQuiz: (id: string) => request<QuizDetail>(`/api/quizzes/${id}`),
  createQuiz: (quiz: any) =>
    request<{ id: string }>('/api/quizzes', {
      method: 'POST',
      body: JSON.stringify(quiz),
    }),
  updateQuiz: (id: string, quiz: any) =>
    request<QuizDetail>(`/api/quizzes/${id}`, {
      method: 'PUT',
      body: JSON.stringify(quiz),
    }),
  deleteQuiz: (id: string) =>
    request<{ success: boolean }>(`/api/quizzes/${id}`, {
      method: 'DELETE',
    }),
  duplicateQuiz: (id: string) =>
    request<{ id: string }>(`/api/quizzes/${id}/duplicate`, {
      method: 'POST',
    }),
  launchRoom: (quizId: string) =>
    request<{ pin: string; host_token: string }>(`/api/rooms`, {
      method: 'POST',
      body: JSON.stringify({ quiz_id: quizId }),
    }),
  getRoomMeta: (pin: string) =>
    request<{ pin: string; quiz_title: string; is_active: boolean }>(`/api/rooms/${pin}`),

  // File Upload
  uploadImage: (dataUrl: string, filename?: string) =>
    request<{ url: string; filename: string }>('/api/upload', {
      method: 'POST',
      body: JSON.stringify({ data: dataUrl, filename }),
    }),

  // History & Player Analytics
  getHistory: () => request<any[]>('/api/history'),
  getQuizHistory: (quizId: string) => request<any[]>(`/api/quizzes/${quizId}/history`),
  getPlayerStats: (email: string) => request<any>(`/api/players/${encodeURIComponent(email)}/stats`),
  getPlayerReview: (gameId: string, playerIdOrEmail: string) =>
    request<import('@/types/game').PlayerQuizSummary>(`/api/history/${encodeURIComponent(gameId)}/review/${encodeURIComponent(playerIdOrEmail)}`),
};
