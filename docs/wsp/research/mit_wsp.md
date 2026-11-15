# Weight-Space Programming (MIT CSAIL): a technical dossier based on the released artifacts

> Source dossier compiled on 13 Nov 2026 by a research subagent (Claude Opus 5.5) for the IGNIS WSP workstream, from the primary sources it names. Evidence tags are the subagent's. The synthesis the workstream relies on is `docs/wsp/RESEARCH.md`; this file is kept for its citations and excerpts.

Compiled 13 Nov 2026 from:
- Blog: https://rikiyat.github.io/blog/weight-space-programming/ (Takehi, Teoh, Zhang, Khattab; published 2 Nov 2026, "Response to Comments" added 12 Nov 2026). Fetched as raw HTML and converted to text.
- Logs: https://github.com/RikiyaT/lm_understand-logs, cloned (`--depth 1`) into
  `<scratch>/mit-wsp/repo`. It has 8,097 files and takes 359 MB. Licence: CC BY 4.0 for logs, data and networks (`LICENSE`). Nothing from the repo was executed. The only processing was reading text, JSON and `.npy`/`.npz` arrays with numpy (`allow_pickle=False`) to count weights.

Below, `repo/` stands for that clone path.

**Evidence tags used throughout**
- **[A]** author-reported. The claim comes from the blog or from an experiment README written by the authors.
- **[R]** I read it in the artifact itself: the prompt, Astra's answer, code, an eval JSON or the network file.
- **[C]** I computed it from the artifacts: weight counts or percentiles from the `.npz`/`.npy` files, or the logic I derived from the written weights by hand.

**Prompt-injection check.** I grepped all `.md`/`.txt` files for text aimed at an AI reader ("ignore previous", "AI assistant", "Claude", and similar). The only hit was a code comment in `repo/exp6_go_from_scratch/runs/budget_25/turns/t040/answer.md:8`: `# at least two liberties. Ignore all prospective territory and captures.` It is Astra's own Go-design comment, not an instruction to a reader. No injected instructions were found.

**Mismatches between the brief and the repo. Read these first.**
- Exp4 is not a general network repair. It is one specific repair of a Kinetix car-on-ramp PPO actor (`repo/exp4_fix_car`).
- Exp6 has **no convolution layers**. The network is an MLP 243 → 512×6 → 82 with ReLU. Every spatial "kernel" was written as an entry of a dense matrix by Python loops over board neighbours (see §6.4).
- Exp8 edits Qwen3.5‑0.8B, with a Gemma 3 270M follow-up (`repo/exp8_math`). The edits were made with a Python kernel and a budget of forward passes; they were not hand-typed numbers.
- In exp5 Astra was **not** given any trained network weights. It got the architecture, with every weight starting at 0.

---

## 1. What WSP is, per the authors, and the shared harness

- [A] The authors define WSP as "treating a network's weights like a program written in a programming language". The programmer is a general-purpose LLM working from a natural-language spec, without gradients (blog, "Weight-Space Programming"). In the 12 Nov FAQ they say it "is not a method" but "the name of an emerging concept" (blog, "Common Questions").
- [A][R] The authoring model is **"Astra" = OpenAI `gpt-6-astra`**, run through the Codex CLI (`codex exec`, or `codex exec resume <thread>`) (`repo/README.md`). The per-call reasoning effort is recorded in each `run.json`:
  - `high` for exps 1, 2 and 4;
  - `xhigh` for exps 3, 5, 6 and 7;
  - `max` for exp 8.

  [R] Two examples: `repo/exp4_fix_car/runs/oneshot_repair/reply/run.json` holds `"-m","gpt-6-astra"` and `model_reasoning_effort="high"`, and exp6 `turns/t001/run.json` holds `xhigh`. The default model in `repo/harness/codex_config.toml` is `gpt-6-luna`, but every call overrides it.
- [R] Codex's tools are switched off (`repo/harness/codex_config.toml`: `shell_tool=false`, `code_mode=false`, `web_search="disabled"`, `multi_agent=false`, …). The system instructions are a single line: "Answer the user's request." (`repo/harness/instructions.md`).
- [A] All 1,125 turns were audited as tool-free (`audit.tool_free`) (`repo/README.md`). Where code was allowed (exps 3, 6, 7 and 8), Astra wrote **one Python block per turn**. The harness ran it in a bubblewrap sandbox with no network and returned the printed output as the next message (`repo/README.md`, `repo/harness/sandbox.py`).

---

## 2. Experiment 4: fix a failing car policy by editing weights (one shot, no tools)

### 2.1 Task
- [A][R] The task is the Kinetix level `l/h2_car_ramp`. A two-wheel car must back up, then drive left fast enough to jump a red strip and touch the blue box with its green wheel. An episode lasts at most 256 steps, and touching red counts as failure (`repo/exp4_fix_car/README.md`; the scene is in `reply/prompt.txt` lines 5–45).
- [A][R] The PPO policy was trained for 1,001,390,080 steps. It scores 0/256 successes, all of them timeouts (`eval/base_result.json`, `eval/ppo_training_result.json`). The trace in the prompt shows the car stalled at body x≈2.00 from step 32 to step 255 (`reply/prompt.txt` lines 51–68).

### 2.2 Architecture [R] (`reply/prompt.txt`, "THE NETWORK")
- The forward pass:
  - `embed = relu(x @ embed.W + embed.b)` with 128 units;
  - `a_0 = tanh(embed @ actor0.W + actor0.b)`;
  - `a_k = tanh(a_(k-1) @ actor<k>.W + actor<k>.b)` for k = 1..4, 128 units each;
  - `logits = a_4 @ actor_head.W + actor_head.b`, giving 16 logits.
- **Input:** 679 float32 numbers, clipped to [-10, 10]. They are arranged as:
  - polygon slots: 26 numbers each, entries 0..233;
  - circle slots: 18 numbers each, entries 234..305;
  - joints: 54 numbers each, entries 306..629;
  - thrusters: 24 numbers each, entries 630..677;
  - gravity, entry 678.

  The first numbers of a polygon slot are "position x, y; velocity x, y; …". In this scene, polygon slot 0 holds entries 0..25, slot 4 holds 26..51, and slot 5 (the car body) holds 52..77. Entry 52 is therefore the car body's x, entry 54 the body's x-velocity, and entry 234 the green wheel's x (circle slot 12).
- **Output:** 16 logits for 6 categorical action entries. Columns 0..2 are entry 0 with values 0 = motor off, 1 = drive +1, 2 = drive −1. Each entry is sampled from its own softmax. Only entry 0 does anything in this scene.
- [A] The network has 171,664 actor weights. The value head is not shown to Astra.

### 2.3 What Astra was given and denied
- [R] Given, in one message of 577,774 characters (`runs/oneshot_repair/loop.log`):
  - the scene geometry;
  - the rules and the reward;
  - the observation layout;
  - the network formula;
  - **all actor weights as integers with one scale per tensor** (e.g. `embed.W (679 x 128), scale 1e-2: one line per row`; `actor0.W … scale 1e-3`; `embed.b … scale 1e-4`);
  - the base result on 256 episodes;
  - one sampled episode logged every 16 steps.
- [R] The task text, verbatim excerpt (`reply/prompt.txt` line 3):
  > "YOUR TASK: change the network's weights by hand so that the car succeeds as often as possible. First read the weights and explain why the trained network fails … Then keep the parts of the trained network that already work and fix only the part that causes the failure. What matters most is that the car succeeds: do not give up success to change fewer weights. You have no tools: no code execution, no calculator, no simulator, nothing to run. You write numbers. This is a single response; you will get no feedback."
- [A][R] Denied:
  - tools;
  - any feedback;
  - any forward pass;
  - a second try.

  [A] The run had **1 round**. [R] The reply used 10,134 output tokens, 9,146 of them reasoning, and took 412.8 s (`reply/run.json`).

### 2.4 The edit language [R] (`reply/prompt.txt` lines 70–80)
```
EDIT FORMAT (numbers only; nothing is run). Write a code block whose opening line is three backticks followed by the word edits, one statement per line ('#' starts a comment; a long list may continue over the next lines):
actor_head.b[0:3] = [0.5, 2.0, -1.0]   # explicit values along one range (as many values as the range has entries)
actor_head.W[7, 0:3] = 0               # fill: row 7, columns 0..2 (a:b is half-open; ':' alone = the whole axis)
embed.W[26, 5] = 1.25                  # one entry: row 26, column 5
actor0.W[:, 3] *= 0.5                  # scale a block (also += and -=)
embed.b[5] -= 0.1
ZERO actor4.W                          # set a whole tensor to 0
Statements apply in order to the network shown below; a later statement overwrites an earlier one. Only the LAST edits block of a reply is read. A statement that cannot be read is skipped (the others still apply). Values: plain decimal numbers (absolute value at most 1e6).
```
- [A] Edits are applied to the *rounded* network as shown (`README.md`).

