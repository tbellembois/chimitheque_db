SELECT 'bookmark' AS table_name, COUNT(*) AS row_count FROM bookmark
UNION ALL
SELECT 'borrowing', COUNT(*) FROM borrowing
UNION ALL
SELECT 'cas_number', COUNT(*) FROM cas_number
UNION ALL
SELECT 'ce_number', COUNT(*) FROM ce_number
UNION ALL
SELECT 'category', COUNT(*) FROM category
UNION ALL
SELECT 'class_of_compound', COUNT(*) FROM class_of_compound
UNION ALL
SELECT 'empirical_formula', COUNT(*) FROM empirical_formula
UNION ALL
SELECT 'linear_formula', COUNT(*) FROM linear_formula
UNION ALL
SELECT 'hazard_statement', COUNT(*) FROM hazard_statement
UNION ALL
SELECT 'precautionary_statement', COUNT(*) FROM precautionary_statement
UNION ALL
SELECT 'physical_state', COUNT(*) FROM physical_state
UNION ALL
SELECT 'signal_word', COUNT(*) FROM signal_word
UNION ALL
SELECT 'symbol', COUNT(*) FROM symbol
UNION ALL
SELECT 'tag', COUNT(*) FROM tag
UNION ALL
SELECT 'producer', COUNT(*) FROM producer
UNION ALL
SELECT 'producer_ref', COUNT(*) FROM producer_ref
UNION ALL
SELECT 'supplier', COUNT(*) FROM supplier
UNION ALL
SELECT 'supplier_ref', COUNT(*) FROM supplier_ref
UNION ALL
SELECT 'name', COUNT(*) FROM name
UNION ALL
SELECT 'unit', COUNT(*) FROM unit
UNION ALL
SELECT 'permission', COUNT(*) FROM permission
UNION ALL
SELECT 'entity', COUNT(*) FROM entity
UNION ALL
SELECT 'person', COUNT(*) FROM person
UNION ALL
SELECT 'product', COUNT(*) FROM product
UNION ALL
SELECT 'store_location', COUNT(*) FROM store_location
UNION ALL
SELECT 'storage', COUNT(*) FROM storage
UNION ALL
SELECT 'productclassesofcompounds', COUNT(*) FROM productclassesofcompounds
UNION ALL
SELECT 'producthazardstatements', COUNT(*) FROM producthazardstatements
UNION ALL
SELECT 'productprecautionarystatements', COUNT(*) FROM productprecautionarystatements
UNION ALL
SELECT 'productsupplierrefs', COUNT(*) FROM productsupplierrefs
UNION ALL
SELECT 'productsymbols', COUNT(*) FROM productsymbols
UNION ALL
SELECT 'productsynonyms', COUNT(*) FROM productsynonyms
UNION ALL
SELECT 'producttags', COUNT(*) FROM producttags
UNION ALL
SELECT 'entitypeople', COUNT(*) FROM entitypeople
UNION ALL
SELECT 'personentities', COUNT(*) FROM personentities
ORDER BY table_name;

