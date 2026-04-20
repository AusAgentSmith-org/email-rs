import styles from './Toolbar.module.css';
import { useAppStore } from '../../store';

type Density = 'compact' | 'cozy' | 'comfy';

function GearIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" width="15" height="15">
      <circle cx="8" cy="8" r="2.5" />
      <path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.22 3.22l1.42 1.42M11.36 11.36l1.42 1.42M3.22 12.78l1.42-1.42M11.36 4.64l1.42-1.42" strokeLinecap="round" />
    </svg>
  );
}

function SunIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" width="15" height="15">
      <circle cx="8" cy="8" r="3" />
      <path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.22 3.22l1.42 1.42M11.36 11.36l1.42 1.42M3.22 12.78l1.42-1.42M11.36 4.64l1.42-1.42" strokeLinecap="round" />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" width="15" height="15">
      <path d="M13.5 10A6 6 0 016 2.5a6 6 0 100 11 6 6 0 007.5-3.5z" strokeLinejoin="round" />
    </svg>
  );
}

const DENSITY_OPTIONS: { value: Density; label: string; title: string }[] = [
  { value: 'compact', label: '▪', title: 'Compact' },
  { value: 'cozy',    label: '▫',  title: 'Cozy'    },
  { value: 'comfy',   label: '□',  title: 'Comfy'   },
];

export function Toolbar() {
  const { theme, density, setTheme, setDensity, openSettings } = useAppStore();

  return (
    <div className={styles.toolbar}>
      <div className={styles.spacer} />

      <div className={styles.densityGroup} role="group" aria-label="Message density">
        {DENSITY_OPTIONS.map(({ value, label, title }) => (
          <button
            key={value}
            type="button"
            title={title}
            className={`${styles.densityBtn}${density === value ? ` ${styles.active}` : ''}`}
            onClick={() => setDensity(value)}
            aria-pressed={density === value}
          >
            {label}
          </button>
        ))}
      </div>

      <button
        type="button"
        className={styles.iconBtn}
        title={theme === 'light' ? 'Switch to dark mode' : 'Switch to light mode'}
        onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}
        aria-label="Toggle theme"
      >
        {theme === 'light' ? <MoonIcon /> : <SunIcon />}
      </button>

      <button
        type="button"
        className={styles.iconBtn}
        title="Settings"
        onClick={openSettings}
        aria-label="Settings"
      >
        <GearIcon />
      </button>
    </div>
  );
}
