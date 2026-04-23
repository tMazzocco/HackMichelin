import { Link } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import { useApp } from "../context/AppContext";
import MapView from "../components/map/MapView";
import MapErrorBoundary from "../components/map/MapErrorBoundary";
import ResizableSplit from "../components/layout/ResizableSplit";
import RestaurantCard from "../components/common/RestaurantCard";
import ArticleCard from "../components/common/ArticleCard";
import LoadingSpinner from "../components/common/LoadingSpinner";
import { articles } from "../data/articles";

export default function HomePage() {
  const { location, restaurants, restaurantsLoading, restaurantsError } = useApp();

  const mapPanel = location ? (
    <MapErrorBoundary>
      <MapView location={location} restaurants={restaurants} zoom={13} interactive />
    </MapErrorBoundary>
  ) : (
    <div className="h-full bg-black/5 flex items-center justify-center">
      <LoadingSpinner />
    </div>
  );

  const contentPanel = (
    <div className="h-full overflow-y-auto no-scrollbar">
      {/* Nearby restaurants */}
      <section className="mt-5 px-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="font-semibold text-base">Nearby</h2>
          <Link to="/map" className="text-primary text-sm flex items-center gap-0.5">
            See all <ChevronRight size={14} />
          </Link>
        </div>
        {restaurantsLoading ? (
          <div className="flex justify-center py-6">
            <LoadingSpinner />
          </div>
        ) : restaurantsError ? (
          <p className="text-text/40 text-sm py-4 text-center">
            Could not reach the server. Check your connection and try again.
          </p>
        ) : restaurants.length === 0 ? (
          <p className="text-text/40 text-sm py-4 text-center">No restaurants found nearby.</p>
        ) : (
          <div className="flex gap-3 overflow-x-auto no-scrollbar pb-1">
            {restaurants.map((r) => (
              <RestaurantCard key={r.id} restaurant={r} />
            ))}
          </div>
        )}
      </section>

      {/* Shorts teaser */}
      <section className="mt-6 px-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="font-semibold text-base">Experiences</h2>
          <Link to="/shorts" className="text-primary text-sm flex items-center gap-0.5">
            Watch <ChevronRight size={14} />
          </Link>
        </div>
        <Link
          to="/shorts"
          className="block rounded-2xl overflow-hidden relative h-40 bg-dark shadow-lg"
        >
          <img
            src="https://picsum.photos/seed/michelin-shorts/800/400"
            alt="Shorts"
            className="w-full h-full object-cover opacity-60"
          />
          <div className="absolute inset-0 flex flex-col items-center justify-center text-white">
            <div className="w-12 h-12 rounded-full bg-white/20 backdrop-blur flex items-center justify-center mb-2">
              <span className="text-2xl">▶</span>
            </div>
            <p className="text-sm font-semibold">Discover restaurant moments</p>
          </div>
        </Link>
      </section>

      {/* Articles */}
      <section className="mt-6 px-4 pb-6">
        <div className="flex items-center justify-between mb-3">
          <h2 className="font-semibold text-base">Articles</h2>
          <Link to="/articles" className="text-primary text-sm flex items-center gap-0.5">
            See all <ChevronRight size={14} />
          </Link>
        </div>
        <div className="flex gap-3 overflow-x-auto no-scrollbar pb-1">
          {articles.map((a) => (
            <ArticleCard key={a.id} article={a} />
          ))}
        </div>
      </section>
    </div>
  );

  return (
    // pt-14: below the fixed TopBar (h-14 = 56px)
    // pb-14: above the fixed BottomNav (~56px)
    <div className="fixed inset-0 pt-14 pb-14">
      <ResizableSplit
        top={mapPanel}
        bottom={contentPanel}
        defaultTopPercent={42}
        minTopPercent={10}
        maxTopPercent={85}
      />
    </div>
  );
}
