import { Copy, Download, RotateCcw, Save, Shuffle, Upload } from 'lucide-react';
import { useCallback, useRef, useState, type ChangeEvent } from 'react';

import { Button, Modal } from '@/components/ui';
import { useThemeStore } from '@/store/themeStore';
import { useUiStore } from '@/store/uiStore';
import { useCustomizerStore } from '@/theme/customizerStore';
import type { BackgroundStyle } from '@/theme/types';

import styles from './Customizer.module.css';
import {
  BackgroundSection,
  ColorsSection,
  PresetsSection,
  StructureSection,
  TypographyMotionSection,
} from './sections';
import {
  copyText,
  downloadTheme,
  parseTheme,
  randomTheme,
  serializeTheme,
} from './themeIo';

export interface CustomizerBodyProps {
  variant?: 'drawer' | 'page';
}

const RANDOM_BACKGROUNDS: readonly BackgroundStyle[] = ['solid', 'gradient', 'mesh', 'noise'];

interface ImportDialogProps {
  open: boolean;
  onClose: () => void;
}

function ImportDialog({ open, onClose }: ImportDialogProps) {
  const [text, setText] = useState('');
  const [error, setError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const pushToast = useUiStore((state) => state.pushToast);

  const handleImport = () => {
    try {
      const theme = parseTheme(text);
      useThemeStore.getState().setTheme(theme);
      setError(null);
      setText('');
      pushToast({ message: 'Thème importé', variant: 'success' });
      onClose();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Import impossible.');
    }
  };

  const handleFile = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.currentTarget.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      setText(typeof reader.result === 'string' ? reader.result : '');
      setError(null);
    };
    reader.onerror = () => setError('Lecture du fichier impossible.');
    reader.readAsText(file);
    event.currentTarget.value = '';
  };

  return (
    <Modal open={open} onClose={onClose} title="Importer un thème">
      <div className={styles.importBody}>
        <p className={styles.importHelp}>
          Collez un thème Prism (JSON) ou chargez un fichier{' '}
          <code>.prism-theme.json</code>.
        </p>
        <textarea
          className={styles.importText}
          value={text}
          spellCheck={false}
          placeholder={'{\n  "id": "mon-theme",\n  "colorMode": "dark",\n  "tokens": { "colors": { "accent": "#1ed760" } }\n}'}
          onChange={(event) => setText(event.currentTarget.value)}
          aria-label="JSON du thème à importer"
        />
        {error ? (
          <p className={styles.importError} role="alert">
            {error}
          </p>
        ) : null}
        <div className={styles.importActions}>
          <input
            ref={fileRef}
            type="file"
            accept="application/json,.json"
            className={styles.fileInput}
            onChange={handleFile}
            tabIndex={-1}
            aria-hidden
          />
          <Button
            variant="ghost"
            size="sm"
            iconLeft={<Upload />}
            onClick={() => fileRef.current?.click()}
          >
            Charger un fichier
          </Button>
          <Button
            variant="primary"
            size="sm"
            iconLeft={<Download />}
            disabled={!text.trim()}
            onClick={handleImport}
          >
            Importer
          </Button>
        </div>
      </div>
    </Modal>
  );
}

/**
 * The full customizer surface, shared by the slide-over drawer and the
 * `/settings` full page. Every control updates the theme (and ambient effects)
 * live through the stores, so the preview is always instant.
 */
export function CustomizerBody({ variant = 'drawer' }: CustomizerBodyProps) {
  const pushToast = useUiStore((state) => state.pushToast);
  const [importOpen, setImportOpen] = useState(false);

  const handleReset = useCallback(() => {
    useThemeStore.getState().reset();
    useCustomizerStore.getState().resetEffects();
    pushToast({ message: 'Personnalisation réinitialisée', variant: 'info' });
  }, [pushToast]);

  const handleSurprise = useCallback(() => {
    useThemeStore.getState().setTheme(randomTheme());
    const background =
      RANDOM_BACKGROUNDS[Math.floor(Math.random() * RANDOM_BACKGROUNDS.length)];
    useCustomizerStore.getState().patchEffects({ background });
    pushToast({ message: 'Surprise ! Un thème tout neuf', variant: 'info' });
  }, [pushToast]);

  const handleSavePreset = useCallback(() => {
    useCustomizerStore
      .getState()
      .saveCustomPreset(useThemeStore.getState().theme);
    pushToast({ message: 'Preset enregistré dans vos presets', variant: 'success' });
  }, [pushToast]);

  const handleCopyJson = useCallback(async () => {
    const ok = await copyText(serializeTheme(useThemeStore.getState().theme));
    pushToast({
      message: ok ? 'Thème copié dans le presse-papier' : 'Copie impossible',
      variant: ok ? 'success' : 'danger',
    });
  }, [pushToast]);

  const handleDownload = useCallback(() => {
    downloadTheme(useThemeStore.getState().theme);
  }, []);

  return (
    <div className={styles.body} data-variant={variant}>
      <PresetsSection />
      <ColorsSection />
      <StructureSection />
      <TypographyMotionSection />
      <BackgroundSection />

      <section className={styles.section}>
        <h3 className={styles.actionsHeading}>Actions</h3>
        <div className={styles.actions}>
          <Button
            variant="secondary"
            size="sm"
            fullWidth
            iconLeft={<RotateCcw />}
            onClick={handleReset}
          >
            Réinitialiser
          </Button>
          <Button
            variant="primary"
            size="sm"
            fullWidth
            iconLeft={<Shuffle />}
            onClick={handleSurprise}
          >
            Surprends-moi
          </Button>
          <Button
            variant="secondary"
            size="sm"
            fullWidth
            iconLeft={<Save />}
            onClick={handleSavePreset}
          >
            Enregistrer le preset
          </Button>
          <Button
            variant="ghost"
            size="sm"
            fullWidth
            iconLeft={<Copy />}
            onClick={handleCopyJson}
          >
            Copier le JSON
          </Button>
          <Button
            variant="ghost"
            size="sm"
            fullWidth
            iconLeft={<Download />}
            onClick={handleDownload}
          >
            Télécharger
          </Button>
          <Button
            variant="ghost"
            size="sm"
            fullWidth
            iconLeft={<Upload />}
            onClick={() => setImportOpen(true)}
          >
            Importer
          </Button>
        </div>
      </section>

      <ImportDialog open={importOpen} onClose={() => setImportOpen(false)} />
    </div>
  );
}
