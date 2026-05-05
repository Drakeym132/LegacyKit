<script lang="ts">
  import {
    extractIpswComponent,
    modifyRamdisk,
    packImg4,
    patchIboot,
    patchKernel,
    repackImg3,
    type IbootBitWidth,
  } from '$lib/api/firmware';
  import { sendBootchain } from '$lib/api/jailbreak';
  import { recordJustBoot } from '$lib/api/justBoot';
  import { deviceStore } from '$lib/stores/deviceStore.svelte';
  import { settingsStore } from '$lib/stores/settingsStore.svelte';
  import { inferProcessorGen } from '$lib/utils/processorGen';
  import { createWorkingController } from '$lib/utils/workingState.svelte';

  let ipswPath = $state('');
  let bootArgs = $state('rd=md0 -v amfi_get_out_of_my_way=0x1 cs_enforcement_disable=1');
  let buildId = $state('');
  let iosVersion = $state('');
  let ibssIpswPath = $state('');
  let ibecIpswPath = $state('');
  let kernelIpswPath = $state('');
  let ramdiskIpswPath = $state('');
  let shshPath = $state('');
  let ramdiskTargetSizeMb = $state(35);

  let extractedIbss = $state('');
  let extractedIbec = $state('');
  let extractedKernel = $state('');
  let extractedRamdisk = $state('');
  let patchedIbss = $state('');
  let patchedIbec = $state('');
  let patchedKernel = $state('');

  const work = createWorkingController();

  let productType = $derived(deviceStore.state.product_type);
  let workspaceExtractedDir = $derived(settingsStore.workspacePaths?.extracted ?? '');
  let workspaceSshBinariesDir = $derived(settingsStore.workspacePaths?.sshBinaries ?? '');
  let processorGen = $derived(inferProcessorGen(productType));
  let bitWidth = $derived<IbootBitWidth>(processorGen !== null && processorGen >= 7 ? 'bits64' : 'bits32');
  let mode = $derived(deviceStore.state.mode);

  let canExtract = $derived(!work.isWorking && !!ipswPath && !!workspaceExtractedDir);
  let hasAnyExtracted = $derived(!!(extractedIbss || extractedIbec || extractedKernel || extractedRamdisk));
  let hasAnyPatched = $derived(!!(patchedIbss || patchedIbec || patchedKernel));

  function joinOut(name: string): string {
    const dir = workspaceExtractedDir.replace(/\/$/, '');
    return `${dir}/${name}`;
  }

  async function handleExtractAll() {
    if (!ipswPath || !workspaceExtractedDir) {
      work.setError('Source IPSW and workspace must be configured.');
      return;
    }
    await work.run('Extract all IPSW components', async () => {
      const [ibss, ibec, kernel, ramdisk] = await Promise.all([
        extractIpswComponent({ ipswPath, componentPath: ibssIpswPath, outputPath: joinOut('iBSS.dec') }),
        extractIpswComponent({ ipswPath, componentPath: ibecIpswPath, outputPath: joinOut('iBEC.dec') }),
        extractIpswComponent({ ipswPath, componentPath: kernelIpswPath, outputPath: joinOut('kernelcache.dec') }),
        extractIpswComponent({ ipswPath, componentPath: ramdiskIpswPath, outputPath: joinOut('ramdisk.dmg') }),
      ]);
      extractedIbss = ibss.outputPath;
      extractedIbec = ibec.outputPath;
      extractedKernel = kernel.outputPath;
      extractedRamdisk = ramdisk.outputPath;
    });
  }

  async function handlePatchIboot() {
    if (!extractedIbss) {
      work.setError('Extract iBSS first.');
      return;
    }
    await work.run('Patch iBSS', async () => {
      const patched = await patchIboot({
        inputPath: extractedIbss,
        outputPath: joinOut('iBSS.patched.bin'),
        bitWidth,
        bootArgs: null,
        bypassRsa: true,
        debug: false,
      });
      patchedIbss = patched.outputPath;
    });
  }

  async function handlePatchIbec() {
    if (!extractedIbec) {
      work.setError('Extract iBEC first.');
      return;
    }
    await work.run('Patch iBEC', async () => {
      const patched = await patchIboot({
        inputPath: extractedIbec,
        outputPath: joinOut('iBEC.patched.bin'),
        bitWidth,
        bootArgs: bootArgs || null,
        bypassRsa: true,
        debug: false,
      });
      patchedIbec = patched.outputPath;
    });
  }

  async function handlePatchKernel() {
    if (!extractedKernel) {
      work.setError('Extract kernel first.');
      return;
    }
    await work.run('Patch kernel', async () => {
      const patched = await patchKernel({
        inputPath: extractedKernel,
        outputPath: joinOut('kernelcache.patched.bin'),
        bitWidth,
        flags: ['-a', '-f'],
      });
      patchedKernel = patched.outputPath;
    });
  }

  async function handleRepackComponent(component: 'ibss' | 'ibec' | 'kernel', inputPath: string, outputName: string) {
    if (!shshPath) {
      work.setError('SHSH blob path is required for repacking.');
      return null;
    }
    return work.run(`Repack ${component}`, async () => {
      if (processorGen !== null && processorGen >= 7) {
        const repacked = await packImg4({ im4pPath: inputPath, shshPath, outputPath: joinOut(outputName), im4mPath: null });
        return repacked.outputPath;
      } else {
        const repacked = await repackImg3({ inputPath, outputPath: joinOut(outputName), templatePath: null, key: null, iv: null });
        return repacked.outputPath;
      }
    });
  }

  async function handleRepackAll() {
    if (!patchedIbss) {
      work.setError('Patch iBSS first.');
      return;
    }
    await work.run('Repack all components', async () => {
      const [ibss, ibec, kernel] = await Promise.all([
        handleRepackComponent('ibss', patchedIbss, 'iBSS.repacked'),
        patchedIbec ? handleRepackComponent('ibec', patchedIbec, 'iBEC.repacked') : Promise.resolve(null),
        patchedKernel ? handleRepackComponent('kernel', patchedKernel, 'kernelcache.repacked') : Promise.resolve(null),
      ]);
      if (ibss) patchedIbss = ibss;
      if (ibec) patchedIbec = ibec;
      if (kernel) patchedKernel = kernel;
    });
  }

  async function handleModifyRamdisk() {
    if (!extractedRamdisk || !workspaceSshBinariesDir) {
      work.setError('Extracted ramdisk and workspace SSH binaries directory are required.');
      return;
    }
    await work.run('Modify ramdisk', async () => {
      const modified = await modifyRamdisk({
        ramdiskPath: extractedRamdisk,
        action: 'resize',
        sourcePath: null,
        targetPath: null,
        sizeMb: ramdiskTargetSizeMb,
      });
      extractedRamdisk = modified.ramdiskPath;
    });
  }

  async function handleBoot() {
    if (!patchedIbss) {
      work.setError('Patched iBSS is required for kloader.');
      return;
    }

    if (buildId.trim() && deviceStore.state.ecid) {
      try {
        await recordJustBoot({
          ecid: deviceStore.state.ecid,
          productType: deviceStore.state.product_type || '',
          deviceName: deviceStore.state.name,
          buildId: buildId.trim(),
          iosVersion: iosVersion.trim() || null,
          bootArgs: bootArgs.trim() || null,
          repackedIbssPath: patchedIbss,
          repackedIbecPath: patchedIbec || null,
          sourceIpswPath: ipswPath,
        });
      } catch (error) {
        console.warn('Failed to record boot in history:', error);
      }
    }

    await work.run('Booting via irecovery', () =>
      sendBootchain({ ibssPath: patchedIbss, ibecPath: patchedIbec || null, processorGeneration: processorGen })
    );
  }
