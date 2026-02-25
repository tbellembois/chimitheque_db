BEGIN TRANSACTION;

DELETE FROM product;

-- =========================
-- CATEGORIES
-- =========================
DELETE FROM category;
INSERT INTO category VALUES
(1,'Cleaning Product'),
(2,'Maintenance, Calibration Reagent'),
(3,'Cellular Growth Factor'),
(4,'Antibody'),
(5,'Cell Culture Medium & supplement'),
(6,'Cellular Viability Reagent'),
(7,'Comercial Kit');

-- =========================
-- CAS NUMBERS
-- =========================
DELETE FROM cas_number;
INSERT INTO cas_number VALUES
(1,'64-17-5',NULL),
(2,'67-56-1',NULL),
(3,'67-64-1',NULL),
(4,'7647-01-0',NULL),
(5,'1310-73-2',NULL),
(6,'7758-99-8',NULL),
(7,'67-68-5',NULL),
(8,'30525-89-4','CMR'),
(9,'9002-07-7',NULL),
(10,'298-93-1',NULL);

-- =========================
-- CE NUMBERS
-- =========================
DELETE FROM ce_number;
INSERT INTO ce_number VALUES
(1,'200-578-6'),
(2,'200-659-6'),
(3,'200-662-2'),
(4,'231-595-7'),
(5,'215-185-5');

-- =========================
-- EMPIRICAL FORMULAS
-- =========================
DELETE FROM empirical_formula;
INSERT INTO empirical_formula VALUES
(1,'C2H6O'),
(2,'CH4O'),
(3,'C3H6O'),
(4,'HCl'),
(5,'NaOH'),
(6,'CuSO4·5H2O'),
(7,'C2H6OS'),
(8,'C18H16BrN5S');

-- =========================
-- NAMES
-- =========================
DELETE FROM name;
INSERT INTO name VALUES
(1,'Ethanol Absolute'),
(2,'Methanol'),
(3,'Acetone'),
(4,'Hydrochloric Acid'),
(5,'Sodium Hydroxide'),
(6,'Copper(II) Sulfate Pentahydrate'),
(7,'Dimethyl Sulfoxide'),
(8,'Paraformaldehyde'),
(9,'Recombinant Human EGF'),
(10,'Recombinant Human bFGF'),
(11,'Anti-GAPDH Antibody'),
(12,'Anti-CD3 Antibody'),
(13,'DMEM High Glucose'),
(14,'Fetal Bovine Serum'),
(15,'Trypsin-EDTA'),
(16,'MTT'),
(17,'96-Well Culture Plate'),
(18,'T25 Culture Flask'),
(19,'Microscope Slides'),
(20,'Cryogenic Vials 2mL');

-- =========================
-- PRODUCERS
-- =========================
DELETE FROM producer;
INSERT INTO producer VALUES
(1,'Merck / Sigma'),
(2,'Acros Organics'),
(3,'Peprotech'),
(4,'R&D Systems'),
(5,'Abcam'),
(6,'ThermoFisher'),
(7,'Gibco'),
(8,'Promega'),
(9,'Corning'),
(10,'Sarstedt');

-- =========================
-- PRODUCER REFS
-- =========================
DELETE FROM producer_ref;
INSERT INTO producer_ref VALUES
(1,'1.00983',1),
(2,'A/4060/17',2),
(3,'AF-100-15',3),
(4,'233-FB',4),
(5,'ab8245',5),
(6,'MA5-14524',6),
(7,'11965-092',7),
(8,'G4100',8),
(9,'3599',9),
(10,'83.3925',10);

DELETE FROM person;
INSERT INTO person (
    person_id,
    person_email
) VALUES
(1, 'admin@chimitheque.fr'),
(2, 'marie.dubois-m1@chimitheque.fr'),
(3, 'thomas.lefevre-m2@chimitheque.fr'),
(4, 'sophie.martin-m3@chimitheque.fr'),
(5, 'julien.moreau-u1@chimitheque.fr'),
(6, 'claire.roux-u2@chimitheque.fr'),
(7, 'antoine.girard-u3@chimitheque.fr');

DELETE FROM entity;
INSERT INTO entity (
entity_id,
entity_name
) VALUES
(1, 'Entity 1'),
(2, 'Entity 2'),
(3, 'Entity 3');

DELETE FROM entitypeople;
INSERT INTO "entitypeople" VALUES
    (1, 2),
    (2, 3),
    (3, 4);

