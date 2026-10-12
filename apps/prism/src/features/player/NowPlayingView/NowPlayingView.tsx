import {
  ChevronDown,
  Heart,
  ListMusic,
  Mic2,
  MonitorSpeaker,
  Pause,
  Play,
  Repeat,
  Repeat1,
  Shuffle,
  SkipBack,
  SkipForward,
  Volume2,
  VolumeX,
} from 'lucide-react';
import { AnimatePresence, motion } from 'framer-motion';
import { useCallback, useEffect, useRef, useState, type CSSProperties } from 'react';
import { createPortal } from 'react-dom';
import { Link } from 'react-router-dom';

import { buildPath } from '@/app/routes/routes';
import { IconButton, Slider } from '@/components/ui';
import { useLibrary, usePlaybackControls, usePlayer, useUi } from '@/hooks';
import { useUiStore } from '@/store/uiStore';
import { cn, fireAndForget } from '@/features/library/lib/utils';
import { formatDuration } from '@/features/library/lib/format';
import { useArtworkGradient } from '@/features/library/lib/artwork';

import { useSmoothProgress } from '../lib/useSmoothProgress';
import { DevicePicker } from '../DevicePicker/DevicePicker';
import { QueueView } from '../QueueView/QueueView';
import { Visualizer } from '../Visualizer/Visualizer';
import styles from './NowPlayingView.module.css';

import type { RepeatState } from '@/lib/spotify/types';

export interface NowPlayingViewProps {
  className?: string;
}

/**
 * Full-screen now-playing experience. Mounted once at the app root and driven
 * by `uiStore.nowPlayingOpen` (the bar's queue button also opens it).
 */
