import { useState, useEffect, useRef, useCallback } from "react";
import { useApp } from "../context/AppContext";
import { getRestaurantPosts } from "../services/posts";
import { Post, Restaurant, UserLocation, timeAgo } from "../types";
import LoadingSpinner from "../components/common/LoadingSpinner";
import MapView from "../components/map/MapView";
import MapErrorBoundary from "../components/map/MapErrorBoundary";
import ResizableSplit from "../components/layout/ResizableSplit";
import { Heart, MessageCircle, MapPin } from "lucide-react";
import { Link } from "react-router-dom";

interface FeedCursor {
  restaurantId: string;
  nextBefore: string | null;
  done: boolean;
}

const BATCH_SIZE = 6;

export default function ShortsPage() {
  const { location, restaurants, restaurantsLoading, locationLoading } = useApp();

  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [currentIdx, setCurrentIdx] = useState(0);

  const cursorsRef = useRef<FeedCursor[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const fetchingRef = useRef(false);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const initialized = useRef(false);

  // ── Batch fetcher ───────────────────────────────────────────────────────────

  const loadBatch = useCallback(async () => {
    if (fetchingRef.current) return;
    const cursors = cursorsRef.current;
    const idx = cursors.findIndex((c) => !c.done);
    if (idx === -1) { setHasMore(false); return; }

    fetchingRef.current = true;
    const cursor = cursors[idx];

    try {
      const response = await getRestaurantPosts(cursor.restaurantId, BATCH_SIZE, cursor.nextBefore ?? undefined);

      if (response.data.length > 0) {
        setPosts((prev) => {
          const seen = new Set(prev.map((p) => p.post_id));
          return [...prev, ...response.data.filter((p) => !seen.has(p.post_id))];
        });
      }

      const exhausted = response.data.length < BATCH_SIZE || !response.next_before;
      const updated = [...cursors];
      updated[idx] = { ...cursor, nextBefore: response.next_before, done: exhausted };
      cursorsRef.current = updated;
      setHasMore(updated.some((c) => !c.done));
    } catch {
      const updated = [...cursors];
      updated[idx] = { ...cursor, done: true };
      cursorsRef.current = updated;
      setHasMore(updated.some((c) => !c.done));
    } finally {
      fetchingRef.current = false;
    }
  }, []);

  // ── Init when restaurants ready ─────────────────────────────────────────────

  useEffect(() => {
    if (locationLoading || restaurantsLoading) return;
    if (initialized.current) return;
    initialized.current = true;

    if (restaurants.length === 0) { setLoading(false); setHasMore(false); return; }

    cursorsRef.current = restaurants.slice(0, 5).map((r) => ({
      restaurantId: r.id,
      nextBefore: null,
      done: false,
    }));

    loadBatch().finally(() => setLoading(false));
  }, [locationLoading, restaurantsLoading, restaurants, loadBatch]);

  // ── Sentinel for infinite scroll ────────────────────────────────────────────

  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !fetchingRef.current && !loading) {
          setLoadingMore(true);
          loadBatch().finally(() => setLoadingMore(false));
        }
      },
      { threshold: 0.1 }
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [loadBatch, loading]);

  // ── Track which short is currently snapped into view ────────────────────────

  const handleScroll = useCallback(() => {
    const container = scrollRef.current;
    if (!container || container.clientHeight === 0) return;
    const idx = Math.round(container.scrollTop / container.clientHeight);
    setCurrentIdx(Math.max(0, Math.min(idx, posts.length - 1)));
  }, [posts.length]);

  // ── Derive map target from the current post ─────────────────────────────────

  const currentPost = posts[currentIdx] ?? null;
  const currentRestaurant: Restaurant | null = currentPost?.restaurant_id
    ? (restaurants.find((r) => r.id === currentPost.restaurant_id) ?? null)
    : null;

  const mapLocation: UserLocation | null =
    currentRestaurant?.latitude && currentRestaurant?.longitude
      ? { lat: currentRestaurant.latitude, lng: currentRestaurant.longitude }
      : location;

  const mapZoom = currentRestaurant ? 15 : 13;

  // ── Map panel ───────────────────────────────────────────────────────────────

  const mapPanel = mapLocation ? (
    <MapErrorBoundary>
      <MapView
        location={mapLocation}
        restaurants={currentRestaurant ? [currentRestaurant] : restaurants.slice(0, 5)}
        zoom={mapZoom}
        interactive
      />
    </MapErrorBoundary>
  ) : (
    <div className="h-full bg-dark flex items-center justify-center">
      <LoadingSpinner size={28} />
    </div>
  );

  // ── Loading / empty states ──────────────────────────────────────────────────

  if (locationLoading || restaurantsLoading || loading) {
    return (
      <div className="fixed inset-0 pb-14 flex flex-col">
        <ResizableSplit
          top={mapPanel}
          bottom={
            <div className="h-full bg-dark flex items-center justify-center">
              <LoadingSpinner size={32} />
            </div>
          }
          defaultTopPercent={35}
        />
      </div>
    );
  }

  if (posts.length === 0) {
    return (
      <div className="fixed inset-0 pb-14 flex flex-col">
        <ResizableSplit
          top={mapPanel}
          bottom={
            <div className="h-full bg-dark flex flex-col items-center justify-center gap-3 text-white/40">
              <p className="text-base font-semibold">No experiences yet</p>
              <p className="text-sm">Be the first to share a restaurant moment.</p>
            </div>
          }
          defaultTopPercent={35}
        />
      </div>
    );
  }

  // ── Main layout ─────────────────────────────────────────────────────────────

  const shortsPanel = (
    <div
      ref={scrollRef}
      className="shorts-container h-full bg-dark"
      onScroll={handleScroll}
    >
      {posts.map((post) => (
        <ShortItem key={post.post_id} post={post} />
      ))}

      {/* Infinite scroll sentinel */}
      <div ref={sentinelRef} className="shorts-item flex items-center justify-center bg-dark">
        {loadingMore ? (
          <LoadingSpinner size={28} />
        ) : hasMore ? (
          <p className="text-white/20 text-xs">Scroll for more</p>
        ) : (
          <p className="text-white/30 text-sm font-medium">You've seen it all ✓</p>
        )}
      </div>
    </div>
  );

  return (
    // No pt needed (no TopBar on /shorts). pb-14 keeps content above BottomNav.
    <div className="fixed inset-0 pb-14">
      <ResizableSplit
        top={mapPanel}
        bottom={shortsPanel}
        defaultTopPercent={35}
        minTopPercent={10}
        maxTopPercent={85}
      />
    </div>
  );
}

