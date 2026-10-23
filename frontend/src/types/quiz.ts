export interface Option {
  id: string;
  text: string;
  is_correct: boolean;
}

export interface Question {
  id: string;
  quiz_id?: string;
  question_text: string;
  question_type?: string;
  time_limit_secs: number;
  points: number;
  order_num: number;
  options: Option[];
}

export interface QuizSummary {
  id: string;
  title: string;
  description: string;
  question_count: number;
  total_time_secs?: number;
  created_at: string;
  theme_logo_url?: string | null;
  theme_primary_color?: string | null;
  theme_secondary_color?: string | null;
  theme_bg_color?: string | null;
  theme_btn_color?: string | null;
}

export interface QuizDetail {
  id: string;
  title: string;
  description: string;
  created_at: string;
  updated_at: string;
  questions: Question[];
  theme_logo_url?: string | null;
  theme_primary_color?: string | null;
  theme_secondary_color?: string | null;
  theme_bg_color?: string | null;
  theme_btn_color?: string | null;
}

export interface OrganizationConfig {
  id: string;
  name: string;
  logo_url: string | null;
  primary_color: string;
  secondary_color: string;
  background_color: string;
  button_color: string;
  updated_at: string;
}

export interface ThemeConfig {
  logo_url: string | null;
  primary_color: string;
  secondary_color: string;
  background_color: string;
  button_color: string;
  org_name: string;
}