### 2.5 The real edit (verbatim, all 51 statements) [R] (`repo/exp4_fix_car/runs/oneshot_repair/reply/answer.md`)
Astra's diagnosis, verbatim excerpt: "embedding unit 116 receives horizontal-velocity weights **2.25, 2.37, and 1.73** … Reverse is effectively never sampled."
```edits
embed.W[:, 6:9] = 0
embed.b[6:9] = [0, 11, 0]
embed.W[52, 6] = 1
embed.W[54, 7] = 1
embed.W[234, 8] = 1

actor0.W[6:9, :] = 0
actor0.W[:, 122:128] = 0
actor0.b[122:128] = [-200, -248, -304, -876, -760, -76]
actor0.W[6, 122] = 80
actor0.W[6, 123] = 80
actor0.W[6, 124] = 80
actor0.W[7, 125] = 80
actor0.W[7, 126] = 80
actor0.W[8, 127] = 80

actor1.W[122:128, :] = 0
actor1.W[:, 124:128] = 0
actor1.b[124:128] = [0, -10, -10, 30]
actor1.W[127, 124] = 10
actor1.W[124, 125] = -10
actor1.W[125, 125] = 10
actor1.W[122, 126] = -10
actor1.W[126, 126] = 10
actor1.W[122, 127] = -10
actor1.W[123, 127] = 10
actor1.W[125, 127] = 10
actor1.W[126, 127] = -10

actor2.W[124:128, :] = 0
actor2.W[:, 126:128] = 0
actor2.b[126:128] = [-10, -30]
actor2.W[124, 126] = 20
actor2.W[125, 126] = 10
actor2.W[126, 126] = 10
actor2.W[124, 127] = 10
actor2.W[125, 127] = -10
actor2.W[126, 127] = -10
actor2.W[127, 127] = 10

actor3.W[126:128, :] = 0
actor3.W[:, 126:128] = 0
actor3.b[126:128] = 0
actor3.W[126, 126] = 10
actor3.W[127, 127] = 10

actor4.W[126:128, :] = 0
actor4.W[:, 126:128] = 0
actor4.b[126:128] = 0
actor4.W[126, 126] = 10
actor4.W[127, 127] = 10

actor_head.W[126, 0:3] = [-100, -100, 200]
actor_head.W[127, 0:3] = [-100, 200, -100]
actor_head.b[0] -= 200
actor_head.b[1] += 100
actor_head.b[2] += 100
```
Astra's own explanation, verbatim excerpt: "Once green's centre passes x = 0.95 … the override switches off. The bias offsets cancel its inactive outputs, restoring the remaining learned policy."

### 2.6 How the function maps onto units [C] (derived by hand from the edit; it matches Astra's explanation)
1. **Isolation.** The circuit takes over 3 embed units (6–8) and a few units per layer. Each one is detached from the trained network first:
   - `embed.W[:,6:9]=0` and `actor0.W[6:9,:]=0` cut the commandeered embed units off;
   - `actor0.W[:,122:128]=0` removes all trained inputs to the actor0 units it takes over;
   - `actor1.W[122:128,:]=0` stops them from leaking into the trained units.

   [A] Most of the 6,073 changed entries are these zeroed rows and columns (`README.md`).
2. **Linear read-out through ReLU with a bias shift.**
   - `embed6 = relu(x[52])` passes body x. Body x is positive in this scene.
   - `embed7 = relu(x[54] + 11)` passes vx. The input is clipped to [−10, 10], so vx + 11 ≥ 1 and the ReLU is always linear.
   - `embed8 = relu(x[234])` passes green-wheel x.
3. **Steep tanh threshold detectors.** The slope is 80, so the transition width is about 1/80 = 0.0125 units. Each bias is −80 × knot:
   - u122 = tanh(80·xb − 200): +1 when xb > 2.5;
   - u123 = tanh(80·xb − 248): +1 when xb > 3.1;
   - u124 = tanh(80·xb − 304): +1 when xb > 3.8;
   - u125 = tanh(80·(vx+11) − 876): +1 when vx > −0.05;
   - u126 = tanh(80·(vx+11) − 760): +1 when vx > −1.5;
   - u127 = tanh(80·xg − 76): +1 when xg > 0.95, i.e. green has not yet passed red.
4. **Boolean logic on ±1 signals, with weights ±10 and the bias setting the gate type** (actor1):
   - v124 = tanh(10·u127) is "override enabled";
   - v125 = tanh(−10·u124 + 10·u125 − 10) is an AND: +1 only when xb < 3.8 and vx > −0.05, i.e. back up from rest;
   - v126 = tanh(−10·u122 + 10·u126 − 10) is an AND: xb < 2.5 and vx > −1.5, i.e. slow near the ramp, which re-triggers the run-up;
   - v127 = tanh(−10·u122 + 10·u123 + 10·u125 − 10·u126 + 30) is a NAND. The +30 bias keeps it +1 unless all four terms are −10.
5. **Second logic layer** (actor2):
   - w126 = tanh(20·v124 + 10·v125 + 10·v126 − 10) means enabled AND (v125 OR v126), i.e. "reverse";
   - w127 = tanh(10·v124 − 10·v125 − 10·v126 + 10·v127 − 30) is a 4-input AND, i.e. "drive left".
6. **Skip/delay through unused depth.** actor3 and actor4 copy each bit with tanh(10·s). Since tanh(±10) ≈ ±1, the ±1 signal survives.
7. **Output with bias cancellation.**
   - When both override units are −1: col0 = +100 + 100 − 200 = 0, col1 = +100 − 200 + 100 = 0, col2 = −200 + 100 + 100 = 0. The **net change is exactly 0** and the learned policy runs unmodified.
   - When "reverse" is +1 (and "drive left" −1): the logits change by (−200, −200, +400), a margin of 600 for value 2.
   - When "drive left" is active: they change by (−200, +400, −200).

   Astra: "Large logit margins make the decisive actions nearly deterministic despite sampling."
8. **Memory without recurrence.** Astra: "Position and velocity distinguish backing from returning without recurrent memory." The phase is inferred from the (x, vx) region, not stored.

### 2.7 Magnitudes [C] (`repo/exp4_fix_car/networks/edited_actor.npz`)
- Trained weights are small. The median |w| is 0.04 for embed.W and about 0.085–0.11 for actor0..4.W, with the 99th percentile at about 0.37–0.45.
- The written path uses **80, 10, 20, 100, 200 and biases up to −876**, i.e. 2–3 orders of magnitude larger. This deliberately saturates tanh, so the path's units carry hard bits.
- [R] `eval/oneshot_result.json` reports `rel_change` of 25.66 overall, and per tensor up to 3,736 (actor0.b).

### 2.8 Result [R] (`eval/oneshot_result.json`, `runs/oneshot_repair/loop.log`)
- 51 of 51 statements were applied with 0 errors, writing 6,318 entries. **6,073 of 171,664 differ (3.54 %)**.
- Search set: 256/256 successes. **Test set: 1,024/1,024 successes**, mean length 73 steps, return 1.374. The base network had 0 %.
- [R] The edited trace starts with action 2 (back up) and moves x from 2.80 to 3.85. Then action 1 drives left, and the run succeeds at step 73 (`oneshot_result.json` → `search_text`).

---

## 3. Experiment 5: write MinAtar policies from scratch (one shot, no tools)

### 3.1 Task, architecture and encoding [R] (`repo/exp5_write_from_scratch/runs/<game>/reply/prompt.txt`, `README.md`)
- The games are Breakout, Seaquest and Asterix from MinAtar 1.0.15. The prompt contains the full game source file.
- The network is `h1 = tanh(x @ W1 + b1)`, `h2 = tanh(h1 @ W2 + b2)`, `logits = h2 @ W3 + b3`, i.e. 600 → 32 → 32 → 6. It has **20,486 parameters, and EVERY WEIGHT STARTS AT 0**.
- **Input encoding:** 600 binary numbers, with index `i = (row*10 + column)*6 + channel`. Rows run 0 (top) to 9, and there are 6 channels: player, danger, reward, projectile, trail, gauge. The prompt hands the model strided ranges explicitly:
  > "all 10 cells of column c in channel ch are rows c*6+ch : 600 : 60 of W1; all 10 cells of row r in channel ch are rows r*60+ch : r*60+60 : 6."
- **Output decoding:** 6 logits for 0 no-op, 1 left, 2 up, 3 right, 4 down, 5 fire. The action is sampled from a softmax, with sticky actions at p = 0.1.

### 3.2 Given, denied and rounds
- [R] Prompt excerpt (Breakout `prompt.txt` line 1):
  > "YOUR TASK: write, from scratch, the weights of a policy network that plays this game as well as possible. There is no training: no data, no loss, no gradients, no fine-tuning; you choose every number yourself. You have no tools … This is a single response and you will get no feedback. First describe your design briefly (the strategy, and what the hidden units compute), then write the edits block."
