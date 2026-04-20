import { useState, useCallback, useEffect } from 'react';
import styles from './ComposeModal.module.css';
import { useAppStore } from '../../store';
import type { AccountSettings } from '../../types';

export function ComposeModal() {
  const { compose, closeCompose } = useAppStore();

  const [to, setTo] = useState('');
  const [subject, setSubject] = useState('');
  const [body, setBody] = useState('');
  const [signature, setSignature] = useState('');
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (compose) {
      setTo(compose.to);
      setSubject(compose.subject);
      setBody(compose.mode === 'forward' ? buildForwardBody(compose.quotedText, compose.quotedFrom) : '');
      setError(null);
      setSending(false);
      // Fetch account signature.
      fetch(`/api/v1/accounts/${compose.accountId}/settings`)
        .then((r) => r.json())
        .then((s: AccountSettings) => { if (s.signature) setSignature(s.signature); })
        .catch(() => {});
    }
  }, [compose]);

  const handleSend = useCallback(async () => {
    if (!compose || !to.trim() || !body.trim()) return;
    setSending(true);
    setError(null);
    try {
      const sigBlock = signature ? `\n\n-- \n${signature}` : '';
      const fullBody = (compose.mode === 'reply' && compose.quotedText
        ? body + '\n\n' + buildQuotedBlock(compose.quotedText, compose.quotedFrom)
        : body) + sigBlock;
      const resp = await fetch('/api/v1/messages', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          accountId: compose.accountId,
          to: to.split(',').map((s) => s.trim()).filter(Boolean),
          cc: [],
          bcc: [],
          subject,
          textBody: fullBody,
          inReplyTo: compose.inReplyTo,
        }),
      });
      if (!resp.ok) {
        const err = await resp.json().catch(() => ({ error: 'Send failed' }));
        setError((err as { error?: string }).error ?? 'Send failed');
      } else {
        closeCompose();
      }
    } catch {
      setError('Network error');
    } finally {
      setSending(false);
    }
  }, [compose, to, subject, body, closeCompose]);

  if (!compose) return null;

  const modeLabel = compose.mode === 'reply' ? 'Reply' : compose.mode === 'forward' ? 'Forward' : 'New message';

  return (
    <div className={styles.pane}>
      <div className={styles.header}>
        <span className={styles.modeTag} data-mode={compose.mode}>{modeLabel}</span>
        <span className={styles.headerSubject}>{compose.subject || 'No subject'}</span>
        <button className={styles.closeBtn} type="button" onClick={closeCompose} aria-label="Discard">
          ×
        </button>
      </div>

      <div className={styles.fields}>
        <div className={styles.fieldRow}>
          <span className={styles.fieldLabel}>To</span>
          <input
            className={styles.fieldInput}
            type="text"
            value={to}
            onChange={(e) => setTo(e.target.value)}
            placeholder="recipient@example.com"
            autoFocus={!to}
          />
        </div>
        <div className={styles.fieldRow}>
          <span className={styles.fieldLabel}>Subject</span>
          <input
            className={styles.fieldInput}
            type="text"
            value={subject}
            onChange={(e) => setSubject(e.target.value)}
            placeholder="Subject"
          />
        </div>
      </div>

      <div className={styles.body}>
        <textarea
          className={styles.bodyArea}
          value={body}
          onChange={(e) => setBody(e.target.value)}
          placeholder="Write your message…"
          autoFocus={!!to}
        />

        {signature && (
          <div className={styles.signatureBlock}>
            <div className={styles.signatureDivider}>--</div>
            <pre className={styles.signatureText}>{signature}</pre>
          </div>
        )}

        {compose.mode === 'reply' && compose.quotedText && (
          <div className={styles.quotedBlock}>
            <div className={styles.quotedFrom}>
              {compose.quotedFrom ? `— ${compose.quotedFrom} wrote:` : '— Original message:'}
            </div>
            <pre className={styles.quotedText}>{compose.quotedText}</pre>
          </div>
        )}
      </div>

      <div className={styles.footer}>
        <button
          className={styles.sendBtn}
          type="button"
          onClick={handleSend}
          disabled={sending || !to.trim() || !body.trim()}
        >
          {sending ? 'Sending…' : 'Send'}
        </button>
        <button className={styles.discardBtn} type="button" onClick={closeCompose}>
          Discard
        </button>
        {error && <span className={styles.errorMsg}>{error}</span>}
      </div>
    </div>
  );
}

function buildQuotedBlock(text: string, from?: string): string {
  const prefix = from ? `${from} wrote:\n` : 'Original message:\n';
  return prefix + text.split('\n').map((l) => `> ${l}`).join('\n');
}

function buildForwardBody(text?: string, from?: string): string {
  if (!text) return '';
  const header = from ? `---------- Forwarded message from ${from} ----------\n\n` : '---------- Forwarded message ----------\n\n';
  return header + text;
}
