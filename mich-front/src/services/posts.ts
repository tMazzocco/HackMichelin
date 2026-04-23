import api from "./api";
import { PostsResponse } from "../types";

export async function getRestaurantPosts(
  restaurantId: string,
  limit = 10,
  before?: string
): Promise<PostsResponse> {
  console.log(`Fetching posts for restaurant ${restaurantId} with limit ${limit} and before ${before}`);
  const { data } = await api.get(`/api/posts/restaurant/${restaurantId}`, {
    params: { limit, ...(before ? { before } : {}) },
  });
  console.log("Fetched posts:", data);
  return data;
}
