table! {
    items (id) {
        id -> Int4,
        title -> Text,
        body -> Text,
        done -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
    }
}

table! {
    votes (user_id, item_id) {
        user_id -> Int4,
        item_id -> Int4,
        ordinal -> Int4,
    }
}

joinable!(votes -> items (item_id));
joinable!(votes -> users (user_id));

allow_tables_to_appear_in_same_query!(
    items,
    users,
    votes,
);
