#!/usr/bin/env python3
"""
HackMichelin - Restaurant importer
Reads all_restaurants.jsonl and bulk-loads into:
  - PostgreSQL (restaurants, restaurant_images, restaurant_cuisines)
  - Elasticsearch (restaurants index with geo + full-text mapping)
"""

import json
import os
import sys
import time

import psycopg2
from psycopg2.extras import execute_batch
from elasticsearch import Elasticsearch, helpers

# ── Config ──────────────────────────────────────────────────
JSONL_PATH = os.getenv("JSONL_PATH", "/data/all_restaurants.jsonl")
PG_DSN     = os.getenv("PG_DSN",    "postgresql://admin:changeme@postgres:5432/hackmichelin")
ES_HOST    = os.getenv("ES_HOST",   "http://elasticsearch:9200")
ES_INDEX   = "restaurants"
BATCH_SIZE = 500


# ── Wait helpers ─────────────────────────────────────────────
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


# ── Elasticsearch index ──────────────────────────────────────
def create_es_index(es):
    if es.indices.exists(index=ES_INDEX):
        print(f"Index '{ES_INDEX}' already exists, skipping creation.")
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
    es.indices.create(index=ES_INDEX, body=mapping)
    print(f"Index '{ES_INDEX}' created.")


# ── Parse one JSONL line ─────────────────────────────────────
def parse_restaurant(r):
    geoloc = r.get("_geoloc") or {}
    lat = geoloc.get("lat")
    lng = geoloc.get("lng")

    city        = (r.get("city") or {}).get("name")
    country     = r.get("country") or {}
    region      = r.get("region") or {}
    distinction = r.get("distinction") or {}
    price_cat   = r.get("price_category") or {}
    main_image  = r.get("main_image") or {}

    pg_row = {
        "id":                   r.get("objectID"),
        "identifier":           r.get("identifier"),
        "slug":                 r.get("slug"),
        "name":                 r.get("name"),
        "chef":                 r.get("chef"),
        "lat":                  lat,
        "lng":                  lng,
        "city":                 city,
        "country_code":         country.get("code"),
        "country_name":         country.get("name"),
        "region_name":          region.get("name"),
        "area_name":            r.get("area_name"),
        "street":               r.get("street"),
        "postcode":             r.get("postcode"),
        "phone":                r.get("phone"),
        "website":              r.get("website"),
        "short_link":           r.get("short_link"),
        "michelin_award":       r.get("michelin_award"),
        "distinction_score":    r.get("distinction_score"),
        "guide_year":           r.get("guide_year"),
        "green_star":           bool(r.get("green_star")),
        "price_category_code":  price_cat.get("code"),
        "price_category_label": price_cat.get("label"),
        "main_image_url":       main_image.get("url"),
        "main_desc":            r.get("main_desc"),
        "online_booking":       bool(r.get("online_booking")),
        "take_away":            bool(r.get("take_away")),
        "delivery":             bool(r.get("delivery")),
        "status":               r.get("status"),
        "published_date":       r.get("published_date"),
        "last_updated":         r.get("last_updated"),
    }

    es_source = {
        "objectID":          r.get("objectID"),
        "identifier":        r.get("identifier"),
        "slug":              r.get("slug"),
        "name":              r.get("name"),
        "chef":              r.get("chef"),
        "main_desc":         r.get("main_desc"),
        "city":              city,
        "country_name":      country.get("name"),
        "country_code":      country.get("code"),
        "region":            region.get("name"),
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

    es_doc = {"_index": ES_INDEX, "_id": r.get("objectID"), "_source": es_source}

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

    cuisines = [
        {
            "restaurant_id": r.get("objectID"),
            "code":          c.get("code"),
            "label":         c.get("label"),
            "slug":          c.get("slug"),
        }
        for c in (r.get("cuisines") or [])
    ]

    return pg_row, es_doc, images, cuisines


# ── Flush batch to PostgreSQL ────────────────────────────────
def flush_pg(cur, restaurants, images, cuisines):
    execute_batch(
        cur,
        """
        INSERT INTO restaurants (
            id, identifier, slug, name, chef, lat, lng,
            city, country_code, country_name, region_name, area_name,
            street, postcode, phone, website, short_link,
            michelin_award, distinction_score, guide_year, green_star,
            price_category_code, price_category_label,
            main_image_url, main_desc, online_booking, take_away, delivery,
            status, published_date, last_updated
        ) VALUES (
            %(id)s, %(identifier)s, %(slug)s, %(name)s, %(chef)s, %(lat)s, %(lng)s,
            %(city)s, %(country_code)s, %(country_name)s, %(region_name)s, %(area_name)s,
            %(street)s, %(postcode)s, %(phone)s, %(website)s, %(short_link)s,
            %(michelin_award)s, %(distinction_score)s, %(guide_year)s, %(green_star)s,
            %(price_category_code)s, %(price_category_label)s,
            %(main_image_url)s, %(main_desc)s, %(online_booking)s, %(take_away)s, %(delivery)s,
            %(status)s,
            to_timestamp(%(published_date)s::bigint / 1000.0),
            to_timestamp(%(last_updated)s::bigint / 1000.0)
        ) ON CONFLICT (id) DO NOTHING
        """,
        restaurants,
        page_size=200,
    )

    if images:
        execute_batch(
            cur,
            """
            INSERT INTO restaurant_images (restaurant_id, identifier, url, copyright, topic, "order")
            VALUES (%(restaurant_id)s, %(identifier)s, %(url)s, %(copyright)s, %(topic)s, %(order)s)
            ON CONFLICT (restaurant_id, identifier) DO NOTHING
            """,
            images,
            page_size=500,
        )

    if cuisines:
        execute_batch(
            cur,
            """
            INSERT INTO restaurant_cuisines (restaurant_id, code, label, slug)
            VALUES (%(restaurant_id)s, %(code)s, %(label)s, %(slug)s)
            ON CONFLICT DO NOTHING
            """,
            cuisines,
            page_size=500,
        )


# ── Main ─────────────────────────────────────────────────────
def main():
    es = Elasticsearch(ES_HOST)
    wait_for_es(es)
    wait_for_pg(PG_DSN)

    create_es_index(es)

    conn = psycopg2.connect(PG_DSN)
    cur  = conn.cursor()

    cur.execute("SELECT COUNT(*) FROM restaurants")
    existing = cur.fetchone()[0]
    if existing > 0:
        print(f"Already imported {existing} restaurants. Skipping.")
        cur.close()
        conn.close()
        return

    pg_restaurants, pg_images, pg_cuisines, es_docs = [], [], [], []
    total = 0

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

            pg_row, es_doc, images, cuisines = parse_restaurant(r)
            pg_restaurants.append(pg_row)
            pg_images.extend(images)
            pg_cuisines.extend(cuisines)
            es_docs.append(es_doc)
            total += 1

            if len(pg_restaurants) >= BATCH_SIZE:
                flush_pg(cur, pg_restaurants, pg_images, pg_cuisines)
                conn.commit()
                helpers.bulk(es, es_docs)
                pg_restaurants, pg_images, pg_cuisines, es_docs = [], [], [], []
                print(f"  → {total} restaurants imported ...")

    if pg_restaurants:
        flush_pg(cur, pg_restaurants, pg_images, pg_cuisines)
        conn.commit()
        helpers.bulk(es, es_docs)

    cur.close()
    conn.close()
    print(f"Done. Total: {total} restaurants imported into PostgreSQL + Elasticsearch.")


if __name__ == "__main__":
    main()
