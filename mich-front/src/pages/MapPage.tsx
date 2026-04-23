import { useState, useCallback } from "react";
import { TextInput, ActionIcon, Paper, Group, Text } from "@mantine/core";
import { Search, X } from "lucide-react";
import { useApp } from "../context/AppContext";
import MapView from "../components/map/MapView";
import MapErrorBoundary from "../components/map/MapErrorBoundary";
import { Loader } from "@mantine/core";
import { Restaurant, awardStars, formatDistance } from "../types";
import { Link } from "react-router-dom";

export default function MapPage() {
  const { location, restaurants, restaurantsLoading } = useApp();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<Restaurant | null>(null);

  const filtered = query.trim()
    ? restaurants.filter((r) =>
        r.name.toLowerCase().includes(query.toLowerCase()) ||
        (r.city ?? "").toLowerCase().includes(query.toLowerCase())
      )
    : restaurants;

  const handleClear = useCallback(() => {
    setQuery("");
    setSelected(null);
  }, []);

  return (
    <div className="fixed inset-0 flex flex-col">
      {/* Search bar */}
      <div className="absolute top-4 inset-x-4 z-[1000]">
        <TextInput
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search restaurants or cities…"
          radius="xl"
          size="md"
          leftSection={<Search size={16} />}
          rightSection={
            query ? (
              <ActionIcon variant="transparent" color="gray" onClick={handleClear}>
                <X size={16} />
              </ActionIcon>
            ) : null
          }
          styles={{
            input: {
              background: "rgba(var(--background-rgb, 251,251,254),0.95)",
              backdropFilter: "blur(12px)",
              boxShadow: "0 4px 20px rgba(0,0,0,0.12)",
            },
          }}
        />

        {/* Filtered list dropdown */}
        {query.trim() && filtered.length > 0 && (
          <Paper radius="xl" shadow="xl" mt={4} style={{ overflow: "hidden", maxHeight: 224, overflowY: "auto", border: "1px solid rgba(0,0,0,0.08)" }}>
            {filtered.slice(0, 10).map((r) => (
              <button
                key={r.id}
                onClick={() => { setSelected(r); setQuery(""); }}
                style={{ width: "100%", background: "none", border: "none", cursor: "pointer", borderBottom: "1px solid rgba(0,0,0,0.05)", padding: 0 }}
              >
                <Group px="md" py="sm" justify="space-between" wrap="nowrap">
                  <div style={{ flex: 1, minWidth: 0, textAlign: "left" }}>
                    <Text size="sm" fw={500} lineClamp={1}>{r.name}</Text>
                    <Text size="xs" c="dimmed" lineClamp={1}>{r.city}</Text>
                  </div>
                  <Group gap="xs" wrap="nowrap">
                    {r.michelin_award && (
                      <Text size="xs" fw={700} c="michelin">{awardStars(r.michelin_award)}</Text>
                    )}
                    {r.distance_meters != null && (
                      <Text size="xs" c="dimmed">{formatDistance(r.distance_meters)}</Text>
                    )}
                  </Group>
                </Group>
              </button>
            ))}
          </Paper>
        )}
      </div>

      {/* Map */}
      <div className="flex-1">
        {location ? (
          <MapErrorBoundary>
            <MapView location={location} restaurants={filtered} zoom={13} interactive />
          </MapErrorBoundary>
        ) : (
          <div className="h-full flex items-center justify-center">
            <Loader color="michelin" size={32} />
          </div>
        )}
      </div>

      {/* Bottom sheet — selected restaurant */}
      {selected && (
        <Paper
          radius="xl"
          shadow="xl"
          p="md"
          style={{ position: "absolute", bottom: 80, left: 16, right: 16, zIndex: 1000 }}
        >
          <Group gap="md" wrap="nowrap">
            <img
              src={selected.main_image_url ?? `https://picsum.photos/seed/${selected.id}/80/80`}
              alt={selected.name}
              style={{ width: 64, height: 64, borderRadius: 12, objectFit: "cover", flexShrink: 0 }}
            />
            <div style={{ flex: 1, minWidth: 0 }}>
              <Text fw={600} size="sm" lineClamp={1}>{selected.name}</Text>
              {selected.city && <Text size="xs" c="dimmed">{selected.city}</Text>}
              {selected.distance_meters != null && (
                <Text size="xs" c="michelin" mt={2}>{formatDistance(selected.distance_meters)}</Text>
              )}
            </div>
            <ActionIcon
              component={Link}
              to={`/restaurant/${selected.id}`}
              color="michelin"
              variant="filled"
              radius="xl"
              size="lg"
              style={{ flexShrink: 0 }}
            >
              →
            </ActionIcon>
          </Group>
        </Paper>
      )}

      {restaurantsLoading && (
        <Paper
          radius="xl"
          shadow="md"
          px="md"
          py="xs"
          style={{ position: "absolute", bottom: 96, left: "50%", transform: "translateX(-50%)", zIndex: 1000 }}
        >
          <Group gap="xs">
            <Loader color="michelin" size={16} />
            <Text size="xs">Loading…</Text>
          </Group>
        </Paper>
      )}
    </div>
  );
}
