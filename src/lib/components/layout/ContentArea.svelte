<script lang="ts">
  import TerminalDeck from './TerminalDeck.svelte';
  import { navigationStore } from '$lib/stores/navigationStore.svelte';

  import HomeView from '$lib/views/HomeView.svelte';
  import RestoreView from '$lib/views/RestoreView.svelte';
  import JailbreakView from '$lib/views/JailbreakView.svelte';
  import SHSHView from '$lib/views/SHSHView.svelte';
  import SSHRamdiskView from '$lib/views/SSHRamdiskView.svelte';
  import AppsView from '$lib/views/AppsView.svelte';
  import DataView from '$lib/views/DataView.svelte';
  import UtilitiesView from '$lib/views/UtilitiesView.svelte';
  import SettingsView from '$lib/views/SettingsView.svelte';
</script>

<main class="flex-1 flex flex-col overflow-hidden bg-[var(--color-bg-primary)]">
  <div class="flex-1 overflow-auto">
    <div class="content-shell">
      {#if navigationStore.currentView === 'home'}
          <HomeView />
        {:else if navigationStore.currentView === 'restore'}
          <RestoreView />
        {:else if navigationStore.currentView === 'jailbreak'}
          <JailbreakView />
        {:else if navigationStore.currentView === 'shsh'}
          <SHSHView />
        {:else if navigationStore.currentView === 'ssh-ramdisk'}
          <SSHRamdiskView />
        {:else if navigationStore.currentView === 'apps'}
          <AppsView />
        {:else if navigationStore.currentView === 'data'}
          <DataView />
        {:else if navigationStore.currentView === 'utilities'}
          <UtilitiesView />
        {:else if navigationStore.currentView === 'settings'}
          <SettingsView />
        {/if}
    </div>
  </div>

  <TerminalDeck />
</main>

<style>
  .content-shell {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    width: 100%;
  }
</style>
