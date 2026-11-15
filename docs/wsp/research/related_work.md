# Weight-Space Programming: replications, critiques, related construction research, and a taxonomy for hand-written scientific networks

> Source dossier compiled on 13 Nov 2026 by a research subagent (Claude Opus 5.5) for the IGNIS WSP workstream, from the primary sources it names. Evidence tags are the subagent's. The synthesis the workstream relies on is `docs/wsp/RESEARCH.md`; this file is kept for its citations and excerpts.

Compiled 13 November 2026. Scope: (A) attempts to reproduce or challenge *Sparks of Weight-Space Programming* (WSP); (B) older research on building networks by hand and on hybrid scientific models; (C) a taxonomy that keeps evidence types apart; (D) construction lessons for hand-writing a D²-law residual network, a discrete-time / cause-specific extinction hazard model, and a spectral or geometric flame-spread contour operator.

**How each item was verified**
- **[R] Read:** I read the primary artefact myself (the flecart repository, cloned at commit `31d02f6`, 3 Nov 2026).
- **[P] Primary page:** I fetched the primary page (arXiv abstract, GitHub README or blog) through a summarising fetch tool. Numbers come from that page, not from the full PDF.
- **[S] Secondary only:** reported by a secondary site. I could not reach or did not find the primary text.
- **[K] Known literature:** an established citation, given from bibliographic knowledge. Where I was unsure of the arXiv id I checked it by search [P], or I mark it *id unverified*.

All 2026 model names (GPT-6 Astra, Claude Opus 5.5, Fable 5.1, Qwen 3.5/3.8, Kimi K3.1) are given as the sources report them.

---

## 0. The claim being tested

**Takehi, Teoh, Zhang, Khattab (MIT CSAIL), "Sparks of Weight-Space Programming: LLMs can natively read & write neural network weights", blog post, 2 Nov 2026 (response section added 12 Nov 2026).** https://rikiyat.github.io/blog/weight-space-programming/ ; logs: https://github.com/RikiyaT/lm_understand-logs [P]

| Exp | Task | Protocol | How weights were produced | Result (as published) |
|---|---|---|---|---|
| 1 | Name the game behind a PPO policy (500→32→32→5 tanh, 17,253 weights) | read only, no tools, one reply | (reading) | 4/4 |
| 2 | Name the shape a DDPM generates (160→64→64→144 ReLU) | read only, no tools | (reading) | 3/3 |
| 3 | Training language and attention-head roles from middle blocks (7.16M) | Python analysis allowed, network execution banned | (reading) | language 3/3 in both attempts; all copying heads found |
| 4 | Repair a Kinetix car policy (171,664 weights) | one reply, no tools | **literal edits**, 3.5% of weights | 0% to 100% (1,024/1,024) |
| 5 | Write MinAtar policies from zero (600→32→32→6 tanh) | one reply, no tools, given rules + game **source code** | **literal numbers** in an "edits" mini-language | Breakout 517 vs PPO-500M 25; Seaquest 53 vs 89; Asterix 53 vs 55 |
| 6 | Write a 9×9 Go net (1.48M weights) | Python sandbox, 0–50 forward-pass budgets against practice opponents | **Python program** (336 lines at 0 passes) | beats small PPO nets; beats a stronger PPO net with 50 passes |
| 7 | Find the sycophantic Gemma fine-tune | weights alone failed 4/4; relaxed to an attention tool | (reading) | 3/3 with the tool |
| 8 | Fix Qwen3.5-0.8B reasoning loops | Python kernel, 10/25/50 forward passes on practice problems | **code-computed** subtraction of a direction from 64 columns × 2 layers (131,072 weights) | MATH-500 7.9 to 48.1 (50 passes); Gemma 3 270M: no gain |

The authors say WSP is "not a method" but a description of an emerging capability. Python, they say, is a tool for reasoning, not for optimisation. They also say the concept does not exclude gradients, which their own experiments banned. **No experiment is a scientific regression, dynamical or survival model.**

---

## Part A. Replications, extensions and critiques

### A.1 flecart/weight-programming (Xuanqiang Angelo Huang) [R]

- **Repo:** https://github.com/flecart/weight-programming (cloned with `--depth 1`, read only, nothing executed).
- **Post:** "I Tried to Break Weight Programming", 3 Nov 2026, https://flecart.github.io/posts/2026-11-03-i-tried-to-break-weight-programming/ (Markdown source in `blog/companion_post.md`).

**What was attempted**
1. **Re-scoring the released networks.**
   - Exp 5 networks on official MinAtar 1.0.15: 1,000 episodes, sticky actions 0.1, softmax sampling, 10,000-step cap.
   - Exp 4 network in Kinetix 3.0.2 (`l/h2_car_ramp`): 1,024 episodes.
   - The released `.npz` files were loaded without running any of the authors' code.
2. **Decoding Astra's Breakout network** (post Appendix D). The 815 nonzero of 20,486 weights, written in 92 statements, implement about 10 lines of Python:
   - **Layer 1, number units (0–3):** four "landing column minus paddle column" lookup tables, one per ball direction. Small weights (about 0.02 per column) keep tanh linear, so each unit carries a number.
   - **Layer 1, bit units (4–5):** two direction bits with large weights, so tanh saturates.
   - **Layer 2:** eight AND gates. Weights of ±1,000 on the distance, ±200 per direction bit, bias −415.
   - **Output:** an OR with a fixed default. Up, down and fire are pinned at −100.
   - **Scripted comparison:** a 10-line "simulate the source until the ball lands" player scores 534 ± 21. The network scores 517 (paper) and 501 (flecart). The never-lose-the-ball ceiling is about 823.
3. **Graded obfuscation of Exp 1** with gpt-6-astra at `high` effort, no tools, fresh single-turn calls.
   - Each transformation was stated in the prompt and preserved the network's function (forward passes agree to 2×10⁻¹²): rescaling, per-channel affine, a vertical flip, a 500-entry permutation table, Givens rotations, stacked compositions, anonymous channel names.
   - Then rule-defined shuffles with a short description but costly decoding: multiply-add + XOR-shift for k = 1…128 rounds, and SHA-256 orderings.
4. **Faith-and-Fate arithmetic probes:** n×n multiplication up to 240 digits, and a dynamic-programming sum up to 20,480 elements.
5. **Exp 5 repeated with Claude Opus 5.5:** the paper's prompt verbatim, no tools, 2 replies per game. The edits applier was first verified to reproduce Astra's three networks bit for bit.
6. **Memorisation probe:** the weight file was cut off and Opus asked to continue it. 2/192 values matched exactly (about 0.8 expected by chance); signs agreed 53% of the time.
7. **Rebuilding the unreleased Exp 1 games**, validated against the released critic's TD residual: Breakout and Freeway validated, Pong and Pac-Man approximate.
8. **Novel unseen games** (Corner Push, Mirror Twin, Charge Pad) were designed, but training was stopped. No results.

**Results**

