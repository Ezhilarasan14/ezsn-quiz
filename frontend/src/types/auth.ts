export interface AdminUser {
  id: string;
  username: string;
  organization_id: string;
}

export interface LoginResponse {
  token: string;
  admin: AdminUser;
  expires_at: string;
}
