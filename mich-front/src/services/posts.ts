import api from "./api";
import { PostsResponse } from "../types";

export async function getRestaurantPosts(
  restaurantId: string,
  limit = 10,
  before?: string
): Promise<PostsResponse> {
  const { data } = await api.get(`/api/posts/restaurant/${restaurantId}`, {
    params: { limit, ...(before ? { before } : {}) },
  });
  return data;
}
