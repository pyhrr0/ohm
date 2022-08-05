table! {
    cosigner (id) {
        id -> Nullable<Integer>,
        uuid -> Text,
        cosigner_type -> Nullable<SmallInt>,
        email -> Nullable<Text>,
        wallet_id -> Nullable<Integer>,
    }
}

table! {
    psbt (id) {
        id -> Nullable<Integer>,
        uuid -> Text,
        data -> Text,
        cosigner_id -> Nullable<Integer>,
        wallet_id -> Nullable<Integer>,
    }
}

table! {
    wallet (id) {
        id -> Nullable<Integer>,
        uuid -> Text,
        address_type -> SmallInt,
        receive_descriptor -> Text,
        receive_address_index -> Integer,
        receive_address -> Text,
        change_descriptor -> Text,
        change_address_index -> Integer,
        change_address -> Text,
        required_signatures -> Integer,
        creation_time -> Timestamp,
    }
}

table! {
    xprv (id) {
        id -> Nullable<Integer>,
        uuid -> Nullable<Text>,
        fingerprint -> Nullable<Text>,
        mnemonic -> Nullable<Text>,
        data -> Nullable<Text>,
        cosigner_id -> Nullable<Integer>,
        wallet_id -> Nullable<Integer>,
    }
}

table! {
    xpub (id) {
        id -> Nullable<Integer>,
        uuid -> Nullable<Text>,
        derivation_path -> Nullable<Text>,
        fingerprint -> Nullable<Text>,
        data -> Text,
        cosigner_id -> Nullable<Integer>,
        wallet_id -> Nullable<Integer>,
    }
}

joinable!(cosigner -> wallet (wallet_id));
joinable!(psbt -> cosigner (cosigner_id));
joinable!(psbt -> wallet (wallet_id));
joinable!(xprv -> cosigner (cosigner_id));
joinable!(xprv -> wallet (wallet_id));
joinable!(xpub -> cosigner (cosigner_id));
joinable!(xpub -> wallet (wallet_id));

allow_tables_to_appear_in_same_query!(cosigner, psbt, wallet, xprv, xpub,);
