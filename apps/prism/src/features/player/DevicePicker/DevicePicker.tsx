import { useQuery } from '@tanstack/react-query';
import { Cast, Check, Laptop, Monitor, Smartphone, Speaker, Tv } from 'lucide-react';
import type { ReactNode } from 'react';

import { Modal, Spinner } from '@/components/ui';
import { useAuth } from '@/hooks';
import { spotifyApi } from '@/lib/spotify/client';
import { cn, fireAndForget } from '@/features/library/lib/utils';

import styles from './DevicePicker.module.css';

import type { PlayerDevice } from '@/lib/spotify/types';

export interface DevicePickerProps {
  open: boolean;
  onClose: () => void;
}

function deviceIcon(type: string): ReactNode {
  switch (type.toLowerCase()) {
    case 'computer':
      return <Laptop />;
    case 'smartphone':
      return <Smartphone />;
    case 'tv':
      return <Tv />;
    case 'speaker':
      return <Speaker />;
    case 'castaudio':
    case 'castvideo':
      return <Cast />;
    default:
      return <Monitor />;
  }
}

/** Modal listing the user's Spotify Connect devices. */
export function DevicePicker({ open, onClose }: DevicePickerProps) {
  const { isAuthenticated } = useAuth();

  const devicesQuery = useQuery({
    queryKey: ['devices'],
    queryFn: () => spotifyApi.getDevices(),
    enabled: open && isAuthenticated,
    refetchInterval: open ? 5000 : false,
  });

  const devices = devicesQuery.data?.devices ?? [];

  const selectDevice = (device: PlayerDevice) => {
    if (!device.id || device.is_restricted) return;
    fireAndForget(spotifyApi.transferPlayback(device.id));
    onClose();
  };

  return (
    <Modal open={open} onClose={onClose} title="Appareils disponibles">
      {devicesQuery.isLoading ? (
        <div className={styles.loading}>
          <Spinner label="Recherche des appareils" />
        </div>
      ) : devices.length === 0 ? (
        <p className={styles.empty}>
          Aucun appareil trouvé. Ouvrez Spotify sur un appareil connecté au même compte.
        </p>
      ) : (
        <ul className={styles.list}>
          {devices.map((device) => {
            const disabled = !device.id || device.is_restricted;
            return (
              <li key={device.id ?? device.name}>
                <button
                  type="button"
                  className={cn(styles.device, device.is_active ? styles.active : '')}
                  disabled={disabled}
                  onClick={() => selectDevice(device)}
                >
                  <span className={styles.icon} aria-hidden>
                    {deviceIcon(device.type)}
                  </span>
                  <span className={styles.meta}>
                    <span className={styles.name}>{device.name}</span>
                    <span className={styles.type}>
                      {device.is_active ? 'Actif' : device.type}
                    </span>
                  </span>
                  {device.is_active ? (
                    <span className={styles.check} aria-hidden>
                      <Check />
                    </span>
                  ) : null}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </Modal>
  );
}
