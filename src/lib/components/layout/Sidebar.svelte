<script lang="ts">
  import DeviceCard from '../device/DeviceCard.svelte';
  import { navigationStore } from '$lib/stores/navigationStore.svelte';
  import type { ViewName } from '$lib/stores/navigationStore.svelte';
  import { settingsStore } from '$lib/stores/settingsStore.svelte';
  import { fly } from 'svelte/transition';
  import { tick, type Component } from 'svelte';
  // Phosphor Icons — closest free analogue to Apple's SF Symbols.
  // We mirror SF Symbols' sidebar convention: `regular` weight when
  // inactive, `fill` weight on the active row, so the selected item
  // reads as the focal point of the column.
  import {
    House,
    DownloadSimple,
    LockOpen,
    BoxArrowDown,
    HardDrives,
    SquaresFour,
    Database,
    Wrench,
    GearSix,
  } from 'phosphor-svelte';

  // ----- Sliding active-tab indicator -----
  // A single absolutely-positioned pill that translates between buttons,
  // giving a subtle slide + crossfade instead of snapping the background
  // colour on each item independently.
  let buttonRefs = $state<Partial<Record<ViewName, HTMLButtonElement>>>({});
  let indicator = $state({ y: 0, h: 0, duration: 280, visible: false });

  // Scale slide duration with distance so short hops stay snappy and long
  // travel (e.g. Settings → Home) doesn't feel rushed.
  // The pill is intentionally faster than the content reel so the active
  // selection lands quickly and the reel catches up underneath it. Tuned to
  // feel snappy on adjacent hops while still scaling up for long sweeps.
  const MIN_MS = 180;
  const MAX_MS = 550;
  const PX_RATE = 0.85;

  // Track the previously-active view in a plain (non-reactive) ref so the
  // distance calculation is always based on the *actual* button-to-button
  // delta of the latest navigation, not on stale `indicator.y` values that
  // can leak across rapid back-to-back clicks.
  let prevView: ViewName | null = null;

  $effect(() => {
    const view = navigationStore.currentView;
    // Re-measure after DOM updates so newly-rendered buttons are positioned.
    void tick().then(() => {
      const btn = buttonRefs[view];
      if (!btn) return;
      const nextY = btn.offsetTop;
      const nextH = btn.offsetHeight;

      let duration = 0;
      if (
        !settingsStore.reduceMotion &&
        indicator.visible &&
        prevView !== null &&
        prevView !== view
      ) {
        const prevBtn = buttonRefs[prevView];
        const prevY = prevBtn?.offsetTop ?? nextY;
        const dy = Math.abs(nextY - prevY);
        duration = Math.min(MAX_MS, Math.max(MIN_MS, MIN_MS + dy * PX_RATE));
      }

      prevView = view;
      indicator = { y: nextY, h: nextH, duration, visible: true };
    });
  });

  type NavItem = { label: string; icon: Component; view: ViewName };
  type NavSection = { title?: string; items: NavItem[] };

  const sections: NavSection[] = [
    {
      items: [{ label: 'Home', icon: House, view: 'home' }],
    },
    {
      title: 'Tools',
      items: [
        { label: 'Restore', icon: DownloadSimple, view: 'restore' },
        { label: 'Jailbreak', icon: LockOpen, view: 'jailbreak' },
        { label: 'SHSH Blobs', icon: BoxArrowDown, view: 'shsh' },
        { label: 'SSH Ramdisk', icon: HardDrives, view: 'ssh-ramdisk' },
      ],
    },
    {
      title: 'Device',
      items: [
        { label: 'Apps', icon: SquaresFour, view: 'apps' },
        { label: 'Data', icon: Database, view: 'data' },
      ],
    },
    {
      title: 'System',
      items: [
        { label: 'Utilities', icon: Wrench, view: 'utilities' },
        { label: 'Settings', icon: GearSix, view: 'settings' },
      ],
    },
  ];
</script>

<aside
  class="w-[244px] h-[calc(100%-var(--shell-inset-top))] mt-[var(--shell-inset-top)] bg-[var(--color-bg-sidebar)] rounded-[var(--content-radius)] border border-[var(--color-border)] shadow-[var(--bezel-shadow)] flex flex-col shrink-0 backdrop-blur-2xl backdrop-saturate-150"
>
  <!-- Drag region behind stoplights. Sized to clear the macOS traffic-light
       buttons and align the first nav item with the toolbar's title baseline.
       The app title now lives in the toolbar (Toolbar.svelte) instead of
       the sidebar, so this drag region is the only thing standing between
       the stoplights and the navigation list. -->
  <div class="h-[52px] shrink-0" data-tauri-drag-region></div>

  <div class="flex flex-col flex-1 min-h-0 px-4 pb-5">
    <nav class="flex-1 overflow-y-auto -mx-1 px-1 relative">
      <!-- Sliding active-tab indicator: a single pill that translates between
           buttons. Hidden until first measurement to avoid a 0,0 flash. -->
      <div
        class="absolute left-1 right-1 rounded-lg bg-[var(--color-accent)] pointer-events-none"
        style="top: {indicator.y}px; height: {indicator.h}px; opacity: {indicator.visible ? 1 : 0}; transition-property: top, height, opacity; transition-duration: {indicator.duration}ms, {indicator.duration}ms, 200ms; transition-timing-function: cubic-bezier(0.45, 0, 0.08, 1), cubic-bezier(0.45, 0, 0.08, 1), ease-out;"
      ></div>
      {#each sections as section, i}
        {#if section.title}
          <div
            class="px-3 {i === 0
              ? 'pt-2'
              : 'pt-5'} pb-1.5 text-[11px] font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]"
          >
            {section.title}
          </div>
        {/if}
        <ul class="flex flex-col gap-1.5 m-0 p-0 list-none">
          {#each section.items as item}
            {@const isActive = navigationStore.currentView === item.view}
            {@const Icon = item.icon}
            <li>
              <button
                bind:this={buttonRefs[item.view]}
                class="relative z-10 w-full px-3 py-2.5 rounded-lg text-[16px] font-medium cursor-pointer transition-colors duration-200 flex items-center gap-3 border-0 text-left bg-transparent
                  {isActive
                  ? 'text-white font-semibold'
                  : 'text-[var(--color-text-primary)] hover:bg-black/[0.05] dark:hover:bg-white/[0.07]'}"
                onclick={() => navigationStore.navigate(item.view)}
              >
                <span
                  class="shrink-0 w-[22px] h-[22px] flex items-center justify-center {isActive
                    ? 'text-white'
                    : 'text-[var(--color-text-secondary)]'}"
                >
                  <!-- SF-Symbols-style behaviour: regular weight inactive,
                       fill weight on the active row so the selected item
                       reads as the focal point of the column. -->
                  <Icon size={22} weight={isActive ? 'fill' : 'regular'} />
                </span>
                <span class="truncate">{item.label}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/each}
    </nav>

    {#if navigationStore.currentView !== 'home'}
      <div class="mt-auto pt-6" transition:fly={{ y: 12, duration: settingsStore.reduceMotion ? 0 : 220 }}>
        <DeviceCard />
      </div>
    {/if}
  </div>
</aside>
