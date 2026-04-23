import { useState, useEffect, useRef, useCallback } from "react";
import { useApp } from "../context/AppContext";
import { getRestaurantPosts } from "../services/posts";
import { Post, timeAgo } from "../types";
import LoadingSpinner from "../components/common/LoadingSpinner";
import { Heart, MessageCircle, MapPin } from "lucide-react";
import { Link } from "react-router-dom";

interface FeedCursor {
  restaurantId: string;
  nextBefore: string | null;
  done: boolean;
}

const BATCH_SIZE = 6;

export default function ShortsPage() {
  const { restaurants, restaurantsLoading, locationLoading } = useApp();
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

  // Use ref so IntersectionObserver closure always reads current cursors
  const cursorsRef = useRef<FeedCursor[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const fetchingRef = useRef(false);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const initialized = useRef(false);

  const loadBatch = useCallback(async () => {
    if (fetchingRef.current) return;
    const cursors = cursorsRef.current;
    const idx = cursors.findIndex((c) => !c.done);
    if (idx === -1) {
      setHasMore(false);
      return;
    }

    fetchingRef.current = true;
    const cursor = cursors[idx];

    try {
      const response = await getRestaurantPosts(
        cursor.restaurantId,
        BATCH_SIZE,
        cursor.nextBefore ?? undefined
      );

      if (response.data.length > 0) {
        setPosts((prev) => {
          const existingIds = new Set(prev.map((p) => p.post_id));
          const fresh = response.data.filter((p) => !existingIds.has(p.post_id));
          return [...prev, ...fresh];
        });
      }

      const exhausted = response.data.length < BATCH_SIZE || !response.next_before;
      const updated = [...cursors];
      updated[idx] = {
        ...cursor,
        nextBefore: response.next_before,
        done: exhausted,
      };
      cursorsRef.current = updated;
      setHasMore(updated.some((c) => !c.done));
    } catch {
      // Mark cursor as done on error to avoid a broken infinite retry loop
      const updated = [...cursors];
      updated[idx] = { ...cursor, done: true };
      cursorsRef.current = updated;
      setHasMore(updated.some((c) => !c.done));
    } finally {
      fetchingRef.current = false;
    }
  }, []);

  // Initialize cursors once restaurants are available
  useEffect(() => {
    if (locationLoading || restaurantsLoading) return;
    if (initialized.current) return;
    initialized.current = true;

    if (restaurants.length === 0) {
      setLoading(false);
      setHasMore(false);
      return;
    }

    cursorsRef.current = restaurants.slice(0, 5).map((r) => ({
      restaurantId: r.id,
      nextBefore: null,
      done: false,
    }));

    loadBatch().finally(() => setLoading(false));
  }, [locationLoading, restaurantsLoading, restaurants, loadBatch]);

  // IntersectionObserver on the sentinel div at the bottom of the list
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

  if (locationLoading || restaurantsLoading || loading) {
    return (
      <div className="fixed inset-0 bg-dark flex items-center justify-center">
        <LoadingSpinner size={32} />
      </div>
    );
  }

  if (posts.length === 0) {
    return (
      <div className="fixed inset-0 bg-dark flex flex-col items-center justify-center gap-4 text-white/40">
        <p className="text-lg font-semibold">No experiences yet</p>
        <p className="text-sm">Be the first to share a restaurant moment.</p>
      </div>
    );
  }

  return (
    <div className="shorts-container fixed inset-0 bg-dark">
      {posts.map((post) => (
        <ShortItem key={post.post_id} post={post} />
      ))}

      {/* Sentinel triggers next page load */}
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
}

function ShortItem({ post }: { post: Post }) {
  const videoRef = useRef<HTMLVideoElement>(null);

  // Play video only when it's in the viewport (saves battery, avoids audio conflicts)
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          video.play().catch(() => {});
        } else {
          video.pause();
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
      {/* Background media */}
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

      {/* Overlay gradient */}
      <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/20" />

      {/* Content */}
      <div className="absolute bottom-24 inset-x-0 px-5 text-white">
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

          {/* Actions */}
          <div className="flex flex-col items-center gap-5 mb-1">
            <button className="flex flex-col items-center gap-1 text-white/80" aria-label="Like">
              <Heart size={26} strokeWidth={1.8} />
            </button>
            <button className="flex flex-col items-center gap-1 text-white/80" aria-label="Comment">
              <MessageCircle size={26} strokeWidth={1.8} />
            </button>
            {post.rating && (
              <div className="flex flex-col items-center">
                <span className="text-secondary font-bold text-lg leading-none">
                  {post.rating === "GOOD" ? "👍" : "👎"}
                </span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
