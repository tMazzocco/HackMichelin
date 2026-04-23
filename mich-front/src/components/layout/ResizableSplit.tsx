import { useState, useRef, useCallback, ReactNode } from "react";

interface Props {
  top: ReactNode;
  bottom: ReactNode;
  defaultTopPercent?: number;
  minTopPercent?: number;
  maxTopPercent?: number;
  onTopPctChange?: (pct: number) => void;
}

export default function ResizableSplit({
  top,
  bottom,
  defaultTopPercent = 40,
  minTopPercent = 0,
  maxTopPercent = 85,
  onTopPctChange,
}: Props) {
  const [topPct, setTopPct] = useState(defaultTopPercent);
  const containerRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  const onPointerDown = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    dragging.current = true;
  }, []);

  const onPointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragging.current || !containerRef.current) return;
      const { top: cTop, height } = containerRef.current.getBoundingClientRect();
      const pct = ((e.clientY - cTop) / height) * 100;
      const clamped = Math.min(Math.max(pct, minTopPercent), maxTopPercent);
      setTopPct(clamped);
      onTopPctChange?.(clamped);
    },
    [minTopPercent, maxTopPercent, onTopPctChange]
  );

  const onPointerUp = useCallback(() => {
    dragging.current = false;
  }, []);

  return (
    <div ref={containerRef} className="flex flex-col h-full w-full overflow-hidden">
      <div className="flex-shrink-0 overflow-hidden" style={{ height: `${topPct}%` }}>
        {top}
      </div>

      <div
        className="flex-shrink-0 z-10 cursor-ns-resize select-none"
        style={{ touchAction: "none", position: "relative", height: 32 }}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        {/* Full-width divider line */}
        <div style={{
          position: "absolute",
          top: "50%",
          left: 0,
          right: 0,
          height: 1,
          background: "rgba(0,0,0,0.08)",
          transform: "translateY(-50%)",
        }} />

        {/* Floating pill centered on the line */}
        <div style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          background: "#fff",
          borderRadius: 99,
          padding: "5px 18px",
          boxShadow: "0 2px 12px rgba(0,0,0,0.13)",
          display: "flex",
          alignItems: "center",
          gap: 6,
        }}>
          <div style={{ width: 18, height: 3, borderRadius: 99, background: "#e8e8e8" }} />
          <div style={{ width: 6, height: 6, borderRadius: 99, background: "#AB152E", boxShadow: "0 0 6px rgba(171,21,46,0.5)" }} />
          <div style={{ width: 18, height: 3, borderRadius: 99, background: "#e8e8e8" }} />
        </div>
      </div>

      <div className="flex-1 overflow-hidden min-h-0">
        {bottom}
      </div>
    </div>
  );
}
