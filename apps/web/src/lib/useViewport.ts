import { useEffect } from 'react';
import { useUI } from '../store';

/* Responsive panel policy for the desktop shell.
   Panels remain user-toggleable, but on narrow widths we retract the
   optional columns so the chat keeps usable width. Runs on mount and
   on resize; only the auto-managed panels are touched. */
export function useViewport() {
  useEffect(() => {
    let lastMembers = true;
    const apply = () => {
      const w = window.innerWidth;
      const s = useUI.getState();
      // member list needs ~1160px to coexist with both left columns
      const wantMembers = w >= 1160;
      if (wantMembers !== lastMembers) {
        lastMembers = wantMembers;
        if (s.membersOpen !== wantMembers) useUI.setState({ membersOpen: wantMembers });
      }
      // collapse the group/channel nav only on very narrow widths
      if (w < 520 && !s.navCollapsed) useUI.setState({ navCollapsed: true });
      if (w >= 680 && s.navCollapsed) useUI.setState({ navCollapsed: false });
    };
    apply();
    window.addEventListener('resize', apply);
    return () => window.removeEventListener('resize', apply);
  }, []);
}
