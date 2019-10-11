table! {
    users (uuid) {
        uuid -> Uuid,
        user_name -> Varchar,
        display_name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        is_active -> Bool,
        is_verified -> Bool,
    }
}