- [A][R] Denied: data, loss, gradients, tools and any rollout. **One reply per game.** Effort was `xhigh` with reasoning summaries set to `detailed`.
- [R] `reply/events.jsonl` holds the reasoning summaries. They are headings only (e.g. Breakout: "Deriving landing intervals", "Preparing manual fold tables", "Calculating F(x+8)").

### 3.3 Edit syntax [R] (prompt)
```
W1[12, 0] = 2.5                     # one entry: row 12, column 0
W1[0:6, 3] = [1, 0, -1, 0, 2, 0]    # explicit values along a range (as many values as entries)
b1[0:4] = -1                        # fill a range (a:b is half-open; ':' alone = the whole axis)
W1[3:600:60, 0] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1]   # column 0 of channel 3, rows 0..9 of the grid -> unit 0
W2[0:2, 0:3] = [[1, -1, 0.5], [0, 2, 0]]   # a 2-D list for a 2-D block (here 2 x 3)
W3[:, 2] *= 2                       # scale (also += and -=)
```
- [A] The first parser rejected `[ [` (a space after the bracket) in the Seaquest and Asterix replies. The parser was fixed and the *same* replies were re-read. The final reading is 122/122 for Seaquest and 55/55 for Asterix, with 0 errors (`README.md`; `runs/*/run.log` vs `apply.json`).

### 3.4 Breakout: real edits (verbatim excerpt, 40 lines) [R] (`runs/breakout/reply/answer.md`)
Design text (verbatim): "The first layer computes four landing-position errors and two direction indicators. … The second layer gates left/right decisions, choosing no-op when positioned correctly."
```edits
# Units 0..3: descending-right, descending-left, ascending-right, ascending-left.
# Subtract paddle position from each target.
W1[540:600:6, 0] = [0, -0.02, -0.04, -0.06, -0.08, -0.10, -0.12, -0.14, -0.16, -0.18]
W1[540:600:6, 1] = [0, -0.02, -0.04, -0.06, -0.08, -0.10, -0.12, -0.14, -0.16, -0.18]
...
# Ball row 0.
W1[3:60:6, 0] = [0.16, 0.18, 0.18, 0.16, 0.14, 0.12, 0.10, 0.08, 0.06, 0.04]
W1[3:60:6, 1] = [0.14, 0.12, 0.10, 0.08, 0.06, 0.04, 0.02, 0, 0, 0.02]
W1[3:60:6, 2] = [0.18, 0.18, 0.16, 0.14, 0.12, 0.10, 0.08, 0.06, 0.04, 0.02]
W1[3:60:6, 3] = [0.16, 0.14, 0.12, 0.10, 0.08, 0.06, 0.04, 0.02, 0, 0]
...
# Unit 4: horizontal direction, including stationary wall-reflection frames.
W1[3:600:60, 4] = 10
W1[9:600:60, 4] = 20
W1[15:600:60, 4] = 40
...
W1[57:600:60, 4] = 170
W1[10:600:60, 4] = -20
W1[16:600:60, 4] = -40
...
W1[58:600:60, 4] = -180

# Unit 5: descending versus ascending; a stationary row-8 frame means paddle bounce.
b1[5] = 5
W1[63:120:6, 5] = 10
W1[123:180:6, 5] = 20
...
W1[544:600:6, 5] = -90

# Four context-gated pairs of left/right comparisons.
W2[0, 0:2] = [-1000, 1000]
W2[1, 2:4] = [-1000, 1000]
W2[2, 4:6] = [-1000, 1000]
W2[3, 6:8] = [-1000, 1000]
W2[4, 0:8] = [200, 200, -200, -200, 200, 200, -200, -200]
W2[5, 0:8] = [200, 200, 200, 200, -200, -200, -200, -200]
b2[0:8] = -415

# Near-deterministic movement when needed; otherwise no-op.
W3[0:8:2, 1] = 30
W3[1:8:2, 3] = 30
b3[:] = [0, 90, -100, 90, -100, -100]
```
(The `...` marks lines I elided. The full block of 92 statements is in the file.)

**How the construction works** [C]:
- **Lookup table → linear "value code".** Ball cell (row r, column c) in channel 3 gets weight 0.02 × (predicted landing column + offset). The paddle cell in row 9, channel 0, at column p gets −0.02·p. Each of units 0–3 therefore pre-activates to 0.02 × (target − paddle), one unit per direction hypothesis. The landing prediction including wall folds is precomputed per cell, one table per (row, direction) ("Preparing manual fold tables").
- **Linear regime of tanh.** The maximum |pre-activation| is about 0.18, so tanh ≈ identity. The units carry a *graded* signed error.
- **Direction bits.** Unit 4 ≈ sign(ball column − trail column), with weights ±20 per column step (special 10/170 values at the walls). Unit 5 ≈ sign(ball row − trail row), with +5 bias and ±10 per row. These saturate to ±1.
- **Gated comparator.** For W2 unit 1 (move right, "descending-right" context), the pre-activation is 1000·h0 + 200·h4 + 200·h5 − 415.
  - With both gate bits at +1 it is 1000·h0 − 15. The unit fires once target − paddle ≥ 1 column (0.02·1000 = 20 > 15) and stays off at 0.
  - If either gate bit is −1 the bias leaves −415 + 1000·h0. Since |h0| ≤ 0.18, the unit is always off.
  - So **±200 gate weights against a −415 bias implement an AND of two context bits**, and **±1000 amplifies a 0.02-per-column error into a hard decision**.
- **Bias-offset priority.** W3 adds +30 per active "left" unit and +30 per active "right" unit. With b3 = 90 and four inactive units at −1 (−120), "left" scores 90 − 120 = −30 against no-op at 0, so no-op wins. One active unit gives 90 − 90 + 30 = +30, so the move wins. Up, down and fire are suppressed at −100.

### 3.5 Seaquest: real edits (verbatim excerpt, 25 lines) [R] (`runs/seaquest/reply/answer.md`)
```edits
# First layer: player row indicators.
b1[0:9] = -6
W1[0:60:6, 0] = 6
W1[60:120:6, 1] = 6
...
# Diver presence in rows 2 through 8.
b1[9:16] = -4
W1[122:180:6, 9] = 8
W1[121:180:6, 9] = -8
...
# Collision risk near the fixed player column.
b1[16:32] = -3
W1[61:120:6, 16] = [0, 0, 2, 4, 7, 7, 7, 5, 2, 0]
W1[62:120:6, 16] = [0, 0, -1.8, -3.6, -1, 0, -1, -3, -1.8, 0]
...
# Diver attraction for fire candidates.
W2[9:16, 0:8] = [
[0.45, 0.6, 0.45, 0.34, 0.255, 0.19, 0.145, 0.11],
[0.34, 0.45, 0.6, 0.45, 0.34, 0.255, 0.19, 0.145],
...
# Candidate offsets.
b2[0:8] = [-11.96, -11.09, -10.63, -10.395, -10.32, -10.395, -10.63, -11.09]
b2[8:16] = [-15.21, -10.64, -10.18, -9.945, -9.87, -9.945, -10.18, -6]
...
# Action logits.
W3[0:8, 5] = 30
W3[8:15, 2] = 30
W3[15, 2] = 40
W3[16:24, 4] = 30
W3[24, 2] = -12
W3[25, 4] = 4
b3[:] = [-100, -100, 238, -100, 244, 241.8]
```
Tricks visible here [C]:
- **Row indicator as a count threshold.** The two-cell submarine in row r gives 6·2 − 6 = +6, i.e. ≈ +1; an empty row gives −6, i.e. ≈ −1.
- **Set difference by channel subtraction.** The "reward" channel holds fish, subs *and* divers, while "danger" holds fish and subs. Reward × (+8) plus danger × (−8) with bias −4 gives +4 for a diver and −4 for an enemy, i.e. a pure diver detector.
- **Shaped spatial kernel per cell.** The collision-risk weights along a row are [0, 0, 2, 4, 7, 7, 7, 5, 2, 0], with trail-channel corrections in the second line.
- **Distance-decaying attraction tables.** Each step multiplies the weight by about 0.75: 0.6 → 0.45 → 0.34 → 0.255 → 0.19 → 0.145 → 0.11 → 0.08. This forms a banded Toeplitz matrix in W2.
- **Action priority by bias offsets that cancel the inactive candidates.** With all candidates inactive (−1): fire = 241.8 − 8·30 = 1.8; up ≈ 238 − 210 − 40 + 12 = 0; down ≈ 244 − 240 − 4 = 0; the rest are −100. **Fire is therefore the default by a margin of 1.8**, and any single active candidate (+60 swing) takes over. The fractional candidate offsets (−11.96, −11.09, −10.63, …) encode a mild preference by row.

