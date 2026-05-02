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
  import { runKloader } from '$lib/api/jailbreak';
  import { recordJustBoot } from '$lib/api/justBoot';
  import { deviceStore } from '$lib/stores/deviceStore.svelte';
  import { createWorkingController } from '$lib/utils/workingState.svelte';

  let ipswPath = $state('');
  let outputDir = $state('');
  let bootArgs = $state('rd=md0 -v amfi_get_out_of_my_way=0x1 cs_enforcement_disable=1');
  let buildId = $state(''); // Added for Just Boot recording
  let iosVersion = $state(''); // Added for Just Boot recording
  let ibssIpswPath = $state('');
  let ibecIpswPath = $state('');
  let kernelIpswPath = $state('');
  let ramdiskIpswPath = $state('');
  let shshPath = $state('');
  let sshBinariesDir = $state('');
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
  let processorGen = $derived(inferProcessorGen(productType));
  let bitWidth = $derived<IbootBitWidth>(processorGen !== null && processorGen >= 7 ? 'bits64' : 'bits32');
  let mode = $derived(deviceStore.state.mode);

  function inferProcessorGen(product: string | null): number | null {
    if (!product) return null;
    if (/^iPhone(1|2),/.test(product) || /^iPod(1|2),/.test(product)) return 1;
    if (product === 'iPod3,1') return 3;
    if (/^iPhone3,/.test(product) || product === 'iPad1,1' || product === 'iPod4,1') return 4;
    if (product === 'iPhone4,1' || /^iPad2,/.test(product) || /^iPad3,[1-3]/.test(product) || product === 'iPod5,1') return 5;
    if (/^iPhone5,/.test(product) || /^iPad3,[4-6]/.test(product)) return 6;
    if (/^iPhone6,/.test(product) || /^iPad4,/.test(product)) return 7;
    if (/^iPhone7,/.test(product) || product === 'iPod7,1' || /^iPad5,/.test(product)) return 8;
    if (/^iPhone8,/.test(product) || /^iPad6,/.test(product)) return 9;
    if (/^iPhone9,/.test(product) || /^iPad7,/.test(product)) return 10;
    return null;
  }

  function joinOut(name: string): string {
    const dir = outputDir.replace(/\/$/, '');
    return `${dir}/${name}`;
  }

  async function handleExtractAll() {
    if (!ipswPath || !outputDir) {
      work.setError('Source IPSW and output directory are required.');
      return;
    }
    const result = await work.run('Extract all IPSW components', async () => {
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
    if (!result) return;
  }

  async function handlePatchIboot() {
    if (!extractedIbss) {
      work.setError('Extract iBSS first.');
      return;
    }
    const result = await work.run('Patch iBSS', async () => {
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
    if (!result) return;
  }

  async function handlePatchIbec() {
    if (!extractedIbec) {
      work.setError('Extract iBEC first.');
      return;
    }
    const result = await work.run('Patch iBEC', async () => {
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
    if (!result) return;
  }

  async function handlePatchKernel() {
    if (!extractedKernel) {
      work.setError('Extract kernel first.');
      return;
    }
    const result = await work.run('Patch kernel', async () => {
      const patched = await patchKernel({
        inputPath: extractedKernel,
        outputPath: joinOut('kernelcache.patched.bin'),
        bitWidth,
        flags: ['-a', '-f'],
      });
      patchedKernel = patched.outputPath;
    });
    if (!result) return;
  }

  async function handleRepackComponent(component: 'ibss' | 'ibec' | 'kernel', inputPath: string, outputName: string) {
    if (!shshPath) {
      work.setError('SHSH blob path is required for repacking.');
      return null;
    }
    const result = await work.run(`Repack ${component}`, async () => {
      if (processorGen !== null && processorGen >= 7) {
        const repacked = await packImg4({ im4pPath: inputPath, shshPath, outputPath: joinOut(outputName), im4mPath: null });
        return repacked.outputPath;
      } else {
        const repacked = await repackImg3({ inputPath, outputPath: joinOut(outputName), templatePath: null, key: null, iv: null });
        return repacked.outputPath;
      }
    });
    return result;
  }

  async function handleRepackAll() {
    if (!patchedIbss) {
      work.setError('Patch iBSS first.');
      return;
    }
    const result = await work.run('Repack all components', async () => {
      const [ibss, ibec, kernel] = await Promise.all([
        handleRepackComponent('ibss', patchedIbss, 'iBSS.repacked'),
        patchedIbec ? handleRepackComponent('ibec', patchedIbec, 'iBEC.repacked') : Promise.resolve(null),
        patchedKernel ? handleRepackComponent('kernel', patchedKernel, 'kernelcache.repacked') : Promise.resolve(null),
      ]);
      if (ibss) patchedIbss = ibss;
      if (ibec) patchedIbec = ibec;
      if (kernel) patchedKernel = kernel;
    });
    if (!result) return;
  }

  async function handleModifyRamdisk() {
    if (!extractedRamdisk || !sshBinariesDir) {
      work.setError('Extracted ramdisk and SSH binaries directory are required.');
      return;
    }
    const result = await work.run('Modify ramdisk', async () => {
      const modified = await modifyRamdisk({
        ramdiskPath: extractedRamdisk,
        action: 'resize',
        sourcePath: null,
        targetPath: null,
        sizeMb: ramdiskTargetSizeMb,
      });
      extractedRamdisk = modified.ramdiskPath;
    });
    if (!result) return;
  }

  async function handleBoot() {
    if (!patchedIbss) {
      work.setError('Patched iBSS is required for kloader.');
      return;
    }
    
    // Record the boot attempt in Just Boot history
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
          sourceIpswPath: ipswPath
        });
      } catch (error) {
        // Silently fail if recording doesn't work - don't break the boot process
        console.warn('Failed to record boot in history:', error);
      }
    }
    
    await work.run('Booting via kloader', () =>
      runKloader({ ibssPath: patchedIbss, ibecPath: patchedIbec || null })
    );
  }
</script>

<div class="view">
  <div class="view-header">
    <div>
      <h1>SSH Ramdisk</h1>
      <p>Build and boot a custom SSH ramdisk in stages. Each step writes its output to your output directory and feeds the next.</p>
    </div>
  </div>

  <section class="device-summary">
    <div>
      <span class="label">Device</span>
      <strong>{productType ?? 'Not detected'}</strong>
    </div>
    <div>
      <span class="label">Mode</span>
      <strong>{mode}</strong>
    </div>
    <div>
      <span class="label">Processor</span>
      <strong>{processorGen ? `A${processorGen}` : 'Unknown'}</strong>
    </div>
  </section>

  <section class="panel">
    <h2>Input</h2>
    <p>Provide the source IPSW and the paths to the components you want to extract.</p>

    <label class="field">
      <span>Source IPSW</span>
      <input bind:value={ipswPath} placeholder="/path/to/firmware.ipsw" />
    </label>

    <label class="field">
      <span>Build ID (for Just Boot recording)</span>
      <input bind:value={buildId} placeholder="e.g. 13G36" />
    </label>

    <label class="field">
      <span>iOS Version (optional)</span>
      <input bind:value={iosVersion} placeholder="e.g. 9.3.5" />
    </label>

    <label class="field">
      <span>Output directory</span>
      <input bind:value={outputDir} placeholder="/path/to/output" />
    </label>

    <label class="field">
      <span>iBSS path inside IPSW</span>
      <input bind:value={ibssIpswPath} placeholder="Firmware/dfu/iBSS.n41ap.RELEASE.im4p" />
    </label>

    <label class="field">
      <span>iBEC path inside IPSW</span>
      <input bind:value={ibecIpswPath} placeholder="Firmware/dfu/iBEC.n41ap.RELEASE.im4p" />
    </label>

    <label class="field">
      <span>Kernel path inside IPSW</span>
      <input bind:value={kernelIpswPath} placeholder="kernelcache.release.n41" />
    </label>

    <label class="field">
      <span>Ramdisk path inside IPSW</span>
      <input bind:value={ramdiskIpswPath} placeholder="058-12345-123.dmg" />
    </label>

    <label class="field">
      <span>SHSH blob</span>
      <input bind:value={shshPath} placeholder="/path/to/blob.shsh" />
    </label>

    <label class="field">
      <span>SSH binaries directory</span>
      <input bind:value={sshBinariesDir} placeholder="/path/to/ssh/binaries" />
    </label>

    <label class="field">
      <span>Boot arguments</span>
      <input bind:value={bootArgs} placeholder="rd=md0 -v amfi_get_out_of_my_way=0x1" />
    </label>

    <label class="field">
      <span>Ramdisk target size (MB)</span>
      <input type="number" bind:value={ramdiskTargetSizeMb} min="20" max="100" />
    </label>

    <div class="actions">
      <button class="secondary" onclick={handleExtractAll} disabled={work.isWorking || !ipswPath || !outputDir}>Extract all</button>
    </div>
  </section>

  <section class="panel">
    <h2>Patch</h2>
    <p>Patch the extracted components with your custom boot arguments.</p>

    <div class="actions">
      <button class="secondary" onclick={handlePatchIboot} disabled={work.isWorking || !extractedIbss}>Patch iBSS</button>
      <button class="secondary" onclick={handlePatchIbec} disabled={work.isWorking || !extractedIbec}>Patch iBEC</button>
      <button class="secondary" onclick={handlePatchKernel} disabled={work.isWorking || !extractedKernel}>Patch kernel</button>
    </div>
  </section>

  <section class="panel">
    <h2>Repack</h2>
    <p>Repack the patched components with your SHSH blob.</p>

    <div class="actions">
      <button class="secondary" onclick={handleRepackAll} disabled={work.isWorking || !patchedIbss || !shshPath}>Repack all</button>
    </div>
  </section>

  <section class="panel">
    <h2>Ramdisk</h2>
    <p>Inject SSH binaries into the ramdisk.</p>

    <div class="actions">
      <button class="secondary" onclick={handleModifyRamdisk} disabled={work.isWorking || !extractedRamdisk || !sshBinariesDir}>Modify ramdisk</button>
    </div>
  </section>

  <section class="panel">
    <h2>Boot</h2>
    <p>Boot the device with kloader using the patched and repacked components.</p>

    <div class="actions">
      <button class="primary" onclick={handleBoot} disabled={work.isWorking || (mode !== 'DFU' && mode !== 'pwnDFU')}>
        kloader iBSS &rarr; iBEC
      </button>
    </div>
  </section>

  {#if work.errorMessage}
    <section class="panel error-panel">
      <h2>Error</h2>
      <p>{work.errorMessage}</p>
    </section>
  {/if}
</div>

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
  }

  .view-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .view-header h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .view-header p {
    margin: 0.25rem 0 0;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
  }

  .device-summary {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .device-summary .label {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .panel {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .panel h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .panel p {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.8125rem;
  }

  .field span {
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .field input {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    padding: 6px 10px;
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.8125rem;
  }

  .field input:disabled {
    opacity: 0.6;
  }

  .actions {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-xs);
  }

  button {
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 14px;
    cursor: pointer;
    border: 1px solid transparent;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .secondary {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    color: var(--color-text-primary);
  }

  .primary {
    background: var(--color-accent);
    border: 1px solid var(--color-accent);
    color: white;
  }

  .error-panel {
    background: color-mix(in srgb, var(--color-danger) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 35%, transparent);
  }

  .error-panel h2 {
    color: var(--color-danger);
  }
</style>