# Tend: Calendar design spec

> Status: **draft for review**. Nothing here is built yet. Read it top to bottom: each section only uses ideas from the sections above it.

---

## 0. The idea in one paragraph

Everything that happens at a time lives in the **Calendar** as an **event**. Some events are a single moment ("call Ravi at 5 PM"); some take time ("standup 10:00–10:15"). An event can **repeat**. An event can have **alerts**, which are the notifications that tell you about it. A small **engine** runs in the background, works out which alerts are due, and rings them. There is no separate Reminders section any more: a "reminder" is just an event whose alert rings and waits for you to mark it done. **Timers** are not events; they are short countdowns shown next to Settings while they run.

---

## 1. Words we use

| Word | Meaning | Example |
|---|---|---|
| **Event** | Something on the calendar | "Pay rent", "Standup", "Ravi's birthday" |
| **Instant** | An event at one moment, with no end | "Call Ravi · 5:00 PM" |
| **Duration** | An event with a start and an end | "Standup · 10:00–10:15" |
| **All-day** | An event for a whole day, with no time | "Ravi's birthday · 9 Oct" |
| **Repeat rule** | How an event comes back | "every month on the 10th" |
| **Occurrence** | One date of a repeating event | "Pay rent on 10 Nov 2026" |
| **Alert** | When to notify you about an occurrence | "10 min before", "day before at 11:58 PM" |
| **Needs done** | The alert keeps ringing until you tick it | "Pay rent" must be ticked |
| **Engine** | Background part that rings alerts | not visible |
| **Notification** | One ring: bell entry, desktop pop-up, sound | "Pay rent · now" |
| **Timer** | A short countdown, not on the calendar | "Tea · 4:32 left" |

---

## 2. Events

### 2.1 Two types

| | Instant | Duration |
|---|---|---|
| Time | one moment (or all-day) | start → end (or all-day span) |
| On the week grid | a thin marker line with a checkbox | a block, as today |
| Typical use | reminders, bills, calls, birthdays, task due dates | meetings, appointments |
| Default alert | at the time | 10 minutes before |
| Needs done (default) | **on** | off |
| Join button | — | when it has a link |

You can switch a type on the event page. Switching from Duration to Instant just drops the end time.

### 2.2 What every event has

| Field | Notes |
|---|---|
| Title | required |
| When | date + time, or date + *all-day*; duration events also have an end |
| Repeat | none, or a repeat rule (section 3) |
| Alerts | zero or more (section 4) |
| Needs done | yes / no |
| Link | optional URL; Join button on duration events |
| For | optional: one person, one task |
| Details | markdown |
| Source | where it came from (section 2.3) |

### 2.3 Where events come from

| Source | Created by | Editable? |
|---|---|---|
| `local` | you | yes |
| `ics` | a subscribed calendar feed | yes. Your edits are kept locally and survive the next sync. |
| `birthday` | a person with a date of birth and *birthday reminder = yes* | title, alerts and details yes; date follows the person's DOB |
| `task_due` | a task with a due date | follows the task. Edit the due date on the task. |

`birthday` and `task_due` events are created and updated automatically. If you delete the person, remove the DOB or clear the due date, the event goes away.

---

## 3. Repeat rules: *when does it come back?*

### 3.1 What a rule is

A rule describes a **pattern**. It never stores a list of dates. The **first date and the time come from the event**; the rule only says how to step forward from there.

```
Event "Pay rent"   starts 2026-10-10 09:00
Rule               every 1 month, on day 10
→ dates            10 Oct, 10 Nov, 10 Dec, 10 Jan 2027 …
```

### 3.2 Choosing a rule (UI)

The Repeat dropdown on the event page:

```
Does not repeat
Daily                 at 9 AM
Weekly                on Saturday
Monthly               on the 10th
Yearly                on 10 Oct
──────────
Custom…
```

The presets copy the event's own date and time. **Custom…** opens a panel:

- **Repeat every** [n] [days | weeks | months | years]
- **weeks:** on M T W T F S S (one or more)
- **months:** either
  - on day [1–31 | last], or
  - on the [1st 2nd 3rd 4th last] (one or more) [Mon … Sun | day | weekday] (one or more)
- **years:** on [month] [day]
- **if the date doesn't exist** (31st, 29 Feb): skip / closest day
- **Ends:** never | on [date] | after [n] times
- A summary line plus the **next 4 dates**, so you can see what you'll get.

### 3.3 Examples

| You want | Rule |
|---|---|
| Every weekday | weekly · M T W T F |
| Every 2 weeks on Friday | every 2 weeks · F |
| 1st of every month | monthly · day 1 |
| Last day of every month | monthly · day *last* |
| Every month on the 10th | monthly · day 10 |
| 2nd Saturday and Sunday | monthly · the 2nd · Sat Sun |
| 1st and 3rd Monday | monthly · the 1st 3rd · Mon |
| Last weekday of the month | monthly · the last · weekday |
| Exactly on 29 Feb | yearly · Feb 29 · *skip* |
| Birthday on 29 Feb | yearly · Feb 29 · *closest* (→ 28 Feb) |

