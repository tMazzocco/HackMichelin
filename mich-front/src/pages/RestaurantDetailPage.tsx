import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Globe, Phone, MapPin, Star } from "lucide-react";
import { getRestaurantById } from "../services/restaurants";
import { getRestaurantPosts } from "../services/posts";
import { Restaurant, Post, awardStars, awardLabel, timeAgo } from "../types";
import LoadingSpinner from "../components/common/LoadingSpinner";
import MapView from "../components/map/MapView";

export default function RestaurantDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [restaurant, setRestaurant] = useState<Restaurant | null>(null);
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;
    Promise.all([getRestaurantById(id), getRestaurantPosts(id, 12)])
      .then(([r, p]) => {
        setRestaurant(r);
        setPosts(p.data);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) {
    return (
      <div className="fixed inset-0 flex items-center justify-center">
        <LoadingSpinner size={32} />
      </div>
    );
  }

  if (!restaurant) {
    return (
      <div className="page pt-14 pb-20 flex items-center justify-center text-text/40">
        Restaurant not found.
      </div>
    );
  }

  const stars = awardStars(restaurant.michelin_award);
  const label = awardLabel(restaurant.michelin_award);
  const heroImg =
    restaurant.main_image_url ?? `https://picsum.photos/seed/${restaurant.id}/800/500`;

  return (
    <div className="page pb-24">
      {/* Hero */}
      <div className="relative h-64">
        <img src={heroImg} alt={restaurant.name} className="w-full h-full object-cover" />
        <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent" />
        <button
          onClick={() => navigate(-1)}
          className="absolute top-12 left-4 w-9 h-9 rounded-full bg-black/40 backdrop-blur flex items-center justify-center text-white"
        >
          <ArrowLeft size={18} />
        </button>
        {stars && (
          <div className="absolute top-12 right-4 bg-primary text-white text-sm font-bold px-3 py-1 rounded-full">
            {stars}
          </div>
        )}
        <div className="absolute bottom-4 left-4 right-4">
          <h1 className="text-white font-bold text-2xl leading-tight">{restaurant.name}</h1>
          {label && <p className="text-secondary text-sm mt-0.5">{label}</p>}
        </div>
      </div>

      {/* Meta */}
      <div className="px-4 mt-4 flex flex-col gap-2">
        {restaurant.city && (
          <div className="flex items-center gap-2 text-text/60 text-sm">
            <MapPin size={14} className="text-primary" />
            <span>
              {[restaurant.street, restaurant.city, restaurant.country_name]
                .filter(Boolean)
                .join(", ")}
            </span>
          </div>
        )}
        {restaurant.phone && (
          <div className="flex items-center gap-2 text-text/60 text-sm">
            <Phone size={14} className="text-primary" />
            <a href={`tel:${restaurant.phone}`}>{restaurant.phone}</a>
          </div>
        )}
        {restaurant.website && (
          <div className="flex items-center gap-2 text-text/60 text-sm">
            <Globe size={14} className="text-primary" />
            <a href={restaurant.website} target="_blank" rel="noreferrer" className="truncate text-primary">
              {restaurant.website.replace(/^https?:\/\//, "")}
            </a>
          </div>
        )}
        {restaurant.price_category_label && (
          <div className="flex items-center gap-2 text-text/60 text-sm">
            <Star size={14} className="text-primary" />
            <span>{restaurant.price_category_label}</span>
          </div>
        )}
      </div>

      {/* Description */}
      {restaurant.main_desc && (
        <p className="px-4 mt-4 text-text/70 text-sm leading-relaxed">{restaurant.main_desc}</p>
      )}

      {/* Stats */}
      {(restaurant.total_posts != null || restaurant.good_pct != null) && (
        <div className="mx-4 mt-4 grid grid-cols-2 gap-3">
          {restaurant.total_posts != null && (
            <div className="rounded-xl bg-black/5 p-3 text-center">
              <p className="font-bold text-lg">{restaurant.total_posts}</p>
              <p className="text-xs text-text/40">Posts</p>
            </div>
          )}
          {restaurant.good_pct != null && (
            <div className="rounded-xl bg-black/5 p-3 text-center">
              <p className="font-bold text-lg">{Math.round(restaurant.good_pct * 100)}%</p>
              <p className="text-xs text-text/40">Positive</p>
            </div>
          )}
        </div>
      )}

      {/* Map */}
      {restaurant.latitude && restaurant.longitude && (
        <div className="px-4 mt-5">
          <h2 className="font-semibold text-sm mb-2">Location</h2>
          <div className="rounded-2xl overflow-hidden h-40 shadow-md">
            <MapView
              location={{ lat: restaurant.latitude, lng: restaurant.longitude }}
              restaurants={[restaurant]}
              zoom={15}
              interactive={false}
            />
          </div>
        </div>
      )}

      {/* Posts grid */}
      {posts.length > 0 && (
        <div className="px-4 mt-5">
          <h2 className="font-semibold text-sm mb-3">Moments</h2>
          <div className="grid grid-cols-3 gap-1">
            {posts.map((post) => {
              const thumb = post.thumbnail_url ?? post.media_url;
              return (
                <div key={post.post_id} className="aspect-square rounded-lg overflow-hidden bg-black/10 relative">
                  {thumb ? (
                    <img src={thumb} alt="" className="w-full h-full object-cover" />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-text/20 text-xs">
                      No media
                    </div>
                  )}
                  {post.rating && (
                    <span className="absolute bottom-1 right-1 text-sm">
                      {post.rating === "good" ? "👍" : "👎"}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
