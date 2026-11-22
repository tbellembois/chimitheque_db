"""sqlseed transform for the product link tables (see sqlfeed_generate.yaml).

Each link table generates 2 * PRODUCTS rows. Row n (from 0) links product
n % PRODUCTS + 1 to item (n % PRODUCTS + n // PRODUCTS) % ITEMS + 1, so every
product gets two different items and no (product, item) pair repeats. Item ids
must run from 1 to ITEMS without gaps, which holds for freshly generated data.
"""

PRODUCTS = 1000

# Product column -> (item column, number of items after generation).
LINKS = {
    "productclassesofcompounds_product_id": ("productclassesofcompounds_class_of_compound_id", 1206),
    "producthazardstatements_product_id": ("producthazardstatements_hazard_statement_id", 113),
    "productprecautionarystatements_product_id": (
        "productprecautionarystatements_precautionary_statement_id",
        169,
    ),
    "productsupplierrefs_product_id": ("productsupplierrefs_supplier_ref_id", 1000),
    "productsymbols_product_id": ("productsymbols_symbol_id", 9),
    "productsynonyms_product_id": ("productsynonyms_name_id", 1000),
    "producttags_product_id": ("producttags_tag_id", 1054),
}


def transform_row(row, ctx):
    n = ctx["row_number"] - 1
    for product_column, (item_column, items) in LINKS.items():
        if product_column in row:
            row[product_column] = n % PRODUCTS + 1
            row[item_column] = (n % PRODUCTS + n // PRODUCTS) % items + 1
    return row
