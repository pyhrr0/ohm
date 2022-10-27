// @generated automatically by Diesel CLI.

diesel::table! {
    cosigner (id) {
        id -> Integer,
        uuid -> Text,
        #[sql_name = "type"]
        type_ -> SmallInt,
        email_address -> Nullable<Text>,
        xpub -> Text,
        xprv -> Nullable<Text>,
        creation_time -> Timestamp,
        wallet_uuid -> Nullable<Text>,
    }
}

diesel::table! {
    wallet (id) {
        id -> Integer,
        uuid -> Text,
        address_type -> SmallInt,
        network -> SmallInt,
        receive_descriptor -> Text,
        receive_address_index -> BigInt,
        receive_address -> Text,
        change_descriptor -> Text,
        change_address_index -> BigInt,
        change_address -> Text,
        balance -> Text,
        required_signatures -> SmallInt,
        creation_time -> Timestamp,
    }
}

diesel::table! {
    psbt (id) {
        id -> Integer,
        uuid -> Text,
        base64 -> Text,
        creation_time -> Timestamp,
        wallet_uuid -> Text,
    }
}
