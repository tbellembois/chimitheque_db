BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS "bookmark" (
	"bookmark_id"	INTEGER,
	"person"	INTEGER NOT NULL,
	"product"	INTEGER NOT NULL,
	PRIMARY KEY("bookmark_id"),
	FOREIGN KEY("person") REFERENCES "person"("person_id") ON DELETE CASCADE,
	FOREIGN KEY("product") REFERENCES "product"("product_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "borrowing" (
	"borrowing_id"	INTEGER,
	"borrowing_comment"	TEXT,
	"person"	INTEGER NOT NULL,
	"borrower"	INTEGER NOT NULL,
	"storage"	INTEGER NOT NULL UNIQUE,
	PRIMARY KEY("borrowing_id"),
	FOREIGN KEY("borrower") REFERENCES "person"("person_id") ON DELETE CASCADE,
	FOREIGN KEY("person") REFERENCES "person"("person_id") ON DELETE CASCADE,
	FOREIGN KEY("storage") REFERENCES "storage"("storage_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "cas_number" (
	"cas_number_id"	INTEGER,
	"cas_number_label"	TEXT NOT NULL UNIQUE,
	"cas_number_cmr"	TEXT,
	PRIMARY KEY("cas_number_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "ce_number" (
	"ce_number_id"	INTEGER,
	"ce_number_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("ce_number_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "category" (
	"category_id"	INTEGER,
	"category_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("category_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "class_of_compound" (
	"class_of_compound_id"	INTEGER,
	"class_of_compound_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("class_of_compound_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "empirical_formula" (
	"empirical_formula_id"	INTEGER,
	"empirical_formula_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("empirical_formula_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "linear_formula" (
	"linear_formula_id"	INTEGER,
	"linear_formula_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("linear_formula_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "hazard_statement" (
	"hazard_statement_id"	INTEGER,
	"hazard_statement_label"	TEXT NOT NULL,
	"hazard_statement_reference"	TEXT NOT NULL UNIQUE,
	"hazard_statement_cmr"	TEXT,
	PRIMARY KEY("hazard_statement_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "precautionary_statement" (
	"precautionary_statement_id"	INTEGER,
	"precautionary_statement_label"	TEXT NOT NULL,
	"precautionary_statement_reference"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("precautionary_statement_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "physical_state" (
	"physical_state_id"	INTEGER,
	"physical_state_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("physical_state_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "signal_word" (
	"signal_word_id"	INTEGER,
	"signal_word_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("signal_word_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "symbol" (
	"symbol_id"	INTEGER,
	"symbol_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("symbol_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "tag" (
	"tag_id"	INTEGER,
	"tag_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("tag_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "producer" (
	"producer_id"	INTEGER,
	"producer_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("producer_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "producer_ref" (
	"producer_ref_id"	INTEGER,
	"producer_ref_label"	TEXT NOT NULL,
	"producer"	INTEGER,
	PRIMARY KEY("producer_ref_id"),
	FOREIGN KEY("producer") REFERENCES "producer"("producer_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "supplier" (
	"supplier_id"	INTEGER,
	"supplier_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("supplier_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "supplier_ref" (
	"supplier_ref_id"	INTEGER,
	"supplier_ref_label"	TEXT NOT NULL,
	"supplier"	INTEGER,
	PRIMARY KEY("supplier_ref_id"),
	FOREIGN KEY("supplier") REFERENCES "supplier"("supplier_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "name" (
	"name_id"	INTEGER,
	"name_label"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("name_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "unit" (
	"unit_id"	INTEGER,
	"unit_label"	TEXT NOT NULL UNIQUE,
	"unit_multiplier"	REAL NOT NULL DEFAULT 1,
	"unit_type"	TEXT,
	"unit"	INTEGER,
	PRIMARY KEY("unit_id"),
	FOREIGN KEY("unit") REFERENCES "unit"("unit_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "permission" (
	"person"	INTEGER NOT NULL,
	"permission_name"	TEXT NOT NULL,
	"permission_item"	TEXT NOT NULL,
	"permission_entity"	INTEGER NOT NULL,
	-- PRIMARY KEY("permission_id"),
	PRIMARY KEY("person", "permission_name", "permission_item", "permission_entity"),
	FOREIGN KEY("person") REFERENCES "person"("person_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "entity" (
	"entity_id"	INTEGER,
	"entity_name"	TEXT NOT NULL UNIQUE,
	"entity_description"	TEXT,
	PRIMARY KEY("entity_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "person" (
	"person_id"	INTEGER,
	"person_email"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("person_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "product" (
	"product_id"	INTEGER,
	"product_type"	TEXT NOT NULL,
	"product_inchi"	TEXT,
	"product_inchikey"	TEXT,
	"product_canonical_smiles"	TEXT,
	"product_specificity"	TEXT,
	"product_msds"	TEXT,
	"product_restricted"	INTEGER DEFAULT 0,
	"product_radioactive"	INTEGER DEFAULT 0,
	"product_threed_formula"	TEXT,
	"product_twod_formula"	TEXT,
	"product_disposal_comment"	TEXT,
	"product_remark"	TEXT,
	"product_qrcode"	TEXT,
	"product_sheet"	TEXT,
	"product_concentration"	REAL,
	"product_temperature"	REAL,
	"product_molecular_weight"	REAL,
	"cas_number"	INTEGER,
	"ce_number"	INTEGER,
	"person"	INTEGER NOT NULL DEFAULT 1,
	"empirical_formula"	INTEGER,
	"linear_formula"	INTEGER,
	"physical_state"	INTEGER,
	"signal_word"	INTEGER,
	"name"	INTEGER NOT NULL,
	"producer_ref"	INTEGER,
	"unit_molecular_weight"	INTEGER,
	"unit_temperature"	INTEGER,
	"category"	INTEGER,
	"product_number_per_carton"	INTEGER,
	"product_number_per_bag"	INTEGER,
	PRIMARY KEY("product_id"),
	FOREIGN KEY("cas_number") REFERENCES "cas_number"("cas_number_id"),
	FOREIGN KEY("category") REFERENCES "category"("category_id"),
	FOREIGN KEY("ce_number") REFERENCES "ce_number"("ce_number_id"),
	FOREIGN KEY("empirical_formula") REFERENCES "empirical_formula"("empirical_formula_id"),
	FOREIGN KEY("linear_formula") REFERENCES "linear_formula"("linear_formula_id"),
	FOREIGN KEY("name") REFERENCES "name"("name_id"),
	FOREIGN KEY("person") REFERENCES "person"("person_id") ON DELETE SET DEFAULT,
	FOREIGN KEY("physical_state") REFERENCES "physical_state"("physical_state_id"),
	FOREIGN KEY("producer_ref") REFERENCES "producer_ref"("producer_ref_id"),
	FOREIGN KEY("signal_word") REFERENCES "signal_word"("signal_word_id"),
	FOREIGN KEY("unit_molecular_weight") REFERENCES "unit"("unit_id"),
	FOREIGN KEY("unit_temperature") REFERENCES "unit"("unit_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "store_location" (
	"store_location_id"	INTEGER,
	"store_location_name"	TEXT NOT NULL,
	"store_location_color"	TEXT,
	"store_location_can_store"	INTEGER DEFAULT 0,
	"store_location_full_path"	TEXT,
	"entity"	INTEGER NOT NULL,
	"store_location"	INTEGER,
	PRIMARY KEY("store_location_id"),
	FOREIGN KEY("entity") REFERENCES "entity"("entity_id"),
	FOREIGN KEY("store_location") REFERENCES "store_location"("store_location_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "storage" (
	"storage_id"	INTEGER,
	"storage_creation_date"	INTEGER NOT NULL DEFAULT current_timestamp,
	"storage_modification_date"	INTEGER NOT NULL DEFAULT current_timestamp,
	"storage_entry_date"	INTEGER,
	"storage_exit_date"	INTEGER,
	"storage_opening_date"	INTEGER,
	"storage_expiration_date"	INTEGER,
	"storage_quantity"	REAL,
	"storage_barecode"	TEXT,
	"storage_comment"	TEXT,
	"storage_reference"	TEXT,
	"storage_batch_number"	TEXT,
	"storage_to_destroy"	INTEGER DEFAULT 0,
	"storage_archive"	INTEGER DEFAULT 0,
	"storage_qrcode"	BLOB,
	"storage_concentration"	REAL,
	"storage_number_of_bag"	INTEGER,
	"storage_number_of_carton"	INTEGER,
	"person"	INTEGER NOT NULL DEFAULT 1,
	"product"	INTEGER NOT NULL,
	"store_location"	INTEGER NOT NULL,
	"unit_concentration"	REAL,
	"unit_quantity"	REAL,
	"supplier"	INTEGER,
	"storage"	INTEGER,
	PRIMARY KEY("storage_id"),
	FOREIGN KEY("person") REFERENCES "person"("person_id") ON DELETE SET DEFAULT,
	FOREIGN KEY("product") REFERENCES "product"("product_id"),
	FOREIGN KEY("storage") REFERENCES "storage"("storage_id"),
	FOREIGN KEY("store_location") REFERENCES "store_location"("store_location_id"),
	FOREIGN KEY("supplier") REFERENCES "supplier"("supplier_id"),
	FOREIGN KEY("unit_concentration") REFERENCES "unit"("unit_id"),
	FOREIGN KEY("unit_quantity") REFERENCES "unit"("unit_id")
) STRICT;

CREATE TABLE IF NOT EXISTS "productclassesofcompounds" (
	"productclassesofcompounds_product_id"	INTEGER NOT NULL,
	"productclassesofcompounds_class_of_compound_id"	INTEGER NOT NULL,
	PRIMARY KEY("productclassesofcompounds_product_id","productclassesofcompounds_class_of_compound_id"),
	FOREIGN KEY("productclassesofcompounds_class_of_compound_id") REFERENCES "class_of_compound"("class_of_compound_id") ON DELETE CASCADE,
	FOREIGN KEY("productclassesofcompounds_product_id") REFERENCES "product"("product_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "producthazardstatements" (
	"producthazardstatements_product_id"	INTEGER NOT NULL,
	"producthazardstatements_hazard_statement_id"	INTEGER NOT NULL,
	PRIMARY KEY("producthazardstatements_product_id","producthazardstatements_hazard_statement_id"),
	FOREIGN KEY("producthazardstatements_hazard_statement_id") REFERENCES "hazard_statement"("hazard_statement_id") ON DELETE CASCADE,
	FOREIGN KEY("producthazardstatements_product_id") REFERENCES "product"("product_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "productprecautionarystatements" (
	"productprecautionarystatements_product_id"	INTEGER NOT NULL,
	"productprecautionarystatements_precautionary_statement_id"	INTEGER NOT NULL,
	PRIMARY KEY("productprecautionarystatements_product_id","productprecautionarystatements_precautionary_statement_id"),
	FOREIGN KEY("productprecautionarystatements_precautionary_statement_id") REFERENCES "precautionary_statement"("precautionary_statement_id") ON DELETE CASCADE,
	FOREIGN KEY("productprecautionarystatements_product_id") REFERENCES "product"("product_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "productsupplierrefs" (
	"productsupplierrefs_product_id"	INTEGER NOT NULL,
	"productsupplierrefs_supplier_ref_id"	INTEGER NOT NULL,
	PRIMARY KEY("productsupplierrefs_product_id","productsupplierrefs_supplier_ref_id"),
	FOREIGN KEY("productsupplierrefs_product_id") REFERENCES "product"("product_id") ON DELETE CASCADE,
	FOREIGN KEY("productsupplierrefs_supplier_ref_id") REFERENCES "supplier_ref"("supplier_ref_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "productsymbols" (
	"productsymbols_product_id"	integer NOT NULL,
	"productsymbols_symbol_id"	integer NOT NULL,
	PRIMARY KEY("productsymbols_product_id","productsymbols_symbol_id"),
	FOREIGN KEY("productsymbols_product_id") REFERENCES "product"("product_id") ON DELETE CASCADE,
	FOREIGN KEY("productsymbols_symbol_id") REFERENCES "symbol"("symbol_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "productsynonyms" (
	"productsynonyms_product_id"	integer NOT NULL,
	"productsynonyms_name_id"	integer NOT NULL,
	PRIMARY KEY("productsynonyms_product_id","productsynonyms_name_id"),
	FOREIGN KEY("productsynonyms_name_id") REFERENCES "name"("name_id") ON DELETE CASCADE,
	FOREIGN KEY("productsynonyms_product_id") REFERENCES "product"("product_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "producttags" (
	"producttags_product_id"	integer NOT NULL,
	"producttags_tag_id"	integer NOT NULL,
	PRIMARY KEY("producttags_product_id","producttags_tag_id"),
	FOREIGN KEY("producttags_product_id") REFERENCES "product"("product_id") ON DELETE CASCADE,
	FOREIGN KEY("producttags_tag_id") REFERENCES "tag"("tag_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "entitypeople" (
	"entitypeople_entity_id"	integer NOT NULL,
	"entitypeople_person_id"	integer NOT NULL,
	PRIMARY KEY("entitypeople_entity_id","entitypeople_person_id"),
	FOREIGN KEY("entitypeople_entity_id") REFERENCES "entity"("entity_id") ON DELETE CASCADE,
	FOREIGN KEY("entitypeople_person_id") REFERENCES "person"("person_id") ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS "personentities" (
	"personentities_person_id"	integer NOT NULL,
	"personentities_entity_id"	integer NOT NULL,
	PRIMARY KEY("personentities_person_id","personentities_entity_id"),
	FOREIGN KEY("personentities_entity_id") REFERENCES "entity"("entity_id") ON DELETE CASCADE,
	FOREIGN KEY("personentities_person_id") REFERENCES "person"("person_id") ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS "idx_bookmark" ON "bookmark" (
	"person","product"
);
CREATE INDEX IF NOT EXISTS "idx_borrowing" ON "borrowing" (
	"person","storage"
);
CREATE INDEX IF NOT EXISTS "idx_cas_number" ON "cas_number" (
	"cas_number_label"
);
CREATE INDEX IF NOT EXISTS "idx_ce_number" ON "ce_number" (
	"ce_number_label"
);
CREATE INDEX IF NOT EXISTS "idx_category" ON "category" (
	"category_label"
);
CREATE INDEX IF NOT EXISTS "idx_class_of_compound" ON "class_of_compound" (
	"class_of_compound_label"
);
CREATE INDEX IF NOT EXISTS "idx_empirical_formula" ON "empirical_formula" (
	"empirical_formula_label"
);
CREATE INDEX IF NOT EXISTS "idx_linear_formula" ON "linear_formula" (
	"linear_formula_label"
);
CREATE INDEX IF NOT EXISTS "idx_hazard_statement" ON "hazard_statement" (
	"hazard_statement_reference"
);
CREATE INDEX IF NOT EXISTS "idx_precautionary_statement" ON "precautionary_statement" (
	"precautionary_statement_reference"
);
CREATE INDEX IF NOT EXISTS "idx_physical_state" ON "physical_state" (
	"physical_state_label"
);
CREATE INDEX IF NOT EXISTS "idx_signal_word" ON "signal_word" (
	"signal_word_label"
);
CREATE INDEX IF NOT EXISTS "idx_symbol" ON "symbol" (
	"symbol_label"
);
CREATE INDEX IF NOT EXISTS "idx_tag" ON "tag" (
	"tag_label"
);
CREATE INDEX IF NOT EXISTS "idx_producer" ON "producer" (
	"producer_label"
);
CREATE INDEX IF NOT EXISTS "idx_producer_ref" ON "producer_ref" (
	"producer","producer_ref_label"
);
CREATE INDEX IF NOT EXISTS "idx_supplier" ON "supplier" (
	"supplier_label"
);
CREATE INDEX IF NOT EXISTS "idx_supplier_ref" ON "supplier_ref" (
	"supplier","supplier_ref_label"
);
CREATE INDEX IF NOT EXISTS "idx_name" ON "name" (
	"name_label"
);
CREATE INDEX IF NOT EXISTS "idx_unit" ON "unit" (
	"unit_label"
);
CREATE INDEX IF NOT EXISTS "idx_permission" ON "permission" (
	"person"
);
CREATE INDEX IF NOT EXISTS "idx_entity" ON "entity" (
	"entity_name"
);
CREATE INDEX IF NOT EXISTS "idx_person" ON "person" (
	"person_email"
);
CREATE INDEX IF NOT EXISTS "idx_product_cas_number" ON "product" (
	"cas_number"
);
CREATE INDEX IF NOT EXISTS "idx_product_empirical_formula" ON "product" (
	"empirical_formula"
);
CREATE INDEX IF NOT EXISTS "idx_product_name" ON "product" (
	"name"
);
CREATE INDEX IF NOT EXISTS "idx_product_type" ON "product" (
	"product_type"
);
CREATE INDEX IF NOT EXISTS "idx_store_location_name" ON "store_location" (
	"store_location_name"
);
CREATE INDEX IF NOT EXISTS "idx_store_location_entity" ON "store_location" (
	"entity"
);
CREATE INDEX IF NOT EXISTS "idx_storage" ON "storage" (
	"product"
);

COMMIT;
