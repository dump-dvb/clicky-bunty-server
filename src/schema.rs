table! {
    users (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        password -> VarChar,
        role -> SmallInt
    }

    region (id) {
        id -> Text,
        name -> Text,
        transport_company -> Text,
        frequency -> BigInt,
        protocol -> Text
    }

    station (id) {
        id -> Uuid,
        token -> Nullable<VarChar>,
        name -> Text,
        lat -> Double,
        lon -> Double,
        region -> Text,
        owner -> Uuid,
        approved -> Bool,
    }
}
