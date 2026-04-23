import { useState } from "react";
import { Title, TextInput, Button, Avatar, Stack, Text, Divider, Paper, Group, Badge } from "@mantine/core";
import { Check } from "lucide-react";
import { useApp } from "../context/AppContext";
import { UserProfile } from "../types";

const FAKE_STATS = {
  stars: 34,
  restaurants: 21,
  countries: 8,
  since: "2019",
};

const FAKE_RECENT = [
  { name: "Guy Savoy", stars: "★★★", location: "Paris, France" },
  { name: "Flocons de Sel", stars: "★★★", location: "Megève, France" },
  { name: "Noma", stars: "★★", location: "Copenhagen, Denmark" },
  { name: "Asador Etxebarri", stars: "★", location: "Atxondo, Spain" },
];

export default function ProfilePage() {
  const { profile, setProfile } = useApp();
  const [form, setForm] = useState<UserProfile>({ firstName: "Alexandre", lastName: "Moreau", avatarUrl: null });
  const [saved, setSaved] = useState(false);

  function handleSave() {
    setProfile(form);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  const initial = form.firstName.charAt(0).toUpperCase() || "?";

  return (
    <div className="page pb-20 px-4 pt-4">
      <Title order={2} fw={700} mt="md" mb="lg">Profile</Title>

      {/* Identity */}
      <Stack align="center" mb="xl" gap="xs">
        <Avatar size={80} radius="xl" color="michelin" src={form.avatarUrl ?? undefined}>
          {initial}
        </Avatar>
        <Text fw={700} size="lg">{form.firstName} {form.lastName}</Text>
        <Text size="xs" c="dimmed">Member since {FAKE_STATS.since}</Text>
      </Stack>

      {/* Star count — hero element */}
      <Paper
        radius="2xl"
        p="xl"
        mb="lg"
        style={{
          background: "linear-gradient(135deg, #AB152E 0%, #6b0e1e 100%)",
          textAlign: "center",
          boxShadow: "0 8px 32px rgba(171,21,46,0.35)",
        }}
      >
        <Text size="xs" c="rgba(255,255,255,0.7)" fw={600} style={{ letterSpacing: 2, textTransform: "uppercase" }} mb={4}>
          Michelin Stars Collected
        </Text>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: 10 }}>
          <svg viewBox="0 0 24 24" width="32" height="32" fill="white" style={{ opacity: 0.9 }}>
            <path d="M12,2 L14.5,7.67 L20.66,7 L17,12 L20.66,17 L14.5,16.33 L12,22 L9.5,16.33 L3.34,17 L7,12 L3.34,7 L9.5,7.67 Z" />
          </svg>
          <Text fw={900} c="white" style={{ fontSize: 56, lineHeight: 1 }}>{FAKE_STATS.stars}</Text>
          <svg viewBox="0 0 24 24" width="32" height="32" fill="white" style={{ opacity: 0.9 }}>
            <path d="M12,2 L14.5,7.67 L20.66,7 L17,12 L20.66,17 L14.5,16.33 L12,22 L9.5,16.33 L3.34,17 L7,12 L3.34,7 L9.5,7.67 Z" />
          </svg>
        </div>
        <Text size="sm" c="rgba(255,255,255,0.65)" mt={6}>across {FAKE_STATS.restaurants} restaurants in {FAKE_STATS.countries} countries</Text>
      </Paper>

      {/* Quick stats */}
      <Group grow mb="lg" gap="sm">
        <Paper withBorder radius="xl" p="sm" ta="center">
          <Text fw={700} size="xl">{FAKE_STATS.restaurants}</Text>
          <Text size="xs" c="dimmed">Restaurants</Text>
        </Paper>
        <Paper withBorder radius="xl" p="sm" ta="center">
          <Text fw={700} size="xl">{FAKE_STATS.countries}</Text>
          <Text size="xs" c="dimmed">Countries</Text>
        </Paper>
        <Paper withBorder radius="xl" p="sm" ta="center">
          <Text fw={700} size="xl">★★★</Text>
          <Text size="xs" c="dimmed">Best tier</Text>
        </Paper>
      </Group>

      {/* Recent visits */}
      <Text fw={600} size="sm" mb="xs">Recent visits</Text>
      <Stack gap="xs" mb="xl">
        {FAKE_RECENT.map((r) => (
          <Paper key={r.name} withBorder radius="xl" px="md" py="sm">
            <Group justify="space-between">
              <div>
                <Text fw={600} size="sm">{r.name}</Text>
                <Text size="xs" c="dimmed">{r.location}</Text>
              </div>
              <Badge color="michelin" variant="light">{r.stars}</Badge>
            </Group>
          </Paper>
        ))}
      </Stack>

      <Divider mb="md" />

      {/* Edit form */}
      <Text fw={600} size="sm" mb="xs">Edit profile</Text>
      <Stack gap="md">
        <TextInput
          label="First name"
          value={form.firstName}
          onChange={(e) => setForm({ ...form, firstName: e.target.value })}
          radius="xl"
          size="md"
        />
        <TextInput
          label="Last name"
          value={form.lastName}
          onChange={(e) => setForm({ ...form, lastName: e.target.value })}
          radius="xl"
          size="md"
        />
        <Button
          onClick={handleSave}
          color={saved ? "green" : "michelin"}
          leftSection={saved ? <Check size={16} /> : null}
          mt="xs"
          size="md"
          radius="xl"
          fullWidth
        >
          {saved ? "Saved" : "Save changes"}
        </Button>
      </Stack>

      <Divider mt="xl" mb="md" />
      <Stack align="center" gap={4}>
        <Text size="xs" c="dimmed">Guide Michelin — v1.0</Text>
        <Text size="xs" c="dimmed" style={{ opacity: 0.5 }}>© Michelin 2026</Text>
      </Stack>
    </div>
  );
}
