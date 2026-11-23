-- Test data of the chimitheque_db tests, loaded by test_utils::init_test_with_fixture
-- after the base data (populate_db_with_base_data), with the foreign keys on.
-- Base data rows (units, symbols, statements, suppliers…) are referred to by
-- their label, the rows below by their id.
--
-- People (person 1, admin@chimitheque.fr, is the admin of the base data):
--   2 chemistry.manager  manager of Chemistry (1)
--   3 chemistry.user     member of Chemistry: writes its storages
--   4 biology.manager    manager of Biology (2)
--   5 biology.user       member of Biology: reads its storages, and the restricted products
--   6 new.user           no entity: reads the products only
--   7 all.reader         no entity: reads the products, and the storages of all the entities
--   8 product.editor     no entity: writes the products (and so reads the restricted ones)
--
-- Entities: 1 Chemistry, 2 Biology, 3 Physics (no member, no store location).
--
-- Store locations:
--   Chemistry  1 Room 101 > 2 Cabinet [CA] > 3 Shelf
--                         > 4 Fridge
--   Biology    5 Room 201 > 6 Cabinet [CB]
--                         > 7 Freezer (empty)
--
-- Producer and supplier references: 1-3 of Acros Organics, 4 of Merck / Sigma;
-- Ethanol uses producer reference 1 and supplier reference 1.
--
-- Products: 1 Water, 2 Ethanol (the most complete), 3 Sodium chloride (no
-- storage), 4 Acetone (restricted), 5 Bovine serum albumin (bio), 6 Pipette
-- tips (consumable, no storage).
--
-- Storages:
--   1 Water    1 L    Shelf (Chemistry)      borrowed by chemistry.user
--   2 Ethanol  500 mL Shelf (Chemistry)      updated once: history row 4
--   3 Ethanol  2 L    Fridge (Chemistry)     archived
--   4 Ethanol  1 L    Shelf (Chemistry)      history row of storage 2
--   5 Acetone  1 L    Cabinet [CA] (Chemistry)
--   6 BSA      5 g    Cabinet [CB] (Biology)
--   7 Ethanol  1 L    Cabinet [CB] (Biology)

-- Entities.
INSERT INTO entity (entity_id, entity_name, entity_description) VALUES
    (1, 'Chemistry', 'Chemistry laboratory'),
    (2, 'Biology', 'Biology laboratory'),
    (3, 'Physics', 'Physics laboratory');

-- People.
INSERT INTO person (person_id, person_email) VALUES
    (2, 'chemistry.manager@example.com'),
    (3, 'chemistry.user@example.com'),
    (4, 'biology.manager@example.com'),
    (5, 'biology.user@example.com'),
    (6, 'new.user@example.com'),
    (7, 'all.reader@example.com'),
    (8, 'product.editor@example.com');

-- Members, and managers.
INSERT INTO personentities (personentities_person_id, personentities_entity_id) VALUES
    (2, 1), (3, 1), (4, 2), (5, 2);
INSERT INTO entitypeople (entitypeople_entity_id, entitypeople_person_id) VALUES
    (1, 2), (2, 4);

-- Permissions, as a person save writes them: reading the products for
-- everyone, reading the entities of the members, all on the managed entities.
INSERT INTO permission (person, permission_name, permission_item, permission_entity) VALUES
    (2, 'r', 'products', NULL), (2, 'r', 'entities', 1), (2, 'all', 'all', 1),
    (3, 'r', 'products', NULL), (3, 'r', 'entities', 1), (3, 'w', 'storages', 1),
    (4, 'r', 'products', NULL), (4, 'r', 'entities', 2), (4, 'all', 'all', 2),
    (5, 'r', 'products', NULL), (5, 'r', 'entities', 2), (5, 'r', 'storages', 2),
    (5, 'r', 'rproducts', NULL),
    (6, 'r', 'products', NULL),
    (7, 'r', 'products', NULL), (7, 'r', 'storages', NULL),
    (8, 'w', 'products', NULL);

-- Store locations, with the full paths create_update_store_location writes.
INSERT INTO store_location (store_location_id, store_location_name, store_location_color, store_location_can_store, store_location_full_path, entity, store_location) VALUES
    (1, 'Room 101', NULL, 0, 'Room 101', 1, NULL),
    (2, 'Cabinet [CA]', NULL, 1, 'Room 101/Cabinet [CA]', 1, 1),
    (3, 'Shelf', NULL, 1, 'Room 101/Cabinet [CA]/Shelf', 1, 2),
    (4, 'Fridge', '#0000ff', 1, 'Room 101/Fridge', 1, 1),
    (5, 'Room 201', NULL, 0, 'Room 201', 2, NULL),
    (6, 'Cabinet [CB]', NULL, 1, 'Room 201/Cabinet [CB]', 2, 5),
    (7, 'Freezer', NULL, 1, 'Room 201/Freezer', 2, 5);

