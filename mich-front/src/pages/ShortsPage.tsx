import { useState, useEffect } from "react";
import { useApp } from "../context/AppContext";
import { getRestaurantPosts } from "../services/posts";
import { Post, timeAgo } from "../types";
import LoadingSpinner from "../components/common/LoadingSpinner";
import { Heart, MessageCircle, MapPin } from "lucide-react";
import { Link } from "react-router-dom";

export default function ShortsPage() {
  const { restaurants } = useApp();
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (restaurants.length === 0) {
      setLoading(false);
      return;
    }
    const ids = restaurants.slice(0, 5).map((r) => r.id);
    Promise.all(ids.map((id) => getRestaurantPosts(id, 4)))
      .then((results) => {
        const all = results.flatMap((r) => r.data);
        // shuffle for variety
        all.sort(() => Math.random() - 0.5);
        setPosts(all);
      })
      .catch(() => setPosts([]))
      .finally(() => setLoading(false));
  }, [restaurants]);

  if (loading) {
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
    </div>
  );
}

function ShortItem({ post }: { post: Post }) {
  const isVideo = post.media_type === "video";
  const media = post.media_url ?? post.thumbnail_url;

  return (
    <div className="shorts-item">
      {/* Background media */}
      {isVideo && post.media_url ? (
        <video
          src={post.media_url}
          className="absolute inset-0 w-full h-full object-cover"
          autoPlay
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
            <button className="flex flex-col items-center gap-1 text-white/80">
              <Heart size={26} strokeWidth={1.8} />
            </button>
            <button className="flex flex-col items-center gap-1 text-white/80">
              <MessageCircle size={26} strokeWidth={1.8} />
            </button>
            {post.rating && (
              <div className="flex flex-col items-center">
                <span className="text-secondary font-bold text-lg leading-none">
                  {post.rating === "good" ? "👍" : "👎"}
                </span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
