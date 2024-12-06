// @generated automatically by Diesel CLI.

diesel::table! {
    EngineState (id) {
        id -> Text,
        position -> Integer,
        steps_per_revolution -> Integer,
    }
}

diesel::table! {
    Led (id) {
        id -> Integer,
        px -> Integer,
        py -> Integer,
        color -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    EngineState,
    Led,
);