import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Account, ConditionGroup, Folder, Label, Message } from '../types';

type View = 'mail' | 'calendar';

type Theme = 'light' | 'dark';

export interface ComposeState {
  accountId: string;
  to: string;
  subject: string;
  inReplyTo?: string;
  mode: 'compose' | 'reply' | 'forward';
  quotedText?: string;
  quotedFrom?: string;
}

interface AppState {
  theme: Theme;
  densityLevel: number;
  view: View;
  selectedCalendarEventId: string | null;
  selectedFolderId: string | null;
  selectedMessageId: string | null;
  selectedLabelId: string | null;
  folderSelectSeq: number;
  folders: Folder[];
  labels: Label[];
  accounts: Account[];
  compose: ComposeState | null;
  searchQuery: string;
  conditionGroup: ConditionGroup | null;
  settingsOpen: boolean;
  paletteOpen: boolean;
  advancedSearchOpen: boolean;
  messages: Message[];
  selectedMessageIds: string[];
  setMessages: (msgs: Message[]) => void;
  patchMessage: (id: string, patch: Partial<Message>) => void;
  removeMessage: (id: string) => void;
  setView: (v: View) => void;
  setSelectedCalendarEvent: (id: string | null) => void;
  setTheme: (t: Theme) => void;
  setDensity: (d: number) => void;
  setSelectedFolder: (id: string) => void;
  setSelectedLabel: (id: string | null) => void;
  setSelectedMessage: (id: string | null) => void;
  setFolders: (folders: Folder[]) => void;
  setLabels: (labels: Label[]) => void;
  setAccounts: (accounts: Account[]) => void;
  openCompose: (state: ComposeState) => void;
  closeCompose: () => void;
  setSearchQuery: (q: string) => void;
  setConditionGroup: (g: ConditionGroup | null) => void;
  openSettings: () => void;
  closeSettings: () => void;
  openPalette: () => void;
  closePalette: () => void;
  openAdvancedSearch: () => void;
  closeAdvancedSearch: () => void;
  toggleMessageSelection: (id: string) => void;
  selectAllMessages: (ids: string[]) => void;
  clearMessageSelection: () => void;
  navigateToMessage: (folderId: string, messageId: string) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      theme: 'light',
      densityLevel: 4,
      view: 'mail',
      selectedCalendarEventId: null,
      selectedFolderId: null,
      selectedMessageId: null,
      selectedLabelId: null,
      folderSelectSeq: 0,
      folders: [],
      labels: [],
      accounts: [],
      compose: null,
      searchQuery: '',
      conditionGroup: null,
      settingsOpen: false,
      paletteOpen: false,
      advancedSearchOpen: false,
      messages: [],
      selectedMessageIds: [],

      setView: (view) => set({ view }),
      setSelectedCalendarEvent: (selectedCalendarEventId) => set({ selectedCalendarEventId }),
      setTheme: (theme) => set({ theme }),
      setDensity: (densityLevel) => set({ densityLevel }),
      setMessages: (msgs) => set({ messages: msgs }),
      patchMessage: (id, patch) => set((s) => ({ messages: s.messages.map((m) => m.id === id ? { ...m, ...patch } : m) })),
      removeMessage: (id) => set((s) => ({ messages: s.messages.filter((m) => m.id !== id), selectedMessageId: s.selectedMessageId === id ? null : s.selectedMessageId })),
      setSelectedFolder: (id) => set((s) => ({ view: 'mail', selectedFolderId: id, selectedLabelId: null, selectedMessageId: null, searchQuery: '', conditionGroup: null, folderSelectSeq: s.folderSelectSeq + 1, messages: [], selectedMessageIds: [] })),
      setSelectedLabel: (id) => set((s) => ({ selectedLabelId: id, selectedFolderId: id ? `label:${id}` : null, selectedMessageId: null, searchQuery: '', conditionGroup: null, folderSelectSeq: s.folderSelectSeq + 1, messages: [], selectedMessageIds: [] })),
      setSelectedMessage: (id) => set({ selectedMessageId: id }),
      setFolders: (folders) => set({ folders }),
      setLabels: (labels) => set({ labels }),
      setAccounts: (accounts) => set({ accounts }),
      openCompose: (compose) => set({ compose }),
      closeCompose: () => set({ compose: null }),
      setSearchQuery: (searchQuery) => set({ searchQuery, conditionGroup: null, selectedMessageId: null, selectedMessageIds: [] }),
      setConditionGroup: (conditionGroup) => set({ conditionGroup, searchQuery: '', selectedMessageId: null, selectedMessageIds: [] }),
      openSettings: () => set({ settingsOpen: true }),
      closeSettings: () => set({ settingsOpen: false }),
      openPalette: () => set({ paletteOpen: true }),
      closePalette: () => set({ paletteOpen: false }),
      openAdvancedSearch: () => set({ advancedSearchOpen: true }),
      closeAdvancedSearch: () => set({ advancedSearchOpen: false }),
      toggleMessageSelection: (id) =>
        set((s) => ({
          selectedMessageIds: s.selectedMessageIds.includes(id)
            ? s.selectedMessageIds.filter((x) => x !== id)
            : [...s.selectedMessageIds, id],
        })),
      selectAllMessages: (ids) => set({ selectedMessageIds: ids }),
      clearMessageSelection: () => set({ selectedMessageIds: [] }),
      navigateToMessage: (folderId, messageId) => set((s) => ({
        selectedFolderId: folderId,
        selectedMessageId: messageId,
        searchQuery: '',
        conditionGroup: null,
        folderSelectSeq: s.folderSelectSeq + 1,
        messages: [],
        selectedMessageIds: [],
      })),
    }),
    {
      name: 'email-rs-app',
      version: 1,
      migrate: (persistedState: unknown, version: number) => {
        const s = persistedState as Record<string, unknown>;
        if (version === 0 && typeof s.density === 'string') {
          const map: Record<string, number> = { compact: 1, cozy: 4, comfy: 7 };
          s.densityLevel = map[s.density as string] ?? 4;
          delete s.density;
        }
        return s;
      },
      partialize: (state) => ({ theme: state.theme, densityLevel: state.densityLevel }),
    },
  ),
);