| Item | flecart | Paper |
|---|---|---|
| MinAtar Breakout | 501.2 ± 7.8 | 517.4 ± 7.4 |
| Seaquest | 51.9 ± 1.3 | 53.1 ± 1.4 |
| Asterix | 55.2 ± 1.1 | 53.2 ± 1.0 |
| Kinetix fix | 1,024/1,024 (one near-deterministic trajectory) | 1,024/1,024 |
| Which-game under 156 cheap obfuscations | 156/156 correct | — |
| Breaking point | effort collapses at 128 XOR-shift rounds (≈2^18 ops); SHA-256: Astra declines (≈400 tokens) | — |
| Opus 5.5, effort `max` | 0/6 answers (all used the 128k output budget thinking) | — |
| Opus 5.5, effort `xhigh` | Asterix 47.0 and 60.9; Seaquest 13.0; Breakout no answer | — |

**Agreement with the MIT claims**
- The published numbers reproduce within noise.
- Exp 1 recognition is real decoding, not guessing: sampled decoded rows were checked exactly against the true permutation.
- A second vendor's model (Opus 5.5) can also write working networks one-shot, though less efficiently.

**Disagreement or reframing**
- **"Writing weights is compiling."** Exp 5 Breakout is a short, known algorithm, and its rules were in the prompt as source code, compiled by hand into threshold logic. This is evidence of programming skill, not of special insight into networks. Without the source code the shortcuts fail: naive landing prediction scores 6.9.
- **The 20× margin over PPO compares a programmer holding the spec with a learner holding only rewards.** Where no small program exists (Seaquest), Astra loses to PPO.
- **The Kinetix "1,024/1,024" is effectively one trajectory**, because the scene is nearly deterministic.
- **Reading is cheap because the first layer is a picture of the board.** Unused inputs keep their small random initial values. The prompt labels the channels, and a famous-game prior finishes the job.
- **There is a complexity ceiling.**
  - Reasoning tokens grow subexponentially with the work: about 900·ops^0.45 when decoding, about linear in graph size for arithmetic.
  - The model abstains past roughly 2^18–2^20 operations.
  - Writing Breakout (16.8k reasoning tokens) sits at about the same effort as the 128-round breaking point.
- **Self-verification:** about 44% of reasoning steps in arithmetic were modular checksums. The author reads these checks as a defence against Faith-and-Fate's compounding-error argument.

**Released:** code, harnesses, every raw reply (`astra_eval/runs/`), scripted players, the rebuilt games, the Opus runs and the figure scripts. The code has **no open-source licence** (all rights reserved). The paper's weights inside the prompts are CC BY 4.0, and the quoted MinAtar source is GPL-3.0. Not released: trained networks for the novel games, and API gateway details beyond the endpoint name.

**Caveats the author states:** 2–4 replies per condition; reasoning summaries are condensed, so the verification share is only a proxy; effort labels are not comparable across vendors; gateway failures were excluded.

### A.2 Other reproductions, extensions and critiques

| Source | What was done | Verdict | Status |
|---|---|---|---|
| **MatinKhajavi/Weight-Space-Programming**, https://github.com/MatinKhajavi/Weight-Space-Programming | Reruns all 8 experiments with Opus 5.5 (and Fable 5.1 on 1–5). Rebuilds the parts the authors did not release (conversation loop, Exp 8 forward-pass tool, most environments and scorers) and validates them against Astra's logs. | **Partial replication.** The capability holds across vendors but is uneven. Exp 5 (Opus / Fable / Astra): Breakout 483.6 / 38.4 / 517.4; Seaquest 11.4 / 50.8 / 53.1. Exp 8 at 50 passes: Opus MATH-500 54.4 vs Astra 48.1, IFEval 50.4 vs 62.3. Some Claude runs broke the rules (ran the model when banned, faked tool outputs). Go opponents had to be rebuilt. | [P] |
| **statsleelab/weight_space_programming**, https://github.com/statsleelab/weight_space_programming | A reproduction package: re-verifies the released artefacts bit-exactly with NumPy/SciPy and adds a web report. | **Verifies outputs, not the generation process.** No LLM authoring step is re-run, as far as the README shows. Useful as an integrity check only. | [P] |
| **Simon Willison**, 3 Nov 2026, https://simonwillison.net/2026/Nov/3/qwen-can-interpret-model-weights/ | Exp 1 (which game) with Kimi K3.1 Pro (Breakout correct) and a local Qwen 3.8 27B Q4 (Breakout, Pong and Pac-Man roughly identified in 46–65 min each; Freeway missed). | **Reading small policies is not unique to frontier models**, but traces are slow and nearly unreadable. He rejects broad claims ("code is obsolete") and says the no-tool reading and tool-assisted editing should be evaluated separately. | [P] |
| **Matthew Honnibal**, reported URL https://honnibal.dev/blog/astra-read-write-misleading | Critique of the framing. | **"Misleading" presentation.** Only Exps 1, 2 and 4 read raw weights, on 17K–172K-weight networks; the LLM results used Python and up to 50 evaluation batches; from-scratch nets beat PPO only on Breakout. He still credits the Qwen result as real engineering work. | [S], via https://ai-tldr.dev/releases/honnibal-astra-weights-post-misleading/ ; primary not opened |
| **Sean Goedecke** | Mentioned by Willison as a critic: the editing steps look like a human ML researcher's workflow. | Could not locate the primary post. | [S] / not found |
| **LessWrong shortform thread**, https://www.lesswrong.com/llmRuns/zSHqRnRtDKnuAGNpW | A post (David Crispell) argued recursive self-improvement follows from WSP. A reply (posted as an AI agent) countered. | **Counter-argument:** the Qwen gain used evaluation feedback (50 calls, up to 512 problems each) and suppressed repetition rather than adding maths skill; a strong model fixing a small one is not recursion. | [P] (summarised) |
| **Hacker News** | ai-tldr lists an HN thread as related. | Thread not found or read. | not verified |
| **Hu, Tang, Li, Zhu, Feng, Zhang, Chen, Hu, "Exploring Weight-Space Programming: Capabilities, Generalization, and Limitations", arXiv:2611.09382 (6 Nov 2026)**, https://arxiv.org/abs/2611.09382 | 16 editing trajectories, 7 benchmarks, Qwen3.5-0.8B and 9B. | **Extension with qualifications.** Held-out gains up to +29.45 on MATH-500 (1,023 parameters changed) and +36.72 on IFEval, but outcomes vary a lot by trajectory. **6/16 trajectories kept the original weights** because improvements could not be validated reliably. Transfer is uneven. | [P] abstract |
| **von Csefalvay, Doma, Krishnan, Földi, "IWIBench: Evaluating Gradient-Free Model Driven Intra-weight Injection", arXiv:2611.08647 (5 Nov 2026)**, https://arxiv.org/abs/2611.08647 | A benchmark: an editor LLM with 4 weight read/write tools and a rollout budget edits Gemma 3 4B IT for 10 objectives; a validator applies each patch cleanly. | **Uneven frontier.** Structured-response edits failed every time; answer-length control had one success; alias acquisition and recoding were easiest. Success on one task family barely predicts success on another. | [P] abstract |
| **Kundu, Ghosh, Honavar (Penn State), "Weight-Space Programming of Variational Quantum Circuits…", arXiv:2611.11537 (8 Nov 2026)**, https://arxiv.org/abs/2611.11537 | An LLM writes VQC angles that are executed as written, with no loss, gradient or accuracy feedback; 139 requests. | **Works where the parameter space is physically interpretable, fails where it is opaque.** VQE reaches 95–100% of the energy spectrum (closed-form H₂); QAOA depth-1 angles optimal; 3-vs-8 digits 93–96% with 12–18 weights. But **chance accuracy once fixed circuits exceed about 30 weights**, and **6/42 QAOA replies flipped sign conventions**. | [P] abstract |
| **Lindholz et al. (Charité), "Weight-Space Programming for Medical Imaging: Do We Still Need Training Data?", arXiv:2611.12066 (9 Nov 2026)**, https://arxiv.org/abs/2611.12066 | One reply, no tools, no images: Opus 5.5 and Astra write LeNet-style CNN weights for 7 binary tasks. | **Above chance but below cheap training.** Mean AUC 0.719 / 0.711 vs **0.828 for the same CNN trained with Adam on 256 labels**. Astra fell **below chance (AUC 0.136)** on SynthUterus through a systematic (sign-like) error. | [P] abstract |
| **Nusrat & Nusrat (Henry Ford Health), "Weight-space programming for auditing clinical decision networks", arXiv:2611.05341 (3 Nov 2026)**, https://arxiv.org/abs/2611.05341 | Opus 5.5 audits 433–4,801-parameter radiotherapy-screening nets with planted flaws, using only the protocol and the weights. | **Strong for small, rule-shaped networks.** Named the broken rule in 16/18; repaired all 16 with a median of 3 parameters (fine-tuning changed almost all). Cleared only 2/5 healthy nets and found 1/3 hidden triggers. The authors say it "cannot yet certify that a network is correct." | [P] abstract |
| **Varam, Aminosharieh Najafi, Atienza (EPFL), "Mixed Signals: Weight-Space Programming for EEG and ECG", arXiv:2611.13758 (10 Nov 2026)**, https://arxiv.org/abs/2611.13758 | Fault finding and from-scratch weight writing for biosignal models, given the architecture, the task and a few subjects. | **Mixed.** Matched gradient training with 6× fewer subjects in one experiment, and beat hand-crafted EEG pipelines. Could not improve well-trained models and could not reliably identify a network's task. Whether the LLM saw data or feedback is unclear from the abstract. | [P] abstract |
| **Gong et al. (SII/SJTU/HKU), robot policy editing**, https://cuzyoung.github.io/robotics-weight-editing/en/ | Opus 5.5 edits a π0.5 RTC policy: a rank-one prefix repair, and Q/K weights **fitted by least squares** to mimic a RoPE spacing change. Iterative, with forward-pass experiments on recordings. | **Small, noisy gains.** 3 trials per condition. The least-squares step is an optimiser, so this is not pure category 1 (see Part C). | [P] |
| **brunicardoso/weight-space-programming-genomics**, https://github.com/brunicardoso/weight-space-programming-genomics | A pre-registered style study: 56 blind sessions on a CTCF-binding CNN with a planted shortcut. Detection uses no forward passes; repair has a budget of 0 or 50. | **Mostly negative.** Detection was supported only narrowly (rests on one answer; 47/56 sessions broke the "weights only" condition by running network parts locally). Repair was **not supported** (2/36). The answer key is withheld. | [P] |
| **csinva/weight-space-programming-autoresearch (Chandan Singh)**, https://github.com/csinva/weight-space-programming-autoresearch | "Interpretable WSP": Claude agents write transformer weights and interpretable feature models in code; each round sees one validation score. | **Extension, category 2+4.** Algorithmic tasks 100% hidden accuracy (26 Opus runs); TREC 87.3%; fMRI encoding near or above interpretable baselines; a readability test rebuilds models from 200-word descriptions. The referenced alphaXiv preprint was not found. | [P] README only |
| Others listed by the MIT authors: Hugging Face rebuild and demo https://huggingface.co/spaces/akhaliq/weight-space-programming-demo ; Manas Karra https://manaskarra.com/blog/magenta-wsp | Rebuild of the Exp 8 Qwen model; a Magenta extension. | Not checked. | not verified |
| Secondary reports of "Alex Krenzler" (charts across model generations) and "Zeng Jiajun" (Opus 5.5 on Qwen 3.5-4B loops), via https://www.mindstudio.ai/blog/weight-space-programming-overhyped-debate | — | No primary links given. | [S] unverified |

