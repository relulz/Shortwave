table! {
    library (id) {
        id -> Nullable<Integer>,
        stationuuid -> Text,
    }
}

allow_tables_to_appear_in_same_query!(library,);
