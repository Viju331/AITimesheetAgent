export interface UserDto {
  id: string;
  name: string;
  email: string | null;
  createdAt: string;
}

export interface AiToolDto {
  id: string;
  name: string;
  displayName: string;
  sessionFolder: string | null;
  isEnabled: boolean;
  createdAt: string;
}

export interface ProjectDto {
  id: string;
  name: string;
  path: string;
  techStack: string | null;
  hasGit: boolean;
  gitRemote: string | null;
  isSelected: boolean;
  createdAt: string;
  lastScanned: string | null;
}

export interface OnboardingStateDto {
  hasUser: boolean;
  hasAiTool: boolean;
  hasProjects: boolean;
  isComplete: boolean;
}

export const ONBOARDING_STEP_COUNT = 5;
