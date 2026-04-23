import { useParams, useNavigate } from "react-router-dom";
import { ActionIcon, Text, Group } from "@mantine/core";
import { ArrowLeft, MapPin } from "lucide-react";
import { articles } from "../data/articles";
import MapView from "../components/map/MapView";
import MapErrorBoundary from "../components/map/MapErrorBoundary";

export default function ArticleDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const article = articles.find((a) => a.id === id);

  if (!article) {
    return (
      <div className="page pt-14 pb-20 flex items-center justify-center">
        <Text c="dimmed">Article not found.</Text>
      </div>
    );
  }

  return (
    <div className="page pb-20">
      {/* Hero */}
      <div className="relative h-64">
        <img src={article.image_url} alt={article.title} className="w-full h-full object-cover" />
        <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-black/20 to-transparent" />
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
        <div className="absolute bottom-4 left-4 right-4">
          <Text size="xs" c="white" style={{ opacity: 0.6 }} mb={4}>{article.author}</Text>
          <Text fw={700} size="xl" c="white" style={{ lineHeight: 1.25 }}>{article.title}</Text>
        </div>
      </div>

      {/* Body */}
      <div className="px-4 mt-5">
        <Text size="sm" fw={500} c="dimmed" mb="md">{article.description}</Text>
        {article.content.split("\n\n").map((para, i) => (
          <Text key={i} size="sm" style={{ lineHeight: 1.7 }} mb="md">
            {para}
          </Text>
        ))}
      </div>

      {/* Restaurant map */}
      <div className="px-4 mt-2">
        <Group gap={4} mb="xs">
          <MapPin size={12} color="var(--mantine-color-dimmed)" />
          <Text size="xs" c="dimmed">{article.restaurant_name}</Text>
        </Group>
        <div className="rounded-2xl overflow-hidden h-44 shadow-md">
          <MapErrorBoundary height="176px">
            <MapView
              location={{ lat: article.restaurant_lat, lng: article.restaurant_lng }}
              restaurants={[]}
              zoom={14}
              interactive={false}
            />
          </MapErrorBoundary>
        </div>
      </div>
    </div>
  );
}
