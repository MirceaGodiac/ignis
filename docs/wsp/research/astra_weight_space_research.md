# Astra research: literal weight-space programming and scientific model growth

Research note for the second model author, 14 November 2026. This is a source ledger and design synthesis, not a validation report. It asks what adjacent work establishes about writing neural parameters, and what those results imply for extending IGNIS models.

## Scope and evidence standard

The key question is narrower than whether neural networks can represent useful functions: what is an authored parameter when the forward is explicit, every stored number is written by an author, and no fitting, search, optimizer, or program chooses values?

I use **literal WSP** for the IGNIS rule: an author writes literal set/set-box/fill-box values against a ModelSpec. This differs from (1) a compiler that generates weights from a source program, (2) a script that computes a grid or weight table, (3) conventional training, and (4) edits chosen after observing development feedback. All can be useful, but the information provenance differs. A source is labelled by how much I read. Abstract-only sources support only the stated abstract-level claim. Full paper means main text and relevant arguments inspected, not independent reproduction of every result.

This note adds no parameters and proposes no candidate. No protected validation or test material, run-level validation outcomes, holdout material, or published per-test outcomes of the modelled series was consulted. The synthesis does not use IGNIS scores to choose a design.

## Summary

Literal WSP is strongest when it writes a **small scientific program in a fixed numerical architecture**: mechanism, regimes, units, transformations and uncertainty scenarios are explicit; numeric values are sparse, typed constants with individual provenance. Expressive power should mostly live in the forward's scientific structure, not an opaque tensor. A table may contain many literal values when those values are externally sourced properties or instrument calibration; parameter count alone does not measure freedom to fit outcomes.

RASP, Tracr and ALTA show that algorithms can be compiled into transformer weights and make their mechanisms inspectable. They do not establish that compiler-generated values meet IGNIS's literal-authoring rule. A compiler is a useful reference and verification aid; the released values still must be literal under the present protocol.

The more recent WSP papers reinforce a practical limit: authors can often encode a known, compact algorithm in weights, but performance falls when the network's weights become opaque, brittle, or more complex than the model can reason about. Mathematical and safety constraints can be checked exactly when a formal model exists; such verification proves properties of the encoded system, not that its scientific assumptions match the experiment.

## Source ledger

### Named WSP and its subsequent primary work

