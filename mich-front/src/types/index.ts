export interface Restaurant {
  id: string;
  name: string;
  slug: string | null;
  chef: string | null;
  latitude: number | null;
  longitude: number | null;
  city: string | null;
  country_code: string | null;
  country_name: string | null;
  region_name: string | null;
  area_name: string | null;
  street: string | null;
  postcode: string | null;
  phone: string | null;
  website: string | null;
  short_link: string | null;
  michelin_award: string | null;
  distinction_score: number | null;
  guide_year: number | null;
  green_star: boolean | null;
  price_category_label: string | null;
  main_image_url: string | null;
  main_desc: string | null;
  online_booking: boolean | null;
  take_away: boolean | null;
  delivery: boolean | null;
  distance_meters?: number;
  total_posts?: number;
  good_pct?: number;
}

export interface Post {
  post_id: string;
  user_id: string | null;
  username: string | null;
  restaurant_id: string | null;
  restaurant_name: string | null;
  media_id: string | null;
  media_type: string | null;
  media_url: string | null;
  thumbnail_url: string | null;
  caption: string | null;
  rating: string | null;
  created_at: string | null;
}

export interface PostsResponse {
  data: Post[];
  next_before: string | null;
}

export interface Article {
  id: string;
  title: string;
  description: string;
  content: string;
  image_url: string;
  restaurant_name: string;
  restaurant_lat: number;
  restaurant_lng: number;
  author: string;
  created_at: string;
}

export interface UserLocation {
  lat: number;
  lng: number;
}

export interface UserProfile {
  firstName: string;
  lastName: string;
  avatarUrl: string | null;
}

export type MichelinAward =
  | "THREE_STARS"
  | "TWO_STARS"
  | "ONE_STAR"
  | "BIB_GOURMAND"
  | null;

export function awardStars(award: string | null): string {
  switch (award) {
    case "THREE_STARS": return "★★★";
    case "TWO_STARS":   return "★★";
    case "ONE_STAR":    return "★";
    case "BIB_GOURMAND": return "●";
    default:            return "";
  }
}

export function awardLabel(award: string | null): string {
  switch (award) {
    case "THREE_STARS": return "3 Stars";
    case "TWO_STARS":   return "2 Stars";
    case "ONE_STAR":    return "1 Star";
    case "BIB_GOURMAND": return "Bib Gourmand";
    default:            return "";
  }
}

export function formatDistance(meters: number): string {
  if (meters < 1000) return `${Math.round(meters)} m`;
  return `${(meters / 1000).toFixed(1)} km`;
}

export function timeAgo(isoDate: string | null): string {
  if (!isoDate) return "";
  const diff = Date.now() - new Date(isoDate).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}
