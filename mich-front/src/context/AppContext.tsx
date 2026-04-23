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

  useEffect(() => {
    if (!location) return;
    setRestaurantsLoading(true);
    setRestaurantsError(false);
    getNearbyRestaurants(location.lat, location.lng, 20_000, 50)
      .then((data) => { setRestaurants(data); setRestaurantsError(false); })
      .catch(() => { setRestaurants([]); setRestaurantsError(true); })
      .finally(() => setRestaurantsLoading(false));
  }, [location?.lat, location?.lng]);

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
