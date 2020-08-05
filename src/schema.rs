table! {
    cards (id) {
        id -> Int4,
        title -> Varchar,
        deck_id -> Int4,
        done -> Bool,
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

joinable!(cards -> decks (deck_id));

allow_tables_to_appear_in_same_query!(
    cards,
    decks,
);
