BEGIN TRANSACTION;

-- ============================================================
-- CLEAN TEST RANGES
-- ============================================================

DELETE FROM borrowing WHERE borrowing_id BETWEEN 8000 AND 8999;
DELETE FROM bookmark WHERE bookmark_id BETWEEN 7000 AND 7999;
DELETE FROM storage WHERE storage_id BETWEEN 6000 AND 6999;
DELETE FROM product WHERE product_id BETWEEN 5000 AND 5999;
DELETE FROM name WHERE name_id BETWEEN 4000 AND 4999;
DELETE FROM supplier_ref WHERE supplier_ref_id BETWEEN 3000 AND 3999;
DELETE FROM producer_ref WHERE producer_ref_id BETWEEN 2000 AND 2999;
DELETE FROM entity WHERE entity_id BETWEEN 200 AND 209;
DELETE FROM person WHERE person_id BETWEEN 2000 AND 2049;
DELETE FROM permission WHERE person BETWEEN 2000 AND 2049;

-- ============================================================
-- ENTITIES (10)
-- ============================================================

INSERT INTO entity VALUES
(200,'Organic Chemistry','Organic laboratory'),
(201,'Inorganic Chemistry','Inorganic lab'),
(202,'Biochemistry','Biochemistry lab'),
(203,'Analytical Chemistry','Analytical unit'),
(204,'Radiochemistry','Radioactive lab'),
(205,'Pharmacology','Pharma unit'),
(206,'Toxicology','Toxicology lab'),
(207,'Quality Control','QC lab'),
(208,'Teaching Lab','Student lab'),
(209,'Research Core','Central research');

-- ============================================================
-- USERS (50)
-- ============================================================

INSERT INTO person
SELECT 2000 + x,
       'user' || printf('%02d',x) || '@test.lab'
FROM (
  SELECT 0 x UNION ALL SELECT 1 UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL
  SELECT 4 UNION ALL SELECT 5 UNION ALL SELECT 6 UNION ALL SELECT 7 UNION ALL
  SELECT 8 UNION ALL SELECT 9 UNION ALL SELECT 10 UNION ALL SELECT 11 UNION ALL
  SELECT 12 UNION ALL SELECT 13 UNION ALL SELECT 14 UNION ALL SELECT 15 UNION ALL
  SELECT 16 UNION ALL SELECT 17 UNION ALL SELECT 18 UNION ALL SELECT 19 UNION ALL
  SELECT 20 UNION ALL SELECT 21 UNION ALL SELECT 22 UNION ALL SELECT 23 UNION ALL
  SELECT 24 UNION ALL SELECT 25 UNION ALL SELECT 26 UNION ALL SELECT 27 UNION ALL
  SELECT 28 UNION ALL SELECT 29 UNION ALL SELECT 30 UNION ALL SELECT 31 UNION ALL
  SELECT 32 UNION ALL SELECT 33 UNION ALL SELECT 34 UNION ALL SELECT 35 UNION ALL
  SELECT 36 UNION ALL SELECT 37 UNION ALL SELECT 38 UNION ALL SELECT 39 UNION ALL
  SELECT 40 UNION ALL SELECT 41 UNION ALL SELECT 42 UNION ALL SELECT 43 UNION ALL
  SELECT 44 UNION ALL SELECT 45 UNION ALL SELECT 46 UNION ALL SELECT 47 UNION ALL
  SELECT 48 UNION ALL SELECT 49
);

-- ============================================================
-- PERMISSIONS (Stress Dataset)
-- ============================================================

-- Global ALL
INSERT INTO permission
SELECT person_id,'entity','all',-1
FROM person WHERE person_id BETWEEN 2000 AND 2009;

-- Global WRITE
INSERT INTO permission
SELECT person_id,'entity','w',-1
FROM person WHERE person_id BETWEEN 2010 AND 2019;

-- Global READ
INSERT INTO permission
SELECT person_id,'entity','r',-1
FROM person WHERE person_id BETWEEN 2020 AND 2029;

-- Mixed entity permissions
INSERT INTO permission
SELECT p.person_id,'entity',
       CASE WHEN (p.person_id + e.entity_id) % 2 = 0 THEN 'r' ELSE 'w' END,
       e.entity_id