### 3.4 Rules the calculation follows

1. **Times are local wall-clock.** 9:00 stays 9:00 across daylight-saving changes.
2. **Dates come from the rule, not from when you pressed Done.** Paying rent late on the 12th doesn't move next month's date.
3. **No backlog.** If you were away for a month, you get the next future date, not a flood of missed ones.

---

## 4. Alerts: *when do you hear about it?*

### 4.1 What an alert is

An alert is a time **relative to each occurrence**. You set it once and it applies to every repeat.

There are two kinds, because timed and all-day events need different wording:

| Kind | For | Written as | Example |
|---|---|---|---|
| **Before** | timed events | N minutes before | `0` = at the time, `10` = 10 min before, `1440` = 1 day before |
| **On day** | all-day events | day offset + clock time | `-1 at 23:58` = day before at 11:58 PM; `0 at 09:00` = that day at 9 AM |

An event can have **several alerts**, for example "1 day before" and "at the time".

### 4.2 Defaults

| Event | Default alerts |
|---|---|
| Instant (timed) | at the time |
| Duration | 10 min before |
| All-day | same day at 9:00 AM |
| Birthday | from Settings → People → *Birthday alerts* |
| Task due | from Settings → Tasks → *Due alerts* (for example, at 9:00 AM on the due date) |

You can change them per event on its page.

### 4.3 Needs done, re-firing and snooze

- **Needs done = off:** each alert rings **once**. That's it.
- **Needs done = on:** after the last alert of an occurrence rings, it **rings again every 5 min, up to 3 rings in total**, until you tick Done. After that it shows as **missed** in the Agenda until you tick it. (5 and 3 are settings.)
- **+10m (snooze):** the next ring for *that occurrence* happens 10 minutes later.
- **Done:** stops all rings for *that occurrence*. Future occurrences are not affected.

---

## 5. Occurrences: *what happened on one date*

### 5.1 Why we need them

A repeating event has endless dates, but we never store them. We only store something for a date when **that one date is different** from the rule:

- you ticked it **done**
- you **snoozed** it
- it **rang** (we count rings for re-firing)
- you **moved** just that date ("pay rent on the 12th this month")
- you **skipped** just that date ("no standup on Diwali")

If a date has no record, it follows the rule and nothing has happened yet.

### 5.2 Example

"Pay rent", monthly on the 10th at 9:00:

| Date | Record | Meaning |
|---|---|---|
| 10 Sep | done 09:12 · rang 1 | paid on time |
| 10 Oct | — | upcoming, normal |
| 10 Nov | moved to 12 Nov 09:00 | just this month, on the 12th |
| 10 Dec | — | upcoming, normal |

### 5.3 Editing a repeating event

When you change a repeating event, Tend asks:

| Choice | What it does |
|---|---|
| **Only this date** | writes an occurrence record (moved / skipped / different time) |
| **This and following** | ends the old rule the day before; creates a new event with the new details from that date on |
| **All** | changes the event and its rule |

Ticking Done and snoozing always apply to **only this date**, so Tend doesn't ask.

---

## 6. The engine

### 6.1 What it does, step by step

It runs **every minute**, and again right after anything changes:

