#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::entity::entity_references;
    use crate::person::person_references;
    use crate::product::product_references;
    use crate::reference::{Reference, find_missing_reference};
    use crate::storage::storage_references;
    use crate::storelocation::store_location_references;
    use chimitheque_types::{
        category::Category, entity::Entity, permission::Permission, person::Person,
        producer::Producer, producerref::ProducerRef, product::Product, storage::Storage,
        storelocation::StoreLocation, symbol::Symbol, tag::Tag,
    };

    #[test]
    fn test_find_missing_reference() {
        let db_connection = crate::test_utils::init_test();
        db_connection
            .execute(
                "INSERT INTO category (category_id, category_label) VALUES (1, 'solvents')",
                [],
            )
            .unwrap();

        let existing = Reference::new("category", "category", Some(1));
        assert_eq!(find_missing_reference(&db_connection, &[]).unwrap(), None);
        assert_eq!(
            find_missing_reference(&db_connection, &[existing]).unwrap(),
            None
        );
        assert_eq!(
            find_missing_reference(
                &db_connection,
                &[
                    Reference::new("category", "category", Some(1)),
                    Reference::new("category", "category", Some(999)),
                ]
            )
            .unwrap(),
            Some("unknown category id 999".to_string())
        );
        assert_eq!(
            find_missing_reference(&db_connection, &[Reference::new("symbols", "symbol", None)])
                .unwrap(),
            Some("symbols has no id".to_string())
        );
    }

    #[test]
    fn test_product_references() {
        let product = Product {
            category: Some(Category {
                category_id: Some(3),
                ..Default::default()
            }),
            // A new tag is created: not listed.
            tags: Some(vec![Tag::default()]),
            // A new producer reference needs an existing producer.
            producer_ref: Some(ProducerRef {
                producer: Producer {
                    producer_id: Some(2),
                    ..Default::default()
                },
                ..Default::default()
            }),
            // A symbol is never created: listed without an id.
            symbols: Some(vec![Symbol::default()]),
            ..Default::default()
        };

        assert_eq!(
            product_references(&product),
            [
                Reference::new("category", "category", Some(3)),
                Reference::new("producer", "producer", Some(2)),
                Reference::new("symbols", "symbol", None),
            ]
        );
    }

    #[test]
    fn test_storage_references() {
        let mut storage = Storage::default();
        storage.product.product_id = Some(1);
        storage.store_location.store_location_id = Some(2);

        assert_eq!(
            storage_references(&storage),
            [
                Reference::new("product", "product", Some(1)),
                Reference::new("store_location", "store_location", Some(2)),
            ]
        );
    }

    #[test]
    fn test_store_location_references() {
        // A new store location needs an entity.
        assert_eq!(
            store_location_references(&StoreLocation::default()),
            [Reference::new("entity", "entity", None)]
        );

        // An updated one keeps its own.
        let store_location = StoreLocation {
            store_location_id: Some(4),
            ..Default::default()
        };
        assert_eq!(
            store_location_references(&store_location),
            [Reference::new("store_location", "store_location", Some(4))]
        );
    }

    #[test]
    fn test_person_references() {
        let person = Person {
            entities: Some(vec![Entity {
                entity_id: Some(1),
                ..Default::default()
            }]),
            managed_entities: Some(vec![Entity::default()]),
            // No entity: a permission on all entities, not listed.
            permissions: Some(vec![Permission::default()]),
            ..Default::default()
        };

        assert_eq!(
            person_references(&person),
            [
                Reference::new("entities", "entity", Some(1)),
                Reference::new("managed_entities", "entity", None),
            ]
        );
    }

    #[test]
    fn test_entity_references() {
        let entity = Entity {
            entity_id: Some(1),
            managers: Some(vec![Person {
                person_id: Some(2),
                ..Default::default()
            }]),
            ..Default::default()
        };

        assert_eq!(
            entity_references(&entity),
            [
                Reference::new("entity", "entity", Some(1)),
                Reference::new("managers", "person", Some(2)),
            ]
        );
    }
}
