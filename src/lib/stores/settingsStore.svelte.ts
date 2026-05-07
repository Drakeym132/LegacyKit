import {
  completeOnboarding,
  ensureWorkspaceLayout,
  getAppSettings,
  revealWorkspace,
  setWorkspaceRoot,
  type WorkspacePaths,
} from '$lib/api/settings';

const REDUCE_MOTION_KEY = 'legacykit.reduceMotion';
const GLASS_CHROME_KEY = 'legacykit.glassChrome';
const LEGACY_FLAT_CHROME_KEY = 'legacykit.flatChrome';
const SIDEBAR_OPACITY_KEY = 'legacykit.sidebarOpacity';
const CONTENT_OPACITY_KEY = 'legacykit.contentOpacity';

const DEFAULT_SIDEBAR_OPACITY = 0.42;
const DEFAULT_CONTENT_OPACITY = 0.38;

function loadOpacity(key: string, fallback: number): number {
  if (typeof window === 'undefined') return fallback;
  const stored = window.localStorage.getItem(key);
  if (stored === null) return fallback;
  const parsed = Number(stored);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(1, Math.max(0, parsed));
}

function loadReduceMotion(): boolean {
  if (typeof window === 'undefined') return false;
  const stored = window.localStorage.getItem(REDUCE_MOTION_KEY);
  if (stored === 'true') return true;
  if (stored === 'false') return false;
  // No explicit user preference yet — default to the OS setting so users who
  // already have "Reduce motion" enabled system-wide get the right behaviour
  // out of the box.
  return window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
}

function loadGlassChrome(): boolean {
  if (typeof window === 'undefined') return false;

  const stored = window.localStorage.getItem(GLASS_CHROME_KEY);
  if (stored === 'true') return true;
  if (stored === 'false') return false;

  // Legacy migration: honour previous "Floating tiles" preference once.
  return window.localStorage.getItem(LEGACY_FLAT_CHROME_KEY) === 'true';
}

class SettingsStore {
  theme = $state<'system' | 'light' | 'dark'>('system');
  terminalVisible = $state<boolean>(false);
  terminalHeight = $state<number>(200);
  autoDetectDevice = $state<boolean>(true);
  pollIntervalMs = $state<number>(15000);
  autoEnterPwnDfu = $state<boolean>(false);
  reduceMotion = $state<boolean>(loadReduceMotion());
  glassChrome = $state<boolean>(loadGlassChrome());
  sidebarOpacity = $state<number>(loadOpacity(SIDEBAR_OPACITY_KEY, DEFAULT_SIDEBAR_OPACITY));
  contentOpacity = $state<number>(loadOpacity(CONTENT_OPACITY_KEY, DEFAULT_CONTENT_OPACITY));
  workspaceRoot = $state<string | null>(null);
  onboarded = $state<boolean>(false);
  loaded = $state<boolean>(false);

  workspacePaths = $state<WorkspacePaths | null>(null);

  setTheme(theme: 'system' | 'light' | 'dark') {
    this.theme = theme;
  }

  setReduceMotion(value: boolean) {
    this.reduceMotion = value;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(REDUCE_MOTION_KEY, value ? 'true' : 'false');
    }
  }

  setGlassChrome(value: boolean) {
    this.glassChrome = value;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(GLASS_CHROME_KEY, value ? 'true' : 'false');
    }
  }

  setSidebarOpacity(value: number) {
    const clamped = Math.min(1, Math.max(0, value));
    this.sidebarOpacity = clamped;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(SIDEBAR_OPACITY_KEY, String(clamped));
    }
  }

  setContentOpacity(value: number) {
    const clamped = Math.min(1, Math.max(0, value));
    this.contentOpacity = clamped;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(CONTENT_OPACITY_KEY, String(clamped));
    }
  }

  toggleTerminal() {
    this.terminalVisible = !this.terminalVisible;
  }

  setTerminalHeight(height: number) {
    this.terminalHeight = Math.max(100, Math.min(600, height));
  }

  setPollInterval(ms: number) {
    this.pollIntervalMs = Math.max(1000, ms);
  }

  async load() {
    const settings = await getAppSettings();
    this.workspaceRoot = settings.workspaceRoot;
    this.onboarded = settings.onboarded;
    this.loaded = true;
    if (this.workspaceRoot) {
      this.workspacePaths = await ensureWorkspaceLayout();
    }
  }

  async setWorkspaceRoot(path: string) {
    const settings = await setWorkspaceRoot(path);
    this.workspaceRoot = settings.workspaceRoot;
    this.onboarded = settings.onboarded;
    this.workspacePaths = await ensureWorkspaceLayout();
  }

  async finishOnboarding() {
    const settings = await completeOnboarding();
    this.workspaceRoot = settings.workspaceRoot;
    this.onboarded = settings.onboarded;
    if (this.workspaceRoot) {
      this.workspacePaths = await ensureWorkspaceLayout();
    }
  }

  async refreshWorkspaceLayout() {
    this.workspacePaths = await ensureWorkspaceLayout();
  }

  async revealWorkspace() {
    await revealWorkspace();
  }

  restartOnboarding() {
    this.onboarded = false;
  }
}

export const settingsStore = new SettingsStore();
