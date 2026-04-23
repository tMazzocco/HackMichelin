import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Skeleton } from "@mantine/core";
import { getRandomPosts } from "../../services/posts";
import { Post } from "../../types";

export default function ExperiencesTriptych() {
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getRandomPosts(3)
      .then(setPosts)
      .catch(() => setPosts([]))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="grid grid-cols-3 gap-2">
        {[0, 1, 2].map((i) => (
          <Skeleton key={i} height={180} radius="md" />
        ))}
      </div>
    );
  }

  if (posts.length === 0) return null;

  return (
    <div className="grid grid-cols-3 gap-2">
      {posts.map((post, idx) => (
        <Link
          key={post.post_id}
          to={post.restaurant_id ? `/restaurant/${post.restaurant_id}` : "/shorts"}
          className="relative rounded-xl overflow-hidden bg-dark block"
          style={{ aspectRatio: "2/3" }}
        >
          {post.thumbnail_url || post.media_url ? (
            <img
              src={(post.thumbnail_url ?? post.media_url)!}
              alt={post.restaurant_name ?? `Post ${idx + 1}`}
              className="absolute inset-0 w-full h-full object-cover"
            />
          ) : (
            <div className="absolute inset-0 bg-black/20" />
          )}
          <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent" />
          <div className="absolute bottom-0 left-0 right-0 p-2">
            {post.restaurant_name && (
              <p className="text-white text-[10px] font-semibold leading-tight line-clamp-2">
                {post.restaurant_name}
              </p>
            )}
          </div>
        </Link>
      ))}
    </div>
  );
}
