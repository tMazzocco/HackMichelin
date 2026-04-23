#!/usr/bin/env python3
"""
HackMichelin - Restaurant importer
Reads all_restaurants.jsonl and bulk-loads into:
  - PostgreSQL (countries, cities, michelin_awards, type_cuisines,
                price_categories, restaurants, serving, costing,
                restaurant_images)
  - Elasticsearch (restaurants index with geo + full-text mapping)
"""

import json
import os
import sys
import time

import psycopg2
from psycopg2.extras import execute_batch
from elasticsearch import Elasticsearch, helpers

# ── Config ──────────────────────────────────────────────────────────────────
JSONL_PATH = os.getenv("JSONL_PATH", "/data/all_restaurants.jsonl")
PG_DSN     = os.getenv("PG_DSN",    "postgresql://admin:changeme@postgres:5432/hackmichelin")
ES_HOST    = os.getenv("ES_HOST",   "http://elasticsearch:9200")
ES_INDEX_RESTAURANTS = "restaurants"
ES_INDEX_USERS       = "users"
BATCH_SIZE           = 500


# ── Wait helpers ─────────────────────────────────────────────────────────────
def wait_for_pg(dsn, retries=30, delay=5):
    for i in range(retries):
        try:
            conn = psycopg2.connect(dsn)
            conn.close()
            print("PostgreSQL ready.")
            return
        except Exception as e:
            print(f"[PG] waiting ({i+1}/{retries}): {e}")
            time.sleep(delay)
    raise RuntimeError("PostgreSQL not ready after retries")


def wait_for_es(es, retries=30, delay=5):
    for i in range(retries):
        try:
            es.cluster.health(wait_for_status="yellow")
            print("Elasticsearch ready.")
            return
        except Exception as e:
            print(f"[ES] waiting ({i+1}/{retries}): {e}")
            time.sleep(delay)
    raise RuntimeError("Elasticsearch not ready after retries")


# ── Elasticsearch indexes ─────────────────────────────────────────────────────
def create_es_restaurants_index(es):
    if es.indices.exists(index=ES_INDEX_RESTAURANTS):
        print(f"Index '{ES_INDEX_RESTAURANTS}' already exists, skipping creation.")
        return
    mapping = {
        "settings": {"number_of_shards": 1, "number_of_replicas": 0},
        "mappings": {
            "properties": {
                "objectID":          {"type": "keyword"},
                "identifier":        {"type": "keyword"},
                "slug":              {"type": "keyword"},
                "name":              {"type": "text", "fields": {"keyword": {"type": "keyword"}}},
                "chef":              {"type": "text"},
                "main_desc":         {"type": "text"},
                "location":          {"type": "geo_point"},
                "city":              {"type": "keyword"},
                "country_name":      {"type": "keyword"},
                "country_code":      {"type": "keyword"},
                "region":            {"type": "keyword"},
                "area_name":         {"type": "keyword"},
                "street":            {"type": "text"},
                "postcode":          {"type": "keyword"},
                "michelin_award":    {"type": "keyword"},
                "distinction_slug":  {"type": "keyword"},
                "distinction_score": {"type": "integer"},
                "guide_year":        {"type": "integer"},
                "green_star":        {"type": "boolean"},
                "price_category":    {"type": "keyword"},
                "cuisines":          {"type": "keyword"},
                "tag_thematic":      {"type": "keyword"},
                "special_diets":     {"type": "keyword"},
                "facilities":        {"type": "keyword"},
                "online_booking":    {"type": "boolean"},
                "take_away":         {"type": "boolean"},
                "delivery":          {"type": "boolean"},
                "phone":             {"type": "keyword"},
                "website":           {"type": "keyword"},
                "short_link":        {"type": "keyword"},
                "url":               {"type": "keyword"},
                "status":            {"type": "keyword"},
                "published_date":    {"type": "date", "format": "epoch_millis"},
                "last_updated":      {"type": "date", "format": "epoch_millis"},
            }
        },
    }
    es.indices.create(index=ES_INDEX_RESTAURANTS, body=mapping)
    print(f"Index '{ES_INDEX_RESTAURANTS}' created.")


