import { createContext, useContext, useState, useEffect, ReactNode } from "react";
import { Restaurant, UserLocation, UserProfile } from "../types";
import { useGeolocation } from "../hooks/useGeolocation";
import { getNearbyRestaurants } from "../services/restaurants";

interface AppContextValue {
  location: UserLocation | null;
  locationLoading: boolean;
  isSearchLocation: boolean;
  setSearchLocation: (loc: UserLocation | null) => void;
  restaurants: Restaurant[];
  restaurantsLoading: boolean;
  restaurantsError: boolean;
  profile: UserProfile;
  setProfile: (p: UserProfile) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

const DEFAULT_PROFILE: UserProfile = {
  firstName: "Guest",
  lastName: "",
  avatarUrl: null,
};

export function AppProvider({ children }: { children: ReactNode }) {
  const { location: gpsLocation, loading: locationLoading } = useGeolocation();
  const [searchLocation, setSearchLocation] = useState<UserLocation | null>(null);
  const [restaurants, setRestaurants] = useState<Restaurant[]>([]);
  const [restaurantsLoading, setRestaurantsLoading] = useState(false);
  const [restaurantsError, setRestaurantsError] = useState(false);
  const [profile, setProfileState] = useState<UserProfile>(() => {
    try {
      const stored = localStorage.getItem("mich_profile");
      return stored ? JSON.parse(stored) : DEFAULT_PROFILE;
    } catch {
      return DEFAULT_PROFILE;
    }
  });

  const location = searchLocation ?? gpsLocation;

  const latKey = location ? Math.round(location.lat * 100) / 100 : null;
  const lngKey = location ? Math.round(location.lng * 100) / 100 : null;

  useEffect(() => {
    if (!location) return;
    let cancelled = false;

    setRestaurantsLoading(true);
    setRestaurantsError(false);

    const { lat, lng } = location;

    Promise.all([
      getNearbyRestaurants(lat, lng, 20_000, 200),
      getNearbyRestaurants(lat, lng, 75_000, 200),
    ])
      .then(([close, far]) => {
        if (cancelled) return;
        const seen = new Set<string>();
        const merged: Restaurant[] = [];
        for (const r of [...close, ...far]) {
          if (!seen.has(r.id)) { seen.add(r.id); merged.push(r); }
        }
        setRestaurants(merged);
        setRestaurantsError(false);
      })
      .catch(() => { if (!cancelled) { setRestaurants([]); setRestaurantsError(true); } })
      .finally(() => { if (!cancelled) setRestaurantsLoading(false); });

    return () => { cancelled = true; };
  }, [latKey, lngKey]); // eslint-disable-line react-hooks/exhaustive-deps

  function setProfile(p: UserProfile) {
    setProfileState(p);
    localStorage.setItem("mich_profile", JSON.stringify(p));
  }

  return (
    <AppContext.Provider
      value={{
        location,
        locationLoading,
        isSearchLocation: searchLocation !== null,
        setSearchLocation,
        restaurants,
        restaurantsLoading,
        restaurantsError,
        profile,
        setProfile,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used inside AppProvider");
  return ctx;
}