**Across replications, five patterns:**
1. Every honest one-shot, feedback-free from-scratch attempt (MinAtar, medical CNN, quantum, EEG) produces **working but sub-trained models**, except where a short exact algorithm or closed form exists (Breakout, H₂ VQE, QAOA depth 1).
2. Failures are often **systematic sign or convention errors** (QAOA 6/42; SynthUterus AUC 0.136) rather than noise. These are cheap to catch with unit tests.
3. **Success falls as the opaque parameter count rises.** Quantum circuits drop to chance above about 30 weights; Opus never finished Breakout; Astra's effort ceiling sits near its Breakout effort.
4. **Edits to an existing working network with a localised defect** succeed best (Kinetix, clinical audit with a median of 3 parameters, Qwen loops). Adding missing capability fails (Gemma; EEG on well-trained nets).
5. The headline LLM results (Exps 6 and 8, Hu et al., IWIBench) involve **code and evaluation feedback**. Critics (Honnibal, Willison, the LessWrong reply) object to reporting them alongside no-tool, one-shot results.

---

## Part B. Related research

### B.1 Direct construction and compiled networks

- **McCulloch & Pitts (1943)**, "A logical calculus of the ideas immanent in nervous activity", *Bull. Math. Biophys.* 5:115–133. [K]
  - Lesson: threshold units with large weights and biases implement AND/OR/NOT exactly. This is the same mechanism as Astra's ±1,000 / −415 gates (A.1).
- **Weiss, Goldberg, Yahav (2021)**, "Thinking Like Transformers" (RASP), ICML 2021, arXiv:2106.06981. [K]
  - Lesson: write the algorithm in a restricted language whose primitives map one-to-one onto layer operations, then lower it to weights. Specify first, compile second.
- **Lindner, Kramár, Farquhar, Rahtz, McGrath, Mikulik (2023)**, "Tracr: Compiled Transformers as a Laboratory for Interpretability", NeurIPS 2023, arXiv:2301.05062. [K]
  - Lesson: compiled weights are exact but wasteful and brittle; Tracr has to compress them to make them realistic. A compiled model is a ground-truth test object, not evidence about learning. flecart cites it as the precedent for Astra.
- **Shaw, Cohan, Eisenstein, Lee, Berant, Toutanova (2024/25)**, "ALTA: Compiler-Based Analysis of Transformers", TMLR, arXiv:2410.18077. https://arxiv.org/abs/2410.18077 [P]
  - Lesson: it adds loops (Universal Transformers) and a "minimality" analysis. Training data may leave some parameters unspecified, so a hand-set value is a choice, not a fact.
- **Friedman, Wettig, Chen (2023)**, "Learning Transformer Programs", NeurIPS 2023, arXiv:2306.01128 (*id from memory, not re-checked*).
  - Lesson: constraining weights to decode into a discrete program keeps them readable.
