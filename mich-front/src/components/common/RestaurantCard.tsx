import { Link } from "react-router-dom";
import { Card, Badge, Text, Group } from "@mantine/core";
import { MapPin } from "lucide-react";
import { Restaurant, awardStars, formatDistance } from "../../types";

interface Props {
  restaurant: Restaurant;
}

export default function RestaurantCard({ restaurant }: Props) {
  const stars = awardStars(restaurant.michelin_award);
  const img = restaurant.main_image_url ?? `https://picsum.photos/seed/${restaurant.id}/400/300`;

  return (
    <Card
      component={Link}
      to={`/restaurant/${restaurant.id}`}
      withBorder
      padding={0}
      radius="xl"
      style={{ flexShrink: 0, width: 224, textDecoration: "none" }}
    >
      <Card.Section style={{ position: "relative", height: 144 }}>
        <img src={img} alt={restaurant.name} style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />
        {stars && (
          <Badge
            color="michelin"
            style={{ position: "absolute", top: 8, right: 8 }}
            size="sm"
          >
            {stars}
          </Badge>
        )}
      </Card.Section>

      <div style={{ padding: "10px 12px" }}>
        <Text fw={600} size="sm" lineClamp={1}>
          {restaurant.name}
        </Text>
        {restaurant.city && (
          <Group gap={4} mt={2}>
            <MapPin size={10} color="var(--mantine-color-dimmed)" />
            <Text size="xs" c="dimmed" lineClamp={1}>
              {restaurant.city}
            </Text>
          </Group>
        )}
        {restaurant.distance_meters != null && (
          <Text size="xs" c="michelin" mt={4}>
            {formatDistance(restaurant.distance_meters)}
          </Text>
        )}
      </div>
    </Card>
  );
}
