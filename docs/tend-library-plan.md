# Tend Library: Words, Snaps, References — build spec

Oct 12, 2026 · Manoj

## Overview

The Library is one bottom-nav tab that holds three areas for things you come across: **Words** (what something means to you), **Snaps** (your own short writing, on a pinboard) and **References** (links and titles worth keeping). Each area keeps its own layout; the Library adds one bar for switching, search and adding.

Principles we settled on:

- Saving is never blocked by detail. Each area needs one field to save; everything else can be added later.
- Write in place. Fields read as plain text on the page; no boxes, borders, or hover/focus backgrounds. A blinking caret is the only edit cue.
- Quiet controls. Icon buttons (a plain +), small chips and segmented switches; no filled call-to-action buttons outside dialogs.
- One tag list across all three areas; several tags per item.
- Undo instead of confirm for deletes and moves.

The reference designs are in `designs/`: `words`, `snaps` and `references`, plus `library`, which mounts the three as they are.

## Library frame

Library is the fourth bottom-nav tab (Notes · Tasks · People · Library). Inside it, one bar sits on top and the chosen area fills the rest.

**Library bar**, left to right:

- A segmented switch: Home · Words · Snaps · References. Library reopens on the last area used.
- A plain + icon button, just left of search.
- Search. Its placeholder names the scope: "Search the library" on Home, "Search snaps" inside Snaps, and so on.

Inside the Library the three areas drop their own bottom nav, header search and +; the bar provides them. Snaps and References also drop their page header (title, counts, tag filter): the Snaps board runs edge to edge between the bar and the nav, and References starts at its column headings.

**Home** shows recent words (word + first line of your meaning), the latest snaps on a small pinboard, and the newest references (title + reference). Each item opens it in its area; "All →" / "Board →" switch area.

**Search scope**

| Where | Searches | Results |
| --- | --- | --- |
| Home | Word titles and meanings, snap text, stack names, reference title, reference and why | One list grouped by area, with counts; a result opens in its area |
| Words | Word titles and meanings | Filters the word list |
| Snaps | Snap text and stack names, across all stacks | A flat board; each result shows "in <stack path>"; dragging is off while searching |
| References | Title, reference and why | Filters the table |

Switching area clears the search; Esc or × clears it too. In-app matching is simple substring for the mockups; the real build uses a text search index.

**The + button**

| Where | What + opens |
| --- | --- |
| Words | The Words add page |
| Snaps | A blank paper (the Snaps editor) in the current stack |
| References | The link-or-text popup |
| Home | A small "what is this?" box (below) |

The Home box only decides where something goes, then hands it over. It guesses as you type: a URL → References ("It's a link."); one line of four words or fewer → Words; anything longer → Snaps. A "Save to: word · snap · reference" switch overrides the guess. Enter (Continue) switches to that area and opens its own add with the text already in: a link starts its title fetch, a word starts its dictionary lookup, longer text lands on a snap paper. Shift+Enter adds a new line.

## Words

Words holds anything you came across and want to understand: a word, idiom, phrase, term or something else. One page serves both adding and editing; only the title is required.

**Layout.** A list on the left grouped by kind (only kinds with entries show), alphabetical within each group. The entry page on the right.

**Entry page, top to bottom**

1. Title: an input styled exactly like the page heading, no border or underline. On a saved entry it renames; empty or a duplicate on blur reverts to the old title.
2. Meta row: kind chip, pronunciation + play (when the dictionary has it), part of speech, then tags.
3. Dictionary (collapsible): senses with examples, synonyms, "dictionaryapi.dev · fetched <date>" and a re-fetch icon.
4. What it is for me (collapsible): **Meaning** and **Notes**, both markdown. Links go in Notes; there is no separate references field.

**Kinds.** Word, idiom, phrase, term / concept, something else. Picked from a small chip that opens a list with a tick on the current kind. Typing guesses word (no spaces) or phrase (spaces); a dictionary hit that says idiom switches to idiom unless you picked a kind yourself. Changing kind on a saved entry moves it to that group.

**Dictionary lookup** runs for word, idiom and phrase only.

- Adding: Enter in the title looks it up; a loading shimmer, then senses, or "Not in the dictionary."
- Term / something else: "No dictionary for this one."
- A title already saved: "Already saved. Open '<title>'" and Save is disabled.

**Meaning and Notes.** Shown formatted; a click on the text (not on a link) opens it as plain markdown in place, no box. Esc, ⌘↵ or clicking away closes it. Supported: bold, code, lists, headings, quotes, links (`[title](url)`, bare URLs, and `title - url` lines). The build should use a real markdown library.

