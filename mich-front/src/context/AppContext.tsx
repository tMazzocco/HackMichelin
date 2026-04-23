import React, { createContext, useContext, useState, useEffect, ReactNode } from "react";
import { Restaurant, UserLocation, UserProfile } from "../types";
import { useGeolocation } from "../hooks/useGeolocation";
import { getNearbyRestaurants } from "../services/restaurants";

interface AppContextValue {
  location: UserLocation | null;
  locationLoading: boolean;
  restaurants: Restaurant[];
  restaurantsLoading: boolean;
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
  const { location, loading: locationLoading } = useGeolocation();
  const [restaurants, setRestaurants] = useState<Restaurant[]>([]);
  const [restaurantsLoading, setRestaurantsLoading] = useState(false);
  const [profile, setProfileState] = useState<UserProfile>(() => {
    try {
      const stored = localStorage.getItem("mich_profile");
      return stored ? JSON.parse(stored) : DEFAULT_PROFILE;
    } catch {
      return DEFAULT_PROFILE;
    }
  });

  useEffect(() => {
    if (!location) return;
    setRestaurantsLoading(true);
    getNearbyRestaurants(location.lat, location.lng)
      .then(setRestaurants)
      .catch(() => setRestaurants([]))
      .finally(() => setRestaurantsLoading(false));
  }, [location]);

  function setProfile(p: UserProfile) {
    setProfileState(p);
    localStorage.setItem("mich_profile", JSON.stringify(p));
  }

  return (
    <AppContext.Provider
      value={{ location, locationLoading, restaurants, restaurantsLoading, profile, setProfile }}
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