</script>

<div class="view">
  <div class="view-header">
    <div>
      <p>Build and boot a custom SSH ramdisk in stages. Each step writes its output to your output directory and feeds the next.</p>
    </div>
  </div>

  <section class="device-summary">
    <div>
      <span class="label">Device</span>
      <strong>{productType ?? 'Not detected'}</strong>
    </div>
    <div>
      <span class="label">Processor</span>
      <strong>{processorGen ? `A${processorGen}` : 'Unknown'}</strong>
    </div>
    <div>
      <span class="label">Mode</span>
      <strong>{mode}</strong>
    </div>
  </section>

  {#if work.errorMessage}
    <div class="error-state">{work.errorMessage}</div>
  {/if}

  <section class="panel">
    <div class="section-title">
      <span>1</span>
      <h2>Sources & Components</h2>
    </div>
    <p class="section-note">Provide the source IPSW and in-archive paths. Outputs resolve into workspace directories automatically.</p>

    <div class="form-grid">
      <label>
        <span>Source IPSW</span>
        <input bind:value={ipswPath} placeholder="/path/to/firmware.ipsw" />
      </label>
      <div class="inline-note span-2">
        Extracted output directory: <code>{workspaceExtractedDir || 'Workspace not configured'}</code>
      </div>
      <label>
        <span>Build ID <em>(for Just Boot history)</em></span>
        <input bind:value={buildId} placeholder="e.g. 13G36" />
      </label>
      <label>
        <span>iOS version <em>(optional)</em></span>
        <input bind:value={iosVersion} placeholder="e.g. 9.3.5" />
      </label>
      <label>
        <span>iBSS path inside IPSW</span>
        <input bind:value={ibssIpswPath} placeholder="Firmware/dfu/iBSS.n41ap.RELEASE.im4p" />
      </label>
      <label>
        <span>iBEC path inside IPSW</span>
        <input bind:value={ibecIpswPath} placeholder="Firmware/dfu/iBEC.n41ap.RELEASE.im4p" />
      </label>
      <label>
        <span>Kernel path inside IPSW</span>
        <input bind:value={kernelIpswPath} placeholder="kernelcache.release.n41" />
      </label>
      <label>
        <span>Ramdisk path inside IPSW</span>
        <input bind:value={ramdiskIpswPath} placeholder="058-12345-123.dmg" />
      </label>
      <label>
        <span>SHSH blob</span>
        <input bind:value={shshPath} placeholder="/path/to/blob.shsh" />
      </label>
      <div class="inline-note">
        SSH binaries directory: <code>{workspaceSshBinariesDir || 'Workspace not configured'}</code>
      </div>
      <label class="span-2">
        <span>Boot arguments</span>
        <input bind:value={bootArgs} placeholder="rd=md0 -v amfi_get_out_of_my_way=0x1" />
      </label>
      <label>
        <span>Ramdisk target size (MB)</span>
        <input type="number" bind:value={ramdiskTargetSizeMb} min="20" max="100" />
      </label>
    </div>

    <div class="actions">
      <button class="secondary" onclick={handleExtractAll} disabled={!canExtract}>Extract all components</button>
    </div>

    {#if hasAnyExtracted}
      <div class="result-grid">
        {#if extractedIbss}
          <div class="result-chip"><span class="chip-label">iBSS</span><code>{extractedIbss}</code></div>
        {/if}
        {#if extractedIbec}
          <div class="result-chip"><span class="chip-label">iBEC</span><code>{extractedIbec}</code></div>
        {/if}
        {#if extractedKernel}
          <div class="result-chip"><span class="chip-label">Kernel</span><code>{extractedKernel}</code></div>
        {/if}
        {#if extractedRamdisk}
          <div class="result-chip"><span class="chip-label">Ramdisk</span><code>{extractedRamdisk}</code></div>
        {/if}
      </div>
    {/if}
  </section>

  <section class="panel">
    <div class="section-title">
      <span>2</span>
      <h2>Patch</h2>
    </div>
    <p class="section-note">Patch iBSS, iBEC, and the kernelcache. Boot arguments are baked into iBEC.</p>

    <div class="actions">
      <button class="secondary" onclick={handlePatchIboot} disabled={work.isWorking || !extractedIbss}>Patch iBSS</button>
      <button class="secondary" onclick={handlePatchIbec} disabled={work.isWorking || !extractedIbec}>Patch iBEC</button>
      <button class="secondary" onclick={handlePatchKernel} disabled={work.isWorking || !extractedKernel}>Patch kernel</button>
    </div>

    {#if hasAnyPatched}
      <div class="result-grid">
        {#if patchedIbss}
          <div class="result-chip"><span class="chip-label">iBSS</span><code>{patchedIbss}</code></div>
        {/if}
        {#if patchedIbec}
          <div class="result-chip"><span class="chip-label">iBEC</span><code>{patchedIbec}</code></div>
        {/if}
        {#if patchedKernel}
          <div class="result-chip"><span class="chip-label">Kernel</span><code>{patchedKernel}</code></div>
        {/if}
      </div>
    {/if}
  </section>

  <section class="panel">
    <div class="section-title">
      <span>3</span>
      <h2>Repack</h2>
    </div>
    <p class="section-note">Repack patched components against the SHSH blob. Img4 is used on A7+, Img3 otherwise.</p>

    <div class="actions">
      <button class="secondary" onclick={handleRepackAll} disabled={work.isWorking || !patchedIbss || !shshPath}>Repack all</button>
    </div>
  </section>

  <section class="panel">
    <div class="section-title">
      <span>4</span>
      <h2>Ramdisk</h2>
    </div>
    <p class="section-note">Resize the ramdisk and inject SSH binaries before booting.</p>

    <div class="actions">
      <button class="secondary" onclick={handleModifyRamdisk} disabled={work.isWorking || !extractedRamdisk || !workspaceSshBinariesDir}>
        Modify ramdisk
      </button>
    </div>
  </section>

  <section class="panel">
    <div class="section-title">
      <span>5</span>
      <h2>Boot</h2>
    </div>
    <p class="section-note">Stage the patched iBSS / iBEC and hand off to kloader. Device must already be in DFU or pwnDFU.</p>

    <div class="actions">
      <button
        class="primary"
        onclick={handleBoot}
        disabled={work.isWorking || (mode !== 'DFU' && mode !== 'pwnDFU')}
      >
        kloader iBSS &rarr; iBEC
      </button>
    </div>
  </section>
</div>

<style>
  .view-header {
    margin-bottom: var(--spacing-lg);
  }

  .view-header p {
    color: var(--color-text-secondary);
    font-size: 0.9rem;
    line-height: 1.5;
    margin: 0;
  }

  .device-summary {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-border);
    margin-bottom: var(--spacing-lg);
  }

  .device-summary div {
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: var(--color-bg-secondary);
    padding: var(--spacing-md);
  }

  .label {
    color: var(--color-text-secondary);
    font-size: 0.75rem;
  }

  .device-summary strong {
    color: var(--color-text-primary);
    font-size: 0.95rem;
  }

  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-secondary);
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .section-title span {
    display: inline-grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--color-accent);
    color: white;
    font-size: 0.75rem;
    font-weight: 700;
  }

  .section-title h2 {
    color: var(--color-text-primary);
    font-size: 1rem;
    margin: 0;
  }

  .section-note {
    color: var(--color-text-secondary);
    font-size: 0.8rem;
    line-height: 1.5;
    margin: 0 0 var(--spacing-md);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--spacing-md);
  }

  .span-2 {
    grid-column: 1 / -1;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    font-weight: 600;
  }

  label em {
    color: var(--color-text-secondary);
    font-style: normal;
    font-weight: 500;
    opacity: 0.75;
  }

  input {
    width: 100%;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font: inherit;
    font-size: 0.85rem;
    padding: 8px 10px;
  }

  input[type='number'] {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
    justify-content: flex-end;
    margin-top: var(--spacing-md);
  }

  button.secondary,
  button.primary {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 12px;
  }

  button.secondary {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
  }

  button.primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: white;
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .error-state {
    border: 1px solid color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
    border-radius: var(--radius-md);
    background: var(--color-bg-secondary);
    color: var(--color-danger);
    padding: var(--spacing-md);
    font-size: 0.875rem;
    margin-bottom: var(--spacing-md);
  }

  .result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: var(--spacing-sm);
    margin-top: var(--spacing-md);
  }

  .result-chip {
    display: flex;
    flex-direction: column;
    gap: 2px;
    border: 1px solid color-mix(in srgb, var(--color-success) 40%, var(--color-border));
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    padding: var(--spacing-sm) var(--spacing-md);
    min-width: 0;
  }

  .chip-label {
    color: var(--color-text-secondary);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .result-chip code {
    color: var(--color-text-primary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  @media (max-width: 720px) {
    .device-summary,
    .form-grid {
      grid-template-columns: 1fr;
    }

    .actions {
      justify-content: flex-start;
    }
  }
</style>
