// @generated automatically by Diesel CLI.

diesel::table! {
    ApplicationState (id) {
        id -> Integer,
        active_preset -> Integer,
        current_engine_pos -> Integer,
        engine_steps_per_rotation -> Integer,
        delay_micros -> Integer,
    }
}

diesel::table! {
    Engine (id) {
        id -> Integer,
        position -> Integer,
        is_target -> Bool,
        associated_preset -> Nullable<Integer>,
    }
}

diesel::table! {
    Led (id) {
        id -> Integer,
        color -> Text,
        brightness -> Integer,
        mode -> Text,
        associated_preset -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    ApplicationState,
    Engine,
    Led,
);