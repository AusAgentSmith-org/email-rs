import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Account, Folder, Message, Screen } from '../types';

interface AppStore {
  // Theme
  theme: 'light' | 'dark';
  setTheme: (t: 'light' | 'dark') => void;

  // Auth
  accounts: Account[];
  setAccounts: (a: Account[]) => void;

  // Navigation
  screen: Screen;
  setScreen: (s: Screen) => void;

  // Folder selection
  folders: Folder[];
  setFolders: (f: Folder[]) => void;
  selectedFolderId: string | null;
  setSelectedFolder: (id: string | null) => void;

  // Message selection (opens detail)
  selectedMessage: Message | null;
  setSelectedMessage: (m: Message | null) => void;

  // Compose
  composeOpen: boolean;
  openCompose: () => void;
  closeCompose: () => void;

  // Search
  searchQuery: string;
  setSearchQuery: (q: string) => void;
}

export const useAppStore = create<AppStore>()(
  persist(
    (set) => ({
      theme: 'light',
      setTheme: (theme) => set({ theme }),

      accounts: [],
      setAccounts: (accounts) => set({ accounts }),

      screen: 'inbox',
      setScreen: (screen) => set({ screen }),

      folders: [],
      setFolders: (folders) => set({ folders }),
      selectedFolderId: null,
      setSelectedFolder: (selectedFolderId) => set({ selectedFolderId }),

      selectedMessage: null,
      setSelectedMessage: (selectedMessage) => set({ selectedMessage }),

      composeOpen: false,
      openCompose: () => set({ composeOpen: true }),
      closeCompose: () => set({ composeOpen: false }),

      searchQuery: '',
      setSearchQuery: (searchQuery) => set({ searchQuery }),
    }),
    {
      name: 'rsmail-store',
      partialize: (s) => ({ theme: s.theme, selectedFolderId: s.selectedFolderId }),
    },
  ),
);
