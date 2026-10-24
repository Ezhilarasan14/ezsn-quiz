export interface AdminUser {
  id: string;
  username: string;
  email?: string;
  name?: string;
  avatar_url?: string;
  role: string;
  organization_id?: string;
}

export interface LoginResponse {
  token: string;
  admin: AdminUser;
  organization?: any;
}

export interface AdminAllowlistEntry {
  email: string;
  role: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface GoogleAuthUrlResponse {
  configured: boolean;
  url?: string;
  state?: string;
  message?: string;
}