def create_es_users_index(es):
    # Users are indexed by SearchService at runtime (on user.registered MQTT events).
    # This just ensures the index exists with the correct mapping on startup.
    if es.indices.exists(index=ES_INDEX_USERS):
        print(f"Index '{ES_INDEX_USERS}' already exists, skipping creation.")
        return
    mapping = {
        "settings": {"number_of_shards": 1, "number_of_replicas": 0},
        "mappings": {
            "properties": {
                "user_id":         {"type": "keyword"},
                "username":        {"type": "text", "fields": {"keyword": {"type": "keyword"}}},
                "bio":             {"type": "text"},
                "avatar_url":      {"type": "keyword", "index": False},
                "stars_collected": {"type": "integer"},
                "followers_count": {"type": "integer"},
            }
        },
    }
    es.indices.create(index=ES_INDEX_USERS, body=mapping)
    print(f"Index '{ES_INDEX_USERS}' created.")


# ── Lookup-table helpers (in-process cache + upsert) ─────────────────────────
# Each helper fetches the surrogate id on first encounter and caches it.
# Subsequent calls for the same key are O(1) dict lookups.

_country_cache = {}   # code -> id
_city_cache    = {}   # (name, country_id) -> id
_award_cache   = {}   # michelin_award string -> id


def get_or_create_country(cur, code, name):
    if not code:
        return None
    if code in _country_cache:
        return _country_cache[code]
    cur.execute(
        """
        INSERT INTO countries (code, name)
        VALUES (%s, %s)
        ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        """,
        (code, name or code),
    )
    row = cur.fetchone()
    _country_cache[code] = row[0]
    return row[0]


def get_or_create_city(cur, name, region_name, area_name, country_id):
    if not name or country_id is None:
        return None
    key = (name, country_id)
    if key in _city_cache:
        return _city_cache[key]
    cur.execute(
        """
        INSERT INTO cities (name, region_name, area_name, country_id)
        VALUES (%s, %s, %s, %s)
        ON CONFLICT (name, country_id) DO UPDATE
            SET region_name = EXCLUDED.region_name,
                area_name   = EXCLUDED.area_name
        RETURNING id
        """,
        (name, region_name, area_name, country_id),
    )
    row = cur.fetchone()
    _city_cache[key] = row[0]
    return row[0]


def get_or_create_award(cur, award_str):
    if not award_str:
        return None
    if award_str in _award_cache:
        return _award_cache[award_str]
    cur.execute(
        """
        INSERT INTO michelin_awards (michelin_award)
        VALUES (%s)
        ON CONFLICT (michelin_award) DO UPDATE SET michelin_award = EXCLUDED.michelin_award
        RETURNING id
        """,
        (award_str,),
    )
    row = cur.fetchone()
    _award_cache[award_str] = row[0]
    return row[0]


def upsert_type_cuisine(cur, code, label):
    if not code:
        return
    cur.execute(
        """
        INSERT INTO type_cuisines (code, label)
        VALUES (%s, %s)
        ON CONFLICT (code) DO NOTHING
        """,
        (code, label),
    )


def upsert_price_category(cur, code, label):
    if not code:
        return
    cur.execute(
        """
        INSERT INTO price_categories (code, label)
        VALUES (%s, %s)
        ON CONFLICT (code) DO NOTHING
        """,
        (code, label),
    )


