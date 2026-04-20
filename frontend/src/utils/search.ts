import type { ConditionField, ConditionGroup } from '../types';

const TEXT_FIELDS: ConditionField[] = ['from', 'to', 'cc', 'subject', 'body', 'filename'];

export function conditionGroupToSearchUrl(g: ConditionGroup): string {
  const active = g.conditions.filter((c) => c.value.trim());
  if (active.length === 0) return '';

  const params = new URLSearchParams();
  params.set('match', g.match);

  // q= keeps the current backend working while we add structured params
  const textVals = active
    .filter((c) => TEXT_FIELDS.includes(c.field))
    .map((c) => c.value.trim());
  if (textVals.length > 0) params.set('q', textVals.join(' '));

  for (const c of active) {
    params.set(c.field, c.value.trim());
    if (TEXT_FIELDS.includes(c.field) && c.operator !== 'contains') {
      params.set(`${c.field}_op`, c.operator);
    }
  }

  return `/api/v1/search?${params.toString()}`;
}
