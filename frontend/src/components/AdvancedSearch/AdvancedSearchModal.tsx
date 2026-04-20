import React, { useState } from 'react';
import styles from './AdvancedSearchModal.module.css';
import { ConditionBuilder, defaultConditionGroup } from '../ConditionBuilder/ConditionBuilder';
import { useAppStore } from '../../store';
import type { ConditionGroup } from '../../types';

interface Props {
  onClose: () => void;
}

export function AdvancedSearchModal({ onClose }: Props) {
  const { conditionGroup, setConditionGroup, setSearchQuery } = useAppStore();
  const [draft, setDraft] = useState<ConditionGroup>(() => conditionGroup ?? defaultConditionGroup());

  const hasActive = draft.conditions.some((c) => c.value.trim());

  const handleSearch = () => {
    setSearchQuery('');
    setConditionGroup(draft);
    onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && hasActive) handleSearch();
    if (e.key === 'Escape') onClose();
  };

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div
        className={styles.modal}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <div className={styles.header}>
          <span className={styles.title}>Advanced search</span>
          <button className={styles.closeBtn} onClick={onClose} type="button">✕</button>
        </div>

        <div className={styles.body}>
          <ConditionBuilder value={draft} onChange={setDraft} />
        </div>

        <div className={styles.footer}>
          <button
            className={styles.clearBtn}
            type="button"
            onClick={() => setDraft(defaultConditionGroup())}
          >
            Clear
          </button>
          <div style={{ flex: 1 }} />
          <button className={styles.cancelBtn} type="button" onClick={onClose}>
            Cancel
          </button>
          <button
            className={styles.searchBtn}
            type="button"
            onClick={handleSearch}
            disabled={!hasActive}
          >
            Search
          </button>
        </div>
      </div>
    </div>
  );
}
