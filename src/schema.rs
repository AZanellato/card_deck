table! {
    cards (id) {
        id -> Int4,
        title -> Varchar,
        deck_id -> Int4,
        finished_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    decks (id) {
        id -> Int4,
        title -> Varchar,
        created_by -> Int4,
    }
}

table! {
    user_tokens (id) {
        id -> Int4,
        token -> Text,
        user_id -> Int4,
        service -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Text,
        name -> Text,
        hash_password -> Text,
    }
}

joinable!(cards -> decks (deck_id));
joinable!(user_tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(cards, decks, user_tokens, users,);
