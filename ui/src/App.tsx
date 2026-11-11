import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import crumbs from "../../design/menubar/crumbs.svg";
import wedgePaidOff from "../../design/menubar/paid-off.svg";
import wedgeFull from "../../design/menubar/wedge-100.svg";
import wedgeOneThird from "../../design/menubar/wedge-33.svg";
import wedgeTwoThirds from "../../design/menubar/wedge-66.svg";
import nibbling from "../../design/nibble-nibbling.svg?raw";
import paidOff from "../../design/nibble-paid-off.svg?raw";
import sleepy from "../../design/nibble-sleepy.svg?raw";
import stillHungry from "../../design/nibble-still-hungry.svg?raw";
import { dollars, headline, suggestionText, weeklyReset } from "./copy";
import { useCountUp, useOpenCount } from "./motion";
import { loading, type Mood, type View, type Wedge } from "./view";

/** The mood drawings as SVG markup, inlined so CSS can animate their parts. */
const mascots: Record<Mood, string> = { nibbling, paidOff, sleepy, stillHungry };
const wedges: Record<Wedge, string> = {
  full: wedgeFull,
  twoThirds: wedgeTwoThirds,
  oneThird: wedgeOneThird,
  crumbs,
  paidOff: wedgePaidOff,
};

/**
 * A CSS mask showing `src`'s shape. The URL is quoted because Vite inlines small SVGs as
 * data URIs containing quotes, which an unquoted `url()` cannot hold.
 */
function maskOf(src: string) {
  const image = `url(${JSON.stringify(src)})`;
  return { maskImage: image, WebkitMaskImage: image };
}

/** The latest View from the shell: fetched once, then pushed on every refresh. */
function useView(): View {
  const [view, setView] = useState<View>(loading);
  useEffect(() => {
    void invoke<View>("get_view").then(setView);
    const unlisten = listen<View>("view", (event) => {
      setView(event.payload);
    });
    return () => {
      void unlisten.then((stop) => {
        stop();
      });
    };
  }, []);
  return view;
}

/**
 * nibble in `mood`, moving. The markup is our own generated SVG (design/scripts/gen.py),
 * so inlining it is safe; re-keying on `opens` replays the entrance.
 */
function Mascot({ mood, opens, label }: { mood: Mood; opens: number; label: string }) {
  return (
    <div
      key={`${mood}-${String(opens)}`}
      className={`mascot mood-${mood}`}
      role="img"
      aria-label={label}
      dangerouslySetInnerHTML={{ __html: mascots[mood] }}
    />
  );
}

export function App() {
  const view = useView();
  const opens = useOpenCount();
  if (view.prompts.celebrate) return <Celebration view={view} opens={opens} />;
  return (
    <main className="card">
      <Mascot mood={view.mood} opens={opens} label={`nibble, ${view.mood}`} />
      <Body view={view} opens={opens} />
      {view.prompts.askAutostart && <AutostartPrompt />}
      <Footer view={view} />
    </main>
  );
}

function Body({ view, opens }: { view: View; opens: number }) {
  const value = useCountUp(view.valueCents, opens);
  if (view.status === "loading") return <p className="lede">Counting crumbs…</p>;
  if (view.status === "noUsage") {
    return (
      <p className="lede">
        No Claude Code usage yet. Use Claude Code and nibble will start eating through your plan.
      </p>
    );
  }
  if (view.status === "signedOut") {
    return (
      <p className="lede">
        nibble can’t find a Claude account. Sign in to Claude Code and it will start counting.
      </p>
    );
  }
  const reading = weeklyReset(view);
  return (
    <>
      <h1>{headline(view)}</h1>
      <div className="amount">
        {view.wedge && (
          <span
            key={`${view.wedge}-${String(opens)}`}
            className="wedge"
            role="img"
            aria-label={`Wedge: ${view.wedge}`}
            style={maskOf(wedges[view.wedge])}
          />
        )}
        <div className="figures">
          <p className="value">
            <span className="mono">{dollars(value)}</span>
            {view.multiplier && (
              <span className="multiplier mono">
                {view.lowerBound ? "≥" : ""}
                {view.multiplier}
              </span>
            )}
          </p>
          {view.weeklyShareCents !== null && (
            <p className="of">of {dollars(view.weeklyShareCents)} this week at API rates</p>
          )}
        </div>
      </div>
      {view.progressPercent !== null && (
        <div className="bar" role="progressbar" aria-valuenow={view.progressPercent}>
          <div key={opens} style={{ width: `${String(Math.min(view.progressPercent, 100))}%` }} />
        </div>
      )}
      {reading && <p className="meta mono">{reading}</p>}
      {view.unpricedResponses > 0 && (
        <p className="meta">
          Plus {view.unpricedResponses} responses from models nibble can’t price yet.
        </p>
      )}
      {view.suggestion && (
        <aside className="suggestion">
          <p>{suggestionText(view.suggestion, view.plan ?? "your plan")}</p>
          <button type="button" onClick={() => void invoke("dismiss")}>
            Not now
          </button>
        </aside>
      )}
      {view.error && <p className="meta error">{view.error}</p>}
    </>
  );
}

function Celebration({ view, opens }: { view: View; opens: number }) {
  return (
    <main className="card celebrate">
      <Mascot mood="paidOff" opens={opens} label="nibble, celebrating" />
      <h1>nibble ate the whole wedge!</h1>
      <p className="lede">
        {view.plan ?? "Your plan"} has paid for itself this week
        {view.multiplier ? `, ${view.multiplier} and counting` : ""}.
      </p>
      <button type="button" className="primary" onClick={() => void invoke("celebrated")}>
        Yum
      </button>
    </main>
  );
}

function AutostartPrompt() {
  const [enable, setEnable] = useState(true);
  return (
    <aside className="suggestion">
      <label className="check">
        <input
          type="checkbox"
          checked={enable}
          onChange={(event) => {
            setEnable(event.target.checked);
          }}
        />
        Start nibble when you log in
      </label>
      <button type="button" onClick={() => void invoke("answer_autostart", { enable })}>
        Done
      </button>
    </aside>
  );
}

function Footer({ view }: { view: View }) {
  return (
    <footer>
      <span>Measures Claude Code only</span>
      <label className="check">
        <input
          type="checkbox"
          checked={view.notificationsOn}
          onChange={(event) => void invoke("set_notifications", { on: event.target.checked })}
        />
        Notify me
      </label>
    </footer>
  );
}
