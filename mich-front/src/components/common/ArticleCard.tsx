import { Link } from "react-router-dom";
import { Card, Text } from "@mantine/core";
import { Article } from "../../types";

interface Props {
  article: Article;
}

export default function ArticleCard({ article }: Props) {
  return (
    <Card
      component={Link}
      to={`/articles/${article.id}`}
      padding={0}
      radius="xl"
      withBorder
      style={{ flexShrink: 0, width: 256, textDecoration: "none" }}
    >
      <Card.Section style={{ position: "relative", height: 144 }}>
        <img src={article.image_url} alt={article.title} style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />
        <div style={{ position: "absolute", inset: 0, background: "linear-gradient(to top, rgba(0,0,0,0.65), transparent)" }} />
        <Text
          fw={600}
          size="sm"
          c="white"
          lineClamp={2}
          style={{ position: "absolute", bottom: 8, left: 12, right: 12 }}
        >
          {article.title}
        </Text>
      </Card.Section>

      <div style={{ padding: "10px 12px" }}>
        <Text size="xs" c="dimmed" lineClamp={2}>{article.description}</Text>
        <Text size="xs" c="dimmed" style={{ opacity: 0.5 }} mt={4}>{article.author}</Text>
      </div>
    </Card>
  );
}
