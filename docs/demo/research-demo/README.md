# Research demo (deck animation)

An animated illustration of the Research workspace for one 16:9 slide (960 × 540, about 12 s,
plays once). It is a product illustration made with the Stageshow skill, **not** a recording of
the running app: for the live demo and the backup video, record the real app (T5).

Story: a question is typed and sent, the answer streams in, its claims arrive labelled for what
they are (source reports, observed, dashed model prediction), the cursor opens the source, and
the source page opens beside the claim with the cited region marked.

| File | What |
|---|---|
| `poster-light.png`, `poster-dark.png` | the final frame (2×), for a static slide |
| `index.tsx`, `ignis.css` | the scene |
| `REHEARSAL.md` | how it was filmed and checked, and what was not checked |

Honesty: no number, quote or finding is invented. The answer only says what the product does;
the source page is filler; its title names a real NASA PSI-69 document (the FLEX test plan).

## Play or record it

The scene imports the Stageshow kit (`../../kit`), which is not in this repository. It runs in
the Stageshow playground:

1. Install the skill once: `npx skills add benjitaylor/skills --skill stageshow`.
2. Copy this folder to `<skill>/examples/ignis-research/`.
3. In the skill folder, run `npm install`, then `npm run dev`.
4. Open `http://127.0.0.1:5173/?capture=ignis-research` and record the window for the deck.