export function NowPlayingView({ className }: NowPlayingViewProps) {
  const { playback, isPlaying, volume } = usePlayer();
  const controls = usePlaybackControls();
  const { isTrackSaved, toggleSavedTrack } = useLibrary();
  const { nowPlayingOpen, setNowPlayingOpen } = useUi();
  const queueOpen = useUiStore((state) => state.queueOpen);
  const setQueueOpen = useUiStore((state) => state.setQueueOpen);

  const progress = useSmoothProgress();
  const [lyricsOpen, setLyricsOpen] = useState(false);
  const [devicePickerOpen, setDevicePickerOpen] = useState(false);
  const [lastVolume, setLastVolume] = useState(volume);
  const rootRef = useRef<HTMLDivElement>(null);

  const track = playback?.item ?? null;
  const palette = useArtworkGradient(track?.album.images?.[0]?.url);

  const close = useCallback(() => {
    setNowPlayingOpen(false);
    setQueueOpen(false);
    setLyricsOpen(false);
  }, [setNowPlayingOpen, setQueueOpen]);

  // The bar's queue button should reveal the queue inside this view.
  useEffect(() => {
    if (queueOpen && !nowPlayingOpen) setNowPlayingOpen(true);
  }, [queueOpen, nowPlayingOpen, setNowPlayingOpen]);

  // Escape to close + body scroll lock + initial focus.
  useEffect(() => {
    if (!nowPlayingOpen) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') close();
    };
    document.addEventListener('keydown', onKeyDown);
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    rootRef.current?.focus();
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      document.body.style.overflow = previousOverflow;
    };
  }, [nowPlayingOpen, close]);

  const toggleQueue = () => {
    setLyricsOpen(false);
    setQueueOpen(!queueOpen);
  };

  const toggleLyrics = () => {
    setQueueOpen(false);
    setLyricsOpen((value) => !value);
  };

  const duration = track && track.duration_ms > 0 ? track.duration_ms : 1;
  const position = Math.min(Math.max(progress, 0), duration);
  const repeat: RepeatState = playback?.repeat_state ?? 'off';
  const shuffle = playback?.shuffle_state ?? false;
  const saved = track ? isTrackSaved(track.id) : false;

  const cycleRepeat = () => {
    const next: RepeatState =
      repeat === 'off' ? 'context' : repeat === 'context' ? 'track' : 'off';
    fireAndForget(controls.setRepeat(next));
  };

  const handleSeek = (value: number) => {
    fireAndForget(controls.seek(Math.round(value)));
  };

  const handleVolume = (value: number) => {
    if (value > 0) setLastVolume(value);
    fireAndForget(controls.setVolume(value));
  };

  const toggleMute = () => {
    const next = volume > 0 ? 0 : lastVolume || 50;
    fireAndForget(controls.setVolume(next));
  };

  const overlay = (
    <AnimatePresence>
      {nowPlayingOpen ? (
        <motion.div
          ref={rootRef}
          className={cn(styles.overlay, className)}
          style={
            {
              '--np-from': palette.from,
              '--np-to': palette.to,
              '--np-rgb': palette.rgb,
            } as CSSProperties
          }
          initial={{ opacity: 0, y: 28 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 28 }}
          transition={{ duration: 0.28, ease: [0.2, 0, 0, 1] }}
          role="dialog"
          aria-modal="true"
          aria-label="Lecture en cours"
          tabIndex={-1}
        >
          <div className={styles.backdrop} aria-hidden />

          <header className={styles.header}>
            <IconButton label="Réduire la vue lecture" onClick={close}>
              <ChevronDown />
            </IconButton>
            <span className={styles.headerTitle}>Lecture en cours</span>
            <div className={styles.headerActions}>
              <IconButton
                label="Paroles"
                className={lyricsOpen ? styles.activeControl : ''}
                onClick={toggleLyrics}
              >
                <Mic2 />
              </IconButton>
              <IconButton
                label="Appareils disponibles"
                onClick={() => setDevicePickerOpen(true)}
              >
                <MonitorSpeaker />
              </IconButton>
              <IconButton
                label="File d’attente"
                className={queueOpen ? styles.activeControl : ''}
                onClick={toggleQueue}
              >
                <ListMusic />
              </IconButton>
            </div>
          </header>

          <div className={styles.body}>
            <div className={styles.main}>
              {track ? (
                <>
                  <div className={styles.artworkWrap}>
                    {track.album.images?.[0]?.url ? (
                      <img
                        className={styles.artwork}
                        src={track.album.images[0].url}
                        alt={track.album.name}
                        draggable={false}
                      />
                    ) : (
                      <span className={styles.artworkPlaceholder} aria-hidden />
                    )}
                  </div>

                  <div className={styles.info}>
                    <div className={styles.infoText}>
                      <h1 className={styles.trackName}>{track.name}</h1>
                      <div className={styles.artists}>
                        {track.artists.map((artist, index) => (
                          <span key={artist.id}>
                            {index > 0 ? ', ' : null}
                            <Link to={buildPath.artist(artist.id)} className={styles.artistLink}>
                              {artist.name}
                            </Link>
                          </span>
                        ))}
                      </div>
                    </div>
                    <IconButton
                      label={saved ? 'Retirer des titres likés' : 'Ajouter aux titres likés'}
                      className={saved ? styles.likeActive : ''}
                      onClick={() => toggleSavedTrack(track.id)}
                    >
                      <Heart fill={saved ? 'currentColor' : 'none'} />
                    </IconButton>
                  </div>

                  <div className={styles.controls}>
                    <div className={styles.buttons}>
                      <IconButton
                        label="Lecture aléatoire"
                        className={shuffle ? styles.activeControl : ''}
                        onClick={() => fireAndForget(controls.setShuffle(!shuffle))}
                      >
                        <Shuffle />
                      </IconButton>
                      <IconButton
                        label="Piste précédente"
                        onClick={() => fireAndForget(controls.previous())}
                      >
                        <SkipBack fill="currentColor" />
                      </IconButton>
                      <IconButton
                        label={isPlaying ? 'Pause' : 'Lecture'}
                        variant="primary"
                        size="lg"
                        className={styles.playButton}
                        onClick={() =>
                          fireAndForget(isPlaying ? controls.pause() : controls.play())
                        }
                      >
                        {isPlaying ? <Pause fill="currentColor" /> : <Play fill="currentColor" />}
                      </IconButton>
                      <IconButton
                        label="Piste suivante"
                        onClick={() => fireAndForget(controls.next())}
                      >
                        <SkipForward fill="currentColor" />
                      </IconButton>
                      <IconButton
                        label="Répéter"
                        className={repeat !== 'off' ? styles.activeControl : ''}
                        onClick={cycleRepeat}
                      >
                        {repeat === 'track' ? <Repeat1 /> : <Repeat />}
                      </IconButton>
                    </div>

                    <div className={styles.progress}>
                      <span className={styles.time}>{formatDuration(position)}</span>
                      <Slider
                        className={styles.slider}
                        value={position}
                        min={0}
                        max={duration}
                        step={1000}
                        ariaLabel="Position de lecture"
                        onChange={handleSeek}
                      />
                      <span className={styles.time}>{formatDuration(track.duration_ms)}</span>
                    </div>

                    <div className={styles.volume}>
                      <IconButton
                        label={volume > 0 ? 'Couper le son' : 'Rétablir le son'}
                        size="sm"
                        onClick={toggleMute}
                      >
                        {volume > 0 ? <Volume2 /> : <VolumeX />}
                      </IconButton>
                      <Slider
                        className={styles.slider}
                        value={volume}
                        min={0}
                        max={100}
                        step={1}
                        ariaLabel="Volume"
                        onChange={handleVolume}
                      />
                    </div>
                  </div>

                  <div className={styles.visualizerWrap}>
                    <Visualizer playing={isPlaying} />
                  </div>
                </>
              ) : (
                <div className={styles.empty}>
                  <p className={styles.emptyTitle}>Aucune lecture en cours</p>
                  <p className={styles.emptyText}>
                    Lancez un titre depuis votre bibliothèque pour le voir ici.
                  </p>
                </div>
              )}
            </div>

            {queueOpen || lyricsOpen ? (
              <aside className={styles.panel} aria-label={queueOpen ? 'File d’attente' : 'Paroles'}>
                {queueOpen ? (
                  <QueueView onClose={() => setQueueOpen(false)} />
                ) : (
                  <div className={styles.lyrics}>
                    <h2 className={styles.lyricsTitle}>Paroles</h2>
                    <p className={styles.lyricsText}>
                      Les paroles ne sont pas disponibles pour ce titre.
                    </p>
                  </div>
                )}
              </aside>
            ) : null}
          </div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );

  return (
    <>
      {createPortal(overlay, document.body)}
      <DevicePicker open={devicePickerOpen} onClose={() => setDevicePickerOpen(false)} />
    </>
  );
}
