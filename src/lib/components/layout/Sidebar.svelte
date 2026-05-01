<script lang="ts">
  import DeviceCard from '../device/DeviceCard.svelte';
  import { navigationStore } from '$lib/stores/navigationStore.svelte';
  import type { ViewName } from '$lib/stores/navigationStore.svelte';
  import { fly } from 'svelte/transition';

  type NavItem = { label: string; icon: string; view: ViewName };
  type NavSection = { title?: string; items: NavItem[] };

  const sections: NavSection[] = [
    {
      items: [
        { label: 'Home', icon: '🏠', view: 'home' },
      ],
    },
    {
      title: 'Tools',
      items: [
        { label: 'Restore', icon: '⬇️', view: 'restore' },
        { label: 'Jailbreak', icon: '🔓', view: 'jailbreak' },
        { label: 'SHSH Blobs', icon: '💾', view: 'shsh' },
        { label: 'SSH Ramdisk', icon: '🖥️', view: 'ssh-ramdisk' },
      ],
    },
    {
      title: 'Device',
      items: [
        { label: 'Apps', icon: '📱', view: 'apps' },
        { label: 'Data', icon: '📦', view: 'data' },
      ],
    },
    {
      title: 'System',
      items: [
        { label: 'Utilities', icon: '🔧', view: 'utilities' },
        { label: 'Settings', icon: '⚙️', view: 'settings' },
      ],
    },
  ];
</script>

<aside class="w-[240px] h-full bg-[var(--color-bg-sidebar)] border-r border-[var(--color-border)] flex flex-col shrink-0 backdrop-blur-2xl">
  <div class="h-7 shrink-0" data-tauri-drag-region></div>
  <div class="flex flex-col flex-1 min-h-0 px-3 pb-3">
    <nav class="flex-1 overflow-y-auto -mx-1 px-1">
      {#each sections as section, i}
        {#if section.title}
          <div class="px-3 {i === 0 ? 'pt-1' : 'pt-4'} pb-1 text-[11px] font-medium text-[var(--color-text-secondary)]">
            {section.title}
          </div>
        {/if}
        <ul class="flex flex-col gap-0.5 m-0 p-0 list-none">
          {#each section.items as item}
            {@const isActive = navigationStore.currentView === item.view}
            <li>
              <button
                class="w-full px-3 py-1.5 rounded-lg text-[13px] cursor-pointer transition-colors duration-100 flex items-center gap-2.5 border-0 text-left
                  {isActive
                    ? 'bg-black/[0.06] dark:bg-white/[0.08] text-[var(--color-accent)] font-medium'
                    : 'bg-transparent text-[var(--color-text-primary)] hover:bg-black/[0.04] dark:hover:bg-white/[0.06]'}"
                onclick={() => navigationStore.navigate(item.view)}
              >
                <span class="text-[15px] leading-none w-5 text-center">{item.icon}</span>
                <span class="truncate">{item.label}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/each}
    </nav>

    {#if navigationStore.currentView !== 'home'}
      <div class="mt-3" transition:fly={{ y: 12, duration: 220 }}>
        <DeviceCard />
      </div>
    {/if}
  </div>
</aside>
