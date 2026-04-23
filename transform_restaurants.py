import json
import re
from html import unescape


INPUT_FILE = "all_restaurants.jsonl"
OUTPUT_FILE = "restaurants_es.jsonl"


def strip_html(text):
    if not text:
        return ""
    text = unescape(text)
    text = re.sub(r"<[^>]+>", "", text)
    return text.strip()


def extract_labels(items):
    if not items:
        return []
    return [item.get("label") for item in items if item.get("label")]


def extract_slugs(items):
    if not items:
        return []
    return [item.get("slug") for item in items if item.get("slug")]


def transform(doc):
    lat = doc.get("_geoloc", {}).get("lat")
    lng = doc.get("_geoloc", {}).get("lng")

    transformed = {
        "id": doc.get("objectID") or doc.get("identifier"),
        "identifier": doc.get("identifier"),
        "name": doc.get("name"),
        "slug": doc.get("slug"),

        "description": strip_html(doc.get("main_desc")),

        "chef": doc.get("chef"),

        "city": doc.get("city", {}).get("name"),
        "city_slug": doc.get("city", {}).get("slug"),

        "country": doc.get("country", {}).get("name"),
        "country_slug": doc.get("country", {}).get("slug"),
        "country_code": doc.get("country", {}).get("code"),

        "region": doc.get("region", {}).get("name"),
        "region_slug": doc.get("region", {}).get("slug"),

        "area_name": doc.get("area_name"),
        "area_slug": doc.get("area_slug"),

        "postcode": doc.get("postcode"),
        "street": doc.get("street"),

        "location": {
            "lat": lat,
            "lon": lng
        } if lat is not None and lng is not None else None,

        "michelin_award": doc.get("michelin_award"),
        "michelin_star": doc.get("michelin_star"),
        "distinction_label": doc.get("distinction", {}).get("label"),
        "distinction_slug": doc.get("distinction", {}).get("slug"),
        "distinction_score": doc.get("distinction_score"),
        "green_star": bool(doc.get("green_star")) if doc.get("green_star") is not None else False,
        "new_table": bool(doc.get("new_table")) if doc.get("new_table") is not None else False,

        "price_low": doc.get("price", {}).get("low"),
        "price_high": doc.get("price", {}).get("high"),
        "price_category_code": doc.get("price_category", {}).get("code"),
        "price_category_label": doc.get("price_category", {}).get("label"),
        "price_category_slug": doc.get("price_category", {}).get("slug"),

        "currency": doc.get("currency"),
        "currency_symbol": doc.get("currency_symbol"),

        "cuisines": extract_labels(doc.get("cuisines")),
        "cuisine_slugs": extract_slugs(doc.get("cuisines")),

        "facilities": extract_labels(doc.get("facilities")),
        "facility_slugs": extract_slugs(doc.get("facilities")),

        "special_diets": extract_labels(doc.get("special_diets")),
        "special_diet_slugs": extract_slugs(doc.get("special_diets")),

        "tags": extract_labels(doc.get("tag_thematic")),
        "tag_slugs": extract_slugs(doc.get("tag_thematic")),

        "online_booking": bool(doc.get("online_booking")),
        "booking_provider": doc.get("booking_provider"),
        "booking_url": doc.get("booking_url"),

        "delivery": bool(doc.get("delivery")),
        "delivery_provider": doc.get("delivery_provider"),
        "delivery_booking_url": doc.get("delivery_booking_url"),

        "take_away": bool(doc.get("take_away")),
        "take_away_booking_url": doc.get("take_away_booking_url"),

        "days_open": doc.get("days_open", []),
        "meal_times": doc.get("meal_times", []),

        "guide_year": doc.get("guide_year"),
        "published_date": doc.get("published_date"),
        "last_updated": doc.get("last_updated"),

        "image": doc.get("image"),
        "website": doc.get("website"),
        "phone": doc.get("phone"),
        "url": doc.get("url"),
        "short_link": doc.get("short_link"),

        "site_name": doc.get("site_name"),
        "site_slug": doc.get("site_slug"),
        "sites": doc.get("sites", []),

        "status": doc.get("status"),
        "object_type": doc.get("objectType"),
    }

    return transformed


def main():
    count_in = 0
    count_out = 0

    with open(INPUT_FILE, "r", encoding="utf-8") as infile, open(OUTPUT_FILE, "w", encoding="utf-8") as outfile:
        for line in infile:
            line = line.strip()
            if not line:
                continue

            count_in += 1

            try:
                raw = json.loads(line)
                clean = transform(raw)
                outfile.write(json.dumps(clean, ensure_ascii=False) + "\n")
                count_out += 1
            except Exception as e:
                print(f"Erreur ligne {count_in}: {e}")

    print(f"Terminé : {count_out}/{count_in} restaurants transformés.")
    print(f"Fichier généré : {OUTPUT_FILE}")


if __name__ == "__main__":
    main()
