import api from "./api";
import { Post, PostsResponse, Restaurant } from "../types";
import { getRestaurantById } from "./restaurants";

// ---------------------------------------------------------------------------
// Temporary: synthesise a Post-shaped item from a Restaurant's main_image_url
// so the restaurant cover can appear as the first card in a post scroll.
// Remove this section once a proper "restaurant cover post" feature is built.
// ---------------------------------------------------------------------------
function restaurantCoverPost(restaurant: Restaurant): Post {
  return {
    post_id:         `cover_${restaurant.id}`,
    user_id:         null,
    username:        null,
    restaurant_id:   restaurant.id,
    restaurant_name: restaurant.name,
    media_id:        null,
    media_type:      "photo",
    media_url:       restaurant.main_image_url,
    thumbnail_url:   restaurant.main_image_url,
    caption:         restaurant.main_desc ?? null,
    rating:          null,
    created_at:      null,
  };
}
// ---------------------------------------------------------------------------

export async function getRestaurantPosts(
  restaurantId: string,
  limit = 10,
  before?: string
): Promise<PostsResponse> {
  const [postsData, restaurant] = await Promise.all([
    api.get(`/api/posts/restaurant/${restaurantId}`, {
      params: { limit, ...(before ? { before } : {}) },
    }).then((r) => r.data as PostsResponse),
    getRestaurantById(restaurantId),
  ]);

  // Prepend the cover only on the first page (no `before` cursor).
  const cover = !before && restaurant.main_image_url
    ? [restaurantCoverPost(restaurant)]
    : [];

  return {
    data:        [...cover, ...postsData.data],
    next_before: postsData.next_before,
  };
}
