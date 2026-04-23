import { useState, useEffect, useRef } from "react";

export interface GeoResult {
  place_id: number;
  display_name: string;
  lat: string;
  lon: string;
}

const MIN_CHARS = 3;
const DEBOUNCE_MS = 600;
const MIN_INTERVAL_MS = 1100;

export function useNominatim(query: string) {
  const [results, setResults] = useState<GeoResult[]>([]);
  const [loading, setLoading] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastFetchAt = useRef<number>(0);

  useEffect(() => {
    const q = query.trim();

    if (q.length < MIN_CHARS) {
      setResults([]);
      return;
    }

    if (timerRef.current) clearTimeout(timerRef.current);

    timerRef.current = setTimeout(async () => {
      const elapsed = Date.now() - lastFetchAt.current;
      if (elapsed < MIN_INTERVAL_MS) {
        await new Promise<void>((r) => setTimeout(r, MIN_INTERVAL_MS - elapsed));
      }

      setLoading(true);
      lastFetchAt.current = Date.now();

      try {
        const url = `https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(q)}&format=json&limit=5&addressdetails=0`;
        const res = await fetch(url, {
          headers: { "User-Agent": "HackMichelin/1.0 (theo.mazzocco47@gmail.com)" },
        });
        setResults(await res.json());
      } catch {
        setResults([]);
      } finally {
        setLoading(false);
      }
    }, DEBOUNCE_MS);

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [query]);

  return { results, loading };
}