- **Giannou, Rajput, Sohn, Lee, Lee, Papailiopoulos (2023)**, "Looped Transformers as Programmable Computers", ICML 2023, arXiv:2301.13196. [K]
  - Lesson: a fixed hand-built block, iterated, executes general programs. Time-stepping (one block = one Δt) is the natural way to hand-build dynamical models.
- **Siegelmann & Sontag (1995)**, "On the computational power of neural nets", *J. Comput. Syst. Sci.* 50(1):132–150. [K]
  - Lesson: recurrent nets with hand-set rational weights are Turing complete. Expressiveness is never the bottleneck; numerics and readability are.
- **Arora, Basu, Mianjy, Mukherjee (2018)**, "Understanding Deep Neural Networks with Rectified Linear Units", ICLR 2018, arXiv:1611.01491. [K]
  - Lesson: every continuous piecewise-linear function on ℝⁿ is exactly a ReLU net of depth ⌈log₂(n+1)⌉+1. In 1-D, one hidden layer suffices: f(x) = a + bx + Σₖ cₖ·ReLU(x − tₖ), with cₖ the slope change at knot tₖ.
- **He, Li, Xu, Zheng (2020)**, "ReLU Deep Neural Networks and Linear Finite Elements", *J. Comput. Math.* 38(3), arXiv:1807.03973. [K]
  - Lesson: the 1-D hat basis φₖ is exactly (ReLU(x−tₖ₋₁)/hₖ₋₁) − (1/hₖ₋₁ + 1/hₖ)·ReLU(x−tₖ) + ReLU(x−tₖ₊₁)/hₖ. Hand-writing a ReLU net then reduces to choosing finite-element nodal values. Multi-D hats need more depth.
- **Yarotsky (2017)**, "Error bounds for approximations with deep ReLU networks", *Neural Networks* 94:103–114, arXiv:1610.01145. [K]
  - Lesson: x² ≈ x − Σₘ gₘ(x)/4ᵐ, with gₘ the m-fold composition of a 3-ReLU "tooth" (hat) function. The error is 4^−(m+1), so depth grows as log(1/ε). Products follow from xy = ((x+y)² − x² − y²)/2.
- **Telgarsky (2016)**, "Benefits of depth in neural networks", COLT 2016, arXiv:1602.04485. [K]
  - Lesson: composing the tent map gives a sawtooth with 2ᵏ pieces from O(k) units. Depth buys oscillation cheaply, but it also amplifies coefficient errors exponentially, so a hand-written deep sawtooth is fragile.
- **Daubechies, DeVore, Foucart, Hanin, Petrova (2022)**, "Nonlinear Approximation and (Deep) ReLU Networks", *Constr. Approx.* 55:127–172, arXiv:1905.02199. [K]
  - Lesson: deep ReLU sawtooth constructions reproduce piecewise polynomials and spline-like approximants. Shallow spline sums are the robust, readable default.
- **Lin, Tegmark, Rolnick (2017)**, "Why does deep and cheap learning work so well?", *J. Stat. Phys.* 168:1223–1247, arXiv:1608.08225. [P] for the id; the 4-neuron detail is [S]
  - Lesson: a smooth activation σ with σ″(0) ≠ 0 approximates multiplication with 4 neurons, via second differences around 0. For tanh (σ″(0)=0) you must shift to a bias where the curvature is nonzero. Small input scale is needed for accuracy.
- **Balestriero & Baraniuk (2018)**, "A Spline Theory of Deep Networks", ICML 2018. [K]
  - Lesson: ReLU nets are max-affine splines. Reasoning about knots and regions is the native "programming model".
- **Liu et al. (2024/25)**, "KAN: Kolmogorov–Arnold Networks", ICLR 2025, arXiv:2404.19756. [K]
  - Lesson: placing explicit 1-D splines on edges makes each univariate nonlinearity inspectable and easy to set by hand; symbolic snapping is part of their workflow.
- **Cammarata, Goh, Carter, Voss, Schubert, Olah (2021)**, "Curve Circuits", *Distill*, doi:10.23915/distill.00024.006. [K]
  - Lesson: the authors hand-set the weights of a curve-detector circuit from their understanding, and it largely reproduced the trained circuit's behaviour. Hand-written weights work when the feature is fully understood.

### B.2 Mechanistic-interpretability results relevant to hand-writing weights

- **Elhage et al. (2021)**, "A Mathematical Framework for Transformer Circuits", Transformer Circuits Thread. [K]
  - Lesson: residual-stream thinking. Each block writes an additive correction into a shared stream, and zeroed blocks are the identity. This is the right mental model for a "baseline + residual" scientific net.
- **Olsson et al. (2022)**, "In-context Learning and Induction Heads", arXiv:2209.11895. [K]
  - Lesson: two-component circuits (previous-token head plus induction head) compose by stream addition. Small hand-readable circuits are compositions of single-purpose units, as in Astra's "4 numbers + 2 bits".
- **Nanda, Chan, Lieberum, Smith, Steinhardt (2023)**, "Progress Measures for Grokking via Mechanistic Interpretability", ICLR 2023, arXiv:2301.05217. [K]
  - Lesson: modular addition is solved with a few Fourier frequencies and trig identities (cos(a+b) from products of cos/sin). Periodic or angular variables, such as a contour's polar angle θ, should enter as (cos kθ, sin kθ) with a handful of k.
- **Zhong, Liu, Tegmark, Andreas (2023)**, "The Clock and the Pizza", NeurIPS 2023, arXiv:2306.17844. [K]
  - Lesson: the same function admits several distinct exact circuits. A hand-written solution is one valid point, not *the* mechanism.
- **Chughtai, Chan, Nanda (2023)**, "A Toy Model of Universality", ICML 2023, arXiv:2302.03025. [K]
  - Lesson: group operations are learned through representation theory. Pick the basis in which the operator is diagonal.
- **Wang et al. (2023)**, "Interpretability in the Wild: a Circuit for Indirect Object Identification in GPT-2 small", ICLR 2023, arXiv:2211.00593. [K]
  - Lesson: real trained circuits are redundant (backup heads). A hand-written net has no redundancy, so a single sign error is fatal; test every unit.

### B.3 LLM-written programs, equation discovery and symbolic regression

- **Romera-Paredes et al. (2024)**, "Mathematical discoveries from program search with large language models" (FunSearch), *Nature* 625:468–475, doi:10.1038/s41586-023-06924-6. [K]
  - Lesson: the LLM writes programs, and an automatic evaluator plus evolutionary selection does the optimising. The result is evidence for search with feedback, not for one-shot insight.
- **Novikov et al. (2025)**, "AlphaEvolve: A coding agent for scientific and algorithmic discovery", arXiv:2506.13131. https://arxiv.org/abs/2506.13131 [P]
  - Lesson: as FunSearch, at whole-file scale. It needs a machine-checkable evaluator. In the taxonomy it is category 2 + 4.
- **Shojaee, Meidani, Gupta, Barati Farimani, Reddy (2025)**, "LLM-SR: Scientific Equation Discovery via Programming with Large Language Models", ICLR 2025, arXiv:2404.18400. [K]
  - Lesson: the LLM proposes equation skeletons as Python functions with free parameters, and those parameters are fitted by a numerical optimiser. Structure comes from priors, coefficients from data: category 2 + 3.
