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
            <svg viewBox="0 0 24 24" width="20" height="20" fill="white">
              <path d="M12,2 L14.5,7.67 L20.66,7 L17,12 L20.66,17 L14.5,16.33 L12,22 L9.5,16.33 L3.34,17 L7,12 L3.34,7 L9.5,7.67 Z" />
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
