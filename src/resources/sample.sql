BEGIN TRANSACTION;

DELETE FROM borrowing;
DELETE FROM storage;
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
(20,'Cryogenic Vials 2mL'),
(21,'Micropipette Tips (20 µL)');

-- =========================
-- PRODUCERS
-- =========================
DELETE FROM producer_ref;
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

-- Supplier data for chemicals, biological reagents, and consumables
DELETE FROM supplier_ref;
DELETE FROM supplier;
INSERT INTO supplier VALUES
(1, 'VWR International'),
(2, 'Avantor Performance Materials'),
(3, 'Carl Roth'),
(4, 'Bio-Rad'),
(5, 'Greiner Bio-One'),
(6, 'Eppendorf'),
(7, 'VWR France'),
(8, 'Thermo Scientific'),
(9, 'Agilent Technologies'),
(10, 'Corning Incorporated');

-- Supplier Reference data linking suppliers to specific products
INSERT INTO supplier_ref VALUES
-- Chemical suppliers
(1, 'EMSURE®, ACS, Reag. Ph Eur', 1),
(2, 'High Performance LC-MS Grade', 2),
(3, 'Suprapur® grade', 3),
(4, 'Pro Analysis' , 4),
(5, 'Ultrapure for HPLC', 5),
(6, 'TraceSELECT™ Ultra', 6),

-- Biological reagent suppliers
(7, 'Chemicon® International', 7),
(8, 'UntraPURE', 8),
(9, ' allows in vitro culture of human fibroblasts at confluence', 9),
(10, 'contains 1 mg/mL Trypsin + 0.53 mM EDTA •4Na', 10),
(11, 'used in a wide range of colorimetric assays including cell viability, proliferation, and cytotoxicity', 1),

-- Consumable suppliers
(12, 'Certified sterile and non-pyrogenic', 2),
(13, 'Optical Quality', 3),
(14, 'Sterile, CELLSTAR®', 4),
(15, 'Graduated, Self Standing', 2),
(16, 'RNA/DNA free, Certified RNase/RNase free', 2),
(17, 'SnakeSkin® Plaqtes', 5);

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
(3, 'Entity 3'),
(4, 'Entity 4');

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
(20,'cons','External thread sterile',NULL,NULL,NULL,NULL,NULL,NULL,1,2,20,10,NULL,NULL,2),
(21,'cons','Generic',NULL,NULL,NULL,NULL,NULL,NULL,1,2,21,10,NULL,NULL,2);

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

-- Storage entries for Store Location ID 2 (root_sl_1/sl_11)
INSERT INTO storage (
    storage_id,
    storage_creation_date,
    storage_modification_date,
    storage_entry_date,
    storage_quantity,
    storage_comment,
    storage_reference,
    storage_batch_number,
    person,
    product,
    store_location,
    supplier,
    unit_quantity
) VALUES
-- Chemical products in location 2 (sl_11)
(1, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 5000, 'New stock', 'REF-ETH-001', 'B23456', 1, 1, 2, NULL, 2),
(2, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 3000, 'For HPLC analysis', 'REF-MET-001', 'A12345', 1, 2, 2, NULL, 2),
(3, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 2000, 'AC grade', 'REF-ACE-001', 'B09876', 1, 3, 2, NULL, 2),
(4, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 1500, 'Corrosive', 'REF-HCL-001', 'C76543', 1, 4, 2, 1, 2),
(5, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 1000, 'Caustic', 'REF-NAOH-001', 'D54321', 1, 5, 2, 2, 2),
-- Storage for consumables in location 2 (sl_11)
(17, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 50, '96-well plates, sterile', 'REF-96WP-001', NULL, 4, 17, 2, NULL, NULL),

-- Storage entries for Store Location ID 3 (root_sl_1/sl_12)
(6, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 800, 'ACS grade', 'REF-CU-001', 'E34567', 1, 6, 3, NULL, 2),
(7, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 500, 'Cell culture grade', 'REF-DMSO-001', 'F23456', 1, 7, 3, 3, 2),
(8, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 300, 'Sensitive to moisture', 'REF-PARA-001', 'G12345', 1, 8, 3, 4, 2),
(19, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 100, 'Microscope slides', 'REF-MS-001', NULL, 5, 19, 3, NULL, NULL),
(20, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 500, '2mL cryogenic vials', 'REF-CV-001', NULL, 6, 20, 3, NULL, NULL),

