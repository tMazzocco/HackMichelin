import { useEffect, useState } from "react";
import { MapContainer, TileLayer, Marker, Popup, useMap } from "react-leaflet";
import L from "leaflet";
import { Link } from "react-router-dom";
import { Restaurant, awardStars, awardLabel, UserLocation } from "../../types";

const CARTO_TILE = "https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}{r}.png";
const CARTO_ATTR =
  '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>';

function awardColor(award: string | null): string {
  switch (award) {
    case "THREE_STARS":
    case "TWO_STARS":
    case "ONE_STAR":
      return "#AB152E";
    case "BIB_GOURMAND":
      return "#E8600A";
    default:
      return "#6b6b6b";
  }
}

function makeMarker(award: string | null) {
  const stars = awardStars(award);
  const bg = awardColor(award);
  return L.divIcon({
    className: "",
    iconSize: [0, 0],
    html: `<div style="background:${bg};color:#fff;font-size:11px;font-weight:700;padding:4px 8px;border-radius:20px;white-space:nowrap;box-shadow:0 2px 8px rgba(0,0,0,0.3);border:2px solid #fff;transform:translate(-50%,-50%);display:inline-block;">${stars || "●"}</div>`,
  });
}

function RecenterMap({ center }: { center: [number, number] }) {
  const map = useMap();
  useEffect(() => {
    map.setView(center, map.getZoom());
  }, [center, map]);
  return null;
}

function MapResizeWatcher() {
  const map = useMap();
  useEffect(() => {
    const container = map.getContainer();
    const observer = new ResizeObserver(() => {
      map.invalidateSize({ animate: false });
    });
    observer.observe(container);
    return () => observer.disconnect();
  }, [map]);
  return null;
}

function RestaurantPopup({ r }: { r: Restaurant }) {
  const stars = awardStars(r.michelin_award);
  const label = awardLabel(r.michelin_award);
  const color = awardColor(r.michelin_award);

  return (
    <div style={{ width: 180, fontFamily: "inherit", padding: 0, margin: 0 }}>
      {r.main_image_url && (
        <div style={{ margin: "-12px -20px 10px", overflow: "hidden", borderRadius: "8px 8px 0 0", height: 100 }}>
          <img
            src={r.main_image_url}
            alt={r.name}
            style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }}
          />
        </div>
      )}

      <div style={{ padding: "0 2px" }}>
        {r.michelin_award && (
          <div style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 5,
            background: color,
            color: "#fff",
            borderRadius: 12,
            padding: "2px 8px",
            fontSize: 11,
            fontWeight: 700,
            marginBottom: 6,
          }}>
            <span>{stars}</span>
            {label && <span style={{ fontWeight: 500 }}>{label}</span>}
            {r.green_star && <span title="Green Star">🌿</span>}
          </div>
        )}

        <p style={{ margin: "0 0 2px", fontWeight: 700, fontSize: 13, lineHeight: 1.3, color: "#1a1a1a" }}>
          {r.name}
        </p>

        {(r.city || r.country_name) && (
          <p style={{ margin: "0 0 6px", fontSize: 11, color: "#6b6b6b" }}>
            {[r.city, r.country_name].filter(Boolean).join(", ")}
          </p>
        )}

        {r.price_category_label && (
          <p style={{ margin: "0 0 8px", fontSize: 11, color: "#888" }}>
            {r.price_category_label}
          </p>
        )}

        <Link
          to={`/restaurant/${r.id}`}
          style={{
            display: "block",
            textAlign: "center",
            background: "#AB152E",
            color: "#fff",
            borderRadius: 8,
            padding: "6px 0",
            fontSize: 12,
            fontWeight: 600,
            textDecoration: "none",
          }}
        >
          View details →
        </Link>
      </div>
    </div>
  );
}

interface Props {
  location: UserLocation;
  restaurants: Restaurant[];
  zoom?: number;
  interactive?: boolean;
}

export default function MapView({ location, restaurants, zoom = 13, interactive = true }: Props) {
  const [instanceKey] = useState(() => `map-${Math.random().toString(36).slice(2)}`);

  return (
    <MapContainer
      key={instanceKey}
      center={[location.lat, location.lng]}
      zoom={zoom}
      zoomControl={false}
      attributionControl={false}
      dragging={interactive}
      scrollWheelZoom={interactive}
      doubleClickZoom={interactive}
      keepBuffer={6}
      style={{ height: "100%", width: "100%" }}
    >
      <TileLayer url={CARTO_TILE} attribution={CARTO_ATTR} />
      <RecenterMap center={[location.lat, location.lng]} />
      <MapResizeWatcher />
      {restaurants.map((r) => {
        if (!r.latitude || !r.longitude) return null;
        return (
          <Marker
            key={r.id}
            position={[r.latitude, r.longitude]}
            icon={makeMarker(r.michelin_award)}
          >
            <Popup closeButton={false}>
              <RestaurantPopup r={r} />
            </Popup>
          </Marker>
        );
      })}
    </MapContainer>
  );
}
