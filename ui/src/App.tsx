import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import crumbs from "../../design/menubar/crumbs.svg";
import wedgePaidOff from "../../design/menubar/paid-off.svg";
import wedgeFull from "../../design/menubar/wedge-100.svg";
import wedgeOneThird from "../../design/menubar/wedge-33.svg";
import wedgeTwoThirds from "../../design/menubar/wedge-66.svg";
import nibbling from "../../design/nibble-nibbling.svg";
import paidOff from "../../design/nibble-paid-off.svg";
import sleepy from "../../design/nibble-sleepy.svg";
import stillHungry from "../../design/nibble-still-hungry.svg";
import { dollars, headline, suggestionText, weeklyReset } from "./copy";
import { loading, type Mood, type View, type Wedge } from "./view";

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

export function App() {
  const view = useView();
  if (view.prompts.celebrate) return <Celebration view={view} />;
  return (
    <main className="card">
      <img className="mascot" src={mascots[view.mood]} alt={`nibble, ${view.mood}`} />
      <Body view={view} />
      {view.prompts.askAutostart && <AutostartPrompt />}
      <Footer view={view} />
    </main>
  );
}

function Body({ view }: { view: View }) {
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
      <p className="amount">
        {view.wedge && (
          <span
            className="wedge"
            role="img"
            aria-label={`Wedge: ${view.wedge}`}
            style={maskOf(wedges[view.wedge])}
          />
        )}
        <span className="mono">{dollars(view.valueCents)}</span>
        {view.weeklyShareCents !== null && (
          <span className="of"> of {dollars(view.weeklyShareCents)} this week at API rates</span>
        )}
        {view.multiplier && (
          <span className="multiplier mono">
            {view.lowerBound ? "≥" : ""}
            {view.multiplier}
          </span>
        )}
      </p>
      {view.progressPercent !== null && (
        <div className="bar" role="progressbar" aria-valuenow={view.progressPercent}>
          <div style={{ width: `${String(Math.min(view.progressPercent, 100))}%` }} />
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

function Celebration({ view }: { view: View }) {
  return (
    <main className="card celebrate">
      <img className="mascot" src={paidOff} alt="nibble, celebrating" />
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
