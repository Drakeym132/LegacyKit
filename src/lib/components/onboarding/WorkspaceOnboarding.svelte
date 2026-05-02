<script lang="ts">
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { settingsStore } from '$lib/stores/settingsStore.svelte';

  let selectedPath = $state(settingsStore.workspaceRoot ?? '');
  let isSaving = $state(false);
  let errorMessage = $state<string | null>(null);

  async function chooseFolder() {
    errorMessage = null;
    const picked = await openDialog({
      directory: true,
      multiple: false,
      title: 'Choose LegacyKit Workspace',
      defaultPath: selectedPath || undefined,
    });
    if (picked && typeof picked === 'string') {
      selectedPath = picked;
    }
  }

  async function confirmWorkspace() {
    errorMessage = null;
    if (!selectedPath.trim()) {
      errorMessage = 'Choose a workspace folder to continue.';
      return;
    }
    isSaving = true;
    try {
      await settingsStore.setWorkspaceRoot(selectedPath.trim());
      await settingsStore.finishOnboarding();
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      isSaving = false;
    }
  }
</script>

<div class="overlay">
  <div class="modal">
    <h2>Choose Workspace Folder</h2>
    <p>
      LegacyKit will store IPSWs, SHSH blobs, extracted files, backups, logs, and temp data here.
      This is required before first use.
    </p>

    <div class="path-row">
      <input value={selectedPath} readonly placeholder="No folder selected" />
      <button class="secondary" onclick={chooseFolder}>Choose folder…</button>
    </div>

    {#if settingsStore.workspacePaths}
      <ul>
        <li><code>{settingsStore.workspacePaths.ipsw}</code></li>
        <li><code>{settingsStore.workspacePaths.ipswCustom}</code></li>
        <li><code>{settingsStore.workspacePaths.shsh}</code></li>
        <li><code>{settingsStore.workspacePaths.backups}</code></li>
      </ul>
    {/if}

    {#if errorMessage}
      <div class="error">{errorMessage}</div>
    {/if}

    <div class="actions">
      <button class="primary" onclick={confirmWorkspace} disabled={isSaving || !selectedPath.trim()}>
        {isSaving ? 'Saving…' : 'Confirm workspace'}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, black 55%, transparent);
    display: grid;
    place-items: center;
    z-index: 9999;
  }
  .modal {
    width: min(760px, calc(100vw - 32px));
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
  }
  .path-row { display: flex; gap: var(--spacing-sm); margin: var(--spacing-md) 0; }
  input {
    flex: 1;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
    color: var(--color-text-primary);
  }
  ul { margin: 0 0 var(--spacing-md) 0; padding-left: 1rem; }
  .error { color: var(--color-danger); margin-bottom: var(--spacing-sm); }
  .actions { display: flex; justify-content: flex-end; }
</style>

