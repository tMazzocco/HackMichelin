import { Link } from "react-router-dom";
import { Article } from "../../types";

interface Props {
  article: Article;
}

export default function ArticleCard({ article }: Props) {
  return (
    <Link to={`/articles/${article.id}`} className="flex-shrink-0 w-64 rounded-2xl overflow-hidden shadow-md bg-white">
      <div className="h-36 relative">
        <img src={article.image_url} alt={article.title} className="w-full h-full object-cover" />
        <div className="absolute inset-0 bg-gradient-to-t from-black/60 to-transparent" />
        <p className="absolute bottom-2 left-3 right-3 text-white font-semibold text-sm leading-tight line-clamp-2">
          {article.title}
        </p>
      </div>
      <div className="p-3">
        <p className="text-text/60 text-xs line-clamp-2">{article.description}</p>
        <p className="text-text/30 text-[10px] mt-1">{article.author}</p>
      </div>
    </Link>
  );
}
