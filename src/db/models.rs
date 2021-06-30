use super::schema::{inventory, tasks, tomatos};
use super::DbColor;

#[derive(Insertable)]
pub struct Tomato {
    pub inventory_id: i32,
    pub task_id: i32,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Queryable, Identifiable, Default)]
#[table_name = "inventory"]
pub struct Inventory {
    pub id: i32,
    pub name: String,
    #[diesel(deserialize_as = "i32")]
    pub color: DbColor,
}

#[derive(Default)]
pub struct NewInventory {
    pub name: String,
    pub color: i32,
}

#[derive(Default, AsChangeset)]
#[table_name = "inventory"]
pub struct EditInventory {
    pub id: i32,
    pub name: Option<String>,
    pub color: Option<i32>,
}

#[derive(Default, AsChangeset)]
#[table_name = "tasks"]
pub struct EditTask {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Queryable, Identifiable, Associations, Default)]
#[belongs_to(Inventory)]
pub struct Task {
    pub id: i32,
    pub inventory_id: i32,
    pub name: String,
    pub spent_minutes: i64,
    pub create_at: i64,
    pub notes: Option<String>,
}

#[derive(Insertable)]
#[table_name = "tasks"]
pub struct TaskRow<'a> {
    pub inventory_id: i32,
    pub name: &'a str,
    pub spent_minutes: i64,
    pub create_at: i64,
    pub notes: Option<&'a str>,
}

#[derive(Default)]
pub struct NewTask {
    pub inventory_id: i32,
    pub name: String,
    pub notes: Option<String>,
}