| Primary source | Reading depth | Supported takeaway | Unresolved / boundary |
|---|---|---|---|
| Takehi, Teoh, Zhang & Khattab, [Sparks of Weight-Space Programming](https://rikiyat.github.io/blog/weight-space-programming/) (2 Nov 2026) | Read author post sections on setup, one-shot MinAtar writing, Go, and Qwen/Gemma editing; checked linked experiment-log repository description. No independent replay. | The cleanest direct-writing evidence is small one-shot networks: model receives task rules/source, architecture and I/O, no data/gradients/rollouts, and emits textual weights. Results vary by task. Later Go uses Python-generated weights and feedback; Qwen editing uses repeated forward passes. These later experiments are different procedures. The post reports that localized loop suppression helped Qwen, but edits did not add general arithmetic skill to Gemma. | A network score does not establish a broader understanding or scientific generalization. Prompt-provided rules may already specify a short algorithm. No peer-reviewed full paper or independent replay was examined. |
| Same authors, [released experiment logs](https://github.com/RikiyaT/lm_understand-logs) | Checked repository description and its purpose; did not enumerate or replay every artifact. | Prompts, traces, code, weights and evaluation artifacts can make the information path auditable. | Repository release is not an independent replication. |
| Gross, [Weight-Space Programming of Policies and Their Verification via Model Checking](https://arxiv.org/abs/2611.11604) (8 Nov 2026) | Read full arXiv paper sections covering formal setup, creation/self-check, experiments, and discussion; not independently rerun. | Combines literal policy weights with probabilistic model checking over an MDP and PCTL requirements. The checker computes property probability rather than estimating it by simulation; author reports optimal probability in 18/18 initial runs before feedback across three environments. Crucially, the paper separately checks that the written network's behavior matches the author's intended actions, catching arithmetic/transposition mistakes before checking policy quality. It also shows one data repair improved accuracy but broke a safety property, and feedback restored it. | A verified MDP policy is only as valid as its MDP, reward/state abstraction, and formal property. This is an excellent separation of encoding correctness from requirement satisfaction; it does not verify physical-model adequacy. Later feedback changes provenance from one-shot. |
| Lehmann, [Compiling Known Rules into Compact Neural Agents: WSP on Unseen Games](https://arxiv.org/abs/2611.16981) (12 Nov 2026) | Read full primary arXiv HTML: abstract, preregistered protocol, comparison arms, main findings and fragility discussion; did not reproduce private games or all supplementary analyses. | Tests 20 private game families and held-out levels/renderings/start states. Written networks are program-like and can generalize, but the fixed 57k-parameter actor reaches at most 56% held-out-level success with feedback while the same writers' Python players reach 97–100%; NumPy players under comparable caps reach 91% versus networks' 79%. Networks are fragile and PPO fine-tuning erases most success. Noise-robustness prompts help some writers. The paper explicitly asks whether weights add anything over an ordinary program or learned network. | This preprint's benchmarks are games, not science. Results emphasize architecture and network format costs, not the scientific value of literal constants. The authors report that prompts/source access were controlled, but sandbox enforcement and reproducibility need ordinary independent audit. |
| Kundu, Ghosh & Honavar, [WSP of Variational Quantum Circuits](https://arxiv.org/abs/2611.11537) (8 Nov 2026) | Read full primary paper's abstract, method, setup and results/discussion sections; did not rerun the simulator. | LLM-authored angles can work when parameters have physical/geometric meaning, including exact closed-form H2 and simple QAOA structures; performance collapses toward chance in opaque fixed circuits above roughly 30 weights. Six of 42 QAOA trials reversed all mixer-angle signs, demonstrating convention errors. Their conclusion centers interpretability and meaningful parameter roles. | Quantum circuit parameters differ from neural tensors and some results use circuit-specific analytic knowledge. The evidence supports small interpretable parameter spaces, not arbitrary large handwritten tensors. |
| Lindholz et al., [WSP for Medical Imaging: Do We Still Need Training Data?](https://arxiv.org/abs/2611.12066) (9 Nov 2026) | Read primary abstract and arXiv metadata; not the full paper. | The authors compare one-shot LLM-written CNN parameters against zero-shot MedSigLIP and the same architecture trained on labels across seven tasks. Abstract claims functional classifiers but does not support replacing training data. | Abstract-only. Do not use numerical claims or detailed protocol until full paper is reviewed. |
| Hu et al., [Exploring WSP: Capabilities, Generalization, and Limitations](https://arxiv.org/abs/2611.09382) | Abstract/landing-page level via existing local ledger; not full paper in this research pass. | Existing ledger reports a set of editing trajectories with variable outcomes and some trajectories left unchanged because improvements could not be established. | Treat quantitative and procedural details as provisional until full paper and artifacts are rechecked. |
| von Csefalvay et al., [IWIBench](https://arxiv.org/abs/2611.08647) | Abstract/landing-page level via existing local ledger. | Existing ledger describes uneven success across edit objectives; direct editing ability depends on task family. | Abstract-only; not a basis for detailed claims here. |

The term WSP must not collapse these procedures. Record which facts the author saw, whether outcomes were present, whether code ran, whether it selected numbers, and how many feedback looks occurred. Label the release by the most data-dependent step, not by its final file format.

### Programs compiled into weights and RASP successors

| Primary source | Reading depth | Supported takeaway | Limit for IGNIS |
|---|---|---|---|
| Weiss, Goldberg & Yahav, [Thinking Like Transformers](https://proceedings.mlr.press/v139/weiss21a.html), ICML 2021 | Read full paper, including RASP primitives, constructions, and layer/head discussion. | RASP maps transformer attention and position-wise computation into symbolic operations. Short programs for histograms, sorting and Dyck languages clarify information flow and resource requirements. | It is a programming-language model of transformers, not evidence that an LLM can type every coefficient. |
| Lindner et al., [Tracr: Compiled Transformers as a Laboratory for Interpretability](https://arxiv.org/abs/2301.05062), NeurIPS 2023 | Read full paper and compilation pipeline; did not run compiler. | Tracr turns restricted RASP into standard transformer weights through an intermediate representation. Known mechanisms offer ground truth for interpretability studies, including superposition. It shows how a source algorithm can make a large tensor readable. | Weights are compiler outputs. The compiler supports a restricted language/architecture; this establishes constructibility, not literal hand-authoring. |
| Shaw et al., [ALTA: Compiler-Based Analysis of Transformers](https://arxiv.org/abs/2410.18077), ICLR 2025 | Read primary abstract and OpenReview PDF introduction/method overview; not every proof. | Extends RASP/Tracr with loops and Universal Transformers and constructs parity, addition, and SCAN solutions. Expressibility and successful training are distinct: an algorithm can be representable without being induced by a training regime. | Compiler-generated, not literal. Quantitative/proof claims remain at overview depth. |
| Strobl et al., [Transformers as Transducers](https://aclanthology.org/2025.tacl-1.9/), TACL 2025 | Read full paper abstract, formal setup and theorem/results statements; did not reproduce proofs. | RASP variants characterize sequence-to-sequence mappings; allowed attention and positional primitives change the formal class of computable transformations. | Expressivity results do not imply useful, stable scientific solvers. |
| Huang et al., [Discovering Interpretable Algorithms by Decompiling Transformers to RASP](https://arxiv.org/abs/2602.08857) | Read abstract and project page method summary, not full text. | In small trained algorithmic models, D-RASP can reparameterize a transformer, prune causally irrelevant parts, and sometimes recover compact interpretable programs. | New preprint and abstract-level reading; not a general inverse for arbitrary weights or a validator of physical meaning. |

**Boundary:** a compiler, generated table, or loop that writes a tensor is programmatic weight generation, even if the source is short and transparent. It may be more reproducible than hand transcription, but it answers a different question. Under IGNIS's present rule it may serve as a reference oracle; it cannot silently become the source of released literals.

### Constructive expressivity: piecewise functions and products

| Primary source | Reading depth | Supported takeaway | Limit |
|---|---|---|---|
| Arora et al., [Understanding Deep Neural Networks with ReLU](https://arxiv.org/abs/1611.01491), ICLR 2018 | Read full primary paper's key constructions/results. | ReLU networks represent continuous piecewise-affine functions; explicit constructions relate depth/width to linear regions. A hinge sum can directly encode slope changes at named knots. | Existence of a representation does not tell us which physical knots or coefficients are correct. |
| Yarotsky, [Error Bounds for Approximations with Deep ReLU Networks](https://arxiv.org/abs/1610.01145), 2017 | Read full paper construction and bounds relevant to squaring/multiplication. | Deep ReLU networks efficiently approximate smooth functions; square approximation plus polarization builds multiplication. | Approximation requires a bounded domain and error allowance; using a neural multiplier where the forward can multiply exactly adds avoidable error. |
| Telgarsky, [Neural Networks and Rational Functions](https://proceedings.mlr.press/v70/telgarsky17a.html) | Read relevant primary construction sections, not all proofs. | Repeated piecewise-linear compositions can approximate nonlinear functions with compact networks. | Expressivity is not a reason to encode known physics in opaque weights. |
| Yang, [Transformers Can Do Arithmetic with the Right Embeddings](https://arxiv.org/abs/2405.17399) | Abstract only. | Input representation can be central to arithmetic generalization; coordinates are part of the inductive bias. | Training study, not literal authored scientific constants. |

For model growth, use three regimes: exact physical law; explicit piecewise law with physically named knots and bounded behavior; approximation block only where an approximation is justified and carries a domain/error bound. A run-indexed outcome lookup is memorization; a sourced material-property table is different, provided its origin and interpolation are fixed independently of outcomes.

### Interpretability, causal interventions, editing

| Primary source | Reading depth | Supported takeaway | Limit |
|---|---|---|---|
| Elhage et al., [A Mathematical Framework for Transformer Circuits](https://transformer-circuits.pub/2021/framework/index.html) | Read full online framework's linearized circuits, QK/OV analysis, superposition, and scope notes. | Matrices can sometimes be factored into legible subcomputations; multi-head composition implements simple algorithms. | A mechanistic story is a hypothesis; small/local insights do not establish whole-model understanding. |
| Olsson et al., [In-Context Learning and Induction Heads](https://arxiv.org/abs/2209.11895) | Read abstract and paper sections on evidence strength. | Induction heads implement repeated-pattern copying. Evidence is causal in small attention-only models and more correlational in larger models. | Language-model mechanism, not combustion evidence. |
| Meng et al., [Locating and Editing Factual Associations in GPT (ROME)](https://arxiv.org/abs/2202.05262), NeurIPS 2022 | Read full paper methods, causal-tracing setup and evaluation. | Interventions can test whether activations causally affect a prediction; ROME then makes a computed rank-one edit and checks generalization/specificity. | ROME computes the edit from data/activations, so it is not literal WSP. An editable network can be edited toward a false assertion. |
| Hase et al., [Does Localization Inform Editing?](https://arxiv.org/abs/2301.04213), NeurIPS 2023 | Read abstract and conclusion summary only. | Localization alone does not establish where an edit will succeed. | Quantitative details not used without full review. |

For a handwritten network, unit labels should be tested, not admired: construct inputs that isolate a named role, trace its effect, and ablate it. Check signs, units, monotonicities, conservation, and limiting behavior. Behavior can support a mechanism without proving that it is the unique mechanism.

### Weight-space symmetries and checkability

| Primary source | Reading depth | Supported takeaway | Limit |
|---|---|---|---|
| Phuong & Lampert, [Functional vs. Parametric Equivalence of ReLU Networks](https://openreview.net/forum?id=Bylx-TNKvH), ICLR 2020 | Read full paper's theorem and definitions. | Hidden-unit permutations and positive rescaling with inverse output rescaling preserve the represented function. For specified architectures/conditions, these characterize equivalences. | Not a universal canonical form. Two different tensors can encode the same function. |
| Ainsworth, Hayase & Srinivasa, [Git Re-Basin](https://arxiv.org/abs/2209.04836), ICLR 2023 | Read full paper's matching methods, results, limitations and counterexample. | Permutation alignment can make trained networks easier to interpolate/merge in studied cases; connectivity is not universal and depends on training/capacity. | This is not a canonical parameterization for hand-authored scientific models. |
| Stock & Gribonval, [An Embedding of ReLU Networks and an Analysis of Their Identifiability](https://arxiv.org/abs/2107.09370) | Abstract and theorem summary only. | An invariant embedding can quotient scale equivalence and support local identifiability under conditions. | Theoretical and assumption-specific. |
| Grigsby, Lindsey & Rolnick, [Hidden Symmetries of ReLU Networks](https://proceedings.mlr.press/v202/grigsby23a.html), ICML 2023 | Read primary abstract and theorem overview. | Besides permutation/scaling, hidden symmetries can arise depending on architecture and parameters. | Does not imply no parameters can be meaningful, but cautions against treating neuron order as unique semantics. |

Use semantic canonicalization: names keyed to scientific role and units, stable branch order, value-level source locators, fixed scale conventions, and function-level neutral-extension tests. This simplifies review without claiming parameter identifiability.

### Expert judgment, priors, and scientific grey boxes

| Primary source | Reading depth | Supported takeaway | Limit |
|---|---|---|---|
| Cooke-style structured expert judgment, [review](https://www.ncbi.nlm.nih.gov/books/NBK571048/) and [classical-model review](https://www.sciencedirect.com/science/article/pii/S0951832007000944) | Read descriptions of seed questions, calibration, informativeness and pooling; Cooke's 1991 book not read. | When inputs are not empirically identified, expert distributions can be elicited and pooled explicitly. Seed questions assess calibration/informativeness rather than assuming expertise. | Requires suitable seed questions and a defensible elicitation design. No combustion expert panel is established here. |
| Zhu & Griffiths, [Eliciting LLM Priors with Iterated In-Context Learning](https://arxiv.org/abs/2406.01860) | Read full primary paper's method and main results. | Iterated queries can recover distributions that qualitatively align with human priors on selected cognitive tasks; an LLM prior can be examined rather than reduced to one point answer. | GPT-4 on selected tasks does not establish calibrated priors for NASA combustion or a scientific property. |
| Yin et al., [APHYNITY](https://arxiv.org/abs/2010.04456) | Read full paper's formulation and decomposition discussion, not reproduction. | Hybrid dynamics split known physical components from a learned residual constrained to explain only the remainder. | It learns the residual from data. Transfer the separation principle, not the fitting step. |
| Buckingham, [On Physically Similar Systems](https://doi.org/10.1098/rsta.1914.0008) | Foundational citation/record, not full derivation. | Dimensional analysis reduces dependence to dimensionless groups when variables and similarity assumptions are correct. | It cannot find omitted variables or validate similarity. |
| Brunton, Proctor & Kutz, [SINDy](https://www.pnas.org/doi/10.1073/pnas.1517384113), PNAS 2016 | Read abstract and method summary. | Dynamics can be organized as sparse combinations of a declared mechanism library. | Coefficients are selected by sparse regression, which violates literal WSP. Reuse the library idea, not learned coefficients. |

An LLM is a hypothesis generator and compiler, not provenance. A defensible use is to list candidate mechanisms/ranges, trace each to primary literature or derivation, and then state the literal choice and uncertainty. Two LLMs are not independent experts simply because their answers differ; training corpora overlap. A model written by a fresh author with only theory and class A/B sources has a cleaner information path, but it still needs independent checking.

Dimensional analysis makes scales and transfer assumptions explicit. A resolved solver may have much code and few outcome-directed parameters, but closures, boundary conditions, grid, time step and constants need provenance. Test conservation, positivity, limiting analytic cases and numerical error separately. Agreement between solvers is a numerical check, not empirical validity.

### Forecast uncertainty, proper scores, description length

| Primary source | Reading depth | Supported takeaway | Limit |
|---|---|---|---|
| Gneiting & Raftery, [Strictly Proper Scoring Rules](https://doi.org/10.1198/016214506000001437), JASA 2007 | Read full paper, including CRPS, Brier/log score, interval and energy-score sections. | Proper scores reward honest predictive distributions in expectation; CRPS/Brier assess more than point accuracy. Sharpness is desirable subject to calibration. | A proper score cannot guarantee calibration in unseen regimes or undo adaptive selection. |
| Grünwald & Roos, [Minimum Description Length Revisited](https://arxiv.org/abs/1908.08484) | Read full overview introduction and two-part-code explanation; not every estimator. | MDL frames generalization as total description length of model plus unexplained data. | Tensor byte count is not an MDL score. The coding grammar/prior must be fixed independently of outcomes. |
| Dwork et al., [Reusable Holdout](https://doi.org/10.1126/science.aaa9375), and Blum & Hardt, [The Ladder](https://arxiv.org/abs/1502.04585) | Read abstracts and method/conclusion summaries. | Adaptive decisions after holdout queries consume holdout validity; the look history matters. | No quantitative look budget derived here; local protocol governs. |

When calibration outcomes are unavailable, separate physical scenario disagreement, run variability, measurement error, property uncertainty, and numerical approximation. Physical bounds, literature ranges and sensitivity envelopes can define provisional uncertainty, but label assumed widths. Equal committee weights mean a transparent convention, not calibrated truth probabilities. A linear pool represents mixture over member explanations; averaging logits or multiplying densities represents different assumptions and can be overconfident. Keep member forecasts visible.

For event forecasts, define the estimand first. Cause-specific hazards concern the instantaneous event rate among those still at risk; cumulative incidence concerns chance of a cause by time t with competing causes. A planned operator stop is not flame extinction. The target definition is scientific, not a network architecture detail.

A better complexity ledger than nonzero count has four columns: (1) sourced exogenous values (properties, geometry, instrument calibration); (2) mechanism choices and regimes; (3) outcome-informed thresholds, widths, scenario weights and revisions; (4) evaluation choices (targets, censoring, exclusions, metric, stopping rule). A large handbook table may add transcription risk without many fit directions. One threshold chosen after many peeks may be a large adaptive choice despite one scalar. MDL is useful only after the model grammar/cost is preregistered.

## First-principles implications

**What is a parameter?** A value used in the forward, but not always a free parameter. A handbook viscosity, an instrument calibration, a literature constant, a scenario weight and a train-informed threshold carry different evidence. Each entry should state physical/statistical meaning, unit/domain, source and chooser, what outcomes were seen, plausible range, and a test that would expose a sign/unit/limit error.

**Where should expressive power sit?** First in explicit state and conservation laws; then boundary conditions, experimental policy, property data and instrument response; then a small set of physically named regime mechanisms; finally in a bounded residual basis and scenario uncertainty. Dense opaque units should not be the main scientific explanation. A model may have thousands of sourced constants but only a few decisions that can chase outcomes.

**What destroys transfer?** Not “many parameters” by itself. Transfer is threatened by hidden dependence on cohort-specific run identities, thresholds or scales sharpened after seeing outcomes, wrong prospective inputs, misspecified sensor/censoring processes, and a physical law applied outside its regime. The claim that stable mechanisms transfer is a strong research hypothesis, not a theorem; five generations cannot establish a general law.

**How can the model grow?** Build in modules: explicit state transitions; shared sourced property registry; separate geometry, planned intervention and sensor process; dimensionless regime dependence; competing mechanism members; resolved transport only where a reduced law has a known failure. Share constants across families to expose common provenance and correlated error, not to claim they are correct.

**Where does handwritten end?** A literal author types each released value with a reason. A program that calculates, estimates, searches, samples, averages, or chooses a value makes it program-selected. Deterministic byte-for-byte copying of an existing literal patch can preserve its provenance if recorded; arithmetic on it is a new choice. Handbook transcription is not fitting, but needs edition/page/unit provenance. A calculator can verify arithmetic; adopting its output should be described as computed, not as direct physical judgment.

**How do two authors combine?** Preserve both releases and their exposure declarations. Compare semantic parameters only when meaning and units match. Keep distinct mechanism hypotheses as members of a committee; equal weights are a convention unless elicited. Do not average tensors just because shapes align. An independent author should check derivations and shared constants. Comparative superiority requires a preregistered evaluation; retaining competing mechanistic hypotheses does not.

## Recommendations for extensions

1. Keep a compact reference equation and compare the forward to it over boundary cases; the reference/test program must not write release weights.
2. Normalize into fixed dimensionless coordinates; knots should encode documented regimes, never convenient train quantiles.
3. Add residual blocks that are neutral by default and preserve predecessor predictions exactly.
4. Use shared, unit-tested property blocks with table provenance, interpolation rule and uncertainty.
5. Name every unit's role and physical input/output; test signs, limits and effects by ablation.
6. Enforce positivity, monotonicity, simplex membership and conservation structurally, then test over the full allowed domain.
7. Separate latent event, sensor observability, censoring and operator intervention.
8. Report uncertainty origins separately; call widths assumed or literature-derived until evaluated calibration supports stronger language.
9. Preregister target, cohort, statistic and decision rule before any development look; log all iterations.
10. Compare function behavior and semantic roles, not raw tensors alone; symmetries make parameter vectors non-unique.
11. Label equal committee weights as convention; use expert elicitation only with explicit distributions and calibration design.
12. Spend new parameters on mechanisms expected to persist across conditions, and isolate local train-informed corrections.

## Unresolved checks

* MIT's WSP is an author post and the logs were not independently replayed. The sharp comparison is with the short program/specification supplied to the model.
* Recent WSP preprints (especially 2611.12066 and the 2026 RASP-decompilation work) need full-text review before quantitative use.
* Symmetry theory gives no single canonical coordinate for arbitrary network architecture/weights.
* LLM prior elicitation does not produce a verified combustion prior; a separate expert-elicitation study would be needed.
* No preregistered MDL grammar exists for the IGNIS model assumptions.
* Proper scores do not make worn development data fresh or establish validity under distribution shift.
* Family-specific combustion mechanisms remain in the dossier's physics and verification files; this note does not rederive them or consult protected per-test series.

**Status:** primary-source research synthesized. Several readings are explicitly abstract/overview-only. No model parameters were changed or proposed as a candidate release.
