import { useState, useCallback } from "react";
import { Search, X } from "lucide-react";
import { useApp } from "../context/AppContext";
import MapView from "../components/map/MapView";
import LoadingSpinner from "../components/common/LoadingSpinner";
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
        <div className="relative flex items-center">
          <Search size={16} className="absolute left-3 text-text/40 pointer-events-none" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search restaurants or cities…"
            className="w-full pl-9 pr-9 py-2.5 rounded-xl bg-background/95 backdrop-blur shadow-lg text-sm border border-black/10 outline-none focus:border-primary/40"
          />
          {query && (
            <button onClick={handleClear} className="absolute right-3 text-text/40">
              <X size={16} />
            </button>
          )}
        </div>

        {/* Filtered list dropdown */}
        {query.trim() && filtered.length > 0 && (
          <div className="mt-1 bg-background rounded-xl shadow-xl overflow-hidden max-h-56 overflow-y-auto border border-black/10">
            {filtered.slice(0, 10).map((r) => (
              <Link
                key={r.id}
                to={`/restaurant/${r.id}`}
                className="flex items-center gap-3 px-4 py-2.5 hover:bg-black/5 border-b border-black/5 last:border-0"
              >
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{r.name}</p>
                  <p className="text-xs text-text/40 truncate">{r.city}</p>
                </div>
                {r.michelin_award && (
                  <span className="text-primary text-xs font-bold">{awardStars(r.michelin_award)}</span>
                )}
                {r.distance_meters != null && (
                  <span className="text-xs text-text/40">{formatDistance(r.distance_meters)}</span>
                )}
              </Link>
            ))}
          </div>
        )}
      </div>

      {/* Map */}
      <div className="flex-1">
        {location ? (
          <MapView location={location} restaurants={filtered} zoom={13} interactive />
        ) : (
          <div className="h-full flex items-center justify-center">
            <LoadingSpinner size={32} />
          </div>
        )}
      </div>

      {/* Bottom sheet — selected restaurant */}
      {selected && (
        <div className="absolute bottom-20 inset-x-4 z-[1000] bg-background rounded-2xl shadow-xl p-4 flex items-center gap-4">
          <img
            src={selected.main_image_url ?? `https://picsum.photos/seed/${selected.id}/80/80`}
            alt={selected.name}
            className="w-16 h-16 rounded-xl object-cover"
          />
          <div className="flex-1 min-w-0">
            <p className="font-semibold text-sm truncate">{selected.name}</p>
            {selected.city && <p className="text-xs text-text/40">{selected.city}</p>}
            {selected.distance_meters != null && (
              <p className="text-xs text-secondary mt-0.5">{formatDistance(selected.distance_meters)}</p>
            )}
          </div>
          <Link
            to={`/restaurant/${selected.id}`}
            className="bg-primary text-white text-xs font-semibold px-3 py-2 rounded-xl"
          >
            View
          </Link>
        </div>
      )}

      {restaurantsLoading && (
        <div className="absolute bottom-24 left-1/2 -translate-x-1/2 z-[1000] bg-background/90 rounded-full px-4 py-2 flex items-center gap-2 shadow">
          <LoadingSpinner size={16} />
          <span className="text-xs">Loading…</span>
        </div>
      )}
    </div>
  );
}
