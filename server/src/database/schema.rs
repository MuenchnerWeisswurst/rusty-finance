// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (id) {
        id -> Int8,
        reservation -> Date,
        receiver -> Text,
        tags -> Array<Nullable<Text>>,
        amount -> Float8,
        currency -> Text,
    }
}