# ── Parse one JSONL line ──────────────────────────────────────────────────────
def parse_restaurant(cur, r):
    geoloc  = r.get("_geoloc") or {}
    lat     = geoloc.get("lat")
    lng     = geoloc.get("lng")

    city_raw    = r.get("city") or {}
    country_raw = r.get("country") or {}
    region_raw  = r.get("region") or {}
    distinction = r.get("distinction") or {}
    price_cat   = r.get("price_category") or {}
    main_image  = r.get("main_image") or {}

    # ── Resolve / create lookup rows ─────────────────────────────────────────
    country_id = get_or_create_country(
        cur,
        country_raw.get("code"),
        country_raw.get("name"),
    )

    city_id = get_or_create_city(
        cur,
        city_raw.get("name"),
        region_raw.get("name"),
        r.get("area_name"),
        country_id,
    )

    michelin_award_id = get_or_create_award(cur, r.get("michelin_award"))

    # Ensure cuisine lookup rows exist (serving rows added in flush_pg)
    for c in (r.get("cuisines") or []):
        upsert_type_cuisine(cur, c.get("code"), c.get("label"))

    # Ensure price category lookup row exists (costing rows added in flush_pg)
    upsert_price_category(cur, price_cat.get("code"), price_cat.get("label"))

    # ── PostgreSQL restaurant row ─────────────────────────────────────────────
    pg_row = {
        "id":                 r.get("objectID"),
        "identifier":         r.get("identifier"),
        "slug":               r.get("slug"),
        "name":               r.get("name"),
        "chef":               r.get("chef"),
        "latitude":           lat,
        "longitude":          lng,
        "street":             r.get("street"),
        "postcode":           r.get("postcode"),
        "phone":              r.get("phone"),
        "website":            r.get("website"),
        "short_link":         r.get("short_link"),
        "distinction_score":  r.get("distinction_score"),
        "guide_year":         r.get("guide_year"),
        "green_star":         bool(r.get("green_star")),
        "main_image_url":     main_image.get("url"),
        "main_desc":          r.get("main_desc"),
        "online_booking":     bool(r.get("online_booking")),
        "take_away":          bool(r.get("take_away")),
        "delivery":           bool(r.get("delivery")),
        "status":             r.get("status"),
        "published_date":     r.get("published_date"),
        "last_updated":       r.get("last_updated"),
        "michelin_award_id":  michelin_award_id,
        "city_id":            city_id,
    }

    # ── Elasticsearch document (field names unchanged for ES) ─────────────────
    es_source = {
        "objectID":          r.get("objectID"),
        "identifier":        r.get("identifier"),
        "slug":              r.get("slug"),
        "name":              r.get("name"),
        "chef":              r.get("chef"),
        "main_desc":         r.get("main_desc"),
        "city":              city_raw.get("name"),
        "country_name":      country_raw.get("name"),
        "country_code":      country_raw.get("code"),
        "region":            region_raw.get("name"),
        "area_name":         r.get("area_name"),
        "street":            r.get("street"),
        "postcode":          r.get("postcode"),
        "michelin_award":    r.get("michelin_award"),
        "distinction_slug":  distinction.get("slug"),
        "distinction_score": r.get("distinction_score"),
        "guide_year":        r.get("guide_year"),
        "green_star":        bool(r.get("green_star")),
        "price_category":    price_cat.get("label"),
        "cuisines":          [c["label"] for c in (r.get("cuisines") or []) if c.get("label")],
        "tag_thematic":      [t["label"] for t in (r.get("tag_thematic") or []) if t.get("label")],
        "special_diets":     [s["label"] for s in (r.get("special_diets") or []) if s.get("label")],
        "facilities":        [f["label"] for f in (r.get("facilities") or []) if f.get("label")],
        "online_booking":    bool(r.get("online_booking")),
        "take_away":         bool(r.get("take_away")),
        "delivery":          bool(r.get("delivery")),
        "phone":             r.get("phone"),
        "website":           r.get("website"),
        "short_link":        r.get("short_link"),
        "url":               r.get("url"),
        "status":            r.get("status"),
        "published_date":    r.get("published_date"),
        "last_updated":      r.get("last_updated"),
    }
    if lat is not None and lng is not None:
        es_source["location"] = {"lat": lat, "lon": lng}

    es_doc = {
        "_index":  ES_INDEX_RESTAURANTS,
        "_id":     r.get("objectID"),
        "_source": es_source,
    }

    images = [
        {
            "restaurant_id": r.get("objectID"),
            "identifier":    img.get("identifier"),
            "url":           img.get("url"),
            "copyright":     img.get("copyright"),
            "topic":         img.get("topic"),
            "order":         img.get("order", 0),
        }
        for img in (r.get("images") or [])
    ]

    servings = [
        {
            "restaurant_id":      r.get("objectID"),
            "type_cuisines_code": c.get("code"),
        }
        for c in (r.get("cuisines") or [])
        if c.get("code")
    ]

    costings = []
    if price_cat.get("code"):
        costings.append({
            "restaurant_id":         r.get("objectID"),
            "price_categories_code": price_cat.get("code"),
        })

    return pg_row, es_doc, images, servings, costings