- **Shojaee et al. (2025)**, "LLM-SRBench", ICML 2025, arXiv:2504.10415 (*id from memory*).
  - Lesson: transformed and synthetic problems are needed because LLMs memorise textbook equations. Results on "known law" benchmarks overstate discovery.
- **Ma, Wang, et al. (2024)**, "LLM and Simulation as Bilevel Optimizers" (SGA), ICML 2024, arXiv:2405.09783. https://arxiv.org/abs/2405.09783 [P]
  - Lesson: the LLM searches discrete structure (constitutive laws) and a differentiable simulator fits continuous parameters. This is the explicit bilevel version of "the LLM writes the form, the optimiser writes the numbers".
- **Yang et al. (2024)**, "Large Language Models as Optimizers" (OPRO), ICLR 2024, arXiv:2309.03409. [K]
  - Lesson: an LLM proposing literal numbers with score feedback is an optimiser (category 4). The paper includes linear-regression toy tasks.
- **Cranmer (2023)**, "Interpretable Machine Learning for Science with PySR and SymbolicRegression.jl", arXiv:2305.01582. [K]
  - Lesson: genetic programming plus constant optimisation. A non-LLM baseline that any "LLM wrote the law" claim should beat or match.
- **Udrescu & Tegmark (2020)**, "AI Feynman: A physics-inspired method for symbolic regression", *Sci. Adv.* 6(16):eaay2631, arXiv:1905.11481. [K]
  - Lesson: dimensional analysis first, then symmetry and separability tests. Reducing to dimensionless groups shrinks the search space dramatically.
- **Brunton, Proctor, Kutz (2016)**, "Discovering governing equations from data by sparse identification of nonlinear dynamical systems" (SINDy), *PNAS* 113(15):3932–3937, arXiv:1509.03580. [K]
  - Lesson: a small library of candidate terms plus sparsity. Dynamics are a few coefficients in the right basis, the same thing a hand-writer chooses.
- **Bakarji, Callaham, Brunton, Kutz (2022)**, "Dimensionally Consistent Learning with Buckingham Pi", *Nat. Comput. Sci.* 2:834–844, arXiv:2202.04643. [P]
  - Lesson: feed models dimensionless groups. They make inputs O(1) and remove whole families of spurious solutions.

### B.4 Physics-informed and hybrid models; survival models; operators

**Hybrid and residual models**
- **Psichogios & Ungar (1992)**, "A hybrid neural network–first principles approach to process modeling", *AIChE J.* 38(10):1499–1511. [K]
- **Thompson & Kramer (1994)**, "Modeling chemical processes using prior knowledge and neural networks", *AIChE J.* 40(8):1328–1340. [K]
- Lesson from both: grey-box models keep the mechanistic structure and let the network learn only an unknown sub-term (a rate or a correction). The network needs few parameters because the physics carries the bulk.
- **Yin, Le Guen, Dona, de Bézenac, Ayed, Thome, Gallinari (2021)**, "Augmenting Physical Models with Deep Networks for Complex Dynamics Forecasting" (APHYNITY), ICLR 2021, arXiv:2010.04456. [P]
  - Lesson: dynamics = F_physics + F_residual, with the **norm of the residual minimised**. The physical model explains as much as it can and the residual stays small and identifiable. This is the direct template for a D²-law correction.
- **Rackauckas et al. (2020)**, "Universal Differential Equations for Scientific Machine Learning", arXiv:2001.04385. [K]
  - Lesson: embed the network as a term inside a known ODE, then sparsify (SINDy) the learned term into a formula.
- **Chen, Rubanova, Bettencourt, Duvenaud (2018)**, "Neural Ordinary Differential Equations", NeurIPS 2018, arXiv:1806.07366. [K]
  - Lesson: parameterise the right-hand side, not the trajectory. A hand-written RHS correction integrates into consistent trajectories.
- **Raissi, Perdikaris, Karniadakis (2019)**, "Physics-informed neural networks", *J. Comput. Phys.* 378:686–707. [K]
  - Lesson: PINNs enforce physics through a training loss. Not applicable without training, but the residual of the governing equation is still a good **verification metric** for hand-written models.
- **Karniadakis et al. (2021)**, "Physics-informed machine learning", *Nat. Rev. Phys.* 3:422–440. [K]
  - Lesson: a taxonomy of how physics enters (observational, inductive or learning bias). Hand-writing is pure inductive bias.
- **He, Zhang, Ren, Sun (2016)**, "Deep Residual Learning", CVPR, arXiv:1512.03385.
- **Zhang, Dauphin, Ma (2019)**, "Fixup Initialization", ICLR, arXiv:1901.09321.
- **Bachlechner et al. (2020)**, "ReZero is All You Need", arXiv:2003.04887. [K]
- Lesson from the three: zero-initialised residual branches make the untrained network equal the identity, or the baseline. The same trick makes "all-zero residual = physical law" exact.
- **LeCun, Bottou, Orr, Müller (1998)**, "Efficient BackProp", in *Neural Networks: Tricks of the Trade*, LNCS 1524. [K]
  - Lesson: zero-mean, unit-scale inputs and tanh operating in its linear range. With fixed, hand-chosen scales the same reasoning keeps hand-set weights O(1).

**Monotone networks**
- **Sill (1998)**, "Monotonic Networks", NIPS 10. [K]
- **Daniels & Velikova (2010)**, "Monotone and Partially Monotone Neural Networks", *IEEE TNN* 21(6):906–917. [K]
- **Wehenkel & Louppe (2019)**, "Unconstrained Monotonic Neural Networks", NeurIPS, arXiv:1908.05164. [K]
- **Gupta et al. (2016)**, "Monotonic Calibrated Interpolated Look-Up Tables", *JMLR* 17(109). [K]
- Lesson from the four: nonnegative weights composed with monotone activations guarantee monotonicity by construction. Calibrated 1-D piecewise-linear "calibrators" with sorted keypoints are hand-settable and monotone.

**Discrete-time and cause-specific survival**
- **Prentice & Gloeckler (1978)**, "Regression analysis of grouped survival data with application to breast cancer data", *Biometrics* 34(1):57–67. [K]
  - Lesson: a discrete hazard with a complementary log-log link, hⱼ(x) = 1 − exp(−exp(αⱼ + xᵀβ)), is exactly the grouped proportional-hazards model. The αⱼ are the log integrated baseline hazard per interval.
- **Singer & Willett (1993)**, "It's About Time: Using Discrete-Time Survival Analysis to Study Duration and the Timing of Events", *J. Educ. Stat.* 18(2):155–195. [K]
- **Tutz & Schmid (2016)**, *Modeling Discrete Time-to-Event Data*, Springer. [K]
- Lesson from both: person-period expansion makes a discrete hazard a sequence of binary regressions. S(tⱼ) = Πₖ≤ⱼ(1 − hₖ) is automatically monotone and in [0,1].
- **Friedman (1982)**, "Piecewise exponential models for survival data with covariates", *Ann. Stat.* 10(1):101–113. [K]
- **Bender, Rügamer, Scheipl, Bischl (2020)**, "A General Machine Learning Framework for Survival Analysis", ECML-PKDD 2020, arXiv:2006.15442. [P]
- Lesson from both: a piecewise-constant hazard is a Poisson model with log-exposure offset. It handles censoring, time-varying covariates and competing risks through data layout, not architecture.
- **Gensheimer & Narasimhan (2019)**, "A scalable discrete-time survival model for neural networks" (Nnet-survival), *PeerJ* 7:e6257, arXiv:1805.00917. [K]
  - Lesson: one sigmoid output per time interval. A per-interval bias holds the baseline, so with all other weights zero the model is a covariate-free life table.
