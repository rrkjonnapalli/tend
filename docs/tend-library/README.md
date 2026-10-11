# Tend Library — design handoff

- `SPEC.md` — the build spec: behaviour, edge cases and the data model.
- `designs/` — the reference screens, as source.

## designs/

| File | What it is |
| --- | --- |
| `library.dc.html` | The Library tab: bar (switch, +, search), Home, and the three areas mounted inside |
| `words.dc.html` | Words: list + entry page, add/edit, dictionary, tags |
| `snaps.dc.html` | Snaps: pinboard, stacks, editor paper, drag and drop |
| `references.dc.html` | References: list + link-or-text popup |
| `tend.css` | Shared Tend tokens and shell classes (nav, explorer, pane, buttons, chips) |
| `words.css`, `snaps.css`, `refs.css`, `library.css` | Each screen's styles, scoped to its root class (`.w-app`, `.s-app`, `.r-app`, `.lib-app`) |

The `.dc.html` files are Design Component source from the design canvas: plain HTML markup with `{{ }}` bindings, `<sc-for>` / `<sc-if>` blocks, and a `<script type="text/x-dc">` holding each screen's sample data and interaction logic (`class Component`). They need the canvas runtime (`support.js`) to run, so read them as a reference rather than opening them in a browser. To click through them, use the canvas.

What to take from them:

- **Markup and CSS** for layout, sizes, colours and fonts (Lato, Montserrat, JetBrains Mono, Caveat).
- **The logic in each `<script>`** for exact behaviours: the add/edit flows, link detection and duplicate checks, the dictionary and title-fetch stand-ins, the stack moves, pointer-based drag with the lifted copy, the tag chip input, and how Library hands an add to each area (`addRequest`, `embedded`, `query` props).

Sample data and the faked fetches (dictionary, page titles) live in each script; replace them with real calls.
