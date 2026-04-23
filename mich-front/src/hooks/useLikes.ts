import { useState, useCallback } from "react";

const KEY = "mich_liked_posts";

function readLikes(): Set<string> {
  try {
    const raw = localStorage.getItem(KEY);
    return raw ? new Set(JSON.parse(raw)) : new Set();
  } catch {
    return new Set();
  }
}

function writeLikes(set: Set<string>) {
  localStorage.setItem(KEY, JSON.stringify([...set]));
}

export function useLikes() {
  const [liked, setLiked] = useState<Set<string>>(readLikes);

  const toggle = useCallback((postId: string) => {
    setLiked((prev) => {
      const next = new Set(prev);
      if (next.has(postId)) next.delete(postId);
      else next.add(postId);
      writeLikes(next);
      return next;
    });
  }, []);

  const isLiked = useCallback((postId: string) => liked.has(postId), [liked]);

  return { isLiked, toggle };
}
