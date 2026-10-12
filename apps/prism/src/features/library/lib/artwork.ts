/**
 * Prism — Library feature: artwork-derived color palettes.
 *
 * Album/playlist covers drive the ambient gradients used by hero headers and
 * the now-playing view. We sample the image on a tiny canvas; when the canvas
 * is tainted (CORS) or the image fails to load we fall back to a deterministic
 * hash of the URL so the UI always has a stable, pleasant palette.
 */

import { useEffect, useState } from 'react';

export interface ArtworkPalette {
  /** `"r, g, b"` triplet, ready to drop inside `rgba(...)`. */
  rgb: string;
  /** Top gradient stop. */
  from: string;
  /** Bottom gradient stop. */
  to: string;
  /** Saturated accent sampled from the artwork. */
  strong: string;
}

/* -------------------------------------------------------------------------- */
/* Color math                                                                  */
/* -------------------------------------------------------------------------- */

function hashString(seed: string): number {
  let hash = 0;
  for (let i = 0; i < seed.length; i += 1) {
    hash = (hash << 5) - hash + seed.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const sN = s / 100;
  const lN = l / 100;
  const c = (1 - Math.abs(2 * lN - 1)) * sN;
  const hp = (((h % 360) + 360) % 360) / 60;
  const x = c * (1 - Math.abs((hp % 2) - 1));
  let r = 0;
  let g = 0;
  let b = 0;
  if (hp < 1) [r, g, b] = [c, x, 0];
  else if (hp < 2) [r, g, b] = [x, c, 0];
  else if (hp < 3) [r, g, b] = [0, c, x];
  else if (hp < 4) [r, g, b] = [0, x, c];
  else if (hp < 5) [r, g, b] = [x, 0, c];
  else [r, g, b] = [c, 0, x];
  const m = lN - c / 2;
  return [
    Math.round((r + m) * 255),
    Math.round((g + m) * 255),
    Math.round((b + m) * 255),
  ];
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  const rN = r / 255;
  const gN = g / 255;
  const bN = b / 255;
  const max = Math.max(rN, gN, bN);
  const min = Math.min(rN, gN, bN);
  const l = (max + min) / 2;
  const delta = max - min;
  if (delta === 0) return [0, 0, l];
  const s = delta / (1 - Math.abs(2 * l - 1));
  let h: number;
  if (max === rN) h = ((gN - bN) / delta) % 6;
  else if (max === gN) h = (bN - rN) / delta + 2;
  else h = (rN - gN) / delta + 4;
  h *= 60;
  if (h < 0) h += 360;
  return [h, s, l];
}

/* -------------------------------------------------------------------------- */
/* Deterministic fallback                                                      */
/* -------------------------------------------------------------------------- */

/** Stable, pleasant palette derived from a string (URL, id, name…). */
export function hashGradient(seed: string): ArtworkPalette {
  const hash = hashString(seed || 'prism');
  const hue = hash % 360;
  const hue2 = (hue + 35 + (hash % 70)) % 360;
  const [r, g, b] = hslToRgb(hue, 52, 42);
  return {
    rgb: `${r}, ${g}, ${b}`,
    from: `hsl(${hue} 58% 40%)`,
    to: `hsl(${hue2} 52% 18%)`,
    strong: `hsl(${hue} 68% 50%)`,
  };
}

/* -------------------------------------------------------------------------- */
/* Canvas extraction                                                           */
/* -------------------------------------------------------------------------- */

function extractPalette(image: HTMLImageElement): ArtworkPalette | null {
  const size = 24;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) return null;

  ctx.drawImage(image, 0, 0, size, size);
  const { data } = ctx.getImageData(0, 0, size, size);

  let rSum = 0;
  let gSum = 0;
  let bSum = 0;
  let count = 0;
  let bestHue = 0;
  let bestSat = -1;
  let bestLum = 0.5;

  for (let i = 0; i < data.length; i += 4) {
    if (data[i + 3] < 128) continue;
    const pr = data[i];
    const pg = data[i + 1];
    const pb = data[i + 2];
    rSum += pr;
    gSum += pg;
    bSum += pb;
    count += 1;
    const [h, s, l] = rgbToHsl(pr, pg, pb);
    if (s > bestSat && l > 0.12 && l < 0.88) {
      bestSat = s;
      bestHue = h;
      bestLum = l;
    }
  }

  if (count === 0) return null;

  const r = Math.round(rSum / count);
  const g = Math.round(gSum / count);
  const b = Math.round(bSum / count);
  const [avgH, avgS, avgL] = rgbToHsl(r, g, b);

  const useSample = bestSat > 0.15;
  const hue = useSample ? bestHue : avgH;
  const sat = Math.max(0.35, Math.min(0.78, useSample ? bestSat : avgS));
  const baseLum = useSample ? bestLum : avgL;

  const fromL = Math.round(Math.max(26, Math.min(52, baseLum * 100 + 10)));
  const toL = Math.round(Math.max(10, Math.min(24, baseLum * 100 - 14)));
  const strongL = Math.round(Math.max(38, Math.min(62, baseLum * 100 + 16)));

  return {
    rgb: `${r}, ${g}, ${b}`,
    from: `hsl(${Math.round(hue)} ${Math.round(sat * 100)}% ${fromL}%)`,
    to: `hsl(${Math.round((hue + 28) % 360)} ${Math.round(sat * 82)}% ${toL}%)`,
    strong: `hsl(${Math.round(hue)} ${Math.round(Math.min(0.92, sat + 0.16) * 100)}% ${strongL}%)`,
  };
}

/* -------------------------------------------------------------------------- */
/* Hook                                                                        */
/* -------------------------------------------------------------------------- */

/**
 * Resolves an ambient palette for an artwork URL. Returns the deterministic
 * fallback immediately, then upgrades to the sampled palette once the image is
 * decoded (if the canvas is readable).
 */
export function useArtworkGradient(url?: string | null): ArtworkPalette {
  const [palette, setPalette] = useState<ArtworkPalette>(() => hashGradient(url ?? 'prism'));

  useEffect(() => {
    if (!url) {
      setPalette(hashGradient('prism'));
      return;
    }

    let cancelled = false;
    setPalette(hashGradient(url));

    const image = new Image();
    image.crossOrigin = 'anonymous';
    image.decoding = 'async';
    image.onload = () => {
      if (cancelled) return;
      try {
        const extracted = extractPalette(image);
        if (extracted) setPalette(extracted);
      } catch {
        // Tainted canvas (CORS) — keep the deterministic fallback.
      }
    };
    image.onerror = () => undefined;
    image.src = url;

    return () => {
      cancelled = true;
    };
  }, [url]);

  return palette;
}
