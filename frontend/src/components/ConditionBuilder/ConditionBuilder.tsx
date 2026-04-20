import styles from './ConditionBuilder.module.css';
import type { Condition, ConditionField, ConditionGroup, ConditionOperator } from '../../types';

type FieldType = 'text' | 'boolean' | 'date';

interface FieldDef {
  label: string;
  type: FieldType;
}

const FIELDS: Record<ConditionField, FieldDef> = {
  from:           { label: 'From',           type: 'text' },
  to:             { label: 'To',             type: 'text' },
  cc:             { label: 'Cc',             type: 'text' },
  subject:        { label: 'Subject',        type: 'text' },
  body:           { label: 'Body',           type: 'text' },
  filename:       { label: 'Filename',       type: 'text' },
  has_attachment: { label: 'Has attachment', type: 'boolean' },
  is_read:        { label: 'Is read',        type: 'boolean' },
  is_flagged:     { label: 'Is starred',     type: 'boolean' },
  date_after:     { label: 'Date after',     type: 'date' },
  date_before:    { label: 'Date before',    type: 'date' },
};

const FIELD_ORDER: ConditionField[] = [
  'from', 'to', 'cc', 'subject', 'body', 'filename',
  'has_attachment', 'is_read', 'is_flagged', 'date_after', 'date_before',
];

const TEXT_OPERATORS: { value: ConditionOperator; label: string }[] = [
  { value: 'contains',     label: 'contains' },
  { value: 'not_contains', label: "doesn't contain" },
  { value: 'equals',       label: 'equals' },
  { value: 'starts_with',  label: 'starts with' },
];

function defaultValue(field: ConditionField): string {
  return FIELDS[field].type === 'boolean' ? 'true' : '';
}

export function newCondition(): Condition {
  return {
    id: Math.random().toString(36).slice(2, 9),
    field: 'from',
    operator: 'contains',
    value: '',
  };
}

export function defaultConditionGroup(): ConditionGroup {
  return { match: 'all', conditions: [newCondition()] };
}

interface Props {
  value: ConditionGroup;
  onChange: (g: ConditionGroup) => void;
}

export function ConditionBuilder({ value, onChange }: Props) {
  const patch = (p: Partial<ConditionGroup>) => onChange({ ...value, ...p });

  const updateCond = (id: string, p: Partial<Condition>) =>
    onChange({ ...value, conditions: value.conditions.map((c) => (c.id === id ? { ...c, ...p } : c)) });

  const removeCond = (id: string) => {
    const rest = value.conditions.filter((c) => c.id !== id);
    onChange({ ...value, conditions: rest.length ? rest : [newCondition()] });
  };

  return (
    <div className={styles.builder}>
      <div className={styles.matchRow}>
        <span className={styles.matchLabel}>Match</span>
        <select
          className={styles.select}
          value={value.match}
          onChange={(e) => patch({ match: e.target.value as 'all' | 'any' })}
        >
          <option value="all">All</option>
          <option value="any">Any</option>
        </select>
        <span className={styles.matchLabel}>of the following conditions</span>
      </div>

      <div className={styles.conditions}>
        {value.conditions.map((cond) => {
          const { type } = FIELDS[cond.field];
          return (
            <div key={cond.id} className={styles.condRow}>
              <select
                className={styles.select}
                value={cond.field}
                onChange={(e) => {
                  const f = e.target.value as ConditionField;
                  updateCond(cond.id, { field: f, value: defaultValue(f), operator: 'contains' });
                }}
              >
                {FIELD_ORDER.map((f) => (
                  <option key={f} value={f}>{FIELDS[f].label}</option>
                ))}
              </select>

              {type === 'text' && (
                <select
                  className={styles.select}
                  value={cond.operator}
                  onChange={(e) => updateCond(cond.id, { operator: e.target.value as ConditionOperator })}
                >
                  {TEXT_OPERATORS.map((op) => (
                    <option key={op.value} value={op.value}>{op.label}</option>
                  ))}
                </select>
              )}

              {type === 'text' && (
                <input
                  className={styles.input}
                  type="text"
                  placeholder={cond.field === 'from' || cond.field === 'to' || cond.field === 'cc' ? 'name or email' : ''}
                  value={cond.value}
                  onChange={(e) => updateCond(cond.id, { value: e.target.value })}
                />
              )}

              {type === 'boolean' && (
                <select
                  className={styles.select}
                  value={cond.value}
                  onChange={(e) => updateCond(cond.id, { value: e.target.value })}
                >
                  <option value="true">Yes</option>
                  <option value="false">No</option>
                </select>
              )}

              {type === 'date' && (
                <input
                  className={`${styles.input} ${styles.dateInput}`}
                  type="date"
                  value={cond.value}
                  onChange={(e) => updateCond(cond.id, { value: e.target.value })}
                />
              )}

              <button
                className={styles.removeBtn}
                onClick={() => removeCond(cond.id)}
                aria-label="Remove condition"
                type="button"
              >
                ✕
              </button>
            </div>
          );
        })}
      </div>

      <button
        className={styles.addBtn}
        type="button"
        onClick={() => onChange({ ...value, conditions: [...value.conditions, newCondition()] })}
      >
        + Add condition
      </button>
    </div>
  );
}