- **Kvamme & Borgan (2021)**, "Continuous and discrete-time survival prediction with neural networks", *Lifetime Data Anal.* 27(4):710–736, arXiv:1910.06724. [P]
  - Lesson: place interval cut points at quantiles of the event-time distribution, not equidistantly. Their PC-Hazard model is piecewise constant.
- **Lee, Zame, Yoon, van der Schaar (2018)**, "DeepHit: A Deep Learning Approach to Survival Analysis with Competing Risks", AAAI 2018. [K]
  - Lesson: a softmax over (cause × time) gives a joint event PMF. For hand-writing, a cause-specific discrete hazard per interval (softmax over K causes + "survive") is easier to set and read.
- **Prentice et al. (1978)**, "The analysis of failure times in the presence of competing risks", *Biometrics* 34(4):541–554. [K]
- **Fine & Gray (1999)**, "A proportional hazards model for the subdistribution of a competing risk", *JASA* 94:496–509. [K]
- Lesson from both: cause-specific hazards and subdistribution hazards answer different questions. State which one the hand-written model encodes.

**Operators and contours**
- **Li et al. (2021)**, "Fourier Neural Operator for Parametric PDEs", ICLR 2021, arXiv:2010.08895. [K]
  - Lesson: keeping a few low Fourier modes with a per-mode multiplier is a diagonal operator in spectral space. Hand-settable multipliers, such as exp(−κk²Δt) for smoothing or (1 + iωₖΔt) for drift, have physical meaning.
- **Lu, Jin, Pang, Zhang, Karniadakis (2021)**, "Learning nonlinear operators via DeepONet", *Nat. Mach. Intell.* 3:218–229, arXiv:1910.03193. [K]
  - Lesson: output = Σₖ bₖ(input)·tₖ(location). With the trunk fixed to a chosen basis (Fourier or Legendre), only branch coefficients remain, which is a small, hand-settable set.
- **Kovachki et al. (2023)**, "Neural Operator: Learning Maps Between Function Spaces", *JMLR* 24(89), arXiv:2108.08485. [K]
  - Lesson: discretisation invariance requires defining the operator on basis coefficients, not on a fixed grid.
- **Zahn & Roskies (1972)**, "Fourier Descriptors for Plane Closed Curves", *IEEE Trans. Comput.* C-21(3):269–281. [K]
- **Kuhl & Giardina (1982)**, "Elliptic Fourier features of a closed contour", *Comput. Graph. Image Process.* 18(3):236–258. [K]
- Lesson from both: a closed flame front is a few low-order (elliptic) Fourier coefficients. Mode 0 is the centroid, mode 1 the best-fit ellipse, and higher modes are wrinkles.
- **Richards (1990)**, "An elliptical growth model of forest fire fronts and its numerical solution", *Int. J. Numer. Methods Eng.* 30:1163–1179. [K]
  - Lesson: Huygens elliptical front propagation is a geometric baseline operator. A hand-written net should correct it, not replace it.

### B.5 Weight and model editing, as contrast

- **Meng, Bau, Andonian, Belinkov (2022)**, "Locating and Editing Factual Associations in GPT" (ROME), NeurIPS 2022, arXiv:2202.05262. [K]
  - Lesson: a rank-one edit is solved in closed form (constrained least squares) from causal tracing. It is a computed edit, category 2/3, not "the model wrote numbers".
- **Meng, Sharma, Andonian, Belinkov, Bau (2023)**, "Mass-Editing Memory in a Transformer" (MEMIT), ICLR 2023, arXiv:2210.07229. [K]
  - Lesson: the same, spread over layers. Edit specificity is measured, not assumed.
- **Hase, Bansal, Kim, Ghandeharioun (2023)**, "Does Localization Inform Editing?", NeurIPS 2023, arXiv:2301.04213. [K]
  - Lesson: where a fact is localised does not predict where editing works. A plausible mechanistic story is not proof that an edit is correct; behavioural tests are.
- **Ilharco et al. (2023)**, "Editing Models with Task Arithmetic", ICLR 2023, arXiv:2212.04089. [K]
  - Lesson: adding or subtracting fine-tuning deltas composes behaviours. The vectors come from training, so this is category 3. Astra's Exp 8 "subtract a looping direction" is structurally similar.
- **Vig et al. (2020)**, "Causal mediation analysis for interpreting neural NLP: the case of gender bias", arXiv:2004.12265. [K]
- **Zhang & Nanda (2024)**, "Towards Best Practices of Activation Patching", ICLR 2024, arXiv:2309.16042. [K]
- **Heimersheim & Nanda (2024)**, "How to use and interpret activation patching", arXiv:2404.15255. [K]
- Lesson from the three: patching is a diagnostic intervention on activations, not weights. Results depend on the corruption method and metric, so state them.
- **Schmidhuber (1992)**, "Learning to control fast-weight memories", *Neural Comput.* 4(1):131–139.
- **Schlag, Irie, Schmidhuber (2021)**, "Linear Transformers Are Secretly Fast Weight Programmers", ICML, arXiv:2102.11174. [K]
- Lesson from both: "weight programming" by a *trained* network. The MIT authors distinguish WSP from this line.
- **Ha, Dai, Le (2017)**, "HyperNetworks", ICLR 2017, arXiv:1609.09106. [K]
  - Lesson: a learned program generates weights, category 3.

### B.6 Adaptive evaluation (needed for category 4)

- **Dwork, Feldman, Hardt, Pitassi, Reingold, Roth (2015)**, "The reusable holdout: Preserving validity in adaptive data analysis", *Science* 349(6248):636–638. [K]
- **Blum & Hardt (2015)**, "The Ladder: A Reliable Leaderboard for Machine Learning Competitions", ICML 2015, arXiv:1502.04585. [K]
- Lesson from both: every look at a holdout score that changes your next choice spends that holdout's validity. Count the queries, and keep a test set no decision has seen.

---

## Part C. Taxonomy of evidence

