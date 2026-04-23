import { Article } from "../types";

export const articles: Article[] = [
  {
    id: "article-1",
    title: "The New Star of Tokyo",
    description:
      "Japan's capital continues to dominate the world gastronomy stage with a record number of Michelin stars.",
    content: `Tokyo remains the most Michelin-starred city in the world, a fact that surprises few who have wandered its narrow alleyways and discovered the extraordinary dedication of its chefs. This year's guide reveals a new wave of young Japanese cooks pushing boundaries while maintaining the obsessive precision that defines the city's culinary identity.

At the heart of this revolution is a renewed interest in hyper-local ingredients. Chefs are partnering directly with small farms in Hokkaido and Kyushu, sourcing vegetables that never appear on any market shelf. The result is a kind of farm-to-counter intimacy rarely seen anywhere else.

The 2026 guide also recognises a surge in contemporary Japanese cuisine — dishes that pair traditional fermentation techniques with Western cooking methods. These are not fusion restaurants in the old sense; they are Japanese restaurants that happen to use a French knife or an Italian pasta maker as one more tool in their pursuit of perfection.`,
    image_url: "https://picsum.photos/seed/tokyo-japan/800/500",
    restaurant_name: "Sushi Saito",
    restaurant_lat: 35.6762,
    restaurant_lng: 139.6503,
    author: "Michelin Guide Asia",
    created_at: "2026-04-20",
  },
  {
    id: "article-2",
    title: "Paris: Bistronomie in the Spotlight",
    description:
      "A new generation of Parisian chefs is redefining French cuisine with surprising creativity and accessibility.",
    content: `The bistronomie movement has taken Paris by storm once more. Born in the early 2000s as a rejection of formal fine dining, it has matured into something far more sophisticated — restaurants where a €45 tasting menu can rival a three-star experience in sheer pleasure, if not grandeur.

This year, the guide's inspectors were struck by the density of talent concentrated in the 10th, 11th, and 20th arrondissements. In these neighbourhoods, far from the gilded dining rooms of the 8th, chefs with world-class pedigrees are cooking in tiny rooms for guests who prefer honest conversation to white-gloved silence.

The common thread is seasonality taken to an extreme. Menus here change not weekly but daily, sometimes with the morning market run. It demands an extraordinary level of creativity and physical stamina — and it shows in every plate.`,
    image_url: "https://picsum.photos/seed/paris-bistro/800/500",
    restaurant_name: "Le Servan",
    restaurant_lat: 48.8629,
    restaurant_lng: 2.3708,
    author: "Guide Michelin France",
    created_at: "2026-04-15",
  },
  {
    id: "article-3",
    title: "Green Stars: Sustainability Takes Centre Stage",
    description:
      "Restaurants earning the Michelin Green Star are leading a quiet revolution in sustainable gastronomy.",
    content: `Since its introduction in 2021, the Michelin Green Star has recognised restaurants that demonstrate exceptional commitment to sustainable gastronomy. Five years on, the distinction has evolved from a niche credential into a mainstream aspiration.

The most compelling Green Star recipients are not those who simply swap out plastic straws. They are restaurants that have restructured their entire supply chain — building relationships with regenerative farms, investing in on-site composting, and training their teams in soil science alongside classical cooking technique.

In Copenhagen, the movement that began with Noma has branched into dozens of smaller, more accessible restaurants where foraging and fermentation are simply how one cooks, not a marketing angle. Similar ecosystems are emerging in the Basque Country, in coastal Portugal, and — perhaps most unexpectedly — in Singapore.`,
    image_url: "https://picsum.photos/seed/green-sustainable/800/500",
    restaurant_name: "Silo London",
    restaurant_lat: 51.5389,
    restaurant_lng: -0.0635,
    author: "Michelin Guide",
    created_at: "2026-04-10",
  },
  {
    id: "article-4",
    title: "A Road Trip Through Basque Country",
    description:
      "Discover why the Basque region boasts the highest concentration of Michelin stars per capita anywhere on earth.",
    content: `Pull off the motorway at almost any point between Bilbao and San Sebastián and you will find yourself within a few kilometres of a Michelin-starred restaurant. This is not an exaggeration. The Basque Country, with a population smaller than many European cities, holds more stars per head of population than anywhere else in the world.

The reasons are cultural as much as culinary. Basque society has always revolved around food — the txokos, private gastronomic societies where men gather to cook for one another, have existed for centuries, building a deep collective knowledge of technique and ingredient that seeps into even the most casual pintxos bar.

Today, the generation trained under Juan Mari Arzak and Martin Berasategui has spread across the region, opening their own restaurants and carrying forward a tradition that manages simultaneously to be intensely rooted in place and startlingly avant-garde.`,
    image_url: "https://picsum.photos/seed/basque-spain/800/500",
    restaurant_name: "Arzak",
    restaurant_lat: 43.3089,
    restaurant_lng: -1.9794,
    author: "Guide Michelin España",
    created_at: "2026-04-05",
  },
];
