-- Migration to version 11, step 2: replace product and recreate its indexes.
-- Dropping a table drops its indexes. Snapshot of shema.sql at version 11.
DROP TABLE product;
ALTER TABLE product_new RENAME TO product;
CREATE INDEX IF NOT EXISTS idx_product_cas_number ON product (cas_number);
CREATE INDEX IF NOT EXISTS idx_product_ce_number ON product (ce_number);
CREATE INDEX IF NOT EXISTS idx_product_empirical_formula ON product (empirical_formula);
CREATE INDEX IF NOT EXISTS idx_product_linear_formula ON product (linear_formula);
CREATE INDEX IF NOT EXISTS idx_product_name ON product (name);
CREATE INDEX IF NOT EXISTS idx_product_type ON product (product_type);
CREATE INDEX IF NOT EXISTS idx_product_person ON product(person);
CREATE INDEX IF NOT EXISTS idx_product_category ON product(category);
CREATE INDEX IF NOT EXISTS idx_product_producer_ref ON product(producer_ref);
CREATE INDEX IF NOT EXISTS idx_product_signal_word ON product(signal_word);
CREATE INDEX IF NOT EXISTS idx_product_unit_temperature ON product(unit_temperature);
CREATE INDEX IF NOT EXISTS idx_product_unit_molecular_weight ON product(unit_molecular_weight);
