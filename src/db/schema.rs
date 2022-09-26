// @generated automatically by Diesel CLI.

diesel::table! {
    cosigner (id) {
        id -> Integer,
        uuid -> Text,
        type_ -> SmallInt,
        email_address -> Text,
        public_key -> Text,
        creation_time -> Timestamp,
    }
}

diesel::table! {
    psbt (id) {
        id -> Integer,
        uuid -> Text,
        data -> Text,
        creation_time -> Timestamp,
        cosigner_id -> Integer,
        wallet_id -> Integer,
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
        required_signatures -> SmallInt,
        creation_time -> Timestamp,
    }
}

diesel::table! {
    xprv (id) {
        id -> Integer,
        uuid -> Text,
        fingerprint -> Text,
        mnemonic -> Text,
        data -> Text,
        creation_time -> Timestamp,
        cosigner_id -> Integer,
        wallet_id -> Integer,
    }
}

diesel::table! {
    xpub (id) {
        id -> Integer,
        uuid -> Text,
        derivation_path -> Text,
        fingerprint -> Text,
        data -> Text,
        creation_time -> Timestamp,
        cosigner_id -> Integer,
        wallet_id -> Integer,
    }
}

diesel::joinable!(psbt -> cosigner (cosigner_id));
diesel::joinable!(psbt -> wallet (wallet_id));
diesel::joinable!(xprv -> cosigner (cosigner_id));
diesel::joinable!(xprv -> wallet (wallet_id));
diesel::joinable!(xpub -> cosigner (cosigner_id));
diesel::joinable!(xpub -> wallet (wallet_id));

diesel::allow_tables_to_appear_in_same_query!(cosigner, psbt, wallet, xprv, xpub,);