FROM person p
CROSS JOIN entity e
WHERE p.person_id BETWEEN 2030 AND 2039;

-- Conflict examples
INSERT INTO permission VALUES (2040,'entity','r',-1);
INSERT INTO permission VALUES (2040,'entity','w',200);
INSERT INTO permission VALUES (2040,'entity','all',201);

INSERT INTO permission VALUES (2041,'entity','w',-1);
INSERT INTO permission VALUES (2041,'entity','r',200);

INSERT INTO permission VALUES (2042,'entity','r',-1);
INSERT INTO permission VALUES (2042,'entity','r',200);

-- ============================================================
-- NAMES (Products)
-- ============================================================

INSERT INTO name VALUES
(4000,'Acetone'),
(4001,'Ethanol'),
(4002,'Methanol'),
(4003,'Hydrochloric Acid'),
(4004,'Sodium Chloride'),
(4005,'Potassium Nitrate'),
(4006,'Sulfuric Acid'),
(4007,'Benzene'),
(4008,'Chloroform'),
(4009,'Formaldehyde');

-- ============================================================
-- PRODUCER & SUPPLIER REFERENCES
-- ============================================================

INSERT INTO producer_ref VALUES (2000,'Sigma Batch A',1);
INSERT INTO producer_ref VALUES (2001,'Merck Batch B',1);

INSERT INTO supplier_ref VALUES (3000,'Fisher Ref A',1);
INSERT INTO supplier_ref VALUES (3001,'VWR Ref B',1);

-- ============================================================
-- PRODUCTS (10 realistic chemicals)
-- ============================================================

INSERT INTO product
(product_id,product_type,product_molecular_weight,
product_radioactive,product_restricted,name,person)
VALUES
(5000,'solvent',58.08,0,0,4000,1),
(5001,'solvent',46.07,0,0,4001,1),
(5002,'solvent',32.04,0,0,4002,1),
(5003,'acid',36.46,0,1,4003,1),
(5004,'salt',58.44,0,0,4004,1),
(5005,'salt',101.10,0,0,4005,1),
(5006,'acid',98.08,0,1,4006,1),
(5007,'aromatic',78.11,0,1,4007,1),
(5008,'halogenated',119.38,0,1,4008,1),
(5009,'aldehyde',30.03,0,1,4009,1);

-- ============================================================
-- STORAGE LOCATIONS
-- ============================================================

INSERT INTO store_location
(store_location_id,store_location_name,entity)
SELECT 1000 + e.entity_id,
       'Main Cabinet ' || e.entity_id,
       e.entity_id
FROM entity e;

-- ============================================================
-- STORAGE ENTRIES (edge cases included)
-- ============================================================

INSERT INTO storage
(storage_id,storage_quantity,product,store_location,person)
SELECT 6000 + row_number() OVER (),
       (abs(random()) % 100) + 1,
       p.product_id,
       1000 + (200 + (p.product_id % 10)),
       1
FROM product p;

-- Expired storage
INSERT INTO storage
(storage_id,storage_quantity,product,store_location,
 storage_expiration_date,person)
VALUES
(6500,5,5003,1000 + 200,strftime('%s','now') - 86400,1);

-- Zero quantity edge case
INSERT INTO storage
(storage_id,storage_quantity,product,store_location,person)
VALUES
(6501,0,5004,1000 + 201,1);

-- Archived
INSERT INTO storage
(storage_id,storage_quantity,product,store_location,storage_archive,person)
VALUES
(6502,10,5005,1000 + 202,1);

-- ============================================================
-- BORROWING
-- ============================================================

INSERT INTO borrowing
(borrowing_id,person,borrower,storage)
VALUES
(8000,2000,2001,6000),
(8001,2002,2003,6001),
(8002,2004,2005,6002);

-- ============================================================
-- BOOKMARKS
-- ============================================================

INSERT INTO bookmark
(bookmark_id,person,product)
VALUES
(7000,2000,5000),
(7001,2001,5001),
(7002,2002,5002);

COMMIT;
