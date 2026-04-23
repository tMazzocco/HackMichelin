import { Link, useNavigate } from "react-router-dom";
import { Avatar, TextInput, ActionIcon } from "@mantine/core";
import { Search, Menu } from "lucide-react";
import { useApp } from "../../context/AppContext";
import { useState } from "react";

export default function TopBar() {
  const { profile } = useApp();
  const navigate = useNavigate();
  const [query, setQuery] = useState("");

  const initial = profile.firstName.charAt(0).toUpperCase() || "?";

  function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    if (query.trim()) navigate(`/search?q=${encodeURIComponent(query.trim())}`);
  }

  return (
    <header className="fixed top-0 inset-x-0 z-50 h-14 flex items-center gap-2 px-3" style={{ background: "none" }}>
      {/* Logo */}
      <Link to="/" style={{ textDecoration: "none", flexShrink: 0 }}>
        <ActionIcon
          size={36}
          radius="xl"
          color="michelin"
          variant="filled"
          aria-label="Home"
          component="span"
        >
          <span style={{ fontSize: 20, fontWeight: 700, lineHeight: 1 }}>✳</span>
        </ActionIcon>
      </Link>

      {/* Search */}
      <form onSubmit={handleSearch} style={{ flex: 1, minWidth: 0 }}>
        <TextInput
          placeholder="Search"
          value={query}
          onChange={(e) => setQuery(e.currentTarget.value)}
          leftSection={<Search size={15} />}
          radius="xl"
          size="sm"
          styles={{
            root: { background: "transparent" },
            input: { backgroundColor: "var(--mantine-color-gray-1)", border: "none" },
          }}
        />
      </form>

      {/* Menu */}
      <ActionIcon size={36} variant="subtle" color="dark" radius="xl" aria-label="Menu">
        <Menu size={20} />
      </ActionIcon>

      {/* Avatar */}
      <Link to="/profile" style={{ textDecoration: "none", flexShrink: 0 }}>
        <Avatar
          src={profile.avatarUrl ?? undefined}
          size={32}
          radius="xl"
          color="michelin"
        >
          {initial}
        </Avatar>
      </Link>
    </header>
  );
}
