import {
  type CSSProperties,
  type MouseEvent,
  type ReactNode,
  useEffect,
  useRef,
  useState,
} from "react";

import nibbling from "../../design/nibble-nibbling.svg?raw";
import paidOff from "../../design/nibble-paid-off.svg?raw";
import sleepy from "../../design/nibble-sleepy.svg?raw";
import stillHungry from "../../design/nibble-still-hungry.svg?raw";
import { dailyReset, dollars, headline, progressText, suggestionText, waiting } from "./copy";
import { reducedMotion, useCountUp, useOpenCount } from "./motion";
import { getView, onView, openExternal, previewing, send } from "./shell";
import { loading, type Mood, type Update, type View } from "./view";

/** The mood drawings as SVG markup, inlined so CSS can animate their parts. */
const mascots: Record<Mood, string> = { nibbling, paidOff, sleepy, stillHungry };

/** The latest View from the shell: fetched once, then pushed on every refresh. */
function useView(): View {
  const [view, setView] = useState<View>(loading);
  useEffect(() => {
    void getView().then(setView);
    return onView(setView);
  }, []);
  return view;
}

/** Where the Paid off card is: up, playing its send-off, or gone until the next Day. */
type Party = "up" | "leaving" | "gone";

export function App() {
  const view = useView();
  const opens = useOpenCount();
  const [settings, setSettings] = useState(false);
  const [party, setParty] = useState<Party>("up");
  // Bumped as the Paid off card leaves, so the reading below replays its entrance.
  const [encore, setEncore] = useState(0);
  const replay = opens + encore;
  if (!view.prompts.celebrate && party !== "up") setParty("up");
  // Every open starts on the reading, not wherever the last visit ended.
  const [seenOpens, setSeenOpens] = useState(opens);
  if (seenOpens !== opens) {
    setSeenOpens(opens);
    setSettings(false);
  }
  useEffect(() => {
    const close = (event: KeyboardEvent) => {
      if (event.key === "Escape") setSettings(false);
    };
    window.addEventListener("keydown", close);
    return () => {
      window.removeEventListener("keydown", close);
    };
  }, []);

  if (settings) {
    return (
      <Settings
        view={view}
        onClose={() => {
          setSettings(false);
        }}
      />
    );
  }
  const celebrating = view.prompts.celebrate && party !== "gone";
  const reading = (
    <main className={`card mood-${view.mood}`}>
      <header className="stage">
        {view.plan && <span className="chip">{view.plan}</span>}
        <button
          type="button"
          className="icon"
          aria-label="Settings"
          onClick={() => {
            setSettings(true);
          }}
        >
          <GearIcon />
        </button>
        <Mascot mood={view.mood} opens={replay} label={`nibble, ${view.mood}`} />
      </header>
      <section className="body">
        {view.status === "ready" ? <Reading view={view} opens={replay} /> : <Waiting view={view} />}
        {view.update && <UpdatePrompt update={view.update} />}
        {view.prompts.askAutostart && <AutostartPrompt />}
      </section>
      <Footer />
    </main>
  );
  // One shape for every state, so neither part remounts as the Paid off card leaves.
  return (
    <>
      {celebrating && party === "up" ? null : reading}
      {celebrating ? (
        <Celebration
          view={view}
          opens={opens}
          onLeave={() => {
            setEncore((count) => count + 1);
            setParty("leaving");
          }}
          onGone={() => {
            setParty("gone");
            send({ name: "celebrated" });
          }}
        />
      ) : null}
    </>
  );
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

function Waiting({ view }: { view: View }) {
  const { title, body } = waiting(view.status);
  return (
    <>
      <h1>{title}</h1>
      <p className="lede">{body}</p>
      {view.error && <p className="note error">{view.error}</p>}
    </>
  );
}

/** Today's reading: what the usage is worth, and how far that goes toward the month's plan. */
function Reading({ view, opens }: { view: View; opens: number }) {
  const value = useCountUp(view.valueCents, opens);
  const progress = progressText(view);
  const reset = dailyReset(view);
  return (
    <>
      <h1>{headline(view)}</h1>
      <div className="reading">
        <p className="label">Claude Code today, at API rates</p>
        <p className="figure">
          <span className="value">{dollars(value)}</span>
          {view.priceCents !== null && <span className="share">of {dollars(view.priceCents)}</span>}
          {view.multiplier && (
            <span className="multiplier">
              {view.lowerBound ? "≥" : ""}
              {view.multiplier}
            </span>
          )}
        </p>
        {view.progressPercent !== null && <Cheese percent={view.progressPercent} replay={opens} />}
        {(progress ?? reset) && (
          <p className="meta">
            <span>{progress}</span>
            <span>{reset}</span>
          </p>
        )}
      </div>
      {view.unpricedResponses > 0 && (
        <p className="note">
          Plus {view.unpricedResponses} responses from models nibble can’t price yet.
        </p>
      )}
      {view.error && <p className="note error">{view.error}</p>}
      {view.suggestion && (
        <aside className="callout">
          <p>{suggestionText(view.suggestion, view.plan ?? "your plan")}</p>
          <button
            type="button"
            onClick={() => {
              send({ name: "dismiss" });
            }}
          >
            Not now
          </button>
        </aside>
      )}
    </>
  );
}

/**
 * Break-even progress as a bar of cheese nibble eats from the left. Past Paid off the bar
 * stays eaten and a tick marks where break-even fell, so 2× sits halfway along.
 */
function Cheese({ percent, replay }: { percent: number; replay: number }) {
  const eaten = Math.min(percent, 100);
  const breakEven = percent > 100 ? 10_000 / percent : null;
  return (
    <div
      className={`cheese${eaten >= 100 ? " gone" : ""}`}
      role="progressbar"
      aria-label="Break-even progress"
      aria-valuenow={percent}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div
        key={replay}
        className="eaten"
        style={{ "--eaten": `${String(eaten)}%` } as CSSProperties}
      />
      {breakEven !== null && (
        <i className="tick" title="Break-even" style={{ left: `${String(breakEven)}%` }} />
      )}
    </div>
  );
}

/** When the send-off starts closing the card, and when it is over, in milliseconds. */
const LEAVE_AT = 950;
const GONE_AT = 1600;

/** The pieces flung from the button: crumbs, little wedges and sparks, all around it. */
const PIECES = Array.from({ length: 32 }, (_, index) => {
  const angle = (index / 32) * Math.PI * 2 + (index % 3) * 0.19;
  const reach = 70 + ((index * 53) % 95);
  return {
    kind: index % 5 === 0 ? "spark" : index % 3 === 0 ? "wedge" : "crumb",
    style: {
      "--dx": `${(Math.cos(angle) * reach).toFixed(1)}px`,
      "--dy": `${(Math.sin(angle) * reach - 36).toFixed(1)}px`,
      "--spin": `${String(((index * 97) % 540) - 270)}deg`,
      "--size": `${String(5 + ((index * 7) % 6))}px`,
      "--delay": `${String((index % 4) * 25)}ms`,
    } as CSSProperties,
  };
});

function Celebration({
  view,
  opens,
  onLeave,
  onGone,
}: {
  view: View;
  opens: number;
  onLeave: () => void;
  onGone: () => void;
}) {
  const [origin, setOrigin] = useState<{ x: number; y: number } | null>(null);
  const [leaving, setLeaving] = useState(false);
  const timers = useRef<number[]>([]);
  useEffect(() => {
    const pending = timers.current;
    return () => {
      pending.forEach((timer) => {
        window.clearTimeout(timer);
      });
    };
  }, []);

  const celebrate = (event: MouseEvent<HTMLButtonElement>) => {
    if (origin) return;
    const box = event.currentTarget.getBoundingClientRect();
    setOrigin({ x: box.left + box.width / 2, y: box.top + box.height / 2 });
    const calm = reducedMotion();
    timers.current.push(
      window.setTimeout(
        () => {
          setLeaving(true);
          onLeave();
        },
        calm ? 0 : LEAVE_AT,
      ),
      window.setTimeout(onGone, calm ? 200 : GONE_AT),
    );
  };

  const classes = ["card", "celebrate", origin && "party", leaving && "leaving"];
  return (
    <main
      className={classes.filter(Boolean).join(" ")}
      style={
        origin
          ? ({ "--x": `${String(origin.x)}px`, "--y": `${String(origin.y)}px` } as CSSProperties)
          : undefined
      }
    >
      <div className="crumbs" key={opens} aria-hidden="true">
        {Array.from({ length: 14 }, (_, index) => (
          <i
            key={index}
            style={{
              left: `${String((index * 37 + 5) % 100)}%`,
              animationDelay: `${String(-(index % 7) * 0.45)}s`,
              animationDuration: `${String(2.6 + (index % 4) * 0.6)}s`,
            }}
          />
        ))}
      </div>
      {origin && (
        <div className="burst" aria-hidden="true">
          <b className="ring" />
          <b className="ring late" />
          {PIECES.map((piece, index) => (
            <i key={index} className={piece.kind} style={piece.style} />
          ))}
        </div>
      )}
      <Mascot mood="paidOff" opens={opens} label="nibble, celebrating" />
      <h1>nibble ate the whole wedge!</h1>
      <p className="lede">
        Today alone paid for your whole month of {view.plan ?? "your plan"}
        {view.multiplier ? `, ${view.multiplier} and counting` : ""}.
      </p>
      <button type="button" className="primary" aria-label="Well earned!" onClick={celebrate}>
        {origin ? <CheckIcon /> : "Well earned!"}
      </button>
    </main>
  );
}

function AutostartPrompt() {
  const answer = (enable: boolean) => () => {
    send({ name: "answer_autostart", enable });
  };
  return (
    <aside className="callout">
      <p>Keep nibble around? It can start when you log in.</p>
      <div className="actions">
        <button type="button" className="primary" onClick={answer(true)}>
          Start at login
        </button>
        <button type="button" onClick={answer(false)}>
          No thanks
        </button>
      </div>
    </aside>
  );
}

function UpdatePrompt({ update }: { update: Update }) {
  const install = () => {
    send({ name: "install_update" });
  };
  switch (update.state) {
    case "available":
      return (
        <aside className="callout">
          <p>A fresh nibble is ready: version {update.version}.</p>
          <div className="actions">
            <button type="button" className="primary" onClick={install}>
              Install & restart
            </button>
          </div>
        </aside>
      );
    case "installing":
      return (
        <aside className="callout" aria-live="polite">
          <p>Installing the update. nibble will be right back.</p>
        </aside>
      );
    case "failed":
      return (
        <aside className="callout" aria-live="polite">
          <p>The update slipped away before it could settle in.</p>
          <div className="actions">
            <button type="button" onClick={install}>
              Try again
            </button>
          </div>
        </aside>
      );
  }
}

function Footer() {
  return (
    <footer>
      <SupportLink url="https://buymeacoffee.com/tonytangdev">Buy me a coffee</SupportLink>
      <SupportLink url="https://tonytang.dev/apps/">More apps</SupportLink>
    </footer>
  );
}

function Settings({ view, onClose }: { view: View; onClose: () => void }) {
  // The shell answers a moment later, so the switches move on the click.
  const [notify, setNotify] = useState(view.switches.notifications);
  const [autostart, setAutostart] = useState(view.switches.autostart);
  const [updateChecks, setUpdateChecks] = useState(view.switches.updateChecks);
  return (
    <main className="card settings">
      <header className="bar">
        <button type="button" className="icon" aria-label="Back" onClick={onClose}>
          <BackIcon />
        </button>
        <h1>Settings</h1>
      </header>
      <section className="rows">
        <Switch
          label="Notify me when it pays off"
          hint="Once a day, when today covers the month."
          on={notify}
          onChange={(on) => {
            setNotify(on);
            send({ name: "set_notifications", on });
          }}
        />
        <Switch
          label="Start at login"
          hint="nibble waits quietly in the menu bar."
          on={autostart}
          onChange={(enable) => {
            setAutostart(enable);
            send({ name: "answer_autostart", enable });
          }}
        />
        <Switch
          label="Check for updates"
          hint="Once a day, nibble asks GitHub for a new version and tells you when one is out."
          on={updateChecks}
          onChange={(on) => {
            setUpdateChecks(on);
            send({ name: "set_update_checks", on });
          }}
        />
        <UpdateRow view={view} />
      </section>
      <section className="about">
        <h2>What nibble counts</h2>
        <p>
          Only Claude Code, read from its logs on this computer. Chats on claude.ai and in the
          desktop app leave no local record, so they are not included.
        </p>
      </section>
      <footer>
        <SupportLink url="https://buymeacoffee.com/tonytangdev">Buy me a coffee</SupportLink>
        <SupportLink url="https://tonytang.dev/apps/">More apps</SupportLink>
        <button
          type="button"
          className="quit"
          onClick={() => {
            send({ name: "quit" });
          }}
        >
          Quit nibble
        </button>
      </footer>
    </main>
  );
}

/**
 * The running version, and a button to look for a newer one now. What it found shows only
 * after a click on this visit, so an old answer never reads as today's. A preview tab has
 * nobody to click for, so it shows the sample's answer straight away.
 */
function UpdateRow({ view }: { view: View }) {
  const [asked, setAsked] = useState(previewing);
  const check = () => {
    setAsked(true);
    send({ name: "check_for_update" });
  };
  const install = () => {
    send({ name: "install_update" });
  };
  const version = view.version ? `nibble ${view.version}` : "nibble";
  let hint = "Look for a new version now.";
  let action: ReactNode = (
    <button type="button" onClick={check}>
      Check now
    </button>
  );
  if (view.update?.state === "installing") {
    hint = "Installing the update. nibble will be right back.";
    action = null;
  } else if (view.update?.state === "failed") {
    hint = "The update slipped away before it could settle in.";
    action = (
      <button type="button" onClick={install}>
        Try again
      </button>
    );
  } else if (asked && view.updateCheck === "checking") {
    hint = "Asking GitHub…";
    action = (
      <button type="button" disabled>
        Checking…
      </button>
    );
  } else if (asked && view.update?.state === "available") {
    hint = `A fresh nibble is ready: version ${view.update.version}.`;
    action = (
      <button type="button" className="primary" onClick={install}>
        Install & restart
      </button>
    );
  } else if (asked && view.updateCheck === "upToDate") {
    hint = "This is the newest nibble. All caught up.";
  } else if (asked && view.updateCheck === "failed") {
    hint = "nibble couldn’t reach GitHub. Try again in a bit.";
  }
  return (
    <div className="row" aria-live="polite">
      <span className="text">
        {version}
        <small>{hint}</small>
      </span>
      {action}
    </div>
  );
}

function Switch(props: {
  label: string;
  hint: string;
  on: boolean;
  onChange: (on: boolean) => void;
}) {
  return (
    <button
      type="button"
      className="switch"
      role="switch"
      aria-checked={props.on}
      onClick={() => {
        props.onChange(!props.on);
      }}
    >
      <span className="text">
        {props.label}
        <small>{props.hint}</small>
      </span>
      <span className="track" aria-hidden="true" />
    </button>
  );
}

/** A link that opens in the default browser, since the popover's webview cannot navigate. */
function SupportLink({ url, children }: { url: string; children: ReactNode }) {
  return (
    <a
      href={url}
      onClick={(event) => {
        event.preventDefault();
        openExternal(url);
      }}
    >
      {children}
    </a>
  );
}

function GearIcon() {
  return (
    <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true">
      <circle
        cx="10"
        cy="10"
        r="6.4"
        fill="none"
        stroke="currentColor"
        strokeWidth="3.2"
        strokeDasharray="2.6 2.427"
      />
      <circle cx="10" cy="10" r="4.4" fill="none" stroke="currentColor" strokeWidth="1.8" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg viewBox="0 0 20 20" width="18" height="18" aria-hidden="true">
      <path
        className="tick-draw"
        d="M4.5 10.5 8.5 14.5 15.5 6"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function BackIcon() {
  return (
    <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true">
      <path
        d="M12 4.5 6.5 10l5.5 5.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
