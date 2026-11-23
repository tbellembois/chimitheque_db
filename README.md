[![Rust](https://github.com/tbellembois/chimitheque_db/actions/workflows/rust.yml/badge.svg)](https://github.com/tbellembois/chimitheque_db/actions/workflows/rust.yml) [![codecov](https://codecov.io/github/tbellembois/chimitheque_db/graph/badge.svg?token=EC9QVS6900)](https://codecov.io/github/tbellembois/chimitheque_db)

Database layer of [Chimithèque](https://github.com/tbellembois/chimitheque_back), a management application for chemical products, biological reagents and laboratory consumables.

It reads and writes the SQLite database with [sea-query](https://github.com/SeaQL/sea-query). The data is validated by [chimitheque_types](https://github.com/tbellembois/chimitheque_types) before being saved here: the database schema is the last safety net.

## SQLite regex extension

The barcode generation uses the [sqlite-regex](https://github.com/asg017/sqlite-regex) extension. `connect` loads `regex0.so` from the directory given by the `SQLITE_EXTENSION_DIR` environment variable. The tests use `src/extensions/regex0.so`.

## Resources

In `src/resources`:

- `shema.sql`: the database schema, created by `create_tables`.
- `ghscode_11.txt`: the GHS hazard and precautionary statements. `populate_db_with_base_data` loads them, with the constants of `chimitheque_defines` (units, symbols, CMR substances...), into a new database.
- `test_fixture.sql`: the test data: a few entities, people, store locations, products and storages, described at the top of the file. `test_utils::init_test_with_fixture` loads it on top of the base data, with the foreign keys on: every test starts from it.
- `migration.sql`: migrates a database of the previous Chimithèque version, attached as `./storage.db`, into a new database. Run it by hand with the `sqlite3` command line tool, in the new database.
- `migration_check.sql`: counts the rows of each table, to compare both databases after a migration.
