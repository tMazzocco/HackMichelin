import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Skeleton } from "@mantine/core";
import { getRandomPosts } from "../../services/posts";
import { Post } from "../../types";

export default function ExperiencesTriptych() {
  const [posts, setPosts] = useState<Post[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

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
        <div
          key={post.post_id}
          onClick={() => navigate("/shorts", { state: { initialPost: post } })}
          className="relative rounded-xl overflow-hidden block cursor-pointer"
          style={{ aspectRatio: "2/3", background: "#111" }}
        >
          {post.thumbnail_url || post.media_url ? (
            <img
              src={(post.thumbnail_url ?? post.media_url)!}
              alt={post.restaurant_name ?? `Post ${idx + 1}`}
              className="absolute inset-0 w-full h-full object-cover"
            />
          ) : null}
          <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent" />
          <div className="absolute bottom-0 left-0 right-0 p-2">
            {post.restaurant_name && (
              <p className="text-white text-[10px] font-semibold leading-tight line-clamp-2">
                {post.restaurant_name}
              </p>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
