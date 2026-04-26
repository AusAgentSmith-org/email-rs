import styles from './ProfileScreen.module.css';
import { useAppStore } from '../store';

export function ProfileScreen() {
  const { accounts, theme, setTheme } = useAppStore();

  return (
    <div className={styles.screen}>
      <header className={styles.header}>
        <span className={styles.title}>Account</span>
      </header>

      <div className={`${styles.body} scroll`}>
        {/* Accounts */}
        {accounts.map((acc) => {
          const hue = acc.name.split('').reduce((n, c) => n + c.charCodeAt(0), 0) % 360;
          return (
            <div key={acc.id} className={styles.accountCard}>
              <div className={styles.accountAvatar} style={{ background: `oklch(62% 0.12 ${hue})` }}>
                {acc.name.split(' ').map((w) => w[0]).join('').slice(0, 2).toUpperCase()}
              </div>
              <div className={styles.accountInfo}>
                <span className={styles.accountName}>{acc.name}</span>
                <span className={styles.accountEmail}>{acc.email}</span>
              </div>
            </div>
          );
        })}

        {/* Settings group */}
        <div className={styles.group}>
          <div className={styles.groupLabel}>Appearance</div>
          <div className={styles.row}>
            <span className={styles.rowLabel}>Theme</span>
            <div className={styles.toggle}>
              <button
                className={`${styles.toggleOption} ${theme === 'light' ? styles.toggleActive : ''}`}
                onClick={() => setTheme('light')}
              >Light</button>
              <button
                className={`${styles.toggleOption} ${theme === 'dark' ? styles.toggleActive : ''}`}
                onClick={() => setTheme('dark')}
              >Dark</button>
            </div>
          </div>
        </div>

        <div className={styles.group}>
          <div className={styles.groupLabel}>About</div>
          <div className={styles.row}>
            <span className={styles.rowLabel}>rsMail</span>
            <span className={styles.rowValue}>v0.1.0</span>
          </div>
        </div>

        <div className={styles.bottomPad} />
      </div>
    </div>
  );
}
