import { useState, useRef } from 'react';
import styles from './SearchScreen.module.css';
import { MailRow } from '../components/MailRow';
import { useAppStore } from '../store';
import { api } from '../api/client';
import type { Message } from '../types';

export function SearchScreen() {
  const { setSelectedMessage, theme } = useAppStore();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Message[]>([]);
  const [loading, setLoading] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleChange = (q: string) => {
    setQuery(q);
    if (timerRef.current) clearTimeout(timerRef.current);
    if (!q.trim()) { setResults([]); return; }
    timerRef.current = setTimeout(async () => {
      setLoading(true);
      try {
        const r = await api.search(q) as { messages: Message[] };
        setResults(r.messages ?? []);
      } catch { setResults([]); }
      setLoading(false);
    }, 300);
  };

  return (
    <div className={styles.screen}>
      <header className={styles.header}>
        <div className={styles.searchBox}>
          <img
            src={theme === 'dark' ? '/icons/search-dark.png' : '/icons/search-light.png'}
            alt=""
            width="16" height="16"
            className={styles.searchIcon}
          />
          <input
            className={styles.input}
            type="search"
            placeholder="Search mail…"
            value={query}
            onChange={(e) => handleChange(e.target.value)}
            autoFocus
          />
          {query && (
            <button className={styles.clearBtn} onClick={() => handleChange('')} aria-label="Clear">
              <svg viewBox="0 0 12 12" fill="currentColor" width="12" height="12">
                <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" fill="none" />
              </svg>
            </button>
          )}
        </div>
      </header>

      <div className={`${styles.results} scroll`}>
        {loading && <div className={styles.hint}>Searching…</div>}
        {!loading && query && results.length === 0 && (
          <div className={styles.hint}>No results for "{query}"</div>
        )}
        {!loading && !query && (
          <div className={styles.hint}>Start typing to search your mail</div>
        )}
        {results.map((msg) => (
          <MailRow key={msg.id} message={msg} onClick={() => setSelectedMessage(msg)} />
        ))}
        <div className={styles.listPad} />
      </div>
    </div>
  );
}
