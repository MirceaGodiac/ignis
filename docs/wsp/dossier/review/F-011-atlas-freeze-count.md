# F-011: split strata were counted as investigations

- **About:** new; PROTOCOL 11.2 and freeze commit summary
- **Reviewer:** GPT-6 Astra (`gpt-6-astra`), 2026-11-14
- **Verdict:** refuted as an investigation count
- **Severity:** record
- **Partitions touched while reviewing:** frozen membership metadata only;
  no experimental outcomes required for this finding

## What the dossier says

PROTOCOL 11.2 says ten investigations are split and five held whole. The owner
relayed the same wording from Fable. The listed held investigations themselves
number six: Saffire I, II and III, DAFT, SAME and FLEX-2.

## What I did

Read the committed split README, PSI-10 metadata and static freeze writer at
. Compared investigation IDs and
stratum rows, without opening the outcome file or any measurement.

## Evidence

Split accession IDs are PSI-10, 20, 21, 22, 23, 39, 101, 106 and 107: **nine**.
PSI-10 contributes normal and inverse strata, giving **ten split strata**.
Held accession IDs are PSI-47, 68, 98, 99, 100 and 102: **six**.
Together they are the fifteen new investigations; the membership JSONs are
internally consistent with these counts.

## What should change

Fable should correct the prose counts to nine split investigations (ten strata)
and six held investigations. No group membership, seed, outcome access or model
number should change. Astra's ledger uses the JSONs and correct denominator;
it does not rewrite Fable's freeze or protocol.

## Response

The owner subsequently asked Astra to resolve the reported issues. Astra corrected
the two prose counts in PROTOCOL 11.2 to nine split investigations (ten strata)
and six held investigations. No group file, rule, seed or test access changed.
Historical commit messages are not rewritten. This finding has no performance conclusion.
