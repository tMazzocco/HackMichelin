import { useState, useEffect } from "react";
import { UserLocation } from "../types";

const PARIS_FALLBACK: UserLocation = { lat: 48.8566, lng: 2.3522 };

export function useGeolocation() {
  const [location, setLocation] = useState<UserLocation | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!navigator.geolocation) {
      setLocation(PARIS_FALLBACK);
      setLoading(false);
      return;
    }
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        setLocation({ lat: pos.coords.latitude, lng: pos.coords.longitude });
        setLoading(false);
      },
      () => {
        setLocation(PARIS_FALLBACK);
        setLoading(false);
      },
      { timeout: 5000 }
    );
  }, []);

  return { location, loading };
}