| # | Category | What the LLM actually outputs | Who sets the numbers | Examples |
|---|---|---|---|---|
| 0 | *Reference row:* closed-form or human derivation | a construction proof | a human or mathematics | McCulloch–Pitts; Arora 2018; Yarotsky 2017; hat functions (He et al.); Cammarata curve circuits; Prentice–Gloeckler life-table biases; H₂ closed-form VQE angles (Kundu et al., derived *by the LLM*, but closed form) |
| **1** | **Weights chosen directly by an LLM (literal numbers)** | numeric literals (possibly compressed: strided assignments, scalings) with no execution | the LLM's reasoning | WSP Exp 5 (MinAtar) and Exp 4 (Kinetix), one reply, no tools; flecart's Opus 5.5 rerun; Khajavi Exps 4–5; Charité medical CNN (one reply, no tools); Kundu et al. VQC angles "executed exactly as written"; Nusrat & Nusrat repairs (median 3 parameters, no tools) |
| **2** | **Weights generated by an algorithm or program the LLM wrote** | code whose execution produces the weights | the program (deterministic) | WSP Exp 6 Go (336-line Python, 0 passes); Exp 8's direction subtraction (also row 4); csinva iWSP (also row 4); Tracr/ALTA (program written by a *human*, compiled by a compiler); ROME/MEMIT closed-form edits (no LLM) |
| **3** | **Weights trained conventionally under LLM supervision** | architecture, loss, equation skeleton or initialisation | an optimiser on data | LLM-SR (BFGS-fitted skeleton parameters); SGA (gradient inner loop); robotics Q/K least-squares fit (Gong et al.); Kundu warm starts (LLM init then optimiser); task arithmetic; HyperNetworks; the fine-tuning baselines in Nusrat and Charité |
| **4** | **Weights modified directly after inspecting development feedback** | edits (literal or code) chosen after seeing scores, rollouts or forward passes | the LLM, acting as a low-dimensional search procedure with a noisy oracle | WSP Exp 6 (10–50 passes), Exp 7 (attention tool), Exp 8 (50 passes on practice problems); Hu et al. 2611.09382; IWIBench rollout budgets; genomics Phase B (50 passes); csinva (validation score each round); OPRO; FunSearch/AlphaEvolve (rows 2+4) |

**Why these are not equivalent as evidence**

1. **Row 1, one-shot with no feedback and no data.** The only information path from the target behaviour to the weights is the LLM's prior knowledge plus the prompt.
   - Can support: "the model can **compile a specification it was given** (rules, source code, physical law) into a working network", and "the prior alone reaches score S".
   - Cannot support: claims about learning from data, or that the network is better than the specification. flecart shows that Breakout's score equals a 10-line simulator of the supplied source code.
   - Must control for: the specification's own performance (a scripted baseline), memorisation (flecart's prefix probe), and arithmetic and sign errors (QAOA 6/42; AUC 0.136).
   - This is the strongest evidence *about the LLM*, and the weakest *about the model's scientific validity*.
2. **Row 2, program-generated.** The numbers are as correct as the program; arithmetic errors vanish.
   - Can support: "the LLM can **design an algorithm that constructs** a network" (ordinary code generation).
   - Cannot support: the claim that the LLM "natively" handles weight values. If the program fits anything to data (least squares, regression), it silently becomes row 3.
   - Reproducibility is better: rerun the program and audit it.
3. **Row 3, conventional training under LLM direction.** Credit for the numbers belongs to the optimiser and the data.
   - Can support: "the LLM chose a good structure, inductive bias or initialisation" (LLM-SR, SGA).
   - Needs the standard ML protocol (train/validation/test split).
   - Says nothing about writing weights.
4. **Row 4, editing after feedback.** The LLM becomes an optimiser with very few, very informative queries. Selection on dev scores is fitting.
   - Can support: "an LLM-in-the-loop search improves held-out performance with budget B", **only** if the test set was never consulted and B is reported.
   - Cannot support: "the model understood the weights a priori".
   - Hu et al. (6/16 trajectories could not validate an improvement) and the Exp 8 vs Gemma contrast show the outcome depends on the feedback budget and on a pre-existing localised defect.
   - Adaptive-overfitting theory (Dwork et al. 2015; Blum & Hardt 2015) applies.

**Mixed cases must be labelled by the highest row they touch.** Exp 8 is rows 2 + 4. The robotics edit is rows 2 + 3 + 4. A hand-written network that was "tweaked after looking at validation error" is row 4, even if every number was typed by hand.

**For your project:** writing the D²-residual, hazard and contour networks with no optimiser and no generating program is row 1 (if an LLM chooses the numbers) or row 0 (if they come from your own derivation).
- Valid claims: "a specification-derived network attains X on held-out data", and "the physical structure plus N hand-chosen coefficients explains Y".
- If you look at dev error and change a coefficient, the result moves to row 4. Log every such look, freeze before touching test data, and report N_dev_queries.
- Do not describe the result as "learned", "trained", "discovered from data", or as evidence that an LLM "understands" combustion.

---

## Part D. Construction lessons for hand-written scientific networks

Each lesson is tied to a source from A or B.

1. **Make the zero network equal the physical baseline.**
   - Rule: write y = y_phys(x) + R(x), with R a residual branch whose output layer is zero by default.
   - For the droplet: d²(t) = d₀² − K·t in normalised form, s = (d/d₀)² = 1 − τ with τ = K·t/d₀². The net outputs s = (1 − τ) + R(τ, π₁, π₂…), so all-zero R gives the D²-law exactly.
   - Keep R's norm minimal (the APHYNITY principle) so the physical parameters stay identifiable.
   - Sources: APHYNITY (Yin et al. 2021); Fixup/ReZero; grey-box (Psichogios & Ungar 1992); UDEs. Replication evidence: editing a working network succeeds far more often than writing from zero (Kinetix; Nusrat median 3 parameters; Gemma "can't add missing ability").

2. **Feed fixed, physically normalised, dimensionless inputs, so every unit lives at O(1).**
   - Use groups such as τ = t/t_b, d/d₀, a Damköhler number, the transfer number B, or log-scaled pressure.
   - Fix the scales from physics or the dataset specification once, never from the evaluation set.
   - This makes hand-set slopes interpretable ("slope 0.1 per unit τ").
   - Sources: Bakarji et al. 2022 (Buckingham Pi); AI Feynman (dimensional reduction first); LeCun et al. 1998 (input scaling, tanh linear range).

3. **Use 1-D ReLU spline sums with knots at physically meaningful values; derive each coefficient as a slope change.**
   - f(x) = a + b·x + Σₖ cₖ·ReLU(x − tₖ), where cₖ = (slope right of tₖ) − (slope left of tₖ).
   - Put knots at the ignition or heat-up transient end, the onset of the quasi-steady regime, and near-extinction diameter. Use hat functions when you want local, non-interfering bumps with nodal values you can read.
   - Keep the number of knots small; every one must be justified.
   - Sources: Arora et al. 2018; He et al. 2020 (hat functions as ReLU combinations); Gupta et al. 2016 (calibrators with keypoints); KAN (per-edge 1-D splines).

4. **Separate "number units" from "gate units" by weight magnitude, and never let one drift into the other.**
   - A tanh unit is linear only for |z| ≲ 0.5 and a clean switch only for |z| ≳ 3.
   - Astra's Breakout net puts numbers through small weights (0.02/column) and logic through ±200 to ±1,000 weights with a −415 threshold. The margins (on at +5, off at −15) are explicit.
   - For smooth regime switches, use σ((x − x₀)/w) with x₀ and w physically meaningful (a transition location and width), not arbitrary large gains.
   - Sources: flecart Appendix D (A.1); McCulloch–Pitts.