// ── ShortItem ──────────────────────────────────────────────────────────────────

function ShortItem({ post }: { post: Post }) {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) video.play().catch(() => {});
        else video.pause();
      },
      { threshold: 0.5 }
    );
    observer.observe(video);
    return () => observer.disconnect();
  }, []);

  const isVideo = post.media_type === "video";
  const media = post.media_url ?? post.thumbnail_url;

  return (
    <div className="shorts-item">
      {isVideo && post.media_url ? (
        <video
          ref={videoRef}
          src={post.media_url}
          className="absolute inset-0 w-full h-full object-cover"
          loop
          muted
          playsInline
        />
      ) : media ? (
        <img src={media} alt="" className="absolute inset-0 w-full h-full object-cover" />
      ) : (
        <div className="absolute inset-0 bg-gradient-to-br from-dark to-primary/30" />
      )}

      <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/20" />

      <div className="absolute bottom-6 inset-x-0 px-5 text-white">
        <div className="flex items-end justify-between gap-4">
          <div className="flex-1 min-w-0">
            {post.restaurant_name && (
              <Link
                to={`/restaurant/${post.restaurant_id}`}
                className="flex items-center gap-1 text-secondary text-xs font-semibold mb-1"
              >
                <MapPin size={12} />
                {post.restaurant_name}
              </Link>
            )}
            <p className="font-semibold text-sm">{post.username ?? "Anonymous"}</p>
            {post.caption && (
              <p className="text-white/80 text-sm mt-1 line-clamp-2">{post.caption}</p>
            )}
            <p className="text-white/40 text-xs mt-1">{timeAgo(post.created_at)}</p>
          </div>

          <div className="flex flex-col items-center gap-5 mb-1">
            <button className="flex flex-col items-center gap-1 text-white/80" aria-label="Like">
              <Heart size={26} strokeWidth={1.8} />
            </button>
            <button className="flex flex-col items-center gap-1 text-white/80" aria-label="Comment">
              <MessageCircle size={26} strokeWidth={1.8} />
            </button>
            {post.rating && (
              <span className="text-secondary font-bold text-lg leading-none">
                {post.rating === "GOOD" ? "👍" : "👎"}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
