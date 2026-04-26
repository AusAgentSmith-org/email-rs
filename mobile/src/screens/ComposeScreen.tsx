import { useState } from 'react';
import styles from './ComposeScreen.module.css';
import { useAppStore } from '../store';

export function ComposeScreen() {
  const { closeCompose } = useAppStore();
  const [to, setTo] = useState('');
  const [subject, setSubject] = useState('');
  const [body, setBody] = useState('');
  const [sending, setSending] = useState(false);

  const handleSend = async () => {
    if (!to.trim() || sending) return;
    setSending(true);
    try {
      await fetch('/api/v1/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ to, subject, body }),
      });
    } catch { /* ignore */ }
    setSending(false);
    closeCompose();
  };

  return (
    <div className={styles.screen}>
      <header className={styles.header}>
        <button className={styles.cancel} onClick={closeCompose}>Cancel</button>
        <span className={styles.title}>New Message</span>
        <button
          className={`${styles.send} ${(!to.trim() || sending) ? styles.sendDisabled : ''}`}
          onClick={handleSend}
          disabled={!to.trim() || sending}
        >
          {sending ? 'Sending…' : 'Send'}
        </button>
      </header>

      <div className={styles.fields}>
        <div className={styles.field}>
          <label className={styles.fieldLabel}>To</label>
          <input
            className={styles.fieldInput}
            type="email"
            value={to}
            onChange={(e) => setTo(e.target.value)}
            placeholder="recipient@example.com"
            autoCapitalize="none"
            autoCorrect="off"
          />
        </div>
        <div className={styles.divider} />
        <div className={styles.field}>
          <label className={styles.fieldLabel}>Subject</label>
          <input
            className={styles.fieldInput}
            type="text"
            value={subject}
            onChange={(e) => setSubject(e.target.value)}
            placeholder="Subject"
          />
        </div>
        <div className={styles.divider} />
        <textarea
          className={styles.bodyInput}
          value={body}
          onChange={(e) => setBody(e.target.value)}
          placeholder="Write your message…"
        />
      </div>
    </div>
  );
}
