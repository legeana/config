// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Nullable<Integer>,
        purpose -> Integer,
        file_type -> Integer,
        path -> Binary,
        update_id -> Nullable<Integer>,
    }
}

diesel::table! {
    updates (id) {
        id -> Nullable<Integer>,
    }
}

diesel::joinable!(files -> updates (update_id));

diesel::allow_tables_to_appear_in_same_query!(files, updates,);
