import api from "./api";
import { Restaurant } from "../types";

export async function getNearbyRestaurants(
  lat: number,
  lng: number,
  radius = 15000,
  limit = 20
): Promise<Restaurant[]> {
  const { data } = await api.get("/api/maps/restaurants/nearby", {
    params: { lat, lng, radius, limit },
  });
  return Array.isArray(data) ? data : [];
}

export async function getRestaurantById(id: string): Promise<Restaurant> {
  const { data } = await api.get(`/api/maps/restaurants/${id}`);
  return data;
}