# ── Flush batch to PostgreSQL ─────────────────────────────────────────────────
def flush_pg(cur, restaurants, images, servings, costings):
    execute_batch(
        cur,
        """
        INSERT INTO restaurants (
            id, identifier, slug, name, chef,
            latitude, longitude,
            street, postcode, phone, website, short_link,
            distinction_score, guide_year, green_star,
            main_image_url, main_desc,
            online_booking, take_away, delivery,
            status, published_date, last_updated,
            michelin_award_id, city_id
        ) VALUES (
            %(id)s, %(identifier)s, %(slug)s, %(name)s, %(chef)s,
            %(latitude)s, %(longitude)s,
            %(street)s, %(postcode)s, %(phone)s, %(website)s, %(short_link)s,
            %(distinction_score)s, %(guide_year)s, %(green_star)s,
            %(main_image_url)s, %(main_desc)s,
            %(online_booking)s, %(take_away)s, %(delivery)s,
            %(status)s,
            to_timestamp(%(published_date)s::bigint / 1000.0),
            to_timestamp(%(last_updated)s::bigint / 1000.0),
            %(michelin_award_id)s, %(city_id)s
        ) ON CONFLICT (id) DO NOTHING
        """,
        restaurants,
        page_size=200,
    )

    if images:
        execute_batch(
            cur,
            """
            INSERT INTO restaurant_images
                (restaurant_id, identifier, url, copyright, topic, "order")
            VALUES
                (%(restaurant_id)s, %(identifier)s, %(url)s,
                 %(copyright)s, %(topic)s, %(order)s)
            ON CONFLICT (restaurant_id, identifier) DO NOTHING
            """,
            images,
            page_size=500,
        )

    if servings:
        execute_batch(
            cur,
            """
            INSERT INTO serving (restaurant_id, type_cuisines_code)
            VALUES (%(restaurant_id)s, %(type_cuisines_code)s)
            ON CONFLICT DO NOTHING
            """,
            servings,
            page_size=500,
        )

    if costings:
        execute_batch(
            cur,
            """
            INSERT INTO costing (restaurant_id, price_categories_code)
            VALUES (%(restaurant_id)s, %(price_categories_code)s)
            ON CONFLICT DO NOTHING
            """,
            costings,
            page_size=500,
        )


# ── Main ─────────────────────────────────────────────────────────────────────
def main():
    es = Elasticsearch(ES_HOST)
    wait_for_es(es)
    wait_for_pg(PG_DSN)

    create_es_restaurants_index(es)
    create_es_users_index(es)

    conn = psycopg2.connect(PG_DSN)
    cur  = conn.cursor()

    cur.execute("SELECT COUNT(*) FROM restaurants")
    existing = cur.fetchone()[0]
    if existing > 0:
        print(f"Already imported {existing} restaurants. Skipping.")
        cur.close()
        conn.close()
        return

    pg_restaurants = []
    pg_images      = []
    pg_servings    = []
    pg_costings    = []
    es_docs        = []
    total          = 0

    print(f"Importing from {JSONL_PATH} ...")
    with open(JSONL_PATH, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"[WARN] JSON parse error: {e}", file=sys.stderr)
                continue

            pg_row, es_doc, images, servings, costings = parse_restaurant(cur, r)

            # Skip rows where city resolution failed (city_id is required)
            if pg_row["city_id"] is None:
                print(f"[WARN] skipping {pg_row['id']}: could not resolve city", file=sys.stderr)
                continue

            pg_restaurants.append(pg_row)
            pg_images.extend(images)
            pg_servings.extend(servings)
            pg_costings.extend(costings)
            es_docs.append(es_doc)
            total += 1

            if len(pg_restaurants) >= BATCH_SIZE:
                flush_pg(cur, pg_restaurants, pg_images, pg_servings, pg_costings)
                conn.commit()
                helpers.bulk(es, es_docs)
                pg_restaurants, pg_images, pg_servings, pg_costings, es_docs = [], [], [], [], []
                print(f"  → {total} restaurants imported ...")

    if pg_restaurants:
        flush_pg(cur, pg_restaurants, pg_images, pg_servings, pg_costings)
        conn.commit()
        helpers.bulk(es, es_docs)

    cur.close()
    conn.close()
    print(f"Done. Total: {total} restaurants imported into PostgreSQL + Elasticsearch.")


if __name__ == "__main__":
    main()
