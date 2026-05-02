<script lang="ts">
  import DeviceCard from '../device/DeviceCard.svelte';
  import { navigationStore } from '$lib/stores/navigationStore.svelte';
  import type { ViewName } from '$lib/stores/navigationStore.svelte';
  import { fly } from 'svelte/transition';

  type IconName =
    | 'home'
    | 'restore'
    | 'jailbreak'
    | 'shsh'
    | 'ramdisk'
    | 'apps'
    | 'data'
    | 'utilities'
    | 'settings';

  type NavItem = { label: string; icon: IconName; view: ViewName };
  type NavSection = { title?: string; items: NavItem[] };

  const sections: NavSection[] = [
    {
      items: [{ label: 'Home', icon: 'home', view: 'home' }],
    },
    {
      title: 'Tools',
      items: [
        { label: 'Restore', icon: 'restore', view: 'restore' },
        { label: 'Jailbreak', icon: 'jailbreak', view: 'jailbreak' },
        { label: 'SHSH Blobs', icon: 'shsh', view: 'shsh' },
        { label: 'SSH Ramdisk', icon: 'ramdisk', view: 'ssh-ramdisk' },
      ],
    },
    {
      title: 'Device',
      items: [
        { label: 'Apps', icon: 'apps', view: 'apps' },
        { label: 'Data', icon: 'data', view: 'data' },
      ],
    },
    {
      title: 'System',
      items: [
        { label: 'Utilities', icon: 'utilities', view: 'utilities' },
        { label: 'Settings', icon: 'settings', view: 'settings' },
      ],
    },
  ];
</script>

<aside
  class="w-[268px] h-full bg-[var(--color-bg-sidebar)] border-r border-[var(--color-border)] flex flex-col shrink-0 backdrop-blur-2xl"
>
  <!-- Drag region behind stoplights -->
  <div class="h-10 shrink-0" data-tauri-drag-region></div>

  <!-- App title -->
  <div
    class="px-5 pt-2 pb-7 shrink-0 select-none"
    data-tauri-drag-region
  >
    <h1
      class="m-0 text-[30px] font-bold tracking-tight text-[var(--color-text-primary)] leading-none"
      data-tauri-drag-region
    >
      LegacyKit
    </h1>
  </div>

  <div class="flex flex-col flex-1 min-h-0 px-4 pb-5">
    <nav class="flex-1 overflow-y-auto -mx-1 px-1">
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
            <li>
              <button
                class="w-full px-3 py-2.5 rounded-lg text-[16px] font-medium cursor-pointer transition-colors duration-100 flex items-center gap-3 border-0 text-left
                  {isActive
                  ? 'bg-[var(--color-accent)] text-white font-semibold'
                  : 'bg-transparent text-[var(--color-text-primary)] hover:bg-black/[0.05] dark:hover:bg-white/[0.07]'}"
                onclick={() => navigationStore.navigate(item.view)}
              >
                <span
                  class="shrink-0 w-[22px] h-[22px] flex items-center justify-center {isActive
                    ? 'text-white'
                    : 'text-[var(--color-text-secondary)]'}"
                >
                  {#if item.icon === 'home'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><path d="M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-4v-7h-8v7H4a1 1 0 0 1-1-1v-9.5Z"/></svg>
                  {:else if item.icon === 'restore'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><path d="M21 15v3.5A2.5 2.5 0 0 1 18.5 21h-13A2.5 2.5 0 0 1 3 18.5V15"/><path d="M7.5 11 12 15.5 16.5 11"/><path d="M12 3v12.5"/></svg>
                  {:else if item.icon === 'jailbreak'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 7.5-2"/></svg>
                  {:else if item.icon === 'shsh'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><path d="M12 3 4 6v6c0 4.5 3.4 8.3 8 9 4.6-.7 8-4.5 8-9V6l-8-3Z"/><path d="m9 12 2 2 4-4"/></svg>
                  {:else if item.icon === 'ramdisk'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><rect x="2.5" y="4" width="19" height="13" rx="2"/><path d="M7 9.5 9.5 12 7 14.5"/><path d="M12.5 14.5h4"/><path d="M9 21h6"/><path d="M12 17v4"/></svg>
                  {:else if item.icon === 'apps'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></svg>
                  {:else if item.icon === 'data'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><ellipse cx="12" cy="5.5" rx="8" ry="2.5"/><path d="M4 5.5v6c0 1.4 3.6 2.5 8 2.5s8-1.1 8-2.5v-6"/><path d="M4 11.5v7c0 1.4 3.6 2.5 8 2.5s8-1.1 8-2.5v-7"/></svg>
                  {:else if item.icon === 'utilities'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><path d="M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.6 2.6-2.7-2.7 2.3-2.9Z"/></svg>
                  {:else if item.icon === 'settings'}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="w-[22px] h-[22px]"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"/></svg>
                  {/if}
                </span>
                <span class="truncate">{item.label}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/each}
    </nav>

    {#if navigationStore.currentView !== 'home'}
      <div class="mt-auto pt-6" transition:fly={{ y: 12, duration: 220 }}>
        <DeviceCard />
      </div>
    {/if}
  </div>
</aside>
