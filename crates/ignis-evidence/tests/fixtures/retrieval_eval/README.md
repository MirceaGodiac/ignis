# Synthetic retrieval-evaluation fixture (R12b)

`origin = synthetic_fixture`. Five retrieval tasks (single-hop, multi-hop, figure) and one task
outside the retrieval sub-set over the PSI-69 FLEX Test Plan fixture (`../PSI-69/`), in the K1.14
`GoldTask` shape (`split: dev`), with hand-written expected terms and a hand-written expansion pack.
They exist so `ignis evidence retrieval-eval` is tested end to end before Tudor's T10 dev set lands.
**Their numbers test the tool; they are never reported as results.** The cited locators are pages of
the Test Plan chosen by reading it, not reviewed gold evidence.
