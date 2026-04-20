import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Account, ConditionGroup, Folder, Message } from '../types';

type Theme = 'light' | 'dark';
type Density = 'compact' | 'cozy' | 'comfy';

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
  density: Density;
  selectedFolderId: string | null;
  selectedMessageId: string | null;
  folderSelectSeq: number;
  folders: Folder[];
  accounts: Account[];
  compose: ComposeState | null;
  searchQuery: string;
  conditionGroup: ConditionGroup | null;
  settingsOpen: boolean;
  messages: Message[];
  selectedMessageIds: string[];
  setMessages: (msgs: Message[]) => void;
  patchMessage: (id: string, patch: Partial<Message>) => void;
  removeMessage: (id: string) => void;
  setTheme: (t: Theme) => void;
  setDensity: (d: Density) => void;
  setSelectedFolder: (id: string) => void;
  setSelectedMessage: (id: string | null) => void;
  setFolders: (folders: Folder[]) => void;
  setAccounts: (accounts: Account[]) => void;
  openCompose: (state: ComposeState) => void;
  closeCompose: () => void;
  setSearchQuery: (q: string) => void;
  setConditionGroup: (g: ConditionGroup | null) => void;
  openSettings: () => void;
  closeSettings: () => void;
  toggleMessageSelection: (id: string) => void;
  selectAllMessages: (ids: string[]) => void;
  clearMessageSelection: () => void;
  navigateToMessage: (folderId: string, messageId: string) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      theme: 'light',
      density: 'cozy',
      selectedFolderId: null,
      selectedMessageId: null,
      folderSelectSeq: 0,
      folders: [],
      accounts: [],
      compose: null,
      searchQuery: '',
      conditionGroup: null,
      settingsOpen: false,
      messages: [],
      selectedMessageIds: [],

      setTheme: (theme) => set({ theme }),
      setDensity: (density) => set({ density }),
      setMessages: (msgs) => set({ messages: msgs }),
      patchMessage: (id, patch) => set((s) => ({ messages: s.messages.map((m) => m.id === id ? { ...m, ...patch } : m) })),
      removeMessage: (id) => set((s) => ({ messages: s.messages.filter((m) => m.id !== id), selectedMessageId: s.selectedMessageId === id ? null : s.selectedMessageId })),
      setSelectedFolder: (id) => set((s) => ({ selectedFolderId: id, selectedMessageId: null, searchQuery: '', conditionGroup: null, folderSelectSeq: s.folderSelectSeq + 1, messages: [], selectedMessageIds: [] })),
      setSelectedMessage: (id) => set({ selectedMessageId: id }),
      setFolders: (folders) => set({ folders }),
      setAccounts: (accounts) => set({ accounts }),
      openCompose: (compose) => set({ compose }),
      closeCompose: () => set({ compose: null }),
      setSearchQuery: (searchQuery) => set({ searchQuery, conditionGroup: null, selectedMessageId: null, selectedMessageIds: [] }),
      setConditionGroup: (conditionGroup) => set({ conditionGroup, searchQuery: '', selectedMessageId: null, selectedMessageIds: [] }),
      openSettings: () => set({ settingsOpen: true }),
      closeSettings: () => set({ settingsOpen: false }),
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
      partialize: (state) => ({ theme: state.theme, density: state.density }),
    },
  ),
);
