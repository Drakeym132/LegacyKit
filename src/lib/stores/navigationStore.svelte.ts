// Navigation views that match the sidebar items
export type ViewName =
  | 'home'
  | 'restore'
  | 'jailbreak'
  | 'shsh'
  | 'ssh-ramdisk'
  | 'apps'
  | 'data'
  | 'utilities'
  | 'settings';

export const viewTitles: Record<ViewName, string> = {
  home: 'Home',
  restore: 'Restore & Downgrade',
  jailbreak: 'Jailbreak',
  shsh: 'SHSH Blobs',
  'ssh-ramdisk': 'SSH Ramdisk',
  apps: 'App Management',
  data: 'Data Management',
  utilities: 'Utilities',
  settings: 'Settings',
};

class NavigationStore {
  currentView = $state<ViewName>('home');
  previousView = $state<ViewName | null>(null);

  navigate(view: ViewName) {
    this.previousView = this.currentView;
    this.currentView = view;
  }

  goBack() {
    if (this.previousView) {
      this.currentView = this.previousView;
      this.previousView = null;
    }
  }
}

export const navigationStore = new NavigationStore();