-- Product identifiers.
INSERT INTO name (name_id, name_label) VALUES
    (1, 'WATER'), (2, 'ETHANOL'), (3, 'SODIUM CHLORIDE'), (4, 'ACETONE'),
    (5, 'BOVINE SERUM ALBUMIN'), (6, 'PIPETTE TIPS'), (7, 'ETHYL ALCOHOL');
INSERT INTO cas_number (cas_number_label) VALUES
    ('7732-18-5'), ('64-17-5'), ('7647-14-5'), ('67-64-1');
INSERT INTO ce_number (ce_number_id, ce_number_label) VALUES
    (1, '231-791-2'), (2, '200-578-6');
INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES
    (1, 'H2O'), (2, 'C2H6O'), (3, 'ClNa'), (4, 'C3H6O');
INSERT INTO linear_formula (linear_formula_id, linear_formula_label) VALUES
    (1, 'CH3CH2OH');
-- Producer and supplier references: three of Acros Organics, one of Merck / Sigma.
INSERT INTO producer_ref (producer_ref_id, producer_ref_label, producer) VALUES
    (1, 'ETOH-96', (SELECT producer_id FROM producer WHERE producer_label = 'Acros Organics')),
    (2, 'ETOH-99', (SELECT producer_id FROM producer WHERE producer_label = 'Acros Organics')),
    (3, 'ACS reagent', (SELECT producer_id FROM producer WHERE producer_label = 'Acros Organics')),
    (4, 'Suprapur® grade', (SELECT producer_id FROM producer WHERE producer_label = 'Merck / Sigma'));
INSERT INTO supplier_ref (supplier_ref_id, supplier_ref_label, supplier) VALUES
    (1, 'A-1234', (SELECT supplier_id FROM supplier WHERE supplier_label = 'Acros Organics')),
    (2, 'A-5678', (SELECT supplier_id FROM supplier WHERE supplier_label = 'Acros Organics')),
    (3, 'A-9999', (SELECT supplier_id FROM supplier WHERE supplier_label = 'Acros Organics')),
    (4, 'Suprapur® grade', (SELECT supplier_id FROM supplier WHERE supplier_label = 'Merck / Sigma'));

-- Products.
INSERT INTO product (product_id, product_type, product_restricted, name, person, cas_number, ce_number, empirical_formula, linear_formula, physical_state, signal_word, producer_ref, category, product_specificity, product_molecular_weight, unit_molecular_weight, product_temperature, unit_temperature, product_concentration, unit_concentration, product_number_per_bag) VALUES
    (1, 'chem', 0, 1, 1,
        (SELECT cas_number_id FROM cas_number WHERE cas_number_label = '7732-18-5'), 1, 1, NULL,
        (SELECT physical_state_id FROM physical_state WHERE physical_state_label = 'liquid'), NULL, NULL, NULL,
        'distilled', 18.015, (SELECT unit_id FROM unit WHERE unit_label = 'g/mol'), NULL, NULL, NULL, NULL, NULL),
    (2, 'chem', 0, 2, 2,
        (SELECT cas_number_id FROM cas_number WHERE cas_number_label = '64-17-5'), 2, 2, 1,
        (SELECT physical_state_id FROM physical_state WHERE physical_state_label = 'liquid'),
        (SELECT signal_word_id FROM signal_word WHERE signal_word_label = 'danger'), 1, NULL,
        'absolute', 46.07, (SELECT unit_id FROM unit WHERE unit_label = 'g/mol'),
        20, (SELECT unit_id FROM unit WHERE unit_label = '°C'),
        96, (SELECT unit_id FROM unit WHERE unit_label = '%'), NULL),
    (3, 'chem', 0, 3, 1,
        (SELECT cas_number_id FROM cas_number WHERE cas_number_label = '7647-14-5'), NULL, 3, NULL,
        (SELECT physical_state_id FROM physical_state WHERE physical_state_label = 'solid'), NULL, NULL, NULL,
        NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL),
    (4, 'chem', 1, 4, 1,
        (SELECT cas_number_id FROM cas_number WHERE cas_number_label = '67-64-1'), NULL, 4, NULL,
        (SELECT physical_state_id FROM physical_state WHERE physical_state_label = 'liquid'),
        (SELECT signal_word_id FROM signal_word WHERE signal_word_label = 'danger'), NULL, NULL,
        NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL),
    (5, 'bio', 0, 5, 4, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
        (SELECT category_id FROM category WHERE category_label = 'Antibody'),
        NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL),
    (6, 'cons', 0, 6, 1, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
        NULL, NULL, NULL, NULL, NULL, NULL, NULL, 100);

