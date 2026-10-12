import styles from './Customizer.module.css';
import { CustomizerBody } from './CustomizerBody';

/**
 * Full-page customizer for the `/settings` route. Same controls as the drawer,
 * laid out as a responsive grid of section cards.
 */
export function CustomizerPage() {
  return (
    <div className={styles.page}>
      <header className={styles.pageHeader}>
        <h1 className={styles.pageTitle}>Personnalisation</h1>
        <p className={styles.pageLead}>
          Façonnez Prism : presets, couleurs, structure, typographie, mouvement et
          ambiance. Tout s’applique en direct.
        </p>
      </header>
      <CustomizerBody variant="page" />
    </div>
  );
}
