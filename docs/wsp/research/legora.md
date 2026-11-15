# Lauritzen / Legora: "Fable hand-wrote the weights" of a legal-citation classifier — what the primary sources show

> Source dossier compiled on 13 Nov 2026 by a research subagent (Claude Opus 5.5) for the IGNIS WSP workstream, from the primary sources it names. Evidence tags are the subagent's. The synthesis the workstream relies on is `docs/wsp/RESEARCH.md`; this file is kept for its citations and excerpts.

Researched 13 Nov 2026. x.com itself returned HTTP 402, xcancel HTTP 451, and nitter.net only shows a takedown notice. The X posts were read through the FxTwitter JSON API (`api.fxtwitter.com`), which returns the tweet record as X serves it, and through threadreaderapp. Both images were downloaded from `pbs.twimg.com` and read directly.

Status marks:
- **VERIFIED**: I read the primary source myself (the post, image or page).
- **AUTHOR-REPORTED**: the author states it and nobody can check it independently.
- **UNVERIFIED**: second-hand, or not found.

## Bottom line

The entire primary record of the experiment is **three tweets and one image of a results table**, all posted by Jacob Lauritzen (@jacsebl) on 3 Nov 2026. He published no architecture, prompt, code, weights, dataset, split or evaluation protocol. He did not answer any of the technical questions in the replies. As of 13 Nov there is no follow-up on his timeline. Every number comes from his table and is AUTHOR-REPORTED. The press coverage (Dealroom, newsletters) only restates the table.

The detailed "hand-written legal-citation net" material that circulates alongside his post comes from **different people with different networks**:
- FluidInference's **CiteNet**, by Alex Wengg (@Alex_tra_memory).
- **Paul Calcraft**'s "replication" thread.

Neither is Lauritzen's implementation, and neither reproduces his protocol.

## 1. Original post and discussion

- **Original post (VERIFIED):** https://x.com/jacsebl/status/2117606206764175543 , posted 2026-11-03 13:13:57 UTC.
  - Text: "Got fable to hand-write weights of a 100k param model to recognize real versus fake legal citations." Second line: "Basically one-shotted a full NN that's better than training it".
  - One image (1113×418): https://pbs.twimg.com/media/HWM_n5IWQAAymo0.jpg
  - Views and engagement when captured: about 908k views, 7.8k likes, 258 replies, 235 quotes.
