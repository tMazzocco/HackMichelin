import { useState, useEffect, useRef, useCallback } from "react";
import { Loader, ActionIcon, Text } from "@mantine/core";
import { Map, ChevronDown, Heart, MessageCircle, MapPin } from "lucide-react";
import { Link } from "react-router-dom";
import { useApp } from "../context/AppContext";
import { getRestaurantPosts } from "../services/posts";
import { Post, Restaurant, UserLocation, timeAgo } from "../types";
import TopBar from "../components/layout/TopBar";
import MapView from "../components/map/MapView";
import MapErrorBoundary from "../components/map/MapErrorBoundary";
import ResizableSplit from "../components/layout/ResizableSplit";

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
  const [mapVisible, setMapVisible] = useState(false);

  const cursorsRef = useRef<FeedCursor[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const fetchingRef = useRef(false);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const initialized = useRef(false);

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

  const handleScroll = useCallback(() => {
    const container = scrollRef.current;
    if (!container || container.clientHeight === 0) return;
    const idx = Math.round(container.scrollTop / container.clientHeight);
    setCurrentIdx(Math.max(0, Math.min(idx, posts.length - 1)));
  }, [posts.length]);

  const currentPost = posts[currentIdx] ?? null;
  const currentRestaurant: Restaurant | null = currentPost?.restaurant_id
    ? (restaurants.find((r) => r.id === currentPost.restaurant_id) ?? null)
    : null;

  const mapLocation: UserLocation | null =
    currentRestaurant?.latitude && currentRestaurant?.longitude
      ? { lat: currentRestaurant.latitude, lng: currentRestaurant.longitude }
      : location;

  const mapZoom = currentRestaurant ? 15 : 13;

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
      <Loader color="michelin" size={28} />
    </div>
  );

  const shortsScroll = (
    <div
      ref={scrollRef}
      className="shorts-container h-full bg-dark"
      onScroll={handleScroll}
    >
      {(locationLoading || restaurantsLoading || loading)
        ? <div className="shorts-item flex items-center justify-center bg-dark"><Loader color="michelin" size={32} /></div>
        : posts.length === 0
          ? (
            <div className="shorts-item flex flex-col items-center justify-center gap-3">
              <Text fw={600} c="white" style={{ opacity: 0.4 }}>No experiences yet</Text>
              <Text size="sm" c="white" style={{ opacity: 0.3 }}>Be the first to share a restaurant moment.</Text>
            </div>
          )
          : posts.map((post) => <ShortItem key={post.post_id} post={post} />)
      }

      {!loading && posts.length > 0 && (
        <div ref={sentinelRef} className="shorts-item flex items-center justify-center bg-dark">
          {loadingMore
            ? <Loader color="michelin" size={28} />
            : hasMore
              ? <Text size="xs" c="white" style={{ opacity: 0.2 }}>Scroll for more</Text>
              : <Text size="sm" fw={500} c="white" style={{ opacity: 0.3 }}>You've seen it all ✓</Text>
          }
        </div>
      )}
    </div>
  );

  /* ── Map toggle button (always visible over shorts) ── */
  const toggleBtn = (
    <ActionIcon
      onClick={() => setMapVisible((v) => !v)}
      radius="xl"
      size="lg"
      color="michelin"
      variant={mapVisible ? "filled" : "light"}
      style={{ position: "absolute", top: mapVisible ? 72 : 16, right: 16, zIndex: 1100 }}
      aria-label={mapVisible ? "Hide map" : "Show map"}
    >
      {mapVisible ? <ChevronDown size={18} /> : <Map size={18} />}
    </ActionIcon>
  );

  if (mapVisible) {
    return (
      <div className="fixed inset-0 pb-14">
        <TopBar />
        {toggleBtn}
        <div className="absolute inset-x-0 bottom-0" style={{ top: 56 }}>
          <ResizableSplit
            top={mapPanel}
            bottom={shortsScroll}
            defaultTopPercent={35}
            minTopPercent={0}
            maxTopPercent={85}
            onTopPctChange={(pct) => { if (pct <= 1) setMapVisible(false); }}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 pb-14">
      {shortsScroll}
      {toggleBtn}
    </div>
  );
}

/* ── ShortItem ── */

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
          loop muted playsInline
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
            <Text fw={600} size="sm" c="white">{post.username ?? "Anonymous"}</Text>
            {post.caption && (
              <Text size="sm" c="white" style={{ opacity: 0.8 }} lineClamp={2} mt={4}>
                {post.caption}
              </Text>
            )}
            <Text size="xs" c="white" style={{ opacity: 0.4 }} mt={4}>
              {timeAgo(post.created_at)}
            </Text>
          </div>

          <div className="flex flex-col items-center gap-5 mb-1">
            <ActionIcon variant="transparent" color="white" size="xl" aria-label="Like">
              <Heart size={26} strokeWidth={1.8} />
            </ActionIcon>
            <ActionIcon variant="transparent" color="white" size="xl" aria-label="Comment">
              <MessageCircle size={26} strokeWidth={1.8} />
            </ActionIcon>
            {post.rating && (
              <Text fw={700} size="lg" c="pink" style={{ lineHeight: 1 }}>
                {post.rating === "GOOD" ? "👍" : "👎"}
              </Text>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
