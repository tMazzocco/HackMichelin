import { useState } from "react";
import { Title, TextInput, Button, Avatar, Stack, Text, Divider } from "@mantine/core";
import { Check } from "lucide-react";
import { useApp } from "../context/AppContext";
import { UserProfile } from "../types";

export default function ProfilePage() {
  const { profile, setProfile } = useApp();
  const [form, setForm] = useState<UserProfile>(profile);
  const [saved, setSaved] = useState(false);

  function handleSave() {
    setProfile(form);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  const initial = form.firstName.charAt(0).toUpperCase() || "?";

  return (
    <div className="page pb-20 px-4" style={{ paddingTop: 56 }}>
      <Title order={2} fw={700} mt="md" mb="lg">Profile</Title>

      <Stack align="center" mb="xl">
        <Avatar size={80} radius="xl" color="michelin" src={form.avatarUrl ?? undefined}>
          {initial}
        </Avatar>
        <Text size="sm" c="dimmed">Avatar from initials</Text>
      </Stack>

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
