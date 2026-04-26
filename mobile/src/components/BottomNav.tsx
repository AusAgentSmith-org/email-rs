import styles from './BottomNav.module.css';
import { useAppStore } from '../store';
import type { Screen } from '../types';

function CalendarIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" width="24" height="24">
      <rect x="3" y="4" width="18" height="18" rx="3" />
      <path d="M3 9h18M8 2v4M16 2v4" />
      <rect x="7" y="13" width="3" height="3" rx="0.5" fill="currentColor" stroke="none" />
    </svg>
  );
}

interface PngTab { id: Screen; label: string; icon: string; iconDark: string; }
const PNG_TABS: PngTab[] = [
  { id: 'inbox',   label: 'Inbox',   icon: '/icons/inbox-open-light.png', iconDark: '/icons/inbox-open-dark.png' },
  { id: 'search',  label: 'Search',  icon: '/icons/search-light.png',     iconDark: '/icons/search-dark.png' },
  { id: 'profile', label: 'Profile', icon: '/icons/profile-light.png',    iconDark: '/icons/profile-dark.png' },
];

export function BottomNav() {
  const { screen, setScreen, theme, openCompose } = useAppStore();

  return (
    <nav className={styles.nav}>
      {/* Inbox + Search on left of FAB */}
      {PNG_TABS.slice(0, 2).map((tab) => (
        <button
          key={tab.id}
          className={`${styles.tab} ${screen === tab.id ? styles.active : ''}`}
          onClick={() => setScreen(tab.id)}
          aria-label={tab.label}
        >
          <img src={theme === 'dark' ? tab.iconDark : tab.icon} alt="" className={styles.icon} />
          <span className={styles.label}>{tab.label}</span>
        </button>
      ))}

      {/* FAB placeholder — rendered as absolute overlay below */}
      <div className={styles.fabPlaceholder} />

      {/* Calendar + Profile on right of FAB */}
      <button
        className={`${styles.tab} ${screen === 'calendar' ? styles.active : ''}`}
        onClick={() => setScreen('calendar')}
        aria-label="Calendar"
      >
        <span className={`${styles.svgIcon} ${screen === 'calendar' ? styles.svgActive : ''}`}>
          <CalendarIcon />
        </span>
        <span className={styles.label}>Calendar</span>
      </button>
      {PNG_TABS.slice(2).map((tab) => (
        <button
          key={tab.id}
          className={`${styles.tab} ${screen === tab.id ? styles.active : ''}`}
          onClick={() => setScreen(tab.id)}
          aria-label={tab.label}
        >
          <img src={theme === 'dark' ? tab.iconDark : tab.icon} alt="" className={styles.icon} />
          <span className={styles.label}>{tab.label}</span>
        </button>
      ))}

      <button className={styles.fab} onClick={openCompose} aria-label="Compose">
        <svg viewBox="0 0 20 20" fill="none" stroke="white" strokeWidth="2" strokeLinecap="round" width="20" height="20">
          <path d="M10 4v12M4 10h12" />
        </svg>
      </button>
    </nav>
  );
}
