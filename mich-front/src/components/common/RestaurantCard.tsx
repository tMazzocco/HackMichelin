import { Link } from "react-router-dom";
import { MapPin } from "lucide-react";
import { Restaurant, awardStars, formatDistance } from "../../types";

interface Props {
  restaurant: Restaurant;
}

export default function RestaurantCard({ restaurant }: Props) {
  const stars = awardStars(restaurant.michelin_award);
  const img = restaurant.main_image_url ?? `https://picsum.photos/seed/${restaurant.id}/400/300`;

  return (
    <Link
      to={`/restaurant/${restaurant.id}`}
      className="flex-shrink-0 w-56 rounded-2xl overflow-hidden bg-dark shadow-lg"
    >
      <div className="relative h-36">
        <img src={img} alt={restaurant.name} className="w-full h-full object-cover" />
        {stars && (
          <span className="absolute top-2 right-2 bg-primary text-white text-xs font-bold px-2 py-0.5 rounded-full">
            {stars}
          </span>
        )}
      </div>
      <div className="p-3">
        <p className="text-white font-semibold text-sm leading-tight line-clamp-1">
          {restaurant.name}
        </p>
        {restaurant.city && (
          <p className="text-white/50 text-xs mt-0.5 flex items-center gap-1">
            <MapPin size={10} />
            {restaurant.city}
          </p>
        )}
        {restaurant.distance_meters != null && (
          <p className="text-secondary text-xs mt-1">{formatDistance(restaurant.distance_meters)}</p>
        )}
      </div>
    </Link>
  );
}
