import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ActionIcon, Badge, Text, Group, Anchor, SimpleGrid, Paper, Loader, Stack, Title } from "@mantine/core";
import { ArrowLeft, Globe, Phone, MapPin, Star } from "lucide-react";
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

  return (
    <div className="page pb-24">
      {/* Hero */}
      <div className="relative h-64">
        <img src={heroImg} alt={restaurant.name} className="w-full h-full object-cover" />
        <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent" />
        <ActionIcon
          onClick={() => navigate(-1)}
          variant="filled"
          color="dark"
          radius="xl"
          size="lg"
          style={{ position: "absolute", top: 48, left: 16, background: "rgba(0,0,0,0.4)", backdropFilter: "blur(8px)" }}
        >
          <ArrowLeft size={18} />
        </ActionIcon>
        {stars && (
          <Badge
            color="michelin"
            size="lg"
            style={{ position: "absolute", top: 48, right: 16 }}
          >
            {stars}
          </Badge>
        )}
        <div className="absolute bottom-4 left-4 right-4">
          <Title order={2} c="white" style={{ lineHeight: 1.2 }}>{restaurant.name}</Title>
          {label && <Text size="sm" c="pink" mt={2}>{label}</Text>}
        </div>
      </div>

      {/* Meta */}
      <Stack gap="xs" px="md" mt="md">
        {restaurant.city && (
          <Group gap="xs">
            <MapPin size={14} color="var(--mantine-color-michelin-6)" />
            <Text size="sm" c="dimmed">
              {[restaurant.street, restaurant.city, restaurant.country_name].filter(Boolean).join(", ")}
            </Text>
          </Group>
        )}
        {restaurant.phone && (
          <Group gap="xs">
            <Phone size={14} color="var(--mantine-color-michelin-6)" />
            <Anchor href={`tel:${restaurant.phone}`} size="sm" c="dimmed">
              {restaurant.phone}
            </Anchor>
          </Group>
        )}
        {restaurant.website && (
          <Group gap="xs">
            <Globe size={14} color="var(--mantine-color-michelin-6)" />
            <Anchor href={restaurant.website} target="_blank" rel="noreferrer" size="sm" c="michelin" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {restaurant.website.replace(/^https?:\/\//, "")}
            </Anchor>
          </Group>
        )}
        {restaurant.price_category_label && (
          <Group gap="xs">
            <Star size={14} color="var(--mantine-color-michelin-6)" />
            <Text size="sm" c="dimmed">{restaurant.price_category_label}</Text>
          </Group>
        )}
      </Stack>

      {/* Description */}
      {restaurant.main_desc && (
        <Text size="sm" c="dimmed" px="md" mt="md" style={{ lineHeight: 1.7 }}>
          {restaurant.main_desc}
        </Text>
      )}

      {/* Stats */}
      {(restaurant.total_posts != null || restaurant.good_pct != null) && (
        <SimpleGrid cols={2} px="md" mt="md" spacing="sm">
          {restaurant.total_posts != null && (
            <Paper withBorder radius="xl" p="sm" ta="center">
              <Text fw={700} size="xl">{restaurant.total_posts}</Text>
              <Text size="xs" c="dimmed">Posts</Text>
            </Paper>
          )}
          {restaurant.good_pct != null && (
            <Paper withBorder radius="xl" p="sm" ta="center">
              <Text fw={700} size="xl">{Math.round(restaurant.good_pct * 100)}%</Text>
              <Text size="xs" c="dimmed">Positive</Text>
            </Paper>
          )}
        </SimpleGrid>
      )}

      {/* Map */}
      {restaurant.latitude && restaurant.longitude && (
        <div className="px-4 mt-5">
          <Text fw={600} size="sm" mb="xs">Location</Text>
          <div className="rounded-2xl overflow-hidden h-40 shadow-md">
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
        <div className="px-4 mt-5">
          <Text fw={600} size="sm" mb="xs">Moments</Text>
          <div className="grid grid-cols-3 gap-1">
            {posts.map((post) => {
              const thumb = post.thumbnail_url ?? post.media_url;
              return (
                <div key={post.post_id} className="aspect-square rounded-lg overflow-hidden bg-black/10 relative">
                  {thumb ? (
                    <img src={thumb} alt="" className="w-full h-full object-cover" />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center">
                      <Text size="xs" c="dimmed">No media</Text>
                    </div>
                  )}
                  {post.rating && (
                    <span className="absolute bottom-1 right-1 text-sm">
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
