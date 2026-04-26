const BASE = '/api/v1';

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`);
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json() as Promise<T>;
}

async function patch(path: string, body: unknown): Promise<void> {
  await fetch(`${BASE}${path}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
}

export const api = {
  accounts:        () => get('/accounts'),
  folders:         (accountId: string) => get(`/accounts/${accountId}/folders`),
  messages:        (folderId: string, page = 1) => get(`/folders/${folderId}/messages?page=${page}&per_page=40`),
  message:         (id: string) => get(`/messages/${id}`),
  search:          (q: string) => get(`/search?q=${encodeURIComponent(q)}&limit=40`),
  markRead:        (id: string) => patch(`/messages/${id}`, { is_read: true }),
  flag:            (id: string, val: boolean) => patch(`/messages/${id}`, { is_flagged: val }),
  archive:         (id: string) => patch(`/messages/${id}/archive`, {}),
  trash:           (id: string) => patch(`/messages/${id}/trash`, {}),
  accountSettings: (accountId: string) => get(`/accounts/${accountId}/settings`),
  updateAccount:   (accountId: string, body: { sync_days_limit?: number | null }) => patch(`/accounts/${accountId}`, body),
  updateFolder:    (folderId: string, body: { is_excluded?: boolean }) => patch(`/folders/${folderId}`, body),
};
