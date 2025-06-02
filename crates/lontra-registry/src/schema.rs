// @generated automatically by Diesel CLI.
// Fixed in accordance with <https://sqlite.org/stricttables.html>.
// See <https://github.com/diesel-rs/diesel/discussions/4628>.

diesel::table! {
    files (id) {
        // Removed Nullable due to STRICT.
        id -> BigInt,
        purpose -> Integer,
        file_type -> Integer,
        path -> Binary,
        // Use BigInt.
        update_id -> Nullable<BigInt>,
    }
}

diesel::table! {
    updates (id) {
        // Removed Nullable due to STRICT.
        id -> BigInt,
    }
}

diesel::joinable!(files -> updates (update_id));

diesel::allow_tables_to_appear_in_same_query!(files, updates,);
