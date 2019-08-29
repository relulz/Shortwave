table! {
    library (id) {
        id -> Nullable<Integer>,
        station_id -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(library,);
