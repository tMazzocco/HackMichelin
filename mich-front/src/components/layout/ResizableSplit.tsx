import { useState, useRef, useCallback, ReactNode } from "react";

interface Props {
  top: ReactNode;
  bottom: ReactNode;
  /** Initial share of the container taken by the top panel (0–100) */
  defaultTopPercent?: number;
  minTopPercent?: number;
  maxTopPercent?: number;
}

export default function ResizableSplit({
  top,
  bottom,
  defaultTopPercent = 40,
  minTopPercent = 10,
  maxTopPercent = 85,
}: Props) {
  const [topPct, setTopPct] = useState(defaultTopPercent);
  const containerRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  const onPointerDown = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    // Capture keeps pointermove firing on this element even when the finger/cursor strays
    e.currentTarget.setPointerCapture(e.pointerId);
    dragging.current = true;
  }, []);

  const onPointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragging.current || !containerRef.current) return;
      const { top: cTop, height } = containerRef.current.getBoundingClientRect();
      const pct = ((e.clientY - cTop) / height) * 100;
      setTopPct(Math.min(Math.max(pct, minTopPercent), maxTopPercent));
    },
    [minTopPercent, maxTopPercent]
  );

  const onPointerUp = useCallback(() => {
    dragging.current = false;
  }, []);

  return (
    <div ref={containerRef} className="flex flex-col h-full w-full overflow-hidden">
      {/* Top panel */}
      <div className="flex-shrink-0 overflow-hidden" style={{ height: `${topPct}%` }}>
        {top}
      </div>

      {/* Drag handle */}
      <div
        className="flex-shrink-0 h-5 flex items-center justify-center bg-background border-y border-black/5 shadow-sm z-10 cursor-ns-resize select-none"
        style={{ touchAction: "none" }}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        <div className="w-10 h-1 rounded-full bg-black/20" />
      </div>

      {/* Bottom panel */}
      <div className="flex-1 overflow-hidden min-h-0">
        {bottom}
      </div>
    </div>
  );
}
