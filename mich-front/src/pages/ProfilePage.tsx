import { useState } from "react";
import { useApp } from "../context/AppContext";
import { UserProfile } from "../types";
import { Check } from "lucide-react";

export default function ProfilePage() {
  const { profile, setProfile } = useApp();
  const [form, setForm] = useState<UserProfile>(profile);
  const [saved, setSaved] = useState(false);

  function handleSave() {
    setProfile(form);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  return (
    <div className="page pt-14 pb-20 px-4">
      <h1 className="font-bold text-xl mt-4 mb-6">Profile</h1>

      {/* Avatar */}
      <div className="flex flex-col items-center mb-8">
        <div className="w-20 h-20 rounded-full bg-primary/10 flex items-center justify-center text-primary font-bold text-3xl shadow">
          {form.firstName.charAt(0).toUpperCase() || "?"}
        </div>
        <p className="mt-2 text-sm text-text/50">Avatar from initials</p>
      </div>

      {/* Form */}
      <div className="flex flex-col gap-4">
        <div>
          <label className="text-xs font-semibold text-text/50 uppercase tracking-wide mb-1 block">
            First name
          </label>
          <input
            type="text"
            value={form.firstName}
            onChange={(e) => setForm({ ...form, firstName: e.target.value })}
            className="w-full px-4 py-3 rounded-xl border border-black/10 bg-white text-sm outline-none focus:border-primary/40"
          />
        </div>
        <div>
          <label className="text-xs font-semibold text-text/50 uppercase tracking-wide mb-1 block">
            Last name
          </label>
          <input
            type="text"
            value={form.lastName}
            onChange={(e) => setForm({ ...form, lastName: e.target.value })}
            className="w-full px-4 py-3 rounded-xl border border-black/10 bg-white text-sm outline-none focus:border-primary/40"
          />
        </div>

        <button
          onClick={handleSave}
          className={`mt-2 py-3 rounded-xl font-semibold text-sm transition-colors flex items-center justify-center gap-2 ${
            saved ? "bg-green-500 text-white" : "bg-primary text-white"
          }`}
        >
          {saved ? (
            <>
              <Check size={16} />
              Saved
            </>
          ) : (
            "Save changes"
          )}
        </button>
      </div>

      {/* App info */}
      <div className="mt-10 pt-6 border-t border-black/5 text-center">
        <p className="text-text/30 text-xs">Guide Michelin — v1.0</p>
        <p className="text-text/20 text-xs mt-1">© Michelin 2026</p>
      </div>
    </div>
  );
}