DELETE FROM permission;
INSERT INTO "permission" VALUES
    (1,'all','all',-1),
    (2,'all','all',1),
    (3,'all','all',2),
    (4,'all','all',3),
    (5,'n','rproducts',-1),
    (5,'n','storages',1),
    (5,'r','products',-1),
    (5,'r','entities',1),
    (6,'w','products',-1),
    (6,'r','rproducts',-1),
    (6,'r','storages',2),
    (6,'r','products',-1),
    (6,'r','entities',2),
    (7,'w','products',-1),
    (7,'r','rproducts',-1),
    (7,'w','storages',3),
    (7,'r','products',-1),
    (7,'r','entities',3);

DELETE FROM personentities;
INSERT INTO "personentities" VALUES
    (2,1),
    (3,2),
    (4,3),
    (5,1),
    (6,2),
    (7,3);

INSERT INTO product (
product_id,
product_type,
product_specificity,
product_molecular_weight,
product_temperature,
product_concentration,
cas_number,
ce_number,
empirical_formula,
physical_state,
signal_word,
name,
producer_ref,
unit_molecular_weight,
unit_temperature,
category
) VALUES

-- CHEM (8)
(1,'chem','≥99.8%',46.07,20,NULL,1,1,1,2,1,1,1,1,2,1),
(2,'chem','HPLC grade',32.04,20,NULL,2,2,2,2,1,2,2,1,2,1),
(3,'chem','ACS reagent',58.08,20,NULL,3,3,3,2,1,3,1,1,2,1),
(4,'chem','37% solution',36.46,20,37,4,4,4,2,1,4,1,1,2,2),
(5,'chem','Pellets ≥98%',40.00,20,NULL,5,5,5,1,1,5,1,1,2,2),
(6,'chem','ACS grade',249.68,20,NULL,6,NULL,6,1,2,6,2,1,2,2),
(7,'chem','Cell culture grade',78.13,20,NULL,7,NULL,7,2,2,7,1,1,2,1),
(8,'chem','EM grade',30.03,20,NULL,8,NULL,NULL,1,1,8,1,1,2,2),

-- BIO (8)
(9,'bio','Carrier-free',6200,4,NULL,NULL,NULL,NULL,1,2,9,3,1,2,3),
(10,'bio','Lyophilized',17000,4,NULL,NULL,NULL,NULL,1,2,10,4,1,2,3),
(11,'bio','Mouse monoclonal',150000,4,NULL,NULL,NULL,NULL,2,2,11,5,1,2,4),
(12,'bio','Rabbit recombinant',145000,4,NULL,NULL,NULL,NULL,2,2,12,6,1,2,4),
(13,'bio','High glucose',NULL,4,NULL,NULL,NULL,NULL,2,2,13,7,1,2,5),
(14,'bio','Heat inactivated',NULL,4,NULL,NULL,NULL,NULL,2,2,14,7,1,2,5),
(15,'bio','0.05% solution',24000,4,NULL,9,NULL,NULL,2,2,15,7,1,2,5),
(16,'bio','Colorimetric assay',414.32,4,NULL,10,NULL,8,1,2,16,8,1,2,6),

-- CONS (4)
(17,'cons','Flat bottom TC treated',NULL,NULL,NULL,NULL,NULL,NULL,1,2,17,9,NULL,NULL,7),
(18,'cons','Vent cap sterile',NULL,NULL,NULL,NULL,NULL,NULL,1,2,18,10,NULL,NULL,7),
(19,'cons','Pre-cleaned glass',NULL,NULL,NULL,NULL,NULL,NULL,1,2,19,NULL,NULL,NULL,2),
(20,'cons','External thread sterile',NULL,NULL,NULL,NULL,NULL,NULL,1,2,20,10,NULL,NULL,2);

DELETE FROM store_location;
INSERT INTO "store_location" VALUES
    (1,'root_sl_1','',1,'root_sl_1',1,NULL),
    (2,'sl_11','',1,'root_sl_1/sl_11',1,1),
    (3,'sl_12','',1,'root_sl_1/sl_12',1,1),
    (4,'root_sl_2','',0,'root_sl_2',2,NULL),
    (5,'[F]sl_21','',1,'root_sl_2/[F]sl_21',2,4),
    (6,'sl_22','',1,'root_sl_2/sl_22',2,4),
    (7,'sl_211','',1,'root_sl_2/[F]sl_21/sl_211',2,5),
    (8,'[I]sl_2111','',1,'root_sl_2/[F]sl_21/sl_211/[I]sl_2111',2,7),
    (9,'sl_21111','',1,'root_sl_2/[F]sl_21/sl_211/[I]sl_2111/sl_21111',2,8),
    (10,'[P]root_sl_3','',1,'[P]root_sl_3',3,NULL);

COMMIT;
