import { Link } from "react-router-dom";
import { useApp } from "../../context/AppContext";

export default function TopBar() {
  const { profile } = useApp();

  return (
    <header className="fixed top-0 inset-x-0 z-50 h-14 flex items-center justify-between px-4 bg-background/80 backdrop-blur-md border-b border-black/5">
      <Link to="/" className="flex items-center gap-2">
        <span className="text-primary font-bold text-lg tracking-tight">MICHELIN</span>
        <span className="text-text/40 text-xs font-medium uppercase tracking-widest">Guide</span>
      </Link>
      <Link to="/profile">
        {profile.avatarUrl ? (
          <img
            src={profile.avatarUrl}
            alt="avatar"
            className="w-8 h-8 rounded-full object-cover border-2 border-primary/20"
          />
        ) : (
          <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center text-primary font-semibold text-sm">
            {profile.firstName.charAt(0).toUpperCase()}
          </div>
        )}
      </Link>
    </header>
  );
}
