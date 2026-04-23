import json

with open("restaurants_es.jsonl", "r") as infile, open("bulk.jsonl", "w") as outfile:
    for line in infile:
        doc = json.loads(line)
        action = { "index": { "_index": "restaurants", "_id": doc["id"] } }
        outfile.write(json.dumps(action) + "\n")
        outfile.write(json.dumps(doc) + "\n")
