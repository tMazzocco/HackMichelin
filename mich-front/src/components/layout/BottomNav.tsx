import React from "react";
import { NavLink } from "react-router-dom";
import { Home, Map, Plus, BookOpen, User } from "lucide-react";

const LEFT_TABS = [
  { to: "/",    icon: Home, label: "Home" },
  { to: "/map", icon: Map,  label: "Search" },
];

const RIGHT_TABS = [
  { to: "/articles", icon: BookOpen, label: "Articles" },
  { to: "/profile",  icon: User,     label: "Profile" },
];

const PRIMARY = "#AB152E";
const INACTIVE = "#aaa";

function TabItem({ to, icon: Icon, label, end }: { to: string; icon: React.ElementType; label: string; end?: boolean }) {
  return (
    <NavLink to={to} end={end} style={{ flex: 1, textDecoration: "none" }}>
      {({ isActive }) => (
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 3 }}>
          <Icon size={22} strokeWidth={1.8} color={isActive ? PRIMARY : INACTIVE} />
          <span style={{ fontSize: 10, color: isActive ? PRIMARY : INACTIVE, fontWeight: isActive ? 600 : 400 }}>
            {label}
          </span>
        </div>
      )}
    </NavLink>
  );
}

export default function BottomNav() {
  return (
    <nav
      style={{
        position: "fixed",
        bottom: 0,
        left: 0,
        right: 0,
        zIndex: 1001,
        backgroundColor: "#fff",
        borderRadius: "20px 20px 0 0",
        boxShadow: "0 -4px 24px rgba(0,0,0,0.10)",
        display: "flex",
        alignItems: "center",
        padding: "0 4px 8px",
        height: 70,
      }}
    >
      {LEFT_TABS.map(({ to, icon, label }) => (
        <TabItem key={to} to={to} icon={icon} label={label} end={to === "/"} />
      ))}

      {/* Elevated center action button */}
      <div style={{ flex: 1, display: "flex", justifyContent: "center", alignItems: "center" }}>
        <NavLink to="/shorts" style={{ textDecoration: "none" }}>
          <div
            style={{
              width: 56,
              height: 56,
              borderRadius: "50%",
              backgroundColor: PRIMARY,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              boxShadow: `0 4px 16px ${PRIMARY}66`,
              marginTop: -24,
              border: "3px solid #fff",
            }}
          >
            <Plus size={28} strokeWidth={2.5} color="#fff" />
          </div>
        </NavLink>
      </div>

      {RIGHT_TABS.map(({ to, icon, label }) => (
        <TabItem key={to} to={to} icon={icon} label={label} />
      ))}
    </nav>
  );
}
