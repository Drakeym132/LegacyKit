<script lang="ts">
  // Toolbar component
</script>

<header
  class="h-[48px] flex items-center justify-between pl-[88px] pr-3 border-b border-[var(--color-border)] bg-[var(--color-bg-sidebar)] backdrop-blur-2xl shrink-0"
  data-tauri-drag-region
>
  <div class="flex items-center gap-2 text-[var(--color-text-secondary)]" data-tauri-drag-region>
    <!-- Contextual actions will go here -->
  </div>
  <div class="flex items-center gap-2">
    <button
      class="w-8 h-8 rounded-md flex items-center justify-center hover:bg-black/5 dark:hover:bg-white/10 transition-colors text-[var(--color-text-secondary)]"
      data-tauri-drag-region="false"
    >
      ⚙️
    </button>
  </div>
</header>
