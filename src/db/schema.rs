table! {
    inventory (id) {
        id -> Integer,
        name -> Text,
        color -> Integer,
    }
}

table! {
    tasks (id) {
        id -> Integer,
        inventory_id -> Integer,
        name -> Text,
        spent_minutes -> BigInt,
        create_at -> BigInt,
        notes -> Nullable<Text>,
    }
}

table! {
    tomatos (id) {
        id -> Integer,
        inventory_id -> Integer,
        task_id -> Integer,
        start_time -> BigInt,
        end_time -> BigInt,
    }
}

allow_tables_to_appear_in_same_query!(inventory, tasks, tomatos,);
