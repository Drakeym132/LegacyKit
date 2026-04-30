<script lang="ts">
  interface Step {
    id: string;
    label: string;
  }
  interface Props {
    steps: Step[];
    activeId: string;
    completedIds?: string[];
  }

  let { steps, activeId, completedIds = [] }: Props = $props();
</script>

<ol class="wizard-steps">
  {#each steps as step, idx}
    <li
      class:active={step.id === activeId}
      class:completed={completedIds.includes(step.id)}
    >
      <span class="num">{idx + 1}</span>
      <span class="label">{step.label}</span>
    </li>
  {/each}
</ol>

<style>
  .wizard-steps {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    list-style: none;
    padding: 0;
    margin: 0 0 var(--spacing-md);
    border-bottom: 1px solid var(--color-border);
  }
  li {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    color: var(--color-text-secondary);
    font-size: 0.8rem;
    font-weight: 600;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
  }
  li.active { color: var(--color-accent); border-bottom-color: var(--color-accent); }
  li.completed { color: var(--color-success); }
  .num {
    display: inline-grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    font-size: 0.7rem;
  }
  li.active .num {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: white;
  }
  li.completed .num {
    background: var(--color-success);
    border-color: var(--color-success);
    color: white;
  }
</style>
