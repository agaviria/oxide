table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        display_name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        is_active -> Bool,
        is_verified -> Bool,
    }
}