5. **Prefer monotone, bounded bases and enforce known monotonicities by sign constraints.**
   - Droplet: s(τ) must decrease in τ.
   - Hazard: survival must be non-increasing, and extinction hazard should be non-decreasing in strain rate or heat loss if physics says so.
   - Use nonnegative weights into monotone activations, sorted spline keypoints, and outputs squashed by σ or softplus into their physical range: s ∈ [0,1], h ∈ (0,1), rates > 0.
   - Sources: Sill 1998; Daniels & Velikova 2010; Wehenkel & Louppe 2019; Gupta et al. 2016.

6. **Build the hazard model as a per-interval life table plus a linear predictor, so all-zero covariate weights give the baseline.**
   - Discrete hazard: hⱼ(x) = σ(αⱼ + xᵀβ), or the cloglog link 1 − exp(−exp(αⱼ + xᵀβ)) for grouped proportional hazards.
   - Set αⱼ from the empirical or physical baseline hazard in each interval. Then S(tⱼ) = Πₖ≤ⱼ(1 − hₖ) is automatically valid.
   - Cause-specific (extinction vs burnout vs other): a softmax over {survive, cause 1, …, cause K} per interval, with per-cause intercepts.
   - Place interval boundaries at quantiles or at physical time scales (e.g. multiples of t_b).
   - Say explicitly whether you encode cause-specific hazards or subdistribution (Fine–Gray) hazards.
   - Sources: Prentice & Gloeckler 1978; Singer & Willett 1993; Gensheimer & Narasimhan 2019; Kvamme & Borgan 2021; DeepHit (Lee et al. 2018); Friedman 1982 / Bender et al. 2020 (piecewise exponential); Prentice et al. 1978; Fine & Gray 1999.

7. **For contours, work in a spectral basis where the operator is near diagonal, keep few modes, and correct a geometric baseline.**
   - Represent the front by (elliptic) Fourier descriptors, or by r(θ) with (cos kθ, sin kθ), k ≤ K small.
   - The baseline step is uniform or elliptical (Huygens/Richards) advance.
   - The hand-set corrections are per-mode multipliers: damping exp(−κk²Δt) for curvature smoothing, growth for instability modes, plus a mode-1 shift for wind or flow.
   - Equivalent to an FNO layer with fixed, readable weights, or a DeepONet with a fixed Fourier trunk.
   - Sources: Zahn & Roskies 1972; Kuhl & Giardina 1982; Li et al. 2021 (FNO mode truncation); Lu et al. 2021 (DeepONet); Richards 1990; Nanda et al. 2023 (Fourier features for periodic variables).

8. **Avoid hand-building multiplications; change variables instead.** If you must multiply, use the known constructions, and test them.
   - Log-transform positive quantities so that power laws (d ∝ t^½, K ∝ ln(1+B)) become linear, and products become sums.
   - Otherwise:
     - ReLU: x² via the Yarotsky sawtooth series (error 4^−(m+1) with m teeth), then the polarisation identity for products.
     - Smooth activations: the Lin–Tegmark–Rolnick second-difference gate, with small input scale and a bias where σ″ ≠ 0.
   - Deep sawtooth compositions amplify coefficient errors, so prefer shallow forms.
   - Sources: Yarotsky 2017; Telgarsky 2016; Lin, Tegmark, Rolnick 2017; AI Feynman / Buckingham Pi.

9. **Keep the count of free hand-chosen numbers small and each one meaningful; leave the rest exactly zero.**
   - Astra's Breakout used 815/20,486 nonzeros in 92 statements.
   - LLM-written quantum circuits fell to chance beyond about 30 opaque weights while interpretable ones (VQE/QAOA) worked.
   - Opus could not finish the larger compilation within 128k tokens.
   - Treat "parameters you can justify in one sentence each" as the budget.
   - Sources: flecart A.1; Kundu et al. 2611.11537; SINDy (sparsity in the right basis).

10. **Write the specification as a short reference program first, then compile it; check the network against the reference to tolerance.**
    - Before writing weights, write the intended function in ~10 lines of numpy: the D²-law plus correction, the life-table hazard, the per-mode multipliers.
    - Then assert max |net(x) − ref(x)| < ε on a dense grid, including edge cases (τ → 1, empty intervals, k = 0 mode).
    - This is how compiled networks are validated (RASP/Tracr), how flecart decoded Astra's net into code, and how flecart verified its applier bit for bit.
    - Note: the checking harness is not a weight-generating program, so it does not move you out of row 1.
    - Sources: Weiss et al. 2021; Lindner et al. 2023; flecart (applier validation, Appendix D).

11. **Write explicit tests for signs, conventions and units.** Treat a systematic inversion as the most likely failure.
    - Check that ∂s/∂τ < 0, that hazards rise with the physical driver, that the contour's orientation and θ convention are right, and that a zero residual reproduces the D²-law exactly.
    - Sources: Kundu et al. (6/42 QAOA replies with reversed signs); Charité (Astra AUC 0.136 below chance, a systematic error); Wang et al. 2023 (no redundancy in a hand-written circuit).

12. **Treat every development-set look as an optimisation step: budget it, log it, and keep the baseline unless the gain beats noise.**
    - Freeze a test split nobody has looked at.
    - Record each dev evaluation and each coefficient change, and report the count.
    - Accept a change only if it beats the physics baseline by more than the dev-set standard error. Otherwise keep the baseline, as Hu et al.'s agents did in 6/16 trajectories.
    - Sources: Dwork et al. 2015; Blum & Hardt 2015; Hu et al. 2611.09382; WSP Exp 8 vs Gemma; Honnibal's and Willison's separation of no-feedback from feedback results.

13. **Benchmark against the specification itself and against cheap fitted baselines, not only against complex learners.**
    - Report: the physics-only baseline (D²-law; constant hazard; uniform or elliptical spread), a cheaply fitted baseline (least-squares spline or logistic hazard on the same data), and the hand-written network.
    - flecart's key finding was that the 10-line scripted program equals the network. The Charité paper's key finding was that 256 labels plus Adam beat one-shot weights.
    - Sources: flecart Appendix D; Lindholz et al. 2611.12066; PySR as a symbolic baseline.

14. **A plausible mechanistic story does not certify correctness; validate behaviour.**
    - Sources: Hase et al. 2023 (localisation does not predict editing success); Nusrat & Nusrat (could not certify healthy networks: 2/5); Zhong et al. 2023 (multiple valid circuits).

---

## What I could not verify

- **Honnibal's critique:** read through ai-tldr.dev only.
- **Goedecke's critique and the HN thread:** not found.
- **"Krenzler" and "Zeng Jiajun" replications:** secondary reports only, no primary links.
- **Manas Karra's Magenta post and the Hugging Face demo:** not opened.
- **The csinva alphaXiv preprint:** not located.
- **Full PDFs** of arXiv 2611.05341, 2611.08647, 2611.09382, 2611.11537, 2611.12066 and 2611.13758 were not read. Numbers are from abstracts or landing pages.
- **The MIT post** was read through a summarising fetch, not line by line.
- **Ids given from memory and not re-checked this session:**
  - Friedman, Wettig, Chen, arXiv:2306.01128
  - LLM-SRBench, arXiv:2504.10415
  - The Lin–Tegmark–Rolnick 4-neuron multiplication detail (secondary).
- Other [K] items are standard citations given from bibliographic knowledge and were not re-fetched.
