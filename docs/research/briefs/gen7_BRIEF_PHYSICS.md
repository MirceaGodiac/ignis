# Brief for a physicist-author: a zero-shot law for one NASA investigation

Repository: D:\dev\SpaceApps-Ardealu, branch feat/handwritten-science. Read `AGENTS.md` ("Literal weight-space
programming") and `docs/wsp/PROTOCOL.md` sections 2, 4 and 12.1 to 12.3.

## The goal of the whole effort
The owner wants a hand-written model with a scored forecast for EVERY one of NASA's 19 flight combustion
investigations. Every model parameter here is a number an author model types, with its physical reason and
source; nothing is fitted. You are the author for the investigation(s) named in your task. Your law will be run
blind on tests nobody has read, so it must be written BEFORE any outcome is seen: zero-shot.

## Zero-shot rule (strict)
Do NOT open anything under `runs/`, `data/`, `holdout/`, and no file of `docs/wsp/feedback/` that reports
outcomes of your investigation. You MAY read: the investigation's card(s) in `docs/wsp/atlas/cards/` and its
section in `docs/wsp/atlas/INVESTIGATIONS.md` (apparatus, conditions flown, what was recorded), the feasibility
note of the data engineer `docs/wsp/feedback/<slug>_feasibility.md` ONLY for its sections on targets, inputs and
units (if it exists yet; it must not be used for baseline scores or anything computed from outcomes), the
earlier author's drafts as a colleague's work to critique and not to copy
(`docs/wsp/dossier/ASTRA_FIRST_PRINCIPLES_ALL_NASA.md`, `docs/wsp/dossier/ASTRA_COVERAGE_19.md`, the kernel file
named in your task), and the published literature (web search and fetch; cite what you use; a number taken from
a paper about this very experiment is class L and must be marked).

## Deliver `docs/wsp/research/phys_<slug>_w0.md`
1. **Mechanism from first principles**: what physically decides the outcome, in dimensionless groups where you
   can, so the statement could serve another apparatus.
2. **The forecast target(s)**: take them from the engineer's feasibility note if it exists; otherwise define
   them yourself from the card (a quantity that matters for fire safety and was recorded for most tests) and say
   so. For each: unit, what a forecast may know.
3. **The law**: a closed form for the median and for the spread of each target (or for the probability of an
   event against time) in terms of pre-test conditions. The spread must state how unsure the theory is.
4. **Every constant as a literal number** with origin class (P property table, T theory, L published
   experiment, J judgement) and its uncertainty; two or three significant figures.
5. **Registered expectations**: a table of forecasts on a grid of the conditions flown, written before any
   comparison.
6. **What would show the law is wrong**, three observations.
7. **Interface for a strict forward with no fitting**: the input features (name, unit, transform), and the law
   restated as a small typed network: additive terms and hinge units over the features for ln(median) or for
   the logit, plus the spread. List the tensors (name, shape, bounds, unit, meaning) and the literal assignments
   in the project's edit-language style (`name[...] = value` lines; examples in `docs/wsp/PROTOCOL.md` section 2
   and in `configs/models/acme/wsp3/normal/d1-fable/edits.wsp`).
8. Sources, and what you could not find or check.

Reason as a physicist: derive, check limits and orders of magnitude, prefer a simple law you can defend.

## Rules
Write only the file named above. No git commands. No sub-agents.

## Final message (concise)
The law in one paragraph, the expectations in a few lines, and the three weakest assumptions.