-- Storage entries for Store Location ID 5 ([F]sl_21)
(9, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 10, 'Carrier-free, -20°C', 'REF-EGF-001', 'H98765', 2, 9, 5, 5, 3),
(10, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 5, 'Lyophilized, -20°C', 'REF-FGF-001', 'I87654', 2, 10, 5, 5, 3),
(11, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 20, 'Mouse monoclonal', 'REF-AGAP-001', 'J56789', 2, 11, 5, 6, 3),
(12, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 15, 'Rabbit recombinant', 'REF-ACD3-001', 'K45678', 2, 12, 5, 6, 3),
(18, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 100, 'T25 flasks, sterile', 'REF-T25-001', NULL, 5, 18, 5, NULL, NULL),

-- Storage entries for Store Location ID 6 (sl_22)
(21, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 5000, 'New stock', 'REF-ETH-002', 'B23457', 1, 1, 6, NULL, 2),
(22, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 3000, 'For HPLC analysis', 'REF-MET-002', 'A12346', 1, 2, 6, NULL, 2),
(23, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 2000, 'AC grade', 'REF-ACE-002', 'B09877', 1, 3, 6, NULL, 2),
(24, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 1500, 'Corrosive', 'REF-HCL-002', 'C76544', 1, 4, 6, 1, 2),
(25, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 1000, 'Caustic', 'REF-NAOH-002', 'D54322', 1, 5, 6, 2, 2),

-- Storage entries for Store Location ID 7 (sl_211)
(26, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 800, 'ACS grade', 'REF-CU-002', 'E34568', 1, 6, 7, NULL, 2),
(27, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 500, 'Cell culture grade', 'REF-DMSO-002', 'F23457', 1, 7, 7, 3, 2),
(28, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 300, 'Sensitive to moisture', 'REF-PARA-002', 'G12346', 1, 8, 7, 4, 2),
(13, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 5, 'High glucose, 4°C', 'REF-DMEM-001', 'L34567', 3, 13, 7, 7, 2),
(14, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 3, 'Heat inactivated, 4°C', 'REF-FBS-001', 'M23456', 3, 14, 7, 7, 2),

-- Storage entries for Store Location ID 8 ([I]sl_2111)
(29, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 5000, 'New stock', 'REF-ETH-003', 'B23458', 1, 1, 8, NULL, 2),
(30, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 3000, 'For HPLC analysis', 'REF-MET-003', 'A12347', 1, 2, 8, NULL, 2),
(31, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 2000, 'AC grade', 'REF-ACE-003', 'B09878', 1, 3, 8, NULL, 2),
(32, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 1500, 'Corrosive', 'REF-HCL-003', 'C76545', 1, 4, 8, 1, 2),
(33, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 1000, 'Caustic', 'REF-NAOH-003', 'D54323', 1, 5, 8, 2, 2),

-- Storage entries for Store Location ID 9 (sl_21111)
(15, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 2, '0.05% solution, 4°C', 'REF-TRYP-001', 'N12345', 3, 15, 9, 7, 2),
(16, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 1, 'Colorimetric assay, RT', 'REF-MTT-001', 'O98765', 3, 16, 9, 8, 3),
(34, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 800, 'ACS grade', 'REF-CU-003', 'E34569', 1, 6, 9, NULL, 2),
(35, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 500, 'Cell culture grade', 'REF-DMSO-003', 'F23458', 1, 7, 9, 3, 2),
(36, strftime('%s', 'now'), strftime('%s', 'now'), strftime('%s', 'now'), 300, 'Sensitive to moisture', 'REF-PARA-003', 'G12347', 1, 8, 9, 4, 2);

-- Add some borrowing records
-- INSERT INTO borrowing (
--     borrowing_id,
--     borrowing_comment,
--     person,
--     borrower,
--     storage
-- ) VALUES
-- (1, 'Samples needed for analysis', 1, 5, 1),
-- (2, 'Used for preparation', 1, 6, 2),
-- (3, 'Using for buffer preparation', 2, 3, 4),
-- (4, 'For cell culture', 3, 1, 9),
-- (5, 'For immunofluorescence', 3, 2, 11),
-- (6, 'For flow cytometry', 3, 4, 17),
-- (7, 'For western blot', 4, 1, 12);

COMMIT;
