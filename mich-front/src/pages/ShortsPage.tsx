import { useState, useEffect, useRef, useCallback } from "react";
import { Loader, ActionIcon, Text } from "@mantine/core";
import { Map, ChevronDown, Heart, MessageCircle, MapPin } from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { useApp } from "../context/AppContext";
import { getRestaurantPosts } from "../services/posts";
import { getNearbyRestaurants } from "../services/restaurants";
import { Post, Restaurant, UserLocation, timeAgo } from "../types";
import TopBar from "../components/layout/TopBar";
import MapView from "../components/map/MapView";
import MapErrorBoundary from "../components/map/MapErrorBoundary";
import { useLikes } from "../hooks/useLikes";
import ResizableSplit from "../components/layout/ResizableSplit";

interface FeedCursor {
  restaurantId: string;
  nextBefore: string | null;
  done: boolean;
}

const BATCH_SIZE = 10;
const RESTAURANT_LIMIT = 50;
const RADII = [20_000, 75_000, 250_000, 1_000_000];

export default function ShortsPage() {
  const { location, restaurants, restaurantsLoading, locationLoading } = useApp();
  const routerLocation = useLocation();
  const initialPost: Post | null = (routerLocation.state as { initialPost?: Post })?.initialPost ?? null;

  const [posts, setPosts] = useState<Post[]>(initialPost ? [initialPost] : []);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [currentIdx, setCurrentIdx] = useState(0);
  const [mapVisible, setMapVisible] = useState(false);

  const cursorsRef = useRef<FeedCursor[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const fetchingRef = useRef(false);
  const expandingRef = useRef(false);
  const radiusIdxRef = useRef(0);
  const usedIdsRef = useRef(new Set<string>());
  const locationRef = useRef(location);
  const prevLocKeyRef = useRef<string | null>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => { locationRef.current = location; }, [location]);

  const addRestaurants = useCallback((list: Restaurant[]) => {
    const fresh = list.filter((r) => !usedIdsRef.current.has(r.id));
    fresh.forEach((r) => usedIdsRef.current.add(r.id));
    const newCursors: FeedCursor[] = fresh.map((r) => ({
      restaurantId: r.id,
      nextBefore: null,
      done: false,
    }));
    cursorsRef.current = [...cursorsRef.current, ...newCursors];
    return fresh.length;
  }, []);

  const expandFeed = useCallback(async (): Promise<boolean> => {
    if (expandingRef.current) return false;
    const loc = locationRef.current;
    if (!loc) return false;
    const nextIdx = radiusIdxRef.current + 1;
    if (nextIdx >= RADII.length) return false;
    expandingRef.current = true;
    radiusIdxRef.current = nextIdx;
    try {
      const list = await getNearbyRestaurants(loc.lat, loc.lng, RADII[nextIdx], RESTAURANT_LIMIT);
      const added = addRestaurants(list);
      return added > 0;
    } catch {
      return false;
    } finally {
      expandingRef.current = false;
    }
  }, [addRestaurants]);

  const loadBatch = useCallback(async () => {
    if (fetchingRef.current) return;

    let idx = cursorsRef.current.findIndex((c) => !c.done);
    if (idx === -1) {
      const expanded = await expandFeed();
      if (!expanded) { setHasMore(false); return; }
      idx = cursorsRef.current.findIndex((c) => !c.done);
      if (idx === -1) { setHasMore(false); return; }
    }

    fetchingRef.current = true;
    const cursor = cursorsRef.current[idx];
    try {
      const response = await getRestaurantPosts(cursor.restaurantId, BATCH_SIZE, cursor.nextBefore ?? undefined);
      if (response.data.length > 0) {
        setPosts((prev) => {
          const seen = new Set(prev.map((p) => p.post_id));
          return [...prev, ...response.data.filter((p) => !seen.has(p.post_id))];
        });
      }
      const exhausted = response.data.length < BATCH_SIZE || !response.next_before;
      const updated = [...cursorsRef.current];
      updated[idx] = { ...cursor, nextBefore: response.next_before, done: exhausted };
      cursorsRef.current = updated;
      setHasMore(true);
    } catch {
      const updated = [...cursorsRef.current];
      updated[idx] = { ...cursor, done: true };
      cursorsRef.current = updated;
    } finally {
      fetchingRef.current = false;
    }
  }, [expandFeed]);

  useEffect(() => {
    if (locationLoading || !location) return;

    const key = `${location.lat.toFixed(2)},${location.lng.toFixed(2)}`;
    if (key === prevLocKeyRef.current) return;
    prevLocKeyRef.current = key;

    const isFirstMount = prevLocKeyRef.current === null;
    cursorsRef.current = [];
    usedIdsRef.current = new Set();
    radiusIdxRef.current = 0;
    fetchingRef.current = false;
    expandingRef.current = false;
    setHasMore(true);
    setLoading(true);
    setPosts(isFirstMount && initialPost ? [initialPost] : []);

    getNearbyRestaurants(location.lat, location.lng, RADII[0], RESTAURANT_LIMIT)
      .then((list) => {
        addRestaurants(list);
        return loadBatch();
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [location, locationLoading, addRestaurants, loadBatch]);

  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !fetchingRef.current && !expandingRef.current && !loading) {
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
      {posts.length === 0
        ? (locationLoading || loading)
          ? <div className="shorts-item flex items-center justify-center" style={{ background: "#111" }}><Loader color="michelin" size={32} /></div>
          : (
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
        <div className="absolute inset-x-0 bottom-0" style={{ top: 0 }}>
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
  const playPromiseRef = useRef<Promise<void> | null>(null);
  const { isLiked, toggle } = useLikes();

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          playPromiseRef.current = video.play();
          playPromiseRef.current?.catch(() => {});
        } else {
          const p = playPromiseRef.current;
          playPromiseRef.current = null;
          if (p) p.then(() => video.pause()).catch(() => {});
          else video.pause();
        }
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
          preload="none"
          poster={post.thumbnail_url ?? undefined}
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
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <Text fw={600} size="sm" c="white">{post.username ?? "Anonymous"}</Text>
              {post.username && (
                <div style={{ display: "flex", alignItems: "center", gap: 3, background: "rgba(171,21,46,0.75)", borderRadius: 20, padding: "1px 7px" }}>
                  <svg viewBox="0 0 24 24" width="9" height="9" fill="white">
                    <path d="M12,2 L14.5,7.67 L20.66,7 L17,12 L20.66,17 L14.5,16.33 L12,22 L9.5,16.33 L3.34,17 L7,12 L3.34,7 L9.5,7.67 Z" />
                  </svg>
                  <Text size="xs" fw={700} c="white" style={{ lineHeight: 1 }}>
                    {(post.username.split("").reduce((a, c) => a + c.charCodeAt(0), 0) % 72) + 3}
                  </Text>
                </div>
              )}
            </div>
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
            <ActionIcon
              variant="transparent"
              color="white"
              size="xl"
              aria-label="Like"
              onClick={() => toggle(post.post_id)}
            >
              <Heart
                size={26}
                strokeWidth={1.8}
                fill={isLiked(post.post_id) ? "#ff4d6d" : "none"}
                color={isLiked(post.post_id) ? "#ff4d6d" : "white"}
              />
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
