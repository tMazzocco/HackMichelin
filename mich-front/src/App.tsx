import { Routes, Route } from "react-router-dom";
import { useLocation } from "react-router-dom";
import TopBar from "./components/layout/TopBar";
import BottomNav from "./components/layout/BottomNav";
import HomePage from "./pages/HomePage";
import MapPage from "./pages/MapPage";
import ShortsPage from "./pages/ShortsPage";
import ArticlesPage from "./pages/ArticlesPage";
import ArticleDetailPage from "./pages/ArticleDetailPage";
import RestaurantDetailPage from "./pages/RestaurantDetailPage";
import ProfilePage from "./pages/ProfilePage";

const FULL_SCREEN_ROUTES = ["/shorts", "/map"];

function Layout() {
  const { pathname } = useLocation();
  const isFullScreen = FULL_SCREEN_ROUTES.some((r) => pathname.startsWith(r));

  return (
    <>
      {!isFullScreen && <TopBar />}
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/map" element={<MapPage />} />
        <Route path="/shorts" element={<ShortsPage />} />
        <Route path="/articles" element={<ArticlesPage />} />
        <Route path="/articles/:id" element={<ArticleDetailPage />} />
        <Route path="/restaurant/:id" element={<RestaurantDetailPage />} />
        <Route path="/profile" element={<ProfilePage />} />
      </Routes>
      <BottomNav />
    </>
  );
}

export default function App() {
  return <Layout />;
}
