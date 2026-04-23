import { useEffect } from "react";
import { MapContainer, TileLayer, Marker, Popup, useMap } from "react-leaflet";
import L from "leaflet";
import { Link } from "react-router-dom";
import { Restaurant, awardStars } from "../../types";
import { UserLocation } from "../../types";

const CARTO_TILE = "https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}{r}.png";
const CARTO_ATTR = '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>';

function makeMarker(award: string | null) {
  const stars = awardStars(award);
  const bg = award && award !== "BIB_GOURMAND" ? "#AB152E" : "#6b6b6b";
  return L.divIcon({
    className: "",
    html: `<div style="background:${bg};color:#fff;font-size:11px;font-weight:700;padding:4px 7px;border-radius:20px;white-space:nowrap;box-shadow:0 2px 8px rgba(0,0,0,0.3);border:2px solid #fff;">${stars || "●"}</div>`,
    iconAnchor: [20, 12],
  });
}

function RecenterMap({ center }: { center: [number, number] }) {
  const map = useMap();
  useEffect(() => {
    map.setView(center, map.getZoom());
  }, [center, map]);
  return null;
}

interface Props {
  location: UserLocation;
  restaurants: Restaurant[];
  zoom?: number;
  interactive?: boolean;
}

export default function MapView({ location, restaurants, zoom = 13, interactive = true }: Props) {
  return (
    <MapContainer
      center={[location.lat, location.lng]}
      zoom={zoom}
      zoomControl={false}
      dragging={interactive}
      scrollWheelZoom={interactive}
      doubleClickZoom={interactive}
      style={{ height: "100%", width: "100%" }}
    >
      <TileLayer url={CARTO_TILE} attribution={CARTO_ATTR} />
      <RecenterMap center={[location.lat, location.lng]} />
      {restaurants.map((r) => {
        if (!r.latitude || !r.longitude) return null;
        return (
          <Marker
            key={r.id}
            position={[r.latitude, r.longitude]}
            icon={makeMarker(r.michelin_award)}
          >
            <Popup>
              <div className="min-w-[140px]">
                <p className="font-semibold text-sm leading-tight">{r.name}</p>
                {r.city && <p className="text-xs text-gray-500 mt-0.5">{r.city}</p>}
                <Link
                  to={`/restaurant/${r.id}`}
                  className="inline-block mt-2 text-xs font-medium text-[#AB152E]"
                >
                  View details →
                </Link>
              </div>
            </Popup>
          </Marker>
        );
      })}
    </MapContainer>
  );
}