- **Author self-replies (VERIFIED, via FxTwitter conversation API and threadreaderapp https://threadreaderapp.com/thread/2117606206764175543.html):**
  - 13:14:53, ID 2117606441678729464: "As expected with more training, performance converges between hand-written weights and trained weights"
  - 13:16:09, ID 2117606758374109438: "Why train when astra/fable can just raw dog the weights directly"
  - 23:30:47, ID 2117761437019861450: "@swyx brave new world"
  - He made no other reply among the roughly 40 replies the API returned (out of 258). His timeline from 3 to 12 Nov, which returned 20 tweets, has no follow-up, no code release and no mention of a dataset.
- **Context the day before (VERIFIED):** On 2 Nov 22:29 UTC, Lauritzen quoted Rikiya Takehi's Weight-Space Programming announcement with "Post training was never the same". Sources: https://x.com/jacsebl/status/2117383524093186519 , quoting https://x.com/rikiyatakehi/status/2117313742891327994
- **Notable critical or probing replies (VERIFIED that they were posted; none was answered):**
  - @jgrimes2: "These numbers don't mean anything without more details." (2117719099769852067)
  - @nuunien: "Can you prove it, beyond a picture of a table?" (2117746956227240421)
  - @AnthonyMaio asks whether Fable also wrote the eval. (2117722361931645133)
  - @victiln asks whether this is on a test set that Fable did not see. (2117721044782039402)
  - @renegadesilicon says "Zero labels" is misleading and that this is distillation of pretrained knowledge into sparse weights. (2117753346924876127)
  - @bioslopper: "this is literally distillation". (2117713624164958420)
  - @thomasahle suggests it coded simple heuristics as weights. (2117753729126707446)
  - Requests for the prompt or workflow came from @CharlesamorAmor, @Bfaviero and @jikkujose.
  - @adarshsolanki asked for the architecture.
  - @MaziyarPanahi asked for a Hugging Face upload.
  - @syfert asked for a run on citebench (https://github.com/Gwonk1/citebench) and on LePhantomCite. (2117689975634624934)
- **Amplification:** Matt Stockton quote-tweeted the post on 3 Nov 19:09, reaching about 308k views (https://x.com/mstockton/status/2117695754278142357). This is the post Dealroom credits.

## 2. Architecture

**Not disclosed. UNVERIFIED / not found.**

The only facts available come from the table:
- The trained baseline has 107,745 non-zero parameters.
- The hand-written net has 3,793 non-zero parameters, rising to 3,828 after one revision.
- The tweet calls it "100k param".

One inference, which is mine and not stated by the author: the hand-written net is probably the same ~107.7k-parameter architecture with about 96.5% of the weights at zero.

The input representation, features, layer sizes, activations and output head are all unknown. The byte-embedding, template-conv and lookup-table design that circulates comes from Calcraft's image and the CiteNet card, not from Lauritzen (see §8).

## 3. How the weights were constructed

**Not disclosed.**

It is not stated whether Fable wrote literal numbers or wrote code that generates them. The table label says "Hand-written". The tweet says "one-shotted". "One revision" is shown as a separate row.

Parameter counts (AUTHOR-REPORTED):

| Version | Non-zero parameters |
|---|---|
| Hand-written | 3,793 |
| Hand-written, one revision | 3,828 |
| Trained baselines | 107,745 |

Exactly how many parameters the architecture has in total is not given beyond "100k".

## 4. Rules or features represented

**Not disclosed by the author.**

Lauritzen never explained what the net checks. The reporter, volume, year and court rules people associate with this experiment come from:
- Calcraft's 2/n tweet: "volume vs year, reporter vs court".
- The CiteNet card.

Both describe other networks.

## 5. What 97.8% and 98.9% measure

The table reads:

| Weights | Labels seen | Non-zero params | Test accuracy |
|---|---|---|---|
| Hand-written | 0 | 3,793 | 97.8% |
| Hand-written, one revision | 5,062 | 3,828 | 98.9% |
| Trained | 512 | 107,745 | 76.5% |
| Trained | 5,062 | 107,745 | 92.9% |
| Trained | 8,192 | 107,745 | 94.6% |

- **97.8%** is the initial hand-written net, with zero labels seen. **98.9%** is after one revision, which had seen 5,062 labels.
- The baselines are the same parameter count trained with 512, 5,062 and 8,192 labels.
- All figures are **AUTHOR-REPORTED**. The image itself is VERIFIED.

The following are **not given**:
- Which dataset was used.
- The test-set size and how the split was made.
- The class balance. This matters because accuracy on an imbalanced real/fake set can be inflated.
- What "fake" means.
- The training recipe for the baselines.
- Whether the test set was ever shown to Fable.

Note on LePhantomCite: the public benchmark has 1,300 excerpts (910 aux_train and 390 eval). Its eval split works out to about 1,014 citations (983 real and 31 phantom, per CiteNet). That does not obviously match "5,062" or "8,192 labels". Lauritzen's dataset is therefore probably something else, possibly synthetic or internal. This is an inference, UNVERIFIED.

## 6. What data the model saw

- **Before writing:** "0 labels seen", per the table (AUTHOR-REPORTED). It is not said whether Fable saw unlabeled examples, a task description, a citation rule set or the data schema.
- **For the revision:** 5,062 labels (AUTHOR-REPORTED). It is not said what feedback was given: errors only, metrics, or examples.

## 7. Public availability

**Nothing public was found.** I found no code, weights, prompts, eval data or licence. There is no GitHub repository, Hugging Face page, blog post or Legora engineering post.

Searches that came back empty of any release:
- X timeline 3–12 Nov
- GitHub, Hugging Face and the Legora site
- Hacker News (`site:news.ycombinator.com`)
- Reddit (`site:reddit.com`)
- LinkedIn

## 8. Third-party "recreations": none reproduces the original

**A. FluidInference CiteNet, by Alex Wengg (@Alex_tra_memory).**
- Sources: https://x.com/Alex_tra_memory/status/2117786388489756866 (4 Nov 01:09 UTC, quote-tweeting Lauritzen) and https://huggingface.co/FluidInference/citenet-coreml
- Licence: Apache-2.0.
- Model card (VERIFIED as written; its numbers are author-reported):
  - Input: 64 character codes.
  - Outputs: `p_fake`, seven evidence values and named concepts.
  - Building blocks: conv template filters (for example "<digit> F.3d "), place-value digit decoders, a ReLU "volume thermometer" and AND-style gates.
  - Size: **397,250 parameters, 13,871 non-zero**.
- How it was built: Claude wrote a spec (reporters, courts, West regional map, volume→year timelines) **and code that compiles the spec into weights**. "no gradient step".
- Evaluation: on LePhantomCite eval, 31/31 phantoms caught. Real citations flagged were 3/983 for v1.0 (blind) and 1/983 for v1.5 (not blind). Only `aux_train` was used for design.
- Scope: it explicitly catches only citations that *cannot exist*. Format-valid invented citations pass.
- Verdict: **this is a different experiment**. The size, data and metric differ, and the model is named only as "Claude", with no Fable version. It is not Lauritzen's network.

**B. Paul Calcraft (@paul_cal), thread of 4 Nov 20:04 UTC.**
- Tweets:
  - https://x.com/paul_cal/status/2118072016670732400
  - https://x.com/paul_cal/status/2118072021095768467
  - https://x.com/paul_cal/status/2118072026435031530
  - https://x.com/paul_cal/status/2118072030654500978
- Content, by tweet:
  - **1/n:** "It's a party trick! People cherry pick problems that small deterministic programs can solve." He also writes "I've replicated this".
  - **2/n:** "Fable can only write a fake legal citation detector net when there is a ruleset it can follow". He cites LePhantomCite, whose fakes can be caught by volume-vs-year or reporter-vs-court mismatches, and quotes the CiteNet post.
  - **3/n:** "At this (low) complexity, the problem is *harder* to solve via gradient descent than a hand-written program". Also: turning an Astra/Fable algorithm into a net "doesn't make this a sensible comparison!"
  - **Final tweet:** "There's no evidence that frontier models can write good neural nets from the void".
- His image (https://pbs.twimg.com/media/HWTg9o2XsAEFjLD.jpg , VERIFIED) is headed "Every net is a rule system compiled into weights". It shows a five-stage pipeline:
  1. Byte embedding, with variants R1 and R2.
  2. Template convs.
  3. Place-value number reader.
  4. Reporter lookup tables "for 173–340 reporters".
  5. ReLU hinge evidence feeding a logit.
- The image also says: "Decompiled to plain Python: identical decisions on all 10,414 eval rows; logits match to about 1e-12".
- What is not stated:
  - Which net(s) he decompiled. R1 and R2 look like two of his own runs, but that is not confirmed.
  - Which model wrote them.
  - Where the 10,414 eval rows come from.
  - Whether there is any repository. None was found.
- Verdict: **his own analogous experiment**, probably on LePhantomCite-style data. It is not a reproduction of Lauritzen's data, protocol or model, because those were never published. It is evidence for the critique that such nets are compiled rule systems. It says nothing about what Lauritzen's net contains. Status: UNVERIFIED beyond the tweets and the image.

**C. Unrelated items to keep separate:**
- LePhantomCite itself: Liu, Stammbach and Henderson (Princeton), arXiv 2606.21155, CC BY 4.0, https://huggingface.co/datasets/ai-law-society-lab/Legal_Phantom_Citation . This is a benchmark, not a network.
- Conventional cite-checkers: citebench, citecheck, Gainsay. These are not hand-written weight networks.

## 9. Connection to MIT CSAIL Weight-Space Programming

- **The MIT post:** Takehi, Teoh, Zhang and Khattab (MIT CSAIL), published 2 Nov 2026, with a response to comments added 12 Nov (https://rikiyat.github.io/blog/weight-space-programming/). It uses "GPT-6 Astra". **It does not mention Lauritzen, Legora, legal citations or Fable** (VERIFIED).
- **Lauritzen's own link to it (VERIFIED):**
  - He quoted Takehi's announcement the evening before, writing "Post training was never the same".
  - His 13:16 reply names "astra/fable".
  - He makes no explicit methodological claim that he followed their method.
- **Commenters:**
  - Calcraft 3/n directly compares the experiment to the MIT Breakout-from-scratch result, quoting https://x.com/rikiyatakehi/status/2117313805977780547 .
  - Matt Stockton frames the post as part of the "frontier models can understand and directly modify weights" discussion.
  - Dealroom (https://dealroom.co/news/x-2117695754278142357-legoras-cto-says-claude-fable-hand-wrote-the-weights-of-a-100k-parameter/ and https://dealroom.co/news/169058-mit-gpt-6-edited-a-rival-ai-s-weights-by-hand-lifting-its-math-score-to-/) and techbyjohan week 46 (https://www.techbyjohan.com/tech-insights-2026-week-46/) present it as following the MIT work. That framing is editorial.

## Claim table

| claim | status | source |
|---|---|---|
| Lauritzen posted on 3 Nov 2026 13:13 UTC that Fable hand-wrote the weights of a 100k-parameter real-vs-fake legal citation net that is "better than training it" | VERIFIED (the post exists, with this wording) | https://x.com/jacsebl/status/2117606206764175543 (via api.fxtwitter.com) |
| "Fable" refers to Anthropic's Claude Fable | AUTHOR-REPORTED by implication; the tweet says only "fable", and Dealroom adds "Claude" | tweet; Dealroom |
| Hand-written: 0 labels, 3,793 non-zero parameters, 97.8% test accuracy | AUTHOR-REPORTED (image VERIFIED) | https://pbs.twimg.com/media/HWM_n5IWQAAymo0.jpg |
| One revision: 5,062 labels, 3,828 non-zero parameters, 98.9% | AUTHOR-REPORTED | same image |
| Trained baselines with 107,745 parameters: 76.5% / 92.9% / 94.6% at 512 / 5,062 / 8,192 labels | AUTHOR-REPORTED | same image |
| "performance converges" with more training | AUTHOR-REPORTED | https://x.com/jacsebl/status/2117606441678729464 |
| Architecture, features and layers | UNVERIFIED (not disclosed) | none |
| Whether the weights are literal numbers or produced by generator code | UNVERIFIED (not disclosed) | none |
| Dataset, test size, split, class balance and test isolation | UNVERIFIED (not disclosed) | none |
| Code, weights, prompts or data released | Not found | timeline, GitHub, Hugging Face, Hacker News, Reddit and LinkedIn searches |
| Lauritzen answered technical questions in the replies | VERIFIED that he did not (among the replies retrievable) | api.fxtwitter.com/2/conversation/2117606206764175543 |
| CiteNet: 397,250 parameters (13,871 non-zero), compiled by Claude-written code, 31/31 and 1/983 on LePhantomCite | AUTHOR-REPORTED by a third party (card VERIFIED); a different network | https://huggingface.co/FluidInference/citenet-coreml ; https://x.com/Alex_tra_memory/status/2117786388489756866 |
| Calcraft "replicated this" and decompiled a net to Python with identical decisions on 10,414 rows | AUTHOR-REPORTED by a third party; no repository; not Lauritzen's net | https://x.com/paul_cal/status/2118072016670732400 ; https://pbs.twimg.com/media/HWTg9o2XsAEFjLD.jpg |
| The MIT Weight-Space Programming post mentions Lauritzen or Legora | VERIFIED that it does not | https://rikiyat.github.io/blog/weight-space-programming/ |
| Lauritzen engaged with the MIT work the day before | VERIFIED | https://x.com/jacsebl/status/2117383524093186519 |

## Queries and URLs tried

- **Direct fetches:**
  - x.com/jacsebl: HTTP 402.
  - xcancel.com: HTTP 451.
  - nitter.net: takedown notice.
  - threadreaderapp: OK.
  - api.fxtwitter.com status, conversation, profile and statuses endpoints: OK.
- **Web searches:**
  - "Jacob Lauritzen Legora Claude hand-wrote weights neural network legal citations"
  - "Lauritzen Fable weights 97.8% citations" (extended)
  - Paul Calcraft replication (extended)
  - "jacsebl … reporter/volume/features/labels" (extended)
  - LePhantomCite (extended)
  - "31/31" fakes (extended)
  - "Claude wrote every weight" (extended)
  - Alex_tra_memory
  - site:news.ycombinator.com
  - site:reddit.com
  - Lauritzen github/huggingface/blog (extended)
  - LinkedIn
- **Pages read:**
  - Dealroom (two articles)
  - techbyjohan week 46
  - the MIT blog
  - Sean Goedecke's post (no mention of Legora)
  - the CiteNet card
  - the LePhantomCite dataset card
  - the X Digest page at minty-lotus-3tpb.here.now
- **Not read:** the Latent Space AINews issue and the LinkedIn post by Abhijit M., which only search snippets surfaced. Both restate the claim.
