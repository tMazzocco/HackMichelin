import { Link } from "react-router-dom";
import { Title, Card, Text, Group, Stack } from "@mantine/core";
import { articles } from "../data/articles";

export default function ArticlesPage() {
  return (
    <div className="page pb-20 px-4 pt-4">
      <Title order={2} fw={700} mt="md" mb="lg">Articles</Title>
      <Stack gap="md">
        {articles.map((a) => (
          <Card
            key={a.id}
            component={Link}
            to={`/articles/${a.id}`}
            withBorder
            padding={0}
            radius="xl"
            style={{ textDecoration: "none" }}
          >
            <Card.Section style={{ position: "relative", height: 176 }}>
              <img src={a.image_url} alt={a.title} style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />
              <div style={{ position: "absolute", inset: 0, background: "linear-gradient(to top, rgba(0,0,0,0.55), transparent)" }} />
              <Text
                fw={700}
                size="md"
                c="white"
                lineClamp={2}
                style={{ position: "absolute", bottom: 12, left: 16, right: 16 }}
              >
                {a.title}
              </Text>
            </Card.Section>

            <div style={{ padding: "12px 16px" }}>
              <Text size="sm" c="dimmed" lineClamp={2}>{a.description}</Text>
              <Group justify="space-between" mt="xs">
                <Text size="xs" c="dimmed" style={{ opacity: 0.6 }}>{a.author}</Text>
                <Text size="xs" c="dimmed" style={{ opacity: 0.6 }}>{a.created_at}</Text>
              </Group>
            </div>
          </Card>
        ))}
      </Stack>
    </div>
  );
}
