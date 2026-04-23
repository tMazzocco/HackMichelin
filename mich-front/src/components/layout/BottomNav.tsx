import { NavLink } from "react-router-dom";
import { Home, Map, Play, BookOpen, User } from "lucide-react";

const tabs = [
  { to: "/",         icon: Home,     label: "Home" },
  { to: "/map",      icon: Map,      label: "Map" },
  { to: "/shorts",   icon: Play,     label: "Shorts" },
  { to: "/articles", icon: BookOpen, label: "Articles" },
  { to: "/profile",  icon: User,     label: "Profile" },
];

export default function BottomNav() {
  return (
    <nav className="fixed bottom-0 inset-x-0 z-50 bg-background border-t border-black/10 flex">
      {tabs.map(({ to, icon: Icon, label }) => (
        <NavLink
          key={to}
          to={to}
          end={to === "/"}
          className={({ isActive }) =>
            `flex-1 flex flex-col items-center justify-center py-2 gap-0.5 text-[10px] transition-colors ${
              isActive ? "text-primary" : "text-text/40"
            }`
          }
        >
          <Icon size={22} strokeWidth={1.8} />
          <span>{label}</span>
        </NavLink>
      ))}
    </nav>
  );
}
