import {
  BarChart3,
  CalendarClock,
  CalendarDays,
  ClipboardList,
  FileText,
  Hash,
  Images,
  ListChecks,
  type LucideIcon,
  Megaphone,
  MessagesSquare,
  MonitorPlay,
  UserPlus,
  Volume2,
} from 'lucide-react';
import type { ChannelType } from '../types';

interface ChannelMeta {
  Icon: LucideIcon;
  label: string;
}

export const channelMeta: Record<ChannelType, ChannelMeta> = {
  text: { Icon: Hash, label: 'Text' },
  announcement: { Icon: Megaphone, label: 'Announcement' },
  voice: { Icon: Volume2, label: 'Voice' },
  stream: { Icon: MonitorPlay, label: 'Stream' },
  forum: { Icon: MessagesSquare, label: 'Forum' },
  calendar: { Icon: CalendarDays, label: 'Calendar' },
  events: { Icon: CalendarClock, label: 'Events' },
  tasks: { Icon: ListChecks, label: 'Tasks' },
  docs: { Icon: FileText, label: 'Docs' },
  media: { Icon: Images, label: 'Media' },
  poll: { Icon: BarChart3, label: 'Polls' },
  applications: { Icon: ClipboardList, label: 'Applications' },
  recruitment: { Icon: UserPlus, label: 'Recruitment' },
};