### 3.6 Asterix: real edits (verbatim excerpt, 25 lines) [R] (`runs/asterix/reply/answer.md`)
```edits
# First-layer biases.
b1[:] = -8

# Player-conditioned upward hazards.
W1[90:570:60, 0:8] = [
[8, 0, 0, 0, 0, 0, 0, 0],
[6, 0, 0, 0, 0, 0, 0, 0],
[3, 4, 0, 0, 0, 0, 0, 0],
...
# Trails cancel receding enemies and already-passed treasure.
# Columns within each block: upward hazard, downward hazard, waiting hazard, treasure.
W1[64:120:6, 0:32:8] = [[0,0,0,0],[0,0,0,-8],[0,0,0,-8],[0,0,-3,-16],[-5,-5,-4,-16],[-6,-6,-9,-24],[-5,-5,-4,-16],[0,0,-3,-16],[0,0,0,-8],[0,0,0,-8]]
...
# Treasure plus trail equals 8 before interception, and 0 after passing.
W1[62:120:6, 24] = [8,8,16,16,24,24,24,16,16,8]
...
# Treasure window decoders.
W2[24, 3:6] = [20,20,24]
W2[25, 6:10] = [24,20,20,24]
...
b2[3:32:4] = 8
b2[4:30:4] = -8
b2[5:30:4] = -21.6
b2[6:32:4] = 21.6

# Safety penalties.
W3[0, 2] = -80
W3[1, 4] = -80
W3[2, 0] = -80

# Prefer waiting when aligned with incoming treasure.
W3[3:32:4, 0] = 16
W3[4:30:4, 0] = -16
...
# Baseline corrections; suppress horizontal movement and redundant fire.
b3[:] = [17, -1000, 0.15, -1000, -0.69, -1000]
```
Tricks [C]:
- **Player-position × hazard conjunction.** Player channel 0 at column 5 (rows 1..8, `90:570:60`) feeds hazard units with weight 8 or 3–4, while enemy cells add 3–11. The bias −8 means a unit only fires when the player is in that row *and* enemies are near. This is an AND by sum and threshold.
- **Trail subtraction encodes motion direction.** An object moving toward the player has its trail on the far side; an object moving away has its trail on the near side, where the negative trail weights cancel it.
- **Windows from differences of shifted tanh steps.** Units with biases +8 / −8 and ±21.6 are read out as pairs with +16 / −16 (W3[3:32:4,0] = 16, W3[4:30:4,0] = −16). A difference of two step functions is a box or bump over a range of the treasure code.
- **Hard action masking by bias −1000** for left, right and fire.

### 3.7 Sparsity and magnitudes [R] (`runs/<game>/apply.json`) and [C] (`networks/<game>/hand.npz`)
| Game | Statements | Nonzero / 20,486 | max abs W1 / W2 / W3 | max abs b1 / b2 / b3 | Median nonzero abs weight [C] | Distinct values [C] |
|---|---|---|---|---|---|---|
| Breakout | 92 | 815 (4.0 %) | 180 / 1000 / 30 | 5 / 415 / 100 | 10 | 62 |
| Seaquest | 122 | 803 (3.9 %) | 8 / 10 / 40 | 6 / 15.21 / 244 | 3.6 | 53 |
| Asterix | 55 | 637 (3.1 %) | 24 / 24 / 80 | 8 / 21.6 / 1000 | 7 | 43 |