**Adding.** + opens the same page empty with a Save / Cancel row at the bottom (⌘↵ saves, Esc cancels). A "new …" row appears at the top of the list and mirrors the title as you type. Saving adds it to its group and opens it. Saved entries keep changes as you go; a delete icon sits in the top bar.

## Snaps

Snaps are your own short writing, pinned to a board. Stacks group them like folders and can nest. Only the text is required.

**The board.** A warm surface (#f4f0e8) with a faint dot grid that scrolls with the content. In the dark theme the board is dark too: the app's own dark greys for the surface and sheets, light ink, and each sticky's hue muted at a low lightness (the `--sticky-night-*` tokens). Snaps and stacks flow into even columns (column width 250px, gap 52px): every snap shows in full, nothing overlaps, positions are not stored. Each item gets a small fixed tilt by position (±1.8° at most) and a red push-pin. Order: pinned-to-top snaps first, then last edited first; stacks are spread evenly between the rest. Empty states: "Nothing on the board yet", "Nothing in this stack yet", "Nothing matches".

**Paper types**, cycled by the rotate button in the editor:

| Paper | Look | Text |
| --- | --- | --- |
| Scrap | Off-white, torn bottom edge | Mono 14/24 |
| Slip | Kraft (#efe3c8), short strip | Mono 13/20 |
| Lined | Notebook lines, red margin rule | Mono 14/24 |
| Plain card | Clean off-white sheet with soft shadow (the stack's top sheet) | Mono 14/24 |
| Sticky | Tinted paper with a deeper glue band | Handwritten (Caveat) 23/28 |

Sticky colour comes from a picker: a 5 × 4 grid of the app palette plus a hex field. The picked colour sets the hue; the paper is a light tint (lightness 87%, band 79%, saturation capped at 85%) so dark text always reads.

**The editor** is a paper over the dimmed board, 340 × 300 for every type so nothing jumps when you rotate:

- Top left: × cancels (Esc). Top right: pin to top, delete (editing only), ✓ saves (⌘↵).
- Bottom, always in the same place: rotate (paper type), stack picker, sticky colour dot (sticky only), tags.
- The middle is the writing area and scrolls when it overflows.
- The stack picker lists "Snaps (loose)", every stack indented by depth, and "+ New stack". New snaps go to the stack you are in.

Clicking a snap opens it in the same editor. Saving an edit moves it to the front and sets its date to "Edited now"; changing its stack moves it ("Saved and moved to …").

**Stacks** are folders: a stack has a parent stack or none. On the board a stack is a small pile under a binder clip, with its name (handwritten), the first lines of its top snap, and a count ("4 snaps · 1 stack"). Opening one shows its contents with a path ("Snaps / Work rules /"); in the Library the path floats at the board's top left. Hover tools: rename (pencil) and remove (bin). Removing a stack keeps its contents and moves them up one level.

**Moving things**

- Drag a snap or stack onto a stack: it goes in. The target stack opens up (top sheet lifts, sheets fan out).
- Drag a snap onto another snap: both go into a new stack, named inline right after.
- Drag onto a part of the path: it moves up to that level.
- A stack can't go inside itself or its own sub-stacks.
- Dragging is pointer-based: a same-size lifted copy follows the pointer (2.5° tilt, soft shadow, no zoom), the original fades in place, and the board auto-scrolls near its edges. A press that moves under 5px is a click. Without dragging, the editor's stack picker moves a snap too.

**Pin to top.** Hover tool or editor toggle. Pinned snaps lead the board and wear a brass pin instead of the red one.

**Delete and undo.** Delete from the hover bin or the editor. Every delete, move or stack removal shows a toast with Undo for 5 seconds.

**Hover tools on touch.** On devices without hover, snap and stack tools are always visible.

## References

A reference is either a **link** or **text** (a book, film, series, anything named). Paste and press Enter is enough to save; title, why and tags can come later.

**List columns**, newest first:

| Column | Link | Text |
| --- | --- | --- |
| Title | Fetched page title, or your own | Same as the reference unless you changed it |
| Reference | The URL as entered, without `https://` and `www.`, with an open-link icon in front | The text as entered |
| Why I kept it | Your note, blank if none | Your note, blank if none |
| Tags | Tag chips | Tag chips |
| Added | Relative date | Relative date |

Clicking a row opens it for editing; the open-link icon opens the page in a new tab. The standalone page also has a tag filter (dropdown with search and counts); inside the Library the header is dropped.

**The popup** (one form for add and edit), top to bottom:

1. Reference: one input for a link or text.
2. A link · text switch, picked automatically (a URL selects link) and changeable by hand. Once changed by hand it stays.
3. Title. For a link it is filled by the page-title fetch ("Getting the page title…" while it runs), with the site name and an open-link icon at the end of the line; a title you typed is never overwritten. For text it mirrors the reference until you change it. Left empty, it falls back to the fetched title or the reference.
4. Why I kept it: optional, written in place.
5. Tags: chip input with up to six "used before" suggestions from the shared list.

Enter in the reference input saves (⌘↵ from any field). Delete appears only when editing, with Undo afterwards.

**Switching type by hand.** To text: no fetch, title mirrors the reference. To link: fetches the title unless you wrote one.

**Duplicates.** Only links are checked, by normalised URL (scheme, trailing slash and case ignored): "You saved this link on <date>. Open it" and Save is disabled.

**Title fetch** happens server-side after save too, so a reference saved before the fetch finishes still gets its title.

## Shared pieces

These behave the same in all three areas; build them once.

**Tags**

- One tag list shared by Words, Snaps and References; several tags per item.
- Allowed characters: a–z, 0–9 and underscore, up to 24. Input is lowercased and anything else is dropped ("Work Notes!" → "worknotes").
- Chip input: space, comma or Enter turns the text into a chip; Backspace on an empty input removes the last chip; clicking a chip removes it.
- Snaps show the first tag plus a count on the paper ("#poem +1").
- Filters pick one tag from a dropdown with a search box and counts; an item matches if it has that tag among others.

**Writing in place.** Inputs and text areas have no border, outline or background on hover or focus. Titles use the heading font; body fields use the body font. The caret is the only cue.

**Toasts and undo.** Short toast at the bottom centre: "Added to …", "Saved", "Moved to …". Destructive actions (delete, move, remove stack) keep Undo for 5 seconds; nothing asks for confirmation.

**Empty states.** One handwritten line plus one hint, centred: "Nothing on the board yet / Press + to pin your first snap." and equivalents per area and for no search results.

**Touch.** Anything shown on hover (snap and stack tools, the open-link icon) is always visible on devices without hover. Moving snaps on touch uses the editor's stack picker; drag can come later with a long-press.

**Keyboard.** Enter saves single-line adds; ⌘↵ saves from any field; Esc closes the innermost popover first, then the dialog. Every icon-only button has an aria-label; pickers use listbox / option roles.

## Data model

Five records; every one also carries `id`, `created_at` and `updated_at`. Stacks reuse the app's existing folder model, like notes and tasks.

| Record | Field | Type | Notes |
| --- | --- | --- | --- |
| word | title | text, required | Unique, case-insensitive |
| word | kind | enum | word, idiom, phrase, term, other |
| word | dictionary | json, nullable | Senses, examples, synonyms, IPA, part of speech, fetched_at; null when not found or not looked up |
| word | meaning | markdown | |
| word | notes | markdown | Links live here |
| snap | text | text, required | |
| snap | folder_id | id, nullable | The stack; null = loose on the board |
| snap | paper | enum | scrap, slip, lined, plain, sticky |
| snap | color | hex, nullable | Sticky only; the picked colour, tint computed on render |
| snap | pinned | bool | Pinned to top |
| stack (folder) | name | text | "Untitled stack" when left empty |
| stack (folder) | parent_id | id, nullable | Null = top level; no cycles |
| reference | type | enum | link, text |
| reference | value | text, required | The URL or the text as entered |
| reference | title | text, nullable | Fetched or typed for links; defaults to value for text |
| reference | why | text | Optional |
| reference | url_key | text, nullable | Normalised URL for duplicate checks; links only, unique |
| tag | name | text | a–z 0–9 _, max 24, unique |
| item_tag | item_type, item_id, tag_id | ids | Join table for words, snaps and references |

Ordering needs no stored positions: Snaps sort by pinned, then `updated_at`; References by `created_at`; Words alphabetically within kind.

Background jobs: the dictionary lookup (dictionaryapi.dev) and the link title fetch, both retried quietly on failure.

## Out of scope for now

Deliberately left for later; none of these block the first build.

- Phone layouts: Snaps at one or two columns, the Words list as its own screen, References as cards, the Library switch as scrolling tabs.
- Dark mode for the board, papers and stickies.
- Failure messages: title fetch failed, dictionary unreachable, offline.
- Drag on touch (long-press) and drag-to-reorder within a level.
- Wanting-to vs done marks for books and films in References.
- A Library Home that reads live data; the mockup uses sample content.

## Open questions

- [ ] Should a snap or reference link to a Words entry (and the other way round), or stay separate?
- [ ] Does Library open on Home every time, or on the last area used? This spec assumes the last area used.
