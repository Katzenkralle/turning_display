// @generated automatically by Diesel CLI.

diesel::table! {
    LED (id) {
        id -> Nullable<Integer>,
        px -> Nullable<Integer>,
        py -> Nullable<Integer>,
        color -> Nullable<Text>,
    }
}
