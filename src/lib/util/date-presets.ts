// The ranges offered beside the calendar in range mode.
import type { DatePreset } from '#lib/types/date.type.ts';
import { addDays, monthBounds, mondayIndex, parseISO } from './dates.ts';

const WEEK_LAST_INDEX = 6;

export function rangePresets(today: string): DatePreset[] {
  const tomorrow = addDays(today, 1);
  const weekday = mondayIndex(parseISO(today));
  const weekStart = addDays(today, -weekday);
  const weekEnd = addDays(today, WEEK_LAST_INDEX - weekday);
  return [
    { name: 'Today', range: [today, today] },
    { name: 'Tomorrow', range: [tomorrow, tomorrow] },
    { name: 'This week', range: [weekStart, weekEnd] },
    { name: 'Next 7 days', range: [today, addDays(today, WEEK_LAST_INDEX)] },
    { name: 'This month', range: monthBounds(today) },
  ];
}
