import { Link } from "react-router-dom";
import { Avatar, TextInput, ActionIcon, Paper, Text, Loader } from "@mantine/core";
import { Search, LocateFixed, X } from "lucide-react";
import { useApp } from "../../context/AppContext";
import { useNominatim } from "../../hooks/useNominatim";
import { useState, useRef, useEffect, useCallback } from "react";
import { createPortal } from "react-dom";

interface DropdownRect {
  top: number;
  left: number;
  width: number;
}

export default function TopBar() {
  const { profile, isSearchLocation, setSearchLocation } = useApp();
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [dropRect, setDropRect] = useState<DropdownRect | null>(null);
  const { results, loading } = useNominatim(query);
  const containerRef = useRef<HTMLDivElement>(null);

  const initial = profile.firstName.charAt(0).toUpperCase() || "?";

  const updateRect = useCallback(() => {
    if (!containerRef.current) return;
    const r = containerRef.current.getBoundingClientRect();
    setDropRect({ top: r.bottom + 6, left: r.left, width: r.width });
  }, []);

  useEffect(() => {
    if (results.length > 0) {
      updateRect();
      setOpen(true);
    }
  }, [results, updateRect]);

  useEffect(() => {
    function onClickOutside(e: MouseEvent) {
      const target = e.target as Node;
      const insideContainer = containerRef.current?.contains(target);
      const insidePortal = (target as Element)?.closest?.("[data-geo-dropdown]");
      if (!insideContainer && !insidePortal) setOpen(false);
    }
    document.addEventListener("mousedown", onClickOutside);
    return () => document.removeEventListener("mousedown", onClickOutside);
  }, []);

  function handleSelect(displayName: string, lat: string, lon: string) {
    setSearchLocation({ lat: parseFloat(lat), lng: parseFloat(lon) });
    setQuery(displayName.split(",").slice(0, 2).join(",").trim());
    setOpen(false);
  }

  function handleReset() {
    setSearchLocation(null);
    setQuery("");
    setOpen(false);
  }

  function handleClear() {
    setQuery("");
    setOpen(false);
    if (isSearchLocation) setSearchLocation(null);
  }

  const dropdown = open && results.length > 0 && dropRect
    ? createPortal(
        <Paper
          data-geo-dropdown
          shadow="md"
          radius="md"
          style={{
            position: "fixed",
            top: dropRect.top,
            left: dropRect.left,
            width: dropRect.width,
            zIndex: 99999,
            overflow: "hidden",
          }}
        >
          {results.map((r) => (
            <div
              key={r.place_id}
              onPointerDown={(e) => {
                e.preventDefault();
                handleSelect(r.display_name, r.lat, r.lon);
              }}
              style={{
                padding: "10px 14px",
                cursor: "pointer",
                borderBottom: "1px solid var(--mantine-color-gray-2)",
              }}
              onMouseEnter={(e) => (e.currentTarget.style.background = "var(--mantine-color-gray-1)")}
              onMouseLeave={(e) => (e.currentTarget.style.background = "")}
            >
              <Text size="sm" lineClamp={1}>{r.display_name}</Text>
            </div>
          ))}
        </Paper>,
        document.body
      )
    : null;

  return (
    <>
      <header
        className="fixed top-0 inset-x-0 h-14 flex items-center gap-2 px-3"
        style={{ background: "none", zIndex: 1001 }}
      >
        {/* Logo */}
        <Link to="/" style={{ textDecoration: "none", flexShrink: 0 }}>
          <ActionIcon size={36} radius="xl" color="michelin" variant="filled" aria-label="Home" component="span">
            <svg viewBox="-20 -200 960 1100" width="20" height="20" fill="white" fillRule="evenodd">
              <path d="M 624 69 v -49 q 0 -74 -47 -122 t -119 -48 q -78 0 -122 49 t -44 131 v 15 q 0 4 1 7 l 1 17 q -74 -53 -133 -53 q -46 0 -88 31 q -40 29 -59 76 q -14 33 -14 66 q 0 101 116 154 l 15 7 q -131 60 -131 162 q 0 69 48 121 t 114 52 q 60 0 119 -43 l 13 -10 q -2 20 -2 48 q 0 75 47 122.5 t 119 47.5 q 79 0 122.5 -49.5 t 43.5 -129.5 v -39 q 74 53 132 53 q 65 0 112.5 -52.5 t 47.5 -120.5 q 0 -102 -116 -155 l -15 -7 q 131 -60 131 -161 q 0 -65 -47 -119 t -113.5 -54 t -117.5 42 Z M 540 238 q 116 -169 215 -169 q 41 0 74.5 37 t 33.5 83 q 0 124 -268 147 v 29 q 133 11 200.5 48 t 67.5 99 q 0 45 -32.5 82.5 t -75.5 37.5 q -101 0 -215 -170 l -26 15 q 57 116 57 196 q 0 124 -113 124 q -54 0 -84 -33 q -29 -35 -29 -88 q 0 -80 57 -199 l -26 -15 q -114 170 -215 170 q -41 0 -74.5 -35.5 t -33.5 -84.5 q 0 -124 267 -147 v -29 q -267 -22 -267 -147 q 0 -44 32 -82 t 76 -38 q 99 0 215 169 l 26 -14 q -57 -117 -57 -200 q 0 -53 29 -88 q 30 -33 84 -33 q 113 0 113 123 q 0 81 -57 198 Z" />
            </svg>
          </ActionIcon>
        </Link>

        {/* Geo search */}
        <div ref={containerRef} style={{ flex: 1, minWidth: 0, position: "relative" }}>
          <TextInput
            placeholder="Search city or country…"
            value={query}
            onChange={(e) => setQuery(e.currentTarget.value)}
            onFocus={() => { if (results.length > 0) { updateRect(); setOpen(true); } }}
            leftSection={loading ? <Loader size={14} color="gray" /> : <Search size={15} />}
            rightSection={
              query.length > 0 ? (
                <ActionIcon size="sm" variant="subtle" color="gray" radius="xl" onClick={handleClear}>
                  <X size={13} />
                </ActionIcon>
              ) : null
            }
            radius="xl"
            size="sm"
            styles={{
              root: { background: "transparent" },
              wrapper: { background: "transparent" },
              input: {
                backgroundColor: "#fff",
                border: isSearchLocation ? "1.5px solid #AB152E" : "none",
              },
            }}
          />
        </div>

        {/* Reset to GPS */}
        {isSearchLocation && (
          <ActionIcon
            size={36}
            radius="xl"
            color="michelin"
            variant="light"
            aria-label="Back to my location"
            onClick={handleReset}
            title="Back to my location"
          >
            <LocateFixed size={18} />
          </ActionIcon>
        )}

        {/* Avatar */}
        <Link to="/profile" style={{ textDecoration: "none", flexShrink: 0 }}>
          <Avatar src={profile.avatarUrl ?? undefined} size={32} radius="xl" color="michelin">
            {initial}
          </Avatar>
        </Link>
      </header>

      {dropdown}
    </>
  );
}