1. **Expand:** for every event, list its dates in the next 60 days (one date if it doesn't repeat).
2. **Adjust:** apply occurrence records. Drop skipped dates, use moved dates, ignore dates already done.
3. **Schedule:** for each date, turn each alert into a clock time (or use the snooze time).
4. **Ring:** for anything due now, create a **notification** (bell + desktop pop-up + sound) and add 1 to that date's ring count.
5. **Re-fire:** for needs-done dates not yet done, ring again 5 min after the last ring, until 3 rings.

### 6.2 What a ring looks like

- **Bell** (next to Settings): a red count badge that blinks until you open it. Each entry has **+10m** and **Done ✓**.
- **Desktop notification** with the same two actions.
- **Sound**, if enabled in Settings.

### 6.3 If the app was closed

On start-up the engine looks back over the time the app was closed. Missed rings show **once** in the bell, marked "missed". Needs-done items show as missed in the Agenda. Nothing re-fires in a burst.

---

## 7. Birthdays, end to end

**Setup (once):** Settings → People → *Birthday alerts* = `day before at 11:58 PM` and `same day at 9:00 AM`. Needs done = on.

**Person:** Ravi Kumar · DOB 1995-10-09 · Birthday reminder = Yes.

**Tend creates:** event "Ravi's birthday" · all-day · instant · yearly on 9 Oct · source `birthday` · for Ravi · alerts copied from the setting.

**What happens in 2026:**

| When | You see |
|---|---|
| 8 Oct, 11:58 PM | 🔔 *Ravi's birthday tomorrow · turns 31* |
| 9 Oct, 9:00 AM | 🔔 *Ravi's birthday today · turns 31* · [+10m] [Done] |
| 9:05, 9:10 | rings again if not done (3 rings total) |
| You tick Done at 9:40 | stops. Recorded for 9 Oct 2026 only. |
| 2027 | both alerts fire again, as normal |

**Changes:**
- Change Ravi's DOB → the event's date moves.
- Set *Birthday reminder* to No, or remove the DOB → the event is removed.
- Change alerts for Ravi only → edit his birthday event. Other birthdays keep the default.
- No birth year → it still works; the text just leaves out "turns 31".

---

## 8. Task due dates, end to end

- A task with a due date gets an **instant** event with source `task_due` on that date (and at its due time, if it has one).
- **Alerts** come from Settings → Tasks → *Due alerts*.
- **Done** comes from the task. Setting the task's status to *Done* marks the event done; you can also tick it in the calendar, which sets the task to Done.
- Changing or clearing the due date on the task updates or removes the event.

---

## 9. Timers (not on the calendar)

| | |
|---|---|
| Start | ⌘T → `mm:ss` (minutes any length, seconds 0–59) + optional text |
| While running | a chip **next to Settings** in the bottom bar: `⏱ 4:32`, plus `+2` if more are running. Hidden when no timers run. |
| Click the chip | popover listing every timer: title, live countdown, **+1m**, **cancel** |
| At zero | rings through the engine (bell + pop-up + sound) once; snooze is allowed; then it's gone |
| Stored | its own small `timers` table; never shown on the calendar |

---

## 10. Screens

### 10.1 Bottom nav

**Notes · Tasks · Calendar · People**. On the right: *(timer chip, only while running)* · Settings · Bell · Feedback.

### 10.2 Calendar views

**Day · Week · Month · Agenda**

- **Week / Day grid:** duration events are blocks. Instants are a thin line at their time, with a checkbox and the title. All-day events sit in a strip at the top of each day.
- **Month:** up to 3 items per day, then "+N more". Instants show their checkbox.
- **Agenda** (replaces the old Reminders list):
  - Groups: **Missed · Now · Today · Next 7 days · Later · Done (today)**.
  - Rows: checkbox (needs-done items) · time · title · link chip · person / task chip.
  - On hover: **+10m** · open · delete.
  - **Done** has undo.
  - The quick-add line at the top creates an instant: `[date] [time] remind me to…`.

### 10.3 Event page

Opened from any event, in any view. It has the same layout as a task page:

```
← Calendar / Pay rent                                     [Mark done] [🗑]

Pay rent                                                   (editable title)

Type        (•) Instant  ( ) Duration          [ ] All day
When        2026-10-10  9:00 AM                (Duration: start → end)
Repeat ↻    Monthly on the 10th ▾              → summary + next 4 dates
Alerts 🔔   at the time  ✕   · + add alert
Needs done  Yes / No
Link 🔗     https://…
For         [person Ravi ✕]  [+ task]

▾ Details   (markdown, pencil → editor)
▾ History   past dates: done / missed / moved / skipped
```

### 10.4 Quick create (unchanged)

- **⌘R** new instant (date + time + text).
- **⌘T** new timer.
- **⌘K** palette: `remind me at 5pm to …`, `event fri 3pm …`, `timer 10m …`.

---

## 11. Data model (summary)

Five tables. Times are INTEGER ms (UTC); local dates and times are text (`YYYY-MM-DD`, `HH:MM`).

```
repeat_rules        id · freq · interval · weekdays · month_day · month_nth · nth_of · month
                    · if_missing · ends_on · ends_after

events              id · kind (instant|duration) · all_day · title · details · link_url
                    · start_date · start_time · end_date · end_time      (local, wall clock)
                    · repeat_rule_id → repeat_rules · needs_done
                    · person_id · task_id · source (local|ics|birthday|task_due)
                    · source_ref (ics UID) · created_at · updated_at · deleted_at

event_alerts        id · event_id → events
                    · minutes_before        (timed events)        ─┐ exactly one
                    · day_offset + at_time  (all-day events)      ─┘

event_occurrences   event_id + occurs_on (the date the rule produced)    ← only when that date differs
                    · done_at · fire_count · last_fired_at · snoozed_until
                    · moved_to_date · moved_to_time · skipped

notifications       id · event_id + occurs_on  |  timer_id · title · fired_at · seen_at · dismissed_at

timers              id · title · ends_at · created_at · state (running|rung|cancelled)
```

Removed compared to the current schema: `reminders`, `reminder_series` (→ `repeat_rules`) and the separate `calendar_sources`-only event table (merged into `events.source`).

---

## 12. Open questions for you

1. **Instant on the week grid:** is a thin line + checkbox + title enough, or should it be a small chip?
2. **Re-fire defaults:** 5 min × 3 rings. Same for everything, or birthdays only once?
3. **ICS events:** can you mark them *needs done*? (Proposed: no; they're meetings.)
4. **Task due:** should ticking the event in the calendar set the task to Done? (Proposed: yes.)
5. **Agenda quick-add:** create instants only, or allow `10:00–11:00` to make a duration event?