-- Product links: Ethanol (2) has them all, Acetone (4) a few.
INSERT INTO productsynonyms (productsynonyms_product_id, productsynonyms_name_id) VALUES (2, 7);
INSERT INTO productsymbols (productsymbols_product_id, productsymbols_symbol_id) VALUES
    (2, (SELECT symbol_id FROM symbol WHERE symbol_label = 'GHS02')),
    (2, (SELECT symbol_id FROM symbol WHERE symbol_label = 'GHS07')),
    (4, (SELECT symbol_id FROM symbol WHERE symbol_label = 'GHS02'));
INSERT INTO producthazardstatements (producthazardstatements_product_id, producthazardstatements_hazard_statement_id) VALUES
    (2, (SELECT hazard_statement_id FROM hazard_statement WHERE hazard_statement_reference = 'H225')),
    (2, (SELECT hazard_statement_id FROM hazard_statement WHERE hazard_statement_reference = 'H319')),
    (4, (SELECT hazard_statement_id FROM hazard_statement WHERE hazard_statement_reference = 'H225'));
INSERT INTO productprecautionarystatements (productprecautionarystatements_product_id, productprecautionarystatements_precautionary_statement_id) VALUES
    (2, (SELECT precautionary_statement_id FROM precautionary_statement WHERE precautionary_statement_reference = 'P210'));
INSERT INTO productclassesofcompounds (productclassesofcompounds_product_id, productclassesofcompounds_class_of_compound_id) VALUES
    (2, (SELECT class_of_compound_id FROM class_of_compound WHERE class_of_compound_label = 'alcohol'));
INSERT INTO producttags (producttags_product_id, producttags_tag_id) VALUES
    (2, (SELECT tag_id FROM tag WHERE tag_label = 'Solvent'));
INSERT INTO productsupplierrefs (productsupplierrefs_product_id, productsupplierrefs_supplier_ref_id) VALUES
    (2, 1);

-- Storages. Dates: 2024-01-15 (1705276800).
INSERT INTO storage (storage_id, storage_creation_date, storage_modification_date, storage_entry_date, storage_quantity, unit_quantity, storage_barecode, storage_batch_number, storage_archive, person, product, store_location, supplier, storage) VALUES
    (1, 1705276800, 1705276800, 1705276800, 1, (SELECT unit_id FROM unit WHERE unit_label = 'L'), 'CA1.1', 'B-001', 0, 3, 1, 3, NULL, NULL),
    (2, 1705276800, 1705276800, 1705276800, 500, (SELECT unit_id FROM unit WHERE unit_label = 'mL'), 'CA2.1', 'B-002', 0, 3, 2, 3,
        (SELECT supplier_id FROM supplier WHERE supplier_label = 'Acros Organics'), NULL),
    (3, 1705276800, 1705276800, 1705276800, 2, (SELECT unit_id FROM unit WHERE unit_label = 'L'), 'CA2.2', 'B-003', 1, 3, 2, 4, NULL, NULL),
    (4, 1705276800, 1705276800, 1705276800, 1, (SELECT unit_id FROM unit WHERE unit_label = 'L'), 'CA2.1', 'B-002', 0, 3, 2, 3, NULL, 2),
    (5, 1705276800, 1705276800, 1705276800, 1, (SELECT unit_id FROM unit WHERE unit_label = 'L'), 'CA4.1', 'B-004', 0, 2, 4, 2, NULL, NULL),
    (6, 1705276800, 1705276800, 1705276800, 5, (SELECT unit_id FROM unit WHERE unit_label = 'g'), 'CB5.1', 'B-005', 0, 5, 5, 6, NULL, NULL),
    (7, 1705276800, 1705276800, 1705276800, 1, (SELECT unit_id FROM unit WHERE unit_label = 'L'), 'CB2.1', 'B-006', 0, 4, 2, 6, NULL, NULL);

-- Storage 1 lent by chemistry.manager to chemistry.user.
INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES
    (1, 'practical work', 2, 3, 1);

-- Bookmarks.
INSERT INTO bookmark (bookmark_id, person, product) VALUES
    (1, 3, 2), (2, 5, 5);
