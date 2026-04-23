import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, MapPin } from "lucide-react";
import { articles } from "../data/articles";
import MapView from "../components/map/MapView";

export default function ArticleDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const article = articles.find((a) => a.id === id);

  if (!article) {
    return (
      <div className="page pt-14 pb-20 flex items-center justify-center text-text/40">
        Article not found.
      </div>
    );
  }

  return (
    <div className="page pb-20">
      {/* Hero */}
      <div className="relative h-64">
        <img src={article.image_url} alt={article.title} className="w-full h-full object-cover" />
        <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-black/20 to-transparent" />
        <button
          onClick={() => navigate(-1)}
          className="absolute top-12 left-4 w-9 h-9 rounded-full bg-black/40 backdrop-blur flex items-center justify-center text-white"
        >
          <ArrowLeft size={18} />
        </button>
        <div className="absolute bottom-4 left-4 right-4">
          <p className="text-white/60 text-xs mb-1">{article.author}</p>
          <h1 className="text-white font-bold text-xl leading-tight">{article.title}</h1>
        </div>
      </div>

      {/* Body */}
      <div className="px-4 mt-5">
        <p className="text-text/60 text-sm font-medium mb-4">{article.description}</p>
        {article.content.split("\n\n").map((para, i) => (
          <p key={i} className="text-text text-sm leading-relaxed mb-4">
            {para}
          </p>
        ))}
      </div>

      {/* Restaurant map */}
      <div className="px-4 mt-2">
        <div className="flex items-center gap-1 mb-2 text-text/50 text-xs">
          <MapPin size={12} />
          <span>{article.restaurant_name}</span>
        </div>
        <div className="rounded-2xl overflow-hidden h-44 shadow-md">
          <MapView
            location={{ lat: article.restaurant_lat, lng: article.restaurant_lng }}
            restaurants={[]}
            zoom={14}
            interactive={false}
          />
        </div>
      </div>
    </div>
  );
}
