import { useEffect, useRef, useState } from "react";

/** Counts up each time the popover gains focus, so entrance animations replay on open. */
export function useOpenCount(): number {
  const [opens, setOpens] = useState(0);
  useEffect(() => {
    const opened = () => {
      setOpens((count) => count + 1);
    };
    window.addEventListener("focus", opened);
    return () => {
      window.removeEventListener("focus", opened);
    };
  }, []);
  return opens;
}

const reducedMotion = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** `target`, eased in from 0 when `replay` changes and from the last value otherwise. */
export function useCountUp(target: number, replay: number, duration = 700): number {
  const [shown, setShown] = useState(target);
  const last = useRef({ value: 0, replay });
  useEffect(() => {
    const start = performance.now();
    const origin = last.current.replay === replay ? last.current.value : 0;
    last.current.replay = replay;
    let frame = requestAnimationFrame(function step(now) {
      const t = reducedMotion() ? 1 : Math.min((now - start) / duration, 1);
      const value = Math.round(origin + (target - origin) * (1 - (1 - t) ** 3));
      last.current.value = value;
      setShown(value);
      if (t < 1) frame = requestAnimationFrame(step);
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [target, replay, duration]);
  return shown;
}
