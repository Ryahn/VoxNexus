import type { ContextMenuItem } from '../store';

/* Context-menu factories. Kept out of components so menus stay
   declarative and consistent. onSelect handlers are no-ops in the
   mock — wire to the API layer later. */

const noop = () => {};

export function menuFor(kind: 'community' | 'channel' | 'group', label: string): ContextMenuItem[] {
  if (kind === 'community') {
    return [
      { label: 'Community Settings', icon: 'settings', shortcut: '', onSelect: noop },
      { label: 'Invite People', icon: 'user-plus', onSelect: noop },
      { label: 'View As…', icon: 'eye', onSelect: noop },
      { label: 'Create Channel', icon: 'plus', onSelect: noop },
      { label: 'Create Group', icon: 'folder-plus', onSelect: noop },
      { separator: true },
      { label: 'Notification Settings', icon: 'bell', onSelect: noop },
      { label: 'Privacy Settings', icon: 'shield', onSelect: noop },
      { separator: true },
      { label: 'Edit Community Profile', icon: 'edit', onSelect: noop },
      { label: `Leave ${label}`, icon: 'log-out', danger: true, onSelect: noop },
    ];
  }
  if (kind === 'group') {
    return [
      { label: 'Mark Group Read', icon: 'check', onSelect: noop },
      { label: 'Group Settings', icon: 'settings', onSelect: noop },
      { label: 'Edit Permissions', icon: 'shield', onSelect: noop },
      { separator: true },
      { label: 'Create Channel', icon: 'plus', onSelect: noop },
      { label: 'Collapse All', icon: 'chevrons-up', onSelect: noop },
    ];
  }
  // channel
  return [
    { label: 'Mark As Read', icon: 'check', onSelect: noop },
    { label: 'Copy Link', icon: 'link', onSelect: noop },
    { separator: true },
    { label: 'Mute Channel', icon: 'bell-off', shortcut: '', onSelect: noop },
    { label: 'Notification Settings', icon: 'bell', onSelect: noop },
    { separator: true },
    { label: 'Edit Permissions', icon: 'shield', onSelect: noop },
    { label: 'Edit Channel', icon: 'edit', onSelect: noop },
    { label: 'Duplicate Channel', icon: 'copy', onSelect: noop },
    { label: 'Delete Channel', icon: 'trash', danger: true, onSelect: noop },
  ];
}

export function messageMenu(): ContextMenuItem[] {
  return [
    { label: 'Add Reaction', icon: 'smile', onSelect: noop },
    { label: 'Reply', icon: 'reply', onSelect: noop },
    { label: 'Start Thread', icon: 'git-branch', onSelect: noop },
    { label: 'Copy Text', icon: 'copy', shortcut: '', onSelect: noop },
    { separator: true },
    { label: 'Pin Message', icon: 'pin', onSelect: noop },
    { label: 'Mark Unread', icon: 'mail', onSelect: noop },
    { label: 'Copy Message Link', icon: 'link', onSelect: noop },
    { separator: true },
    { label: 'Edit Message', icon: 'edit', onSelect: noop },
    { label: 'Delete Message', icon: 'trash', danger: true, onSelect: noop },
  ];
}

export function memberMenu(name: string): ContextMenuItem[] {
  return [
    { label: `Message ${name}`, icon: 'message', onSelect: noop },
    { label: 'View Profile', icon: 'user', onSelect: noop },
    { label: 'Call', icon: 'phone', onSelect: noop },
    { separator: true },
    { label: 'Add Friend', icon: 'user-plus', onSelect: noop },
    { label: 'Assign Roles', icon: 'shield', onSelect: noop },
    { separator: true },
    { label: 'Timeout', icon: 'clock', onSelect: noop },
    { label: 'Kick', icon: 'log-out', danger: true, onSelect: noop },
    { label: 'Ban', icon: 'ban', danger: true, onSelect: noop },
  ];
}
