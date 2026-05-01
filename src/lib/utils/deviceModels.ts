export const DEVICE_MODELS: Record<string, string> = {
  'iPhone1,1': 'iPhone',
  'iPhone1,2': 'iPhone 3G',
  'iPhone2,1': 'iPhone 3GS',
  'iPhone3,1': 'iPhone 4 (GSM)',
  'iPhone3,2': 'iPhone 4 (GSM, Rev A)',
  'iPhone3,3': 'iPhone 4 (CDMA)',
  'iPhone4,1': 'iPhone 4S',
  'iPhone5,1': 'iPhone 5 (GSM)',
  'iPhone5,2': 'iPhone 5 (Global)',
  'iPhone5,3': 'iPhone 5c (GSM)',
  'iPhone5,4': 'iPhone 5c (Global)',
  'iPhone6,1': 'iPhone 5s (GSM)',
  'iPhone6,2': 'iPhone 5s (Global)',

  'iPad1,1': 'iPad',
  'iPad2,1': 'iPad 2 (Wi-Fi)',
  'iPad2,2': 'iPad 2 (GSM)',
  'iPad2,3': 'iPad 2 (CDMA)',
  'iPad2,4': 'iPad 2 (Wi-Fi, Rev A)',
  'iPad2,5': 'iPad mini (Wi-Fi)',
  'iPad2,6': 'iPad mini (GSM)',
  'iPad2,7': 'iPad mini (Global)',
  'iPad3,1': 'iPad (3rd gen, Wi-Fi)',
  'iPad3,2': 'iPad (3rd gen, CDMA)',
  'iPad3,3': 'iPad (3rd gen, GSM)',
  'iPad3,4': 'iPad (4th gen, Wi-Fi)',
  'iPad3,5': 'iPad (4th gen, GSM)',
  'iPad3,6': 'iPad (4th gen, Global)',
  'iPad4,1': 'iPad Air (Wi-Fi)',
  'iPad4,2': 'iPad Air (Cellular)',
  'iPad4,3': 'iPad Air (China)',
  'iPad4,4': 'iPad mini 2 (Wi-Fi)',
  'iPad4,5': 'iPad mini 2 (Cellular)',
  'iPad4,6': 'iPad mini 2 (China)',
  'iPad4,7': 'iPad mini 3 (Wi-Fi)',
  'iPad4,8': 'iPad mini 3 (Cellular)',
  'iPad4,9': 'iPad mini 3 (China)',

  'iPod1,1': 'iPod touch',
  'iPod2,1': 'iPod touch (2nd gen)',
  'iPod3,1': 'iPod touch (3rd gen)',
  'iPod4,1': 'iPod touch (4th gen)',
  'iPod5,1': 'iPod touch (5th gen)',
};

export function getDeviceFriendlyName(productType: string | null | undefined): string | null {
  if (!productType) return null;
  return DEVICE_MODELS[productType] ?? null;
}

// Productids covered by bundled images in /devices/{id}.png.
// Kept in sync with scripts/download-device-images.sh.
export const BUNDLED_DEVICE_IMAGES: ReadonlySet<string> = new Set(Object.keys(DEVICE_MODELS));

export function hasBundledDeviceImage(productType: string | null | undefined): boolean {
  return !!productType && BUNDLED_DEVICE_IMAGES.has(productType);
}

export function bundledDeviceImageUrl(productType: string): string {
  return `/devices/${productType}.png`;
}

// Best-guess color for the AppleDB CDN when device_color isn't reported
// (e.g. DFU/Recovery on a newer device we don't bundle locally).
const DEVICE_DEFAULT_COLORS: Record<string, string> = {
  iPhone: 'Space Gray',
  iPad: 'Space Gray',
  iPod: 'Space Gray',
};

export function cdnDeviceImageUrl(productType: string, deviceColor: string | null | undefined): string | null {
  const family = productType.replace(/[0-9].*$/, '');
  const color = deviceColor || DEVICE_DEFAULT_COLORS[family];
  if (!color) return null;
  return `https://img.appledb.dev/device@256/${encodeURIComponent(productType)}/${encodeURIComponent(color)}.png`;
}
