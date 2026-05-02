<script lang="ts">
  import TerminalDeck from './TerminalDeck.svelte';
  import { navigationStore, viewOrder, type ViewName } from '$lib/stores/navigationStore.svelte';
  import { settingsStore } from '$lib/stores/settingsStore.svelte';

  import HomeView from '$lib/views/HomeView.svelte';
  import RestoreView from '$lib/views/RestoreView.svelte';
  import JailbreakView from '$lib/views/JailbreakView.svelte';
  import SHSHView from '$lib/views/SHSHView.svelte';
  import SSHRamdiskView from '$lib/views/SSHRamdiskView.svelte';
  import AppsView from '$lib/views/AppsView.svelte';
  import DataView from '$lib/views/DataView.svelte';
  import UtilitiesView from '$lib/views/UtilitiesView.svelte';
  import SettingsView from '$lib/views/SettingsView.svelte';

  // ----- Vertical view reel -----
  // The content area renders every view stacked vertically; navigating slides
  // the entire reel so intermediate views are visible during the journey
  // (e.g. Home → Settings sweeps through Restore, Jailbreak, etc.).
  //
  // Duration & easing mirror the sidebar's active-pill indicator so the
  // reel and the pill arrive at the destination together.
  //   - Easing: cubic-bezier(0.45, 0, 0.08, 1) — a near-linear middle so
  //     intermediate panes scroll past at a steady, perceptible rate, with a
  //     gentle slow-start and a soft ease-out tail for a calm landing. No
  //     overshoot.
  //   - Duration: longer ceiling than the sidebar pill alone would need so
  //     that on long sweeps (e.g. Home↔Settings) every intermediate view is
  //     actually on-screen long enough to register, not just blurred past.
  const MIN_MS = 300;
  const MAX_MS = 980;
  const PX_RATE = 1.75;
  // Approximate vertical distance between adjacent sidebar pill positions
  // (button height + gap, averaged across section headers). Tuned to match
  // the sidebar's measured durations closely enough to feel synchronized.
  const PX_PER_STEP = 50;

  // `prevIndex` is a plain ref (not $state) so updating it doesn't itself
  // trigger reactivity. We only care about the *delta at the moment the
  // navigation happens*, computed synchronously before the DOM is patched
  // — otherwise the browser would start the transition using the previous
  // run's `transition-duration` and only pick up the new value on the next
  // frame, which manifests as short hops inheriting long-sweep durations.
  let prevIndex = viewOrder.indexOf(navigationStore.currentView);
  let duration = $state(0);

  const currentIndex = $derived(viewOrder.indexOf(navigationStore.currentView));

  $effect.pre(() => {
    const next = currentIndex;
    if (next === prevIndex) return;
    if (settingsStore.reduceMotion) {
      // Honour the accessibility preference: jump to the new view without
      // animating. Update prevIndex so the next "real" navigation still
      // computes the correct delta if the user toggles the setting back off.
      duration = 0;
    } else {
      const dy = Math.abs(next - prevIndex) * PX_PER_STEP;
      duration = Math.min(MAX_MS, Math.max(MIN_MS, MIN_MS + dy * PX_RATE));
    }
    prevIndex = next;
  });

  const views: { view: ViewName; component: typeof HomeView }[] = [
    { view: 'home', component: HomeView },
    { view: 'restore', component: RestoreView },
    { view: 'jailbreak', component: JailbreakView },
    { view: 'shsh', component: SHSHView },
    { view: 'ssh-ramdisk', component: SSHRamdiskView },
    { view: 'apps', component: AppsView },
    { view: 'data', component: DataView },
    { view: 'utilities', component: UtilitiesView },
    { view: 'settings', component: SettingsView },
  ];
</script>

<main class="flex-1 flex flex-col overflow-hidden bg-[var(--color-bg-primary)]">
  <div class="flex-1 relative overflow-hidden">
    <div
      class="reel"
      style="transform: translateY(-{currentIndex * 100}%); transition-duration: {duration}ms;"
    >
      {#each views as { view, component: View } (view)}
        <section class="slot" aria-hidden={view !== navigationStore.currentView}>
          <div class="content-shell">
            <View />
          </div>
        </section>
      {/each}
    </div>
  </div>

  <TerminalDeck />
</main>

<style>
  .reel {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    /* Match the sidebar pill's easing curve so both motions feel unified. */
    transition-property: transform;
    /* Near-linear middle with a soft ease-out tail. The flatter middle
       keeps intermediate views moving past at a perceptible, steady rate
       during long sweeps (e.g. Home → Settings) so every pane registers,
       while the curved tail still gives a soft landing at the destination. */
    transition-timing-function: cubic-bezier(0.45, 0, 0.08, 1);
    will-change: transform;
  }

  .slot {
    flex: 0 0 100%;
    height: 100%;
    overflow-y: auto;
  }

  .content-shell {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    width: 100%;
  }
</style>
