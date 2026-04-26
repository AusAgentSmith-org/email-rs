import { useEffect } from 'react';
import { useAppStore } from './store';
import { BottomNav } from './components/BottomNav';
import { InboxScreen } from './screens/InboxScreen';
import { ProfileScreen } from './screens/ProfileScreen';
import { MailDetailScreen } from './screens/MailDetailScreen';
import { ComposeScreen } from './screens/ComposeScreen';

export function App() {
  const { screen, theme, selectedMessage, composeOpen } = useAppStore();

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  return (
    <div style={{ height: '100%', position: 'relative', background: 'var(--bg)' }}>
      {/* Active screen */}
      {screen === 'inbox'    && <InboxScreen />}
      {screen === 'profile'  && <ProfileScreen />}
      {screen === 'calendar' && <InboxScreen />}  {/* placeholder */}

      {/* Bottom nav — always visible except when detail/compose is open */}
      {!selectedMessage && !composeOpen && <BottomNav />}

      {/* Overlays */}
      {selectedMessage && <MailDetailScreen />}
      {composeOpen     && <ComposeScreen />}
    </div>
  );
}
