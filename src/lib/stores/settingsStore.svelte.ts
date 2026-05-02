import {
  completeOnboarding,
  ensureWorkspaceLayout,
  getAppSettings,
  revealWorkspace,
  setWorkspaceRoot,
  type WorkspacePaths,
} from '$lib/api/settings';

class SettingsStore {
  theme = $state<'system' | 'light' | 'dark'>('system');
  terminalVisible = $state<boolean>(false);
  terminalHeight = $state<number>(200);
  autoDetectDevice = $state<boolean>(true);
  pollIntervalMs = $state<number>(3000);
  workspaceRoot = $state<string | null>(null);
  onboarded = $state<boolean>(false);
  loaded = $state<boolean>(false);

  workspacePaths = $state<WorkspacePaths | null>(null);

  setTheme(theme: 'system' | 'light' | 'dark') {
    this.theme = theme;
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
