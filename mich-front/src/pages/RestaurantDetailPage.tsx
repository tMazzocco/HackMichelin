import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Text, Anchor, Loader } from "@mantine/core";
import { ArrowLeft, Heart, Share2, MapPin, Phone, Globe } from "lucide-react";
import { getRestaurantById } from "../services/restaurants";
import { getRestaurantPosts } from "../services/posts";
import { Restaurant, Post, awardStars, awardLabel } from "../types";
import MapView from "../components/map/MapView";
import MapErrorBoundary from "../components/map/MapErrorBoundary";

export default function RestaurantDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [restaurant, setRestaurant] = useState<Restaurant | null>(null);
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    if (!id) return;
    setError(false);
    getRestaurantById(id)
      .then((r) => {
        setRestaurant(r);
        return getRestaurantPosts(id, 12).then((p) => setPosts(p.data)).catch(() => {});
      })
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) {
    return (
      <div className="fixed inset-0 flex items-center justify-center">
        <Loader color="michelin" size={32} />
      </div>
    );
  }

  if (error || !restaurant) {
    return (
      <div className="page pt-14 pb-20 flex flex-col items-center justify-center gap-3 px-6 text-center">
        <Text size="md" fw={600} c="dimmed">Restaurant not found</Text>
        <Text size="sm" c="dimmed">This restaurant may have been removed or is temporarily unavailable.</Text>
        <Anchor component="button" c="michelin" size="sm" fw={500} mt="xs" onClick={() => navigate(-1)}>
          Go back
        </Anchor>
      </div>
    );
  }

  const stars = awardStars(restaurant.michelin_award);
  const label = awardLabel(restaurant.michelin_award);
  const heroImg = restaurant.main_image_url ?? `https://picsum.photos/seed/${restaurant.id}/800/500`;
  const addressLine = [restaurant.street, restaurant.postcode].filter(Boolean).join(" ");
  const cityCountry = [restaurant.city, restaurant.country_name].filter(Boolean).join(", ");

  const badgeText = stars
    ? `${stars} ${label}${restaurant.guide_year ? ` · Guide ${restaurant.guide_year}` : ""}`
    : `Selected${restaurant.guide_year ? ` (Guide ${restaurant.guide_year})` : ""}`;

  const iconBtn: React.CSSProperties = {
    width: 36, height: 36, borderRadius: "50%",
    background: "rgba(255,255,255,0.92)", border: "none", cursor: "pointer",
    display: "flex", alignItems: "center", justifyContent: "center",
    backdropFilter: "blur(8px)", boxShadow: "0 2px 8px rgba(0,0,0,0.15)",
  };

  const outlineBtn: React.CSSProperties = {
    display: "flex", alignItems: "center", justifyContent: "center", gap: 8,
    background: "#fff", border: "1.5px solid #AB152E", color: "#AB152E",
    borderRadius: 10, padding: "11px 0", fontSize: 14, fontWeight: 600,
    textDecoration: "none", cursor: "pointer",
  };

  const chip: React.CSSProperties = {
    display: "inline-flex", alignItems: "center", gap: 5,
    border: "1px solid #e5e5e5", borderRadius: 20,
    padding: "5px 12px", fontSize: 12, fontWeight: 500, color: "#444",
    whiteSpace: "nowrap",
  };

  return (
    <div className="page pb-24" style={{ background: "#fff" }}>
      {/* Floating top nav */}
      <div style={{ position: "fixed", top: 0, left: 0, right: 0, zIndex: 100, display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 16px" }}>
        <button onClick={() => navigate(-1)} style={iconBtn}>
          <ArrowLeft size={18} color="#111" />
        </button>
        <div style={{ display: "flex", gap: 8 }}>
          <button style={iconBtn}><Heart size={18} color="#111" /></button>
          <button style={iconBtn}><Share2 size={18} color="#111" /></button>
        </div>
      </div>

      {/* Hero */}
      <div style={{ position: "relative", height: 260 }}>
        <img src={heroImg} alt={restaurant.name} style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />
        <div style={{
          position: "absolute", bottom: 12, left: 12,
          background: "#AB152E", color: "#fff",
          borderRadius: 20, padding: "5px 12px",
          display: "inline-flex", alignItems: "center", gap: 5,
          fontSize: 12, fontWeight: 600,
        }}>
          <MapPin size={11} />
          {badgeText}
          {restaurant.green_star && <span style={{ marginLeft: 2 }}>🌿</span>}
        </div>
      </div>

      {/* Content */}
      <div style={{ padding: "20px 16px 0" }}>
        {/* Name */}
        <h1 style={{ fontSize: 28, fontWeight: 800, margin: "0 0 10px", lineHeight: 1.15, color: "#111" }}>
          {restaurant.name}
        </h1>

        {/* Address */}
        {(addressLine || cityCountry) && (
          <div style={{ display: "flex", alignItems: "flex-start", gap: 6, marginBottom: 18, color: "#555" }}>
            <MapPin size={14} color="#AB152E" style={{ flexShrink: 0, marginTop: 2 }} />
            <div style={{ fontSize: 13, lineHeight: 1.5 }}>
              {addressLine && <div>{addressLine}</div>}
              {cityCountry && <div>{cityCountry}</div>}
            </div>
          </div>
        )}

        {/* Chips */}
        {(restaurant.price_category_label || restaurant.online_booking) && (
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 20 }}>
            {restaurant.price_category_label && (
              <span style={chip}>€ {restaurant.price_category_label}</span>
            )}
            {restaurant.online_booking && (
              <span style={chip}>📅 Reservable</span>
            )}
            {restaurant.take_away && (
              <span style={chip}>🥡 Take away</span>
            )}
            {restaurant.delivery && (
              <span style={chip}>🛵 Delivery</span>
            )}
          </div>
        )}

        {/* CTA buttons */}
        {(restaurant.phone || restaurant.website) && (
          <div style={{ display: "grid", gridTemplateColumns: restaurant.phone && restaurant.website ? "1fr 1fr" : "1fr", gap: 10, marginBottom: 28 }}>
            {restaurant.phone && (
              <a href={`tel:${restaurant.phone}`} style={outlineBtn}>
                <Phone size={16} />
                Call
              </a>
            )}
            {restaurant.website && (
              <a href={restaurant.website} target="_blank" rel="noreferrer" style={outlineBtn}>
                <Globe size={16} />
                Website
              </a>
            )}
          </div>
        )}

        {/* About */}
        {restaurant.main_desc && (
          <section style={{ marginBottom: 28 }}>
            <h2 style={{ fontSize: 18, fontWeight: 700, margin: "0 0 10px", color: "#111" }}>About</h2>
            <p style={{ fontSize: 14, lineHeight: 1.75, color: "#555", margin: 0 }}>
              {restaurant.main_desc}
            </p>
          </section>
        )}

        {/* Stats */}
        {(restaurant.total_posts != null || restaurant.good_pct != null) && (
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, marginBottom: 28 }}>
            {restaurant.total_posts != null && (
              <div style={{ border: "1px solid #eee", borderRadius: 12, padding: "14px", textAlign: "center" }}>
                <div style={{ fontSize: 22, fontWeight: 700, color: "#111" }}>{restaurant.total_posts}</div>
                <div style={{ fontSize: 12, color: "#888", marginTop: 2 }}>Posts</div>
              </div>
            )}
            {restaurant.good_pct != null && (
              <div style={{ border: "1px solid #eee", borderRadius: 12, padding: "14px", textAlign: "center" }}>
                <div style={{ fontSize: 22, fontWeight: 700, color: "#111" }}>{Math.round(restaurant.good_pct * 100)}%</div>
                <div style={{ fontSize: 12, color: "#888", marginTop: 2 }}>Positive</div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Map */}
      {restaurant.latitude && restaurant.longitude && (
        <div style={{ padding: "0 16px", marginBottom: 28 }}>
          <h2 style={{ fontSize: 18, fontWeight: 700, margin: "0 0 10px", color: "#111" }}>Location</h2>
          <div style={{ borderRadius: 16, overflow: "hidden", height: 160 }}>
            <MapErrorBoundary height="160px">
              <MapView
                location={{ lat: restaurant.latitude, lng: restaurant.longitude }}
                restaurants={[restaurant]}
                zoom={15}
                interactive={false}
              />
            </MapErrorBoundary>
          </div>
        </div>
      )}

      {/* Posts grid */}
      {posts.length > 0 && (
        <div style={{ padding: "0 16px" }}>
          <h2 style={{ fontSize: 18, fontWeight: 700, margin: "0 0 10px", color: "#111" }}>Moments</h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 4 }}>
            {posts.map((post) => {
              const thumb = post.thumbnail_url ?? post.media_url;
              return (
                <div
                  key={post.post_id}
                  style={{ aspectRatio: "1", borderRadius: 8, overflow: "hidden", background: "#f0f0f0", position: "relative", cursor: "pointer" }}
                  onClick={() => navigate("/shorts", { state: { initialPost: post } })}
                >
                  {thumb && <img src={thumb} alt="" style={{ width: "100%", height: "100%", objectFit: "cover" }} />}
                  {post.media_type === "video" && (
                    <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
                      <div style={{ width: 28, height: 28, borderRadius: "50%", background: "rgba(0,0,0,0.45)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                        <div style={{ width: 0, height: 0, borderTop: "6px solid transparent", borderBottom: "6px solid transparent", borderLeft: "10px solid white", marginLeft: 2 }} />
                      </div>
                    </div>
                  )}
                  {post.rating && (
                    <span style={{ position: "absolute", bottom: 4, right: 4, fontSize: 14 }}>
                      {post.rating === "GOOD" ? "👍" : "👎"}
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
