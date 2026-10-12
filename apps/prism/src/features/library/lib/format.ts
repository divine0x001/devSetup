/**
 * Prism — Library feature: French-locale formatting helpers.
 */

const compactNumber = new Intl.NumberFormat('fr-FR', {
  notation: 'compact',
  maximumFractionDigits: 1,
});

const longDate = new Intl.DateTimeFormat('fr-FR', {
  day: 'numeric',
  month: 'long',
  year: 'numeric',
});

/** `3:07` style duration from milliseconds. */
export function formatDuration(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return '0:00';
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

/** `1,2 M` / `12,4 k` compact count. */
export function formatCount(value: number): string {
  if (!Number.isFinite(value)) return '0';
  return compactNumber.format(value);
}

/** `12 mars 2024` from an ISO date string. */
export function formatDate(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return longDate.format(date);
}

/** `2024` from a Spotify `release_date` (which may be `YYYY`, `YYYY-MM`…). */
export function formatYear(releaseDate: string): string {
  return releaseDate.slice(0, 4);
}

/** `1 h 23 min` / `45 min` total duration. */
export function formatTotalDuration(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return '0 min';
  const totalMinutes = Math.round(ms / 60_000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours <= 0) return `${minutes} min`;
  if (minutes === 0) return `${hours} h`;
  return `${hours} h ${minutes} min`;
}

/** Time-of-day greeting used on the home view. */
export function getGreeting(date: Date = new Date()): string {
  const hour = date.getHours();
  if (hour < 6) return 'Bonne nuit';
  if (hour < 12) return 'Bonjour';
  if (hour < 18) return 'Bon après-midi';
  return 'Bonsoir';
}

/** Joins artist names with a French comma separator. */
export function formatArtists(artists: Array<{ name: string }>): string {
  return artists.map((artist) => artist.name).join(', ');
}
