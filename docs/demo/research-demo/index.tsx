import { Bar, Collapse, Field, Stage, Stream, Text, Window, type Act } from "../../kit";
import "./ignis.css";

// IGNIS Research workspace, for the pitch deck (16:9 slide). Copied from the ai-chat gold (thread,
// streaming answer, composer) with a source card straddling the window's right margin ("panel over
// product"). Beats: the question is typed and sent; a short search step; the answer streams in; its
// claims arrive, each labelled for what it is; the cursor opens the source; the source page slides in
// beside the claim with the cited region marked. Plays once; the poster is the payoff.
//
// Honesty: no invented numbers or quotes. The source page is filler bars; its title names a real
// document of NASA PSI-69 (the FLEX test plan) listed in the repository's manifests.
//
// Real strings: the question, the answer, the three claim labels and lines, "Open source", the
// source title.

const Q = "How does ambient pressure change methanol droplet burning in microgravity?";
const A = "The FLEX reports describe how the runs vary the ambient pressure. Each claim below is bound to its evidence and labelled for what it is.";
const WORDS = A.split(" ").length;

type S = {
  draft: string;
  sent: boolean;
  phase: "idle" | "searching" | "answer";
  words: number;
  claims: boolean;
  source: boolean;
};

const initial: S = { draft: "", sent: false, phase: "idle", words: 0, claims: false, source: false };
const poster: S = { draft: "", sent: true, phase: "answer", words: WORDS, claims: true, source: true };

async function script(a: Act<S>) {
  await a.wait(600);
  a.mark("type");
  await a.type("draft", Q, { interval: 30 });
  await a.wait(350);
  a.mark("send");
  a.put({ sent: true, draft: "" });
  await a.wait(400);
  a.put({ phase: "searching" });
  await a.wait(1200);
  a.mark("stream");
  a.put({ phase: "answer" });
  for (let i = 1; i <= WORDS; i++) {
    a.put({ words: i });
    await a.wait(38);
  }
  await a.wait(300);
  a.mark("claims");
  a.put({ claims: true });
  await a.wait(1000);
  a.mark("open");
  await a.move("open-source");
  await a.click();
  a.put({ source: true });
  await a.wait(3200);
}

export default function IgnisResearch() {
  return (
    <Stage
      width={960}
      height={540}
      initial={initial}
      poster={poster}
      script={script}
      play="once"
      accent="#1d4ed8"
      cursor={{ at: { x: 780, y: 480 } }}
      className="ig"
    >
      {(s) => (
        <>
          <Window title="IGNIS · Research" className="ig-win">
            <div className="ig-body">
              <div className="ig-main">
              <div className="ig-thread">
                <div className="ig-turn ig-turn--past">
                  <Bar w={180} h={30} r={15} tone={1} className="ig-past-q" />
                  <Bar w="78%" h={8} r={4} tone={2} />
                  <Bar w="66%" h={8} r={4} tone={2} />
                  <Bar w="40%" h={8} r={4} tone={2} />
                </div>
                <div className="ig-turn ig-turn--past">
                  <Bar w={220} h={30} r={15} tone={1} className="ig-past-q" />
                  <Bar w="84%" h={8} r={4} tone={2} />
                  <Bar w="58%" h={8} r={4} tone={2} />
                </div>
                <Collapse open={s.sent} reveal="drop" from="below">
                  <div className="ig-q">
                    <Text>{Q}</Text>
                  </div>
                </Collapse>
                <Collapse open={s.phase === "searching"}>
                  <Text className="ig-searching">Searching NASA sources</Text>
                </Collapse>
                <Collapse open={s.phase === "answer"}>
                  <div className="ig-a" data-on>
                    <Stream className="ig-stream" text={A} count={s.words} />
                    <div className="ig-claims" data-on={s.claims || undefined}>
                      <div className="ig-claim">
                        <span className="ig-chip ig-chip--source">source reports</span>
                        <span className="ig-claim-text">FLEX test plan, PSI-69</span>
                        <span className="ig-open" data-target="open-source" data-cursor="pointer" data-open={s.source || undefined}>
                          Open source
                        </span>
                      </div>
                      <div className="ig-claim">
                        <span className="ig-chip ig-chip--observed">observed</span>
                        <span className="ig-claim-text">Measured diameter at the cutoff</span>
                      </div>
                      <div className="ig-claim">
                        <span className="ig-chip ig-chip--predicted">model prediction</span>
                        <span className="ig-claim-text">Predicted diameter at the horizon</span>
                      </div>
                    </div>
                  </div>
                </Collapse>
              </div>
              <Field className="ig-composer" value={s.draft} placeholder="Ask about the NASA experiments" focused={!s.sent} />
              </div>

              {/* The rail, as in the product: Activity from the first frame; the opened source
                  comes in over its top. Activity is filler: the eye belongs on the claims. */}
              <aside className="ig-rail">
                <Collapse open={s.source} reveal="drop">
          <div className="ig-source" data-on={s.source || undefined} aria-hidden={!s.source}>
            <div className="ig-source-head">
              <Text className="ig-source-title">FLEX test plan</Text>
              <Text className="ig-source-meta">NASA PSI-69 · cited region</Text>
            </div>
            <div className="ig-page">
              <Bar w="62%" h={7} r={3} tone={1} />
              <Bar w="92%" h={6} r={3} tone={2} />
              <Bar w="88%" h={6} r={3} tone={2} />
              <div className="ig-cited">
                <Bar w="94%" h={6} r={3} tone={1} />
                <Bar w="90%" h={6} r={3} tone={1} />
                <Bar w="70%" h={6} r={3} tone={1} />
              </div>
              <Bar w="86%" h={6} r={3} tone={2} />
              <Bar w="91%" h={6} r={3} tone={2} />
              <Bar w="54%" h={6} r={3} tone={2} />
            </div>
          </div>
                </Collapse>
                <Text className="ig-rail-title">Activity</Text>
                {[64, 48, 72, 56, 40, 66, 52].map((w, i) => (
                  <div key={i} className="ig-row">
                    <Bar w={34} h={6} r={3} tone={2} />
                    <Bar w={`${w}%`} h={7} r={3} tone={1} />
                  </div>
                ))}
              </aside>
            </div>
          </Window>
        </>
      )}
    </Stage>
  );
}
