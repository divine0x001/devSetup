import { useEffect, useRef, useState, type RefObject } from 'react';

import { cn } from '@/features/library/lib/utils';

import styles from './Visualizer.module.css';

export interface VisualizerProps {
  /**
   * Optional audio element to analyse with the Web Audio API. The Spotify Web
   * Playback SDK does not expose one, so this is usually omitted and the
   * synthetic animation is used instead.
   */
  audioRef?: RefObject<HTMLAudioElement | null>;
  playing?: boolean;
  bars?: number;
  className?: string;
}

/**
 * Ambient bar visualizer. Uses a real `AnalyserNode` when an audio element is
 * available, otherwise animates synthetic bars. Renders a static frame when
 * paused or when the user prefers reduced motion. Never throws.
 */
export function Visualizer({ audioRef, playing = false, bars = 48, className }: VisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const [hasAnalyser, setHasAnalyser] = useState(false);

  // Best-effort Web Audio wiring.
  useEffect(() => {
    const audio = audioRef?.current;
    if (!audio) return;

    let context: AudioContext | null = null;
    try {
      const Ctor =
        window.AudioContext ??
        (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
      if (!Ctor) return;
      context = new Ctor();
      const source = context.createMediaElementSource(audio);
      const analyser = context.createAnalyser();
      analyser.fftSize = 256;
      source.connect(analyser);
      analyser.connect(context.destination);
      analyserRef.current = analyser;
      setHasAnalyser(true);
    } catch {
      // Already connected / CORS / unsupported — keep the synthetic fallback.
      analyserRef.current = null;
      setHasAnalyser(false);
    }

    return () => {
      analyserRef.current = null;
      setHasAnalyser(false);
      void context?.close().catch(() => undefined);
    };
  }, [audioRef]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const accent =
      getComputedStyle(canvas).getPropertyValue('--prism-color-accent').trim() || '#1ed760';

    let width = 0;
    let height = 0;
    const resize = () => {
      const dpr = window.devicePixelRatio || 1;
      width = canvas.clientWidth;
      height = canvas.clientHeight;
      canvas.width = Math.max(1, Math.floor(width * dpr));
      canvas.height = Math.max(1, Math.floor(height * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(canvas);

    const analyser = analyserRef.current;
    const frequencyData = analyser ? new Uint8Array(analyser.frequencyBinCount) : null;
    const animate = playing || hasAnalyser;
    const start = performance.now();
    let frame = 0;

    const barWidth = () => width / bars;
    const gap = () => Math.max(1, barWidth() * 0.28);

    const draw = (now: number) => {
      ctx.clearRect(0, 0, width, height);
      ctx.fillStyle = accent;
      const bw = barWidth();
      const g = gap();
      const w = Math.max(1, bw - g);

      if (analyser && frequencyData) {
        analyser.getByteFrequencyData(frequencyData);
        for (let i = 0; i < bars; i += 1) {
          const value = frequencyData[Math.floor((i / bars) * frequencyData.length)] / 255;
          const h = Math.max(2, value * height);
          ctx.globalAlpha = 0.35 + value * 0.65;
          ctx.fillRect(i * bw + g / 2, height - h, w, h);
        }
      } else {
        const t = (now - start) / 1000;
        for (let i = 0; i < bars; i += 1) {
          const phase = i * 0.35;
          const value =
            playing && !reduceMotion
              ? (Math.sin(t * 2 + phase) * 0.5 + 0.5) *
                (0.55 + 0.45 * Math.sin(t * 1.3 + phase * 0.7))
              : 0.22;
          const h = Math.max(2, value * height);
          ctx.globalAlpha = 0.4 + value * 0.6;
          ctx.fillRect(i * bw + g / 2, height - h, w, h);
        }
      }
      ctx.globalAlpha = 1;

      if (animate) frame = requestAnimationFrame(draw);
    };

    frame = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, [bars, playing, hasAnalyser]);

  return (
    <canvas
      ref={canvasRef}
      className={cn(styles.root, className)}
      aria-hidden
      data-playing={playing || undefined}
    />
  );
}
