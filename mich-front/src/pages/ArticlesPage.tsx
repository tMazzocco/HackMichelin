import { articles } from "../data/articles";
import { Link } from "react-router-dom";

export default function ArticlesPage() {
  return (
    <div className="page pt-14 pb-20 px-4">
      <h1 className="font-bold text-xl mt-4 mb-5">Articles</h1>
      <div className="flex flex-col gap-4">
        {articles.map((a) => (
          <Link
            key={a.id}
            to={`/articles/${a.id}`}
            className="rounded-2xl overflow-hidden shadow-sm border border-black/5 bg-white flex flex-col"
          >
            <div className="relative h-44">
              <img src={a.image_url} alt={a.title} className="w-full h-full object-cover" />
              <div className="absolute inset-0 bg-gradient-to-t from-black/50 to-transparent" />
              <p className="absolute bottom-3 left-4 right-4 text-white font-bold text-base leading-tight">
                {a.title}
              </p>
            </div>
            <div className="p-4">
              <p className="text-text/60 text-sm leading-relaxed">{a.description}</p>
              <div className="flex items-center justify-between mt-3">
                <span className="text-text/30 text-xs">{a.author}</span>
                <span className="text-text/30 text-xs">{a.created_at}</span>
              </div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
