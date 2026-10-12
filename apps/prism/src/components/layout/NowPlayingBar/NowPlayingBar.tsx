import {
  ChevronUp,
  Heart,
  ListMusic,
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
import { useState } from 'react';
import { Link } from 'react-router-dom';

import { buildPath } from '@/app/routes/routes';
import { useLibrary, usePlaybackControls, usePlayer, useUi } from '@/hooks';
import { useUiStore } from '@/store/uiStore';
import { Avatar, IconButton, Marquee, Slider } from '@/components/ui';

import styles from './NowPlayingBar.module.css';
import type { RepeatState } from '@/lib/spotify/types';

export interface NowPlayingBarProps {
  className?: string;
}

function formatTime(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return '0:00';
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function fireAndForget(promise: Promise<unknown>): void {
  promise.catch(() => undefined);
}

/**
 * Premium now-playing bar. Reads `usePlayer()` / `useUi()` and hides itself
 * entirely when nothing is playing.
 */
export function NowPlayingBar({ className }: NowPlayingBarProps) {
  const { playback, isPlaying, progressMs, volume, setProgressMs } = usePlayer();
  const { isTrackSaved, toggleSavedTrack } = useLibrary();
  const controls = usePlaybackControls();
  const { setNowPlayingOpen } = useUi();
  const queueOpen = useUiStore((state) => state.queueOpen);
  const setQueueOpen = useUiStore((state) => state.setQueueOpen);
  const [lastVolume, setLastVolume] = useState(volume);

  const track = playback?.item ?? null;
  if (!track) return null;

  const duration = track.duration_ms > 0 ? track.duration_ms : 1;
  const position = Math.min(Math.max(progressMs, 0), duration);
  const repeat: RepeatState = playback?.repeat_state ?? 'off';
  const shuffle = playback?.shuffle_state ?? false;
  const saved = isTrackSaved(track.id);

  const cycleRepeat = () => {
    const next: RepeatState =
      repeat === 'off' ? 'context' : repeat === 'context' ? 'track' : 'off';
    fireAndForget(controls.setRepeat(next));
  };

  const handleSeek = (value: number) => {
    setProgressMs(value);
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

  return (
    <footer
      className={[styles.root, className ?? ''].filter(Boolean).join(' ')}
      aria-label="Lecture en cours"
    >
      <div className={styles.trackInfo}>
        <button
          type="button"
          className={styles.coverButton}
          onClick={() => setNowPlayingOpen(true)}
          aria-label="Ouvrir la vue lecture"
        >
          <Avatar
            src={track.album.images?.[0]?.url ?? null}
            alt={track.album.name}
            size={56}
            className={styles.cover}
          />
        </button>

        <div className={styles.meta}>
          <Marquee className={styles.trackName}>{track.name}</Marquee>
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
          size="sm"
          className={[styles.like, saved ? styles.likeActive : ''].filter(Boolean).join(' ')}
          onClick={() => toggleSavedTrack(track.id)}
        >
          <Heart fill={saved ? 'currentColor' : 'none'} />
        </IconButton>
      </div>

      <div className={styles.controls}>
        <div className={styles.buttons}>
          <IconButton
            label="Lecture aléatoire"
            size="sm"
            className={shuffle ? styles.activeControl : ''}
            onClick={() => fireAndForget(controls.setShuffle(!shuffle))}
          >
            <Shuffle />
          </IconButton>
          <IconButton
            label="Piste précédente"
            size="sm"
            onClick={() => fireAndForget(controls.previous())}
          >
            <SkipBack fill="currentColor" />
          </IconButton>

          <IconButton
            label={isPlaying ? 'Pause' : 'Lecture'}
            variant="primary"
            size="lg"
            className={styles.playButton}
            onClick={() => fireAndForget(isPlaying ? controls.pause() : controls.play())}
          >
            {isPlaying ? <Pause fill="currentColor" /> : <Play fill="currentColor" />}
          </IconButton>

          <IconButton
            label="Piste suivante"
            size="sm"
            onClick={() => fireAndForget(controls.next())}
          >
            <SkipForward fill="currentColor" />
          </IconButton>
          <IconButton
            label="Répéter"
            size="sm"
            className={repeat !== 'off' ? styles.activeControl : ''}
            onClick={cycleRepeat}
          >
            {repeat === 'track' ? <Repeat1 /> : <Repeat />}
          </IconButton>
        </div>

        <div className={styles.progress}>
          <span className={styles.time}>{formatTime(position)}</span>
          <Slider
            className={styles.progressSlider}
            value={position}
            min={0}
            max={duration}
            step={1000}
            ariaLabel="Position de lecture"
            onChange={handleSeek}
          />
          <span className={styles.time}>{formatTime(track.duration_ms)}</span>
        </div>
      </div>

      <div className={styles.right}>
        <IconButton
          label="File d’attente"
          size="sm"
          className={queueOpen ? styles.activeControl : ''}
          onClick={() => setQueueOpen(!queueOpen)}
        >
          <ListMusic />
        </IconButton>

        <div className={styles.volume}>
          <IconButton
            label={volume > 0 ? 'Couper le son' : 'Rétablir le son'}
            size="sm"
            onClick={toggleMute}
          >
            {volume > 0 ? <Volume2 /> : <VolumeX />}
          </IconButton>
          <Slider
            className={styles.volumeSlider}
            value={volume}
            min={0}
            max={100}
            step={1}
            ariaLabel="Volume"
            onChange={handleVolume}
          />
        </div>

        <IconButton
          label="Agrandir la vue lecture"
          size="sm"
          onClick={() => setNowPlayingOpen(true)}
        >
          <ChevronUp />
        </IconButton>
      </div>
    </footer>
  );
}
