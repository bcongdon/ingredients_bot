CREATE TABLE food(
  "fdc_id" INTEGER,
  "data_type" TEXT,
  "description" TEXT,
  "food_category_id" TEXT,
  "publication_date" TEXT
);

CREATE TABLE branded_food(
  "fdc_id" INTEGER,
  "brand_owner" TEXT,
  "gtin_upc" TEXT,
  "ingredients" TEXT,
  "serving_size" TEXT,
  "serving_size_unit" TEXT,
  "household_serving_fulltext" TEXT,
  "branded_food_category" TEXT,
  "data_source" TEXT,
  "modified_date" TEXT,
  "available_date" TEXT
);

.mode csv
.import food.csv food
.import branded_food.csv branded_food