There are two scale regimes. The *graded* path keeps weights around 0.01–0.2 so that tanh stays linear (Breakout's targets). The *logic* path uses 6–1000 so that tanh saturates to ±1.

### 3.8 Evaluation and numbers [R] (`eval/minatar_eval.json`; procedure in `README.md`)
- **Procedure:** 1,000 episodes of official MinAtar 1.0.15, actions sampled from the softmax, sticky actions 0.1, a cap of 10,000 steps.
- **PPO baselines:** the same 600‑32‑32‑6 tanh actor, CleanRL-style. 100M steps uses seeds 0–9 (Seaquest 0–2) and 500M steps uses seeds 0–2. The post shows the best seed of each. [A] The checkpoints are not released.

| Game | PPO 100M (best seed) | PPO 500M (best seed) | Astra one-shot (mean ± SE) |
|---|---|---|---|
| Breakout | 15.779 (s2) | 25.344 (s0) | **517.411 ± 7.42** (max 800, mean length 7,417 steps) |
| Seaquest | 5.617 (s2) | **88.884** (s2) | 53.11 ± 1.36 |
| Asterix | 25.82 (s6) | **55.204** (s1) | 53.178 ± 1.02 |

- [C] Caveat visible in the data: Breakout's mean episode length of 7,417 under a 10,000-step cap means a large share of episodes were probably cut by the cap. The Breakout score is therefore partly bounded by evaluation length. Per-episode lengths are not in the file.
- [R] PPO varies widely across seeds. For example, Seaquest 500M seeds score 7.49, 4.38 and 88.88, so the "best seed" baseline is dominated by one seed.

---

## 4. Experiment 6: write a 1.48M-weight Go network (Python sandbox with 0/10/25/50 forward "passes")

### 4.1 Task and architecture [R] (`repo/exp6_go_from_scratch/runs/budget_50/message.txt`)
- The game is 9×9 Go (pgx `go_9x9`) with area scoring and komi 7.5.
- **Input:** 243 binary numbers, with `input (row*9+col)*3 + plane`. Plane 0 is the mover's stones, plane 1 the opponent's stones, and plane 2 is 1 everywhere when White is to move.
- **Network:** `h_1 = relu(x @ L1.W + L1.b)` (512 units); `h_k = relu(h_(k-1) @ Lk.W + Lk.b)` for k = 2..6; `logits = h_6 @ out.W + out.b` (82 outputs). It has **1,480,274 weights, all zero at the start**.
- **Output:** 82 logits, 81 board points plus pass. Illegal moves are masked, moves are sampled at temperature 1, and there is no search. **There are no convolution layers.**

### 4.2 What Astra was given and denied [R] (same file)
- Given: a Python sandbox (numpy, scipy, sklearn, PyTorch CPU, pandas, sympy, networkx; no Go library, no internet, no GPU) with up to 120 runs of 900 s each. The helpers `load()`, `save()`, `inp(row,col,plane)`, `mv(row,col)` and `PASS=81` are predefined.
- Prompt excerpt (RULES):
  > "Not allowed: (a) gradient descent or any other training … no fitting of weights to data or labels … (b) search … (c) running the network outside the pass tools: do not compute your network's output or activations on any input in the sandbox … Allowed: writing the weights by code from your own design (formulas, loops, symmetries, linear algebra on the weights themselves), Go-rule code in the sandbox …"
- **The "limited forward-pass feedback"** [R], verbatim from the prompt:
  - A **games pass** "plays up to 128 games of your current network … against practice opponents" ("random", "weak" = a REINFORCE network, "strong" = a PPO seed-1 network, none of which are evaluation opponents). Astra gets "a summary per opponent (score, wins, mean area margin, stones captured and lost, passes, game length, how the games ended) and, in /work/pass_<k>/, every game's moves, your network's top-5 moves with their probabilities at each of its moves, and the board after every move."
  - A **positions pass** "runs your current network on up to 256 positions" and returns the top 3 moves, plus logits, probabilities and optionally every hidden activation in `/work/pass_<k>/`.
  - [R] An example of the returned summary (pass 1, `runs/budget_50/tool_calls.jsonl`; also in `turns/t003/prompt.txt`): `"strong": {"games": 64, "score": 0.578125, … "score_as_black": 0.375, "score_as_white": 0.78125, "mean_margin": 13.09, … "end_repetition": 2}`.
- Budget 0 has **no pass tools at all** (`diff budget_0/message.txt budget_50/message.txt`).
- [A] Rounds (`README.md` runs table, matched in `loop.log`):

  | Budget | Turns | Python runs | Passes used (games / positions) | Wall time |
  |---|---|---|---|---|
  | 0 | 2 | 1 | 0 | 22 min |
  | 10 | 28 | 17 | 10 games | 29 min |
  | 25 | 56 | 30 | 24 games / 1 positions | 57 min |
  | 50 | 90 | 39 | 47 games / 3 positions | 3 h 52 min, about 2 h 15 min of which was a harness pause |

  Effort was `xhigh`. [R] The Codex event streams in these runs hold no reasoning summaries (`README.md`).

### 4.3 Real construction code, budget 0 (verbatim excerpts from the single 336-line run) [R] (`runs/budget_0/tool_calls.jsonl`, field `code`)
Unit layout: `unit(channel, point) = channel*81 + point`, i.e. 6 feature planes × 81 = 486 units, plus global scalars at 486–493. Neighbourhood operators are written as dense matrices by looping over neighbours.
```python
TOTAL, WHITE, OWN_COUNT, OPP_COUNT = 486, 487, 488, 489
PHASE, LATE, AHEAD, BEHIND = 490, 491, 492, 493

# L1: stones, direct liberties on stones, and territorial density penalties.
W, b = params["L1.W"], params["L1.b"]
for p in range(N):
    W[3*p, unit(0, p)] = 1.0
    W[3*p+1, unit(1, p)] = 1.0

    for color, channel in ((0, 2), (1, 3)):
        j = unit(channel, p)
        b[j] = len(neighbors[p]) - 4.0
        W[3*p+color, j] += 4.0
        for q in neighbors[p]:
            W[3*q, j] -= 1.0
            W[3*q+1, j] -= 1.0
...
# L2: certified liberty evidence and strategic move scores.
#
# In a same-color star, an individual liberty contributes at most eight
# to the weighted liberty sum below. Thus sums >8 and >16 certify
# at least two and three DISTINCT liberties, respectively.
...
    for color, source_lib, destination, threshold in (
        (0, 2, 2, 8.0),
        (1, 3, 3, 8.0),
        (0, 2, 4, 16.0),
    ):
        j = unit(destination, p)
        b[j] = -threshold - 64.0
        W[unit(color, p), j] += 64.0
        W[unit(source_lib, p), j] += 8.0
...
W[TOTAL, PHASE], b[PHASE] = 1.0, -18.0
W[TOTAL, LATE], b[LATE] = 1.0, -52.0
W[OWN_COUNT, AHEAD] = 1.0
W[OPP_COUNT, AHEAD] = -1.0
W[WHITE, AHEAD] = 7.5
b[AHEAD] = -1.0
W[OWN_COUNT, BEHIND] = -1.0
W[OPP_COUNT, BEHIND] = 1.0
W[WHITE, BEHIND] = -7.5
b[BEHIND] = 1.0

# L3: propagate certified liberty evidence through same-color stones.
# A pair (z,g) encodes stone=z-g exactly and nonnegative evidence=g.
W, b = params["L3.W"], params["L3.b"]
M = 481.0
for p in range(N):
    neighborhood = [p] + neighbors[p]
    for stone_channel, evidence_channel, z_channel, g_channel in (
        (0, 2, 0, 1),
        (1, 3, 2, 3),
    ):
        for destination, subtraction in ((z_channel, 0.0), (g_channel, 1.0)):
            j = unit(destination, p)
            W[unit(stone_channel, p), j] += M + 1.0
            for q in neighborhood:
                W[unit(evidence_channel, q), j] += 1.0
            b[j] = -M - subtraction
...
# Output policy.
W, b = params["out.W"], params["out.b"]
for p in range(N):
    b[p] = -OFFSET
    for channel, coefficient in (
        (0, 4.6),
        (1, 7.1),
        (2, 1.0),
        (3, -64.0),
        (4, -9.0),
        (5, -1.6),
    ):
        W[unit(channel, p), p] = coefficient
```
Kernel table and boundary renormalisation (same file):
```python
# Density, friendly shape, and opponent approach kernels.
kernels = {
    (0, 1): (1.000, 0.550, -0.650),
    (1, 1): (0.750, 0.700,  0.100),
    (0, 2): (0.700, 0.750,  0.250),
    ...
    (2, 4): (0.015, 0.000,  0.000),
}
...
        d, f, e = kernels.get(
            tuple(sorted((abs(r - s), abs(c - t)))),
            (0.0, 0.0, 0.0),
        )
...
    factor = min(
        1.65,
        np.sqrt(full_density_mass / float(density[p].sum()))
    )
    density[p] *= factor
```
[R] The run's printed output: `Parameters: 1480274`, `Nonzero parameters: 34190` (`runs/budget_0/state.json`, field `next`).

### 4.4 How the Go functions map onto units [C] (derived from the code above)
- **Liberty count gated by presence, using only bias and ReLU.** The liberty unit computes relu(4·s_p + (deg − 4) − Σ_{q∈N(p)} occupied_q). With a stone (s = 1) this equals deg − occupied = the number of empty neighbours. Without a stone it is ≤ 0, i.e. 0. **A count is multiplied by a binary flag without any multiplication.** The flag carries weight 4 = max(deg), and the bias offset cancels it.
- **Thresholds that certify a count.** relu(64·s + Σ weighted evidence − threshold − 64) with threshold 8 or 16. The orientation weights 3/4/5 are chosen so that one liberty contributes ≤ 8. A sum above 8 or above 16 therefore proves at least 2 or 3 *distinct* liberties.
- **Big-M gating plus a flag/value pair.** In L3, z = relu((M+1)·s + e − M) and g = relu((M+1)·s + e − M − 1). With M = 481 > the maximum evidence: s = 1 gives z = e + 1 and g = e, so z − g = 1 recovers the stone flag exactly; s = 0 gives z = g = 0. L4 repeats this with M = 2401 because evidence grows after propagation. **A binary flag and a non-negative magnitude travel together through ReLU layers and are recovered linearly.**
- **Multi-step propagation = depth.** L3 and L4 each spread "liberty evidence" one step along same-colour neighbours: `for q in neighborhood: W[unit(evidence_channel, q), j] += 1.0`. Each layer is one diffusion or relaxation step restricted to a mask.
- **ReLU hinge ramps (piecewise-linear in a global count).** PHASE = relu(TOTAL − 18) and LATE = relu(TOTAL − 52) are knots at 18 and 52 stones. They feed the pass logit with slopes 0.050 and 0.045.
- **ReLU pair for a signed quantity.** AHEAD = relu(own − opp + 7.5·white − 1) and BEHIND = relu(−own + opp − 7.5·white + 1) are the positive and negative parts of the komi-adjusted margin, with a small offset. They have separate output slopes (+0.025 and −0.035).
- **Offset trick to carry a signed value through ReLU.** The L2 strategic score has bias `OFFSET + positional` with OFFSET = 128. It is copied through the next layers and finally cancelled by `out.b[p] = -OFFSET`.
- **Identity skip paths.** `copy_globals`: `W[g,g] = 1` at every layer for the global scalars (which are non-negative, so ReLU is the identity on them). Budget 50 also carries the board planes (`carry_board(k)`).
- **Neighbourhood "convolutions" written as dense matrices.**
  - The kernel coefficients are keyed by the sorted pair (|dr|, |dc|), which makes them isotropic and symmetric.
  - Three kernels are written at once: density, friendly shape and enemy shape.
  - Near the board edge, a row is rescaled by min(1.65, sqrt(full_mass / row_mass)), so that **truncated stencils are renormalised**.
- **Masking by a large output weight.** The "true eye" channel goes to the output with −64, which effectively forbids filling one's own eyes.
- **Ternary packing (budget 50).** `final_policy_design.py`: "At layer 4, encode own/empty/enemy as 1, 1/2, 0. This frees a full channel". The code is `B[3][u(0,p)] = 0.5`, `W[3][u(0,p),u(0,p)] = 0.5`, `W[3][u(1,p),u(0,p)] = -0.5`. It is decoded later with `(ch, sign, bias) in [(0,2,-1),(1,-2,1)]`.

### 4.5 With passes (budget 50): what changed [R]
- The design moved into a parametrised builder `build(save, overrides)` with a coefficient dict (`runs/budget_50/work/final_policy_design.py`). Passes were used to adjust those coefficients and to swap in structural blocks. The successive files `policy_design_v2.py`…`v25.py` are in `work/`.

  | Coefficient | DEFAULTS | `final_policy_config.json` |
  |---|---|---|
  | rescue | 6 | 8 |
  | contact | 1.4 | 3.0 |
  | temperature_scale | 1.8 | 2.5 |
  | repeat_risk | none | 80 |
  | pass_white | 0.35 | 3.35 |

- The output temperature is a global knob, applied as `OW *= cfg["temperature_scale"]; OB *= cfg["temperature_scale"]`.
- **Static verification without a forward pass.** Turn 75 (`turns/t075/answer.md`) permutes every weight matrix under the 8 board symmetries and checks invariance (`Exact symmetry on all valid broadcast-colour inputs`).
- **Validation over repeated passes.** `work/final_validation_manifest.json` lists validation passes 28–49 on the frozen network (sha256 `dc92ab2b…`), accumulating 2,000 games against "strong" with a win rate of 0.8335 and a Wilson 95 % CI of [0.817, 0.849].
- [A] The blog says: "it plays practice games, looks at how it loses, changes the part of the network behind it, and keeps a change only if it wins more."

### 4.6 Sparsity and magnitudes [C] (`networks/budget_N/*.npy`)
| Budget | Nonzero / 1,480,274 | Median nonzero abs weight | 99th percentile | Largest | Distinct values |
|---|---|---|---|---|---|
| 0 | 34,190 (2.31 %) | 0.81 | 2,402 | L4: 2,402 (M + 1) | 575 |
| 10 | 26,761 (1.81 %) | 1.0 | 40 | out.b: 350 | 62 |
| 25 | 23,944 (1.62 %) | 1.0 | 4,096 | L4/L5: 4,096 | 45 |
| 50 | 25,947 (1.75 %) | 1.0 | 100 | out.b: 250; L3: 100 | 65 |
| PPO same size (comparison) | 100 % | 0.046 | 0.18 | 0.71 | about 1.47M |

### 4.7 Evaluation [A] (`README.md`) and [R] (`eval/budget_N/eval_submission.json`, `h2h_*.json`)
- **Hidden scoring:** 2,000 games against each of two small PPO networks (243→32→32→82, seeds 0 and 2, 20,000 iterations × 1,024 games). Astra's network plays Black in half the games, moves are sampled, and there are no draws (komi 7.5).

| Budget | vs PPO s0 | vs PPO s2 | vs REINFORCE s0 / s2 | vs random |
|---|---|---|---|---|
| 0 | 54.15 % | 54.90 % | 98.8 / 99.7 % | 99.55 % |
| 10 | 72.85 % | 59.95 % | 99.0 / 99.9 % | 99.55 % |
| 25 | 76.00 % | 62.25 % | 98.05 / 98.1 % | 98.75 % |
| 50 | 84.00 % | 80.80 % | 99.85 / 99.9 % | 100 % |

- **Same-size PPO baseline:** the identical 512×6 ReLU network trained with PPO for 20.5M games. The learning rate of 3e‑5 was chosen from {1e‑4, 3e‑5, 1e‑5} using the practice opponent. Head-to-head over 2,000 games:
  - **Budget 0 loses** against seeds 0, 1 and 2: 42.6 %, 26.3 % and 36.3 % (e.g. [R] `eval/budget_0/h2h_ppo_big_lr3e-5_s0_iter20000.json` → `"score": 0.4255`).
  - **Budget 50 wins:** 74.2 %, 70.0 % and 78.5 %.
  - [A] Untuned PPO at lr 3e‑4 loses 92–98 % to the written networks (README "Other evaluation files"). This is why the lr scan matters.

---

## 5. Experiment 8: diagnose and fix Qwen3.5‑0.8B's math looping by targeted edits (Python kernel + forward-pass budget)

### 5.1 Task and model [R] (`repo/exp8_math/runs/p50c/message.txt`)
- The model is Qwen3.5‑0.8B, text part, with thinking on: "24 layers of width 1,024; layers 3, 7, 11, 15, 19 and 23 are full-attention layers … the other 18 are Gated DeltaNet linear-attention layers … each layer has a SwiGLU MLP of width 3,584; RMSNorm; vocabulary 248,320; input embedding and output layer tied."
- Task excerpt: "Your job is to make it better at mathematics by EDITING ITS WEIGHTS DIRECTLY: find out what goes wrong inside the network when it fails at math problems and repair that with targeted changes."
- Refused by `save_edit`:
  - the final norm;
  - the embedding / lm_head;
  - "any edit that only multiplies a whole tensor by a constant".

  Prompt: "Global knobs … are not what we are looking for, and neither is making the model skip its thinking or close it early".

### 5.2 Given, denied and feedback [R] (same file)
- Given:
  - a persistent CPU Python kernel, with `load(name)`, `load_original`, `save_edit`, `drop_edit`, `edited()` and `tok`; gradients are disabled; at most 300 code runs;
  - practice sets (7,469 GSM8K train, 7,275 MATH train, 714 AIME answers);
  - **10, 25 or 50 forward passes**, of two kinds:
    - (a) **generation:** up to 512 answers with the eval sampling. `"model": "both"` gives a *paired* comparison (accuracy difference with SE, rescues and regressions), and residual-stream / MLP activations are optional;
    - (b) **teacher-forced:** up to 1,024 texts, returning token log-probs, top-k, hidden states, MLP activations and attention.
- The prompt itself prescribes the method (excerpt): "Diagnose first … Localize causally. That a component is more active on correct answers is not enough … show that changing it changes the behaviour you predicted, and try several strengths to see a dose-response. … Measure with enough power … Keep an edit only if its gain is clearly larger than its standard error on problems that were not used to choose it."
- Denied:
  - training of any kind ("no backward passes, autograd or optimizers, and no iterative fitting of weights to a loss");
  - running or simulating the model in the kernel;
  - internet.
- [A] Mid-run notices from the experimenters (`notices/001…004`) said:
  - a pass costs one regardless of size;
  - separate generation passes are not seed-reproducible, so compare only within a "both" pass;
  - submit the best edit even if it is not significant;
  - a failure-count correction.
- [A] Rounds:

  | Run | Turns | Python runs | Conversations (handoff every 40 turns) | Hours |
  |---|---|---|---|---|
  | p10c | 74 | 59 | 2 | 11.6 |
  | p25c | 86 | 45 | 3 | 8.5 |
  | p50c | 158 | 80 | 4 | 20.3 |

  Effort was `max` (`README.md`).

### 5.3 The edits (exact formulas) [R]
- **p50c (50 passes):** `W = W0 - 6*outer(u, mask64*(u@W0))`, applied to `model.language_model.layers.{15,19}.mlp.down_proj.weight` (`runs/p50c/work_notes/final_report.md`).
  - Here u is a unit "repetition direction" in the residual stream (dimension 1,024), and mask64 keeps 64 selected MLP neurons (columns) per layer.
  - The 64 neuron IDs per layer are listed in `work_notes/frozen_weights_manifest.json` (e.g. layer 15: 0, 2, 11, 25, 34, …, 3003; layer 19: 1, 5, 8, 11, …, 3343).
  - [C] Note that α = 6 removes the component **more than** completely: within those columns the u-component becomes −5× its original value, so it is reversed. α = 1 would be a plain projection.
  - [R] Relative Frobenius change is 10.64 % for layer 15 and 8.11 % for layer 19. The edit touches 131,072 weights, of which **130,188 differ after bf16 rounding** (`edit/manifest.json`).
- Verbatim code that wrote the final edit [R] (`runs/p50c/tool_calls.jsonl`, entry 97):
```python
# Hypothesis: increasing only the selected repetition-writer columns yields a better benefit/side-effect tradeoff.
def set_subset_edit(alpha):
    for l in (15, 19):
        w = base_mlp_weights[l]
        u = u_rep[l].float()
        writer = u @ w
        masked = torch.zeros_like(writer)
        ix = writer_subsets[l].long()
        masked[ix] = writer[ix]
        save_edit(f"model.language_model.layers.{l}.mlp.down_proj.weight",
                  w - float(alpha) * torch.outer(u, masked))
set_subset_edit(6)
```
- How u and the neurons were found [R]:
  - Entry 9 of `tool_calls.jsonl` builds, from a teacher-forced pass on **matched productive versus repetitive continuations** (`fwd_4`), a mean residual difference per layer: `diffH = (a[1::2]-a[::2]).mean(0)` over the last 36 tokens. It then normalises it: `d = d/d.norm()`.
  - Each MLP neuron j is scored as Δactivation_j × (d · W_down[:, j]), and the decomposition is printed by layer (layer 15 has the largest MLP contribution, 0.2634).
  - The first causal probe ablated one neuron (`w[:,11]=0` in layer 19).
  - Entry 36 recovers u as the dominant column of the alpha-1 edit delta.
  - Entry 90 keeps the 64 neurons with the highest attribution ("writer_subsets"). Those 64 carry, for example, 0.204 of 0.254 of the repetitive-context attribution at layer 15.
- **p10c (10 passes):** 30 rank-one edits, `W' = W - 1.5 u(u^T W)`, on the attention and MLP output projections of layers 8–22 (`runs/p10c/final_answer.md`).
- **p25c (25 passes):** "zeroed columns `[1792:2048]` of `model.language_model.layers.15.self_attn.o_proj.weight`", i.e. head 7 switched off (`runs/p25c/final_answer.md`).
- [R] Evidence and controls reported by Astra (`p50c/work_notes/final_report.md`):
  - dose-response at α = 1/2/4 reduced repetitive-branch log-prob by 0.024/0.049/0.104 nats/token, with productive branches barely changed;
  - single-layer (L15-only and L19-only) tests;
  - a **norm-matched orthogonal random control** gained only +1.56 pp (SE 3.52).

### 5.4 Evaluation [A][R]
- [R] Diagnosis at the first pass: 11 of 96 correct, **85 of 96 cut at 32,768 tokens**, mean 29,427 tokens (`runs/p50c/requests/fwd_1.done.json`).
- [R] Astra's own held-out *practice* confirmation (p50c, 1,152 problems, paired): 10.6 % → 60.4 %, a gain of +49.8 pp (SE 1.5), with 579 rescues and 5 regressions. 576 of the rescues were originally truncated (`final_report.md`).
- [A][R] Benchmarks, hidden from Astra (`README.md` table; `eval/<run>/merged.json`):

  | Model | MATH-500 | GSM8K | IFEval | Cut at token limit |
  |---|---|---|---|---|
  | Base | 7.9 | 13.8 | 21.0 | 93.0 % |
  | 10 passes | 12.3 | 25.7 | 49.0 | 87.2 % |
  | 25 passes | 14.1 | 25.1 | 39.9 | 86.4 % |
  | 50 passes | 48.1 | 67.1 | 62.3 | 33.1 % |

  Other benchmarks at 0 / 10 / 25 / 50 passes:
  - AMC23: 0.5 / 1.1 / 1.1 / 23.4;
  - MMLU-Pro: 40.3 / 40.0 / 44.7 / 45.5;
  - GPQA: 22.7 / 29.4 / 30.1 / 36.6.

  The post's figure showed an interim base GPQA of 24.5 (`README.md` note).
- **Negative result, the Gemma 3 270M follow-up** [A] (`gemma_followup/README.md`):

  | Passes | MATH-500 (lenient) |
  |---|---|
  | 0 (base) | 6.8 |
  | 5 | 7.0 |
  | 10 | 7.6 |
  | 25 | 7.7 |
  | 50 | 7.0 |

  The edits were column rescalings, e.g. "doubles neuron 920 of layer 17's MLP". Gemma rarely loops (8.5 % of answers cut), and there was "no single component behind" the errors.

---

## 6. Cross-cutting: edit representation, construction tricks, numerics

### 6.1 Weight-edit representation, exactly as practised
- **One-shot, tool-free (exps 4 and 5).** The representation is a fenced code block of type `edits` with a small assignment language: NumPy-style indexing with half-open `a:b` and strided `a:b:s` ranges, and the operators `=`, `*=`, `+=`, `-=` and `ZERO T`. The right side is a scalar broadcast, a 1-D list, or a 2-D nested list. Statements apply in order, later ones overwrite earlier ones, and only the last block counts. Unreadable statements are skipped (`|v| ≤ 1e6`) (§2.4, §3.3).
  - In exp5 everything starts at zero, so Astra only writes the nonzeros (637–815 per network).
  - In exp4 Astra first **zeroes slices** (`W[:, 6:9] = 0`, `W[122:128, :] = 0`) and then writes **single elements** (`actor0.W[6, 122] = 80`), **bias vectors** (`actor0.b[122:128] = [...]`) and **incremental bias patches** (`actor_head.b[0] -= 200`).
  - Strided slices are how grid geometry is addressed: `W1[3:600:60, 4]` is one board column in one channel.
- **Code-generated (exp6).** Zero-initialised numpy arrays are filled by Python loops over points and neighbours with `+=` accumulation (`W[3*q, j] -= 1.0`), driven by coefficient dicts, then `save(name, array)`. This is a parametrised **builder function** rather than per-element edits.
- **Edit to a trained LLM (exp8).** Whole-tensor replacement through `save_edit(name, tensor)`, built as low-rank, column-masked updates (`w - alpha*outer(u, masked)`) or column zeroing (`o_proj[:, 1792:2048] = 0`).
- **Reading representation (exps 1, 2 and 4).** Weights are shown as text: rounded to 3 decimals in exps 1–2, and as integers × one scale per tensor in exp4 (`README.md` of each).

### 6.2 Catalogue of construction tricks actually present (with numbers)
| Trick | Where (file) | Numbers |
|---|---|---|
| Steep tanh step at a chosen knot | exp4 `answer.md` | slope 80, bias −80·knot (−200 → 2.5, −248 → 3.1, −304 → 3.8, −76 → 0.95) |
| Bias shift so ReLU passes a signed variable linearly | exp4 | `embed.b[7] = 11` for vx ∈ [−10, 10] |
| Offset carried and cancelled downstream | exp6 b0 | OFFSET = 128, `out.b = -128` |
| AND / OR / NAND on ±1 units by bias | exp4 actor1/2 | ±10 weights; bias −10 gives AND of 2; −30 gives AND of 4; +30 gives NAND |
| Gated comparator: hard decision from a small graded error | exp5 Breakout | ±1000 on a 0.02-per-column error, ±200 gates, bias −415 |
| Bias cancellation of inactive branches (exact zero contribution) | exp4 head; exp5 b3 | (−100, −100, 200) / (−100, 200, −100) with b offsets (−200, +100, +100); Breakout b3 = 90 vs 4×30 |
| Action priority via bias offsets | exp5 Seaquest/Asterix | b3 = [−100, −100, 238, −100, 244, 241.8] → default fire by 1.8; −1000 masks |
| Isolated parallel pathway inside a trained net | exp4 | zero rows/cols of commandeered units; 6,073 of 6,318 written entries are mostly these |
| Linear value code in the tanh linear regime | exp5 Breakout | 0.02 per column, max \|pre\| ≈ 0.18 |
| Count gated by binary presence (no multiply) | exp6 L1 | flag weight 4, bias deg − 4, −1 per occupied neighbour |
| Threshold certificate on a weighted count | exp6 L2 | bias −8 − 64 / −16 − 64, +64 from the presence flag |
| Big-M gating + (z, g) flag/value pair | exp6 L3/L4 | M = 481, then 2,401; z − g recovers the flag exactly |
| Hinge ramps at knots (piecewise-linear) | exp6 globals | PHASE = relu(T − 18), LATE = relu(T − 52) |
| ReLU pair for positive and negative parts | exp6 AHEAD/BEHIND | ±(own − opp ± 7.5·white) ∓ 1 |
| Identity skip for globals/planes | exp6 `copy_globals`, `carry_board` | W[g, g] = 1 at every layer |
| Stencil as dense matrix, isotropic table, edge renormalisation | exp6 b0 `kernels`, `factor` | up to radius 4; factor min(1.65, sqrt(mass_full / mass_row)) |
| Repeated-layer propagation (one neighbour step per layer) | exp6 L3, L4 | neighbourhood sum +1 per neighbour |
| Window = difference of shifted steps | exp5 Asterix | biases ±8 / ±21.6, readout +16 / −16 |
| Geometric distance decay tables | exp5 Seaquest W2 | 0.6, 0.45, 0.34, 0.255, 0.19, 0.145, 0.11, 0.08 |
| Channel subtraction to separate overlapping inputs | exp5 Seaquest | reward +8, danger −8, bias −4 |
| Ternary packing in one unit | exp6 b50 | own/empty/enemy = 1, 1/2, 0 |
| Global output temperature knob | exp6 b50 | `temperature_scale` 1.8 → 2.5 |
| Weight-level symmetry check (no forward pass) | exp6 b50 t075 | 8 dihedral permutations, max error 0 |
| Column-masked low-rank directional correction | exp8 p50c | α = 6, 64 columns per layer, 2 layers |

### 6.3 Magnitude conventions and numerical hygiene (observed)
- **Two regimes coexist.**
  1. Graded or linear signals use small weights: exp5 Breakout uses 0.01–0.2, and exp6 kernels use 0.015–1.0.
  2. Logic and gating use saturating magnitudes: 6–20 for ±1 logic, 80 for steps, 200–1,000 for gates and masks, and M = 481–4,096 for ReLU big-M.
- **Saturation is deliberate.** tanh(10) ≈ 1 − 4·10⁻⁹, so logic bits are effectively exact. Logit margins of 60–600 make sampling nearly deterministic.
- **Hygiene in the code (exp6).**
  - `assert array.dtype == np.float32` and `assert np.isfinite(array).all()` before saving (budget 0).
  - The print of the nonzero count.
  - The symmetry check (budget 50).
  - The sha256 of the frozen network (exp6 `final_validation_manifest.json`; exp8 manifests).
- **Precision.** In exp8 the float32 submission is cast to bf16 for evaluation. 884 of 131,072 touched values rounded back to the original (`edit/manifest.json` note).
  - [C] Constants around 2,402 in float32 are fine.
  - [C] bf16 would quantise values near 2,402 in steps of 16 and would break the exact (z, g) cancellation. Exp6 was evaluated with float32 tensors (`networks/*.npy` are float32).
- **Sparsity.**
  - exp5: 3.1–4.0 % nonzero.
  - exp6: 1.6–2.3 % nonzero.
  - exp4: 3.54 % of entries changed.
  - exp8: 0.016 % of weights changed.

### 6.4 What "convolution kernels" look like in exp6
- There are no conv layers. Each kernel is a coefficient table keyed by the offset (|dr|, |dc|), sorted for 8-fold symmetry, and written into a dense 486×486 block (or 243×512 in L1) by looping over all point pairs (p, q).
- This yields translation-equivariant and dihedral-equivariant weights, apart from the edge renormalisation. Budget 50 uses the same scheme with `rings[p] = (q, |dr|, |dc|)` and classes adjacent / diagonal / two-step / knight's move (`final_policy_design.py`).

---

## 7. Limitations and caveats

**Stated by the authors [A]:**
- It is "a statement about an emerging capability", not a method. The baselines test "only the narrow claim of its experiment" (blog FAQ).
- Results are "still limited, specifically due to inefficiencies in context length and harnesses" (blog, "What Comes Next").
- Exp7: from weights alone Astra could not identify the sycophant in 4 attempts. It succeeded only with an attention-pattern tool (`repo/exp7_sycophancy/README.md`).
- Exp8 follow-up: editing "could remove an obstacle that hid an ability, but it could not add an ability the model lacked" (Gemma; blog).
- Exp4: Astra itself wrote "This is a physics-based repair, not a simulated or verified result" (`answer.md`).
- Exp6: Astra's final answers say "Strength against the hidden opponents remains untested" and "Hidden-opponent performance remains unmeasured" (`runs/budget_0/final_answer.md`, `runs/budget_50/final_answer.md`).
- Exp8: "These are held-out PRACTICE results, not unseen benchmark scores" (`final_report.md`). The base model's 93 % truncation rate comes from thinking-on sampling. Turning thinking off would raise the base score (blog FAQ).
- One run per condition in exps 4, 5 and 6. Earlier prompt versions and trial runs are excluded (`repo/README.md`; exp6 README mentions an unreported 1-pass run and a v2 prompt).
- Exp5: the first reply parser had to be fixed after the fact. The same replies were re-read (`exp5 README.md`).

**Evident from the artifacts [C]:**
- **Exp4 generalisation is narrow.** The scene is fixed and deterministic ("every episode starts from exactly the same state", `prompt.txt` line 6). The 1,024 test episodes differ only in action sampling, and the override hard-codes scene coordinates (2.5, 3.1, 3.8, 0.95).
- **Exp5 is one sample per game,** with no variance across replies. Breakout is probably capped by the 10,000-step limit (mean length 7,417). The PPO baselines are best-of-seeds with high variance across seeds.
- **Exp6 budget 0 loses to the same-size PPO network** (26–43 %). The headline win needs 50 passes of game feedback. The hidden opponents are small (32×32) PPO networks. Large constants (M = 2,401–4,096) make the network sensitive to precision and to any input outside the binary domain.
- **Exp8's large gain depends on one pathological failure (looping).** The 10- and 25-pass runs gave small gains (MATH-500 +4.4 and +6.2 pp). Astra chose α = 6 empirically through forward passes, i.e. the edit was tuned with feedback, not derived purely from reading the weights.
- **No smooth nonlinear function was hand-built** in any artifact (no exp, log, power or product of two continuous variables). Every construction is linear, piecewise-linear (ReLU hinges) or thresholded (saturated tanh steps, logic). The only "continuous" lookup is a per-cell table over a *discrete* grid input.

---

## 8. Mechanisms that transfer to scientific regression / dynamical systems / hazard models / neural operators

Only mechanisms evidenced above are listed. "Maps to" is my proposal.

**Transfers**
1. **Steep sigmoid/tanh step at a chosen knot** (exp4: slope 80, bias −80·knot). This maps to regime indicators such as extinction or ignition thresholds and Damköhler or diameter cut-offs. The width of the transition is about 1/slope, so you can choose how sharp a regime boundary is.
2. **ReLU hinge ramps at chosen knots** (exp6: relu(T − 18), relu(T − 52)). A sum of hinges with chosen slopes gives a piecewise-linear table. For example, a residual correction to the D² law, Δ(t/d₀²), could be tabulated over a normalised coordinate.
   - [C] Only single hinges are evidenced. A multi-knot table is the direct generalisation.
3. **ReLU positive/negative pair for a signed quantity** (exp6 AHEAD/BEHIND). This carries a signed residual or deviation, e.g. (T − T_ref), through ReLU layers with different slopes on each side: an asymmetric, piecewise-linear response.
4. **Bias shift or carried offset for linear pass-through** (exp4 `+11`; exp6 OFFSET = 128, cancelled at the output). Shift a bounded signed input such as log-pressure or the log diameter ratio into ReLU's positive range, then subtract the constant in the output bias, so the base physics term flows linearly.
5. **Bias-cancelled inactive branch, i.e. an exact-zero residual** (exp4 head offsets; exp5 b3). Add a correction branch whose contribution is exactly 0 outside its regime. This keeps a baseline model such as the D² law or a baseline hazard untouched where the correction is not wanted.
6. **Isolated parallel pathway inside an existing net** (exp4: zero rows and columns of the units you commandeer). This bolts a hand-written correction onto a trained surrogate without interference.
7. **Logic gates on ±1 units** (exp4: ±10 weights, bias −10, −30 or +30). Combine regime indicators, e.g. "small droplet AND high ambient T", to switch branches. The output is piecewise-constant, so use it for selection, not for smooth fitting.
8. **Linear value code in the linear regime of tanh** (exp5: 0.02 per unit, |pre| ≤ 0.18). If the architecture forces tanh, keep pre-activations small to approximate a linear map, and amplify downstream.
9. **Window or bump = difference of two shifted steps** (exp5 Asterix: ±16 readout of bias-shifted units). This builds a piecewise-constant hazard basis, e.g. a baseline hazard h₀(t) on intervals.
   - [C] With softer slopes it becomes a smooth bump.
10. **Count or quantity gated by a binary mask without multiplication** (exp6 L1: flag weight = maximum count, bias offset). Compute a quantity only where a mask is on, e.g. only on burning or flame-front cells, provided the quantity has a known bound.
11. **Big-M gating plus a flag/value pair** (exp6 L3/L4, M = 481 or 2,401). Carry a mask bit and a non-negative field value together through ReLU layers and recover both linearly. This needs a known upper bound on the value, and float32.
12. **Stencils as dense matrices from neighbour loops, isotropic tables, edge renormalisation** (exp6 `kernels`, factor min(1.65, sqrt(full/row))). This writes Laplacian, curvature or smoothing operators for a flame-spread contour field on a grid.
    - Key coefficients by (|dr|, |dc|) for rotation and reflection symmetry.
    - Renormalise the stencil at the boundary.
    - Each layer gives one explicit step (exp6 L3/L4 propagate one neighbour step per layer). This is a natural fit for explicit time-stepping or front propagation.
13. **Geometric distance-decay kernels** (exp5 Seaquest: ratio ≈ 0.75 per cell). These give influence or attenuation kernels, e.g. radiative or diffusive influence with distance, as banded Toeplitz blocks.
14. **Identity skip channels for global scalars** (exp6 `copy_globals`). Carry global parameters such as pressure, ambient O₂ or gravity level unchanged to deep layers where they modulate outputs.
15. **Global output scale ("temperature") as one knob** (exp6 `OW *= temperature_scale`). A single multiplier on the output layer sets the overall scale, e.g. of a log-hazard or a correction magnitude. Tune it last.
16. **Weight-level invariance tests without a forward pass** (exp6 t075: permute the weights under symmetries and check that the error is 0). Verify that an operator's weights are exactly symmetric under rotations and reflections, or under reversal of a 1-D axis, before any evaluation.
17. **Parametrised builder function plus coefficient dict, then feedback tuning** (exp6 budget 50 `build(save, overrides)`). Write the network as code with physically meaningful coefficients and adjust only those coefficients when checking against data. This keeps the design readable.
18. **Directional, column-masked low-rank correction of a trained net** (exp8: W − α·u·(mask ⊙ uᵀW)). If an existing trained surrogate has a systematic bias that is linear in some hidden direction, subtract that direction in the selected output columns.
    - This needs paired evaluation and dose-response, and a random-direction control (exp8 `final_report.md`).
19. **Evaluation discipline** (exp8 prompt and p50c):
    - freeze the edit before held-out confirmation;
    - make paired original-versus-edited comparisons on the same inputs;
    - run dose-response sweeps (α = 1, 2, 4, 6);
    - include a norm-matched random control.

**Does NOT transfer, or not evidenced**
- **Hard action priorities, −1000 masks and argmax/softmax margins** (exps 4 and 5). These are specific to categorical policies. A regression output has no softmax to absorb large offsets, and a bias of −1,000 would simply be wrong.
- **Saturated tanh logic used for smooth fits.** It produces discontinuities. Use it only to select regimes, not to model a smooth law.
- **Smooth nonlinear primitives (exp, log, power, products of two continuous inputs, d² = d₀² − K·t, Arrhenius terms) were never constructed** in these artifacts. Nothing here shows how to hand-build multiplication or exponentials. Any such construction (e.g. log-space inputs so that products become sums, or piecewise-linear approximation of exp) is beyond the evidence.
- **Discrete-grid lookup tables** (exp5 Breakout's per-cell landing tables) rely on one-hot inputs. They do not carry over directly to continuous scalar inputs; there you need hinge or step bases (items 1, 2 and 9).
- **Exp8-style gains depend on a single dominant failure mode in a trained model.** The Gemma follow-up shows that targeted edits do not add missing capability.
- **Big-M constants (481–4,096) and the exact (z, g) cancellation** are safe only with bounded inputs and float32. They break with bf16, unbounded continuous inputs, or input noise.
- **The reading experiments (exps 1, 2, 3 and 7)** inform interpretation, not construction. They give no hand-writing mechanism beyond the text representation of weights.
