/**
 * Prism — Library feature barrel (Worker 5).
 *
 * Views are mounted by W1's route table; components and helpers are shared with
 * the player feature.
 */

/* Views -------------------------------------------------------------------- */
export { HomeView } from './views/HomeView/HomeView';
export { LibraryView } from './views/LibraryView/LibraryView';
export type { LibraryViewProps, LibraryTab } from './views/LibraryView/LibraryView';
export { SearchView } from './views/SearchView/SearchView';
export { PlaylistView } from './views/PlaylistView/PlaylistView';
export type { PlaylistViewProps } from './views/PlaylistView/PlaylistView';
export { AlbumView } from './views/AlbumView/AlbumView';
export type { AlbumViewProps } from './views/AlbumView/AlbumView';
export { ArtistView } from './views/ArtistView/ArtistView';
export type { ArtistViewProps } from './views/ArtistView/ArtistView';

/* Components --------------------------------------------------------------- */
export { AuthGate } from './components/AuthGate/AuthGate';
export type { AuthGateProps } from './components/AuthGate/AuthGate';
export { LoginView } from './components/LoginView/LoginView';
export type { LoginViewProps } from './components/LoginView/LoginView';
export { EmptyState } from './components/EmptyState/EmptyState';
export type { EmptyStateProps } from './components/EmptyState/EmptyState';
export { ErrorState } from './components/ErrorState/ErrorState';
export type { ErrorStateProps } from './components/ErrorState/ErrorState';
export { Equalizer } from './components/Equalizer/Equalizer';
export type { EqualizerProps } from './components/Equalizer/Equalizer';
export { HeroHeader } from './components/HeroHeader/HeroHeader';
export type { HeroHeaderProps } from './components/HeroHeader/HeroHeader';
export { MediaCard } from './components/MediaCard/MediaCard';
export type { MediaCardProps } from './components/MediaCard/MediaCard';
export { Menu } from './components/Menu/Menu';
export type { MenuProps, MenuItem } from './components/Menu/Menu';
export { Reveal } from './components/Reveal/Reveal';
export type { RevealProps } from './components/Reveal/Reveal';
export { SectionHeader } from './components/SectionHeader/SectionHeader';
export type { SectionHeaderProps } from './components/SectionHeader/SectionHeader';
export { Shelf } from './components/Shelf/Shelf';
export type { ShelfProps } from './components/Shelf/Shelf';
export { TrackList } from './components/TrackList/TrackList';
export type { TrackListProps } from './components/TrackList/TrackList';
export { TrackRow } from './components/TrackRow/TrackRow';
export type { TrackRowProps } from './components/TrackRow/TrackRow';
export { TrackListSkeleton } from './components/TrackListSkeleton/TrackListSkeleton';
export type { TrackListSkeletonProps } from './components/TrackListSkeleton/TrackListSkeleton';
export { ShelfSkeleton } from './components/ShelfSkeleton/ShelfSkeleton';
export type { ShelfSkeletonProps } from './components/ShelfSkeleton/ShelfSkeleton';
export { GridSkeleton } from './components/GridSkeleton/GridSkeleton';
export type { GridSkeletonProps } from './components/GridSkeleton/GridSkeleton';

/* Helpers ------------------------------------------------------------------ */
export { cn, fireAndForget } from './lib/utils';
export {
  formatDuration,
  formatCount,
  formatDate,
  formatYear,
  formatTotalDuration,
  formatArtists,
  getGreeting,
} from './lib/format';
export { useArtworkGradient, hashGradient } from './lib/artwork';
export type { ArtworkPalette } from './lib/artwork';
export { useSavedAlbums } from './lib/queries';
