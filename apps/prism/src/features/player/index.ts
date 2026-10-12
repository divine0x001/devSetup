/**
 * Prism — Player feature barrel (Worker 5).
 *
 * `NowPlayingView` is mounted once at the app root (see the integration notes);
 * the queue store and helpers are shared with the library feature.
 */

export { NowPlayingView } from './NowPlayingView/NowPlayingView';
export type { NowPlayingViewProps } from './NowPlayingView/NowPlayingView';
export { QueueView } from './QueueView/QueueView';
export type { QueueViewProps } from './QueueView/QueueView';
export { DevicePicker } from './DevicePicker/DevicePicker';
export type { DevicePickerProps } from './DevicePicker/DevicePicker';
export { Visualizer } from './Visualizer/Visualizer';
export type { VisualizerProps } from './Visualizer/Visualizer';
export { useQueueStore } from './queueStore';
export type { QueueStoreState } from './queueStore';
export { useSmoothProgress } from './lib/useSmoothProgress';
