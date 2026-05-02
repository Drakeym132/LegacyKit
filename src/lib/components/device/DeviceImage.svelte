<script lang="ts">
  import {
    bundledDeviceImageUrl,
    cdnDeviceImageUrl,
    hasBundledDeviceImage,
  } from '../../utils/deviceModels';

  interface Props {
    productType: string | null | undefined;
    deviceColor: string | null | undefined;
    width?: number;
    height?: number;
  }

  let { productType, deviceColor, width = 92, height = 190 }: Props = $props();

  const sources = $derived.by(() => {
    if (!productType) return [];
    const urls: string[] = [];
    if (hasBundledDeviceImage(productType)) {
      urls.push(bundledDeviceImageUrl(productType));
    }
    const cdn = cdnDeviceImageUrl(productType, deviceColor);
    if (cdn) urls.push(cdn);
    return urls;
  });

  let attempt = $state(0);

  $effect(() => {
    // Reset when the source list changes (e.g. device swap).
    sources;
    attempt = 0;
  });

  let currentSrc = $derived(sources[attempt] ?? null);

  // ---- Module-level decoded-image cache ----
  // Keeps an `HTMLImageElement` reference per URL so the browser retains the
  // decoded bitmap across HomeView remounts (the `{#key}` carousel transition
  // destroys/recreates the DOM each time the user navigates back to Home).
  // Without this, every visit triggers a fresh network/decode pass and the
  // device image flashes in late.
  const decodedCache = (
    globalThis as unknown as { __lkDeviceImgCache?: Map<string, HTMLImageElement> }
  ).__lkDeviceImgCache ??
    ((globalThis as unknown as { __lkDeviceImgCache?: Map<string, HTMLImageElement> })
      .__lkDeviceImgCache = new Map<string, HTMLImageElement>());

  const failedCache = (
    globalThis as unknown as { __lkDeviceImgFailed?: Set<string> }
  ).__lkDeviceImgFailed ??
    ((globalThis as unknown as { __lkDeviceImgFailed?: Set<string> })
      .__lkDeviceImgFailed = new Set<string>());

  // Whether the current src has been decoded at least once (synchronous answer).
  let isWarm = $derived(currentSrc !== null && decodedCache.has(currentSrc));

  // Warm the cache for any new src. Skipping URLs we've already failed on so
  // we move straight to the next fallback.
  $effect(() => {
    const src = currentSrc;
    if (!src) return;
    if (failedCache.has(src)) {
      handleError();
      return;
    }
    if (decodedCache.has(src)) return;
    const img = new Image();
    img.decoding = 'async';
    img.src = src;
    img
      .decode()
      .then(() => {
        decodedCache.set(src, img);
      })
      .catch(() => {
        failedCache.add(src);
        handleError();
      });
  });

  function handleError() {
    if (currentSrc) failedCache.add(currentSrc);
    if (attempt < sources.length - 1) attempt += 1;
    else attempt = sources.length; // exhausted -> render SVG fallback
  }
</script>

{#if currentSrc}
  <img
    class="device-img"
    src={currentSrc}
    alt={productType ?? 'Device'}
    {width}
    {height}
    loading="eager"
    fetchpriority="high"
    decoding={isWarm ? 'sync' : 'async'}
    onerror={handleError}
  />
{:else}
  <svg class="device-img" {width} {height} viewBox="0 0 96 200" fill="none" aria-hidden="true">
    <rect x="1" y="1" width="94" height="198" rx="15" fill="#1c2b4a" />
    <rect x="3" y="3" width="90" height="194" rx="13" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="1" />
    <rect x="7" y="30" width="82" height="140" rx="5" fill="#dbeafe" />
    <rect x="7" y="30" width="82" height="140" rx="5" fill="url(#sg)" />
    <defs>
      <linearGradient id="sg" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="white" stop-opacity="0.18" />
        <stop offset="100%" stop-color="white" stop-opacity="0" />
      </linearGradient>
    </defs>
    <circle cx="48" cy="185" r="8" fill="none" stroke="rgba(255,255,255,0.15)" stroke-width="1.5" />
    <circle cx="48" cy="185" r="4" fill="rgba(255,255,255,0.06)" />
    <circle cx="48" cy="18" r="3.5" fill="rgba(255,255,255,0.12)" />
    {#each [37, 41, 45, 49, 53, 57, 61] as x}
      <rect {x} y="16" width="2" height="4" rx="1" fill="rgba(255,255,255,0.1)" />
    {/each}
    <rect x="15" y="46" width="66" height="10" rx="3" fill="rgba(79,142,247,0.3)" />
    <rect x="15" y="62" width="46" height="5" rx="2" fill="rgba(79,142,247,0.15)" />
    <rect x="15" y="73" width="56" height="5" rx="2" fill="rgba(79,142,247,0.1)" />
    <rect x="15" y="84" width="38" height="5" rx="2" fill="rgba(79,142,247,0.08)" />
    {#each [15, 37, 59] as x}
      <rect {x} y="100" width="18" height="18" rx="4" fill="rgba(79,142,247,0.12)" />
    {/each}
  </svg>
{/if}

<style>
  .device-img {
    position: relative;
    z-index: 1;
    object-fit: contain;
    height: 100%;
    width: auto;
    max-width: 100%;
    max-height: 100%;
  }
</style>
