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

// Canonical sidebar order. Used to compute slide direction so the content
// area transitions feel like a horizontal carousel: forward (down the list)
// slides the new view in from the right, backward slides it in from the left.
export const viewOrder: ViewName[] = [
  'home',
  'restore',
  'jailbreak',
  'shsh',
  'ssh-ramdisk',
  'apps',
  'data',
  'utilities',
  'settings',
];

class NavigationStore {
  currentView = $state<ViewName>('home');
  previousView = $state<ViewName | null>(null);
  /** +1 = navigating forward (down the sidebar), -1 = backward (up). */
  direction = $state<1 | -1>(1);

  navigate(view: ViewName) {
    if (view === this.currentView) return;
    const from = viewOrder.indexOf(this.currentView);
    const to = viewOrder.indexOf(view);
    // Update direction BEFORE the view so transition params capture the
    // correct value on the same render tick.
    this.direction = to >= from ? 1 : -1;
    this.previousView = this.currentView;
    this.currentView = view;
  }

  goBack() {
    if (this.previousView) {
      this.navigate(this.previousView);
    }
  }
}

export const navigationStore = new NavigationStore();
