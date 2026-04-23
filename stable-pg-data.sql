CREATE TABLE users(
   id SERIAL,
   username VARCHAR(50)  NOT NULL,
   email VARCHAR(255)  NOT NULL,
   password_hash VARCHAR(255)  NOT NULL,
   bio TEXT,
   avatar_url TEXT,
   created_at TIMESTAMP,
   updated_at TIMESTAMP,
   PRIMARY KEY(id),
   UNIQUE(username),
   UNIQUE(email)
);

CREATE TABLE type_cuisines(
   code VARCHAR(50) ,
   label VARCHAR(100) ,
   PRIMARY KEY(code)
);

CREATE TABLE media_types(
   id SERIAL,
   label VARCHAR(10) ,
   PRIMARY KEY(id)
);

CREATE TABLE countries(
   id SERIAL,
   code VARCHAR(10) ,
   name VARCHAR(10) ,
   PRIMARY KEY(id)
);

CREATE TABLE cities(
   id SERIAL,
   name VARCHAR(100) ,
   region_name VARCHAR(150) ,
   countries_id INTEGER NOT NULL,
   PRIMARY KEY(id),
   FOREIGN KEY(countries_id) REFERENCES countries(id)
);

CREATE TABLE michelin_awards(
   id SERIAL,
   michelin_award VARCHAR(50) ,
   PRIMARY KEY(id)
);

CREATE TABLE price_categories(
   code VARCHAR(20) ,
   label VARCHAR(100) ,
   PRIMARY KEY(code)
);

CREATE TABLE restaurants(
   id SERIAL,
   identifier VARCHAR(50) ,
   slug VARCHAR(255) ,
   thumbnail_url TEXT NOT NULL,
   name VARCHAR(255)  NOT NULL,
   chef VARCHAR(255) ,
   latitude NUMERIC(15,2)  ,
   longitude NUMERIC(15,2)  ,
   street TEXT,
   postcode VARCHAR(20) ,
   phone VARCHAR(50) ,
   website TEXT,
   short_link TEXT,
   distinction_score INTEGER,
   guide_year INTEGER,
   green_star BOOLEAN,
   main_image_url TEXT,
   main_desc TEXT,
   online_booking BOOLEAN,
   take_away BOOLEAN,
   delivery BOOLEAN,
   status VARCHAR(50) ,
   published_date TIMESTAMP,
   last_update TIMESTAMP,
   michelin_awards_id INTEGER NOT NULL,
   cities_id INTEGER NOT NULL,
   PRIMARY KEY(id),
   UNIQUE(identifier),
   FOREIGN KEY(michelin_awards_id) REFERENCES michelin_awards(id),
   FOREIGN KEY(cities_id) REFERENCES cities(id)
);

CREATE TABLE media(
   id SERIAL,
   thumbnail_url TEXT,
   filename VARCHAR(255)  NOT NULL,
   mime_type VARCHAR(100) ,
   size_bytes BIGINT,
   width INTEGER,
   height INTEGER,
   duration_sec NUMERIC(15,2)  ,
   caption TEXT,
   created_at TIMESTAMP,
   location TEXT,
   restaurants_id INTEGER NOT NULL,
   users_id INTEGER NOT NULL,
   media_types_id INTEGER NOT NULL,
   PRIMARY KEY(id),
   FOREIGN KEY(restaurants_id) REFERENCES restaurants(id),
   FOREIGN KEY(users_id) REFERENCES users(id),
   FOREIGN KEY(media_types_id) REFERENCES media_types(id)
);

CREATE TABLE serving(
   restaurants_id INTEGER,
   type_cuisines_code VARCHAR(50) ,
   PRIMARY KEY(restaurants_id, type_cuisines_code),
   FOREIGN KEY(restaurants_id) REFERENCES restaurants(id),
   FOREIGN KEY(type_cuisines_code) REFERENCES type_cuisines(code)
);

CREATE TABLE costing(
   restaurants_id INTEGER,
   price_categories_code VARCHAR(20) ,
   PRIMARY KEY(restaurants_id, price_categories_code),
   FOREIGN KEY(restaurants_id) REFERENCES restaurants(id),
   FOREIGN KEY(price_categories_code) REFERENCES price_categories(code)
);