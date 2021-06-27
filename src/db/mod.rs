mod models;
pub mod schema;
use diesel::dsl::sql;
pub use models::*;

use diesel::prelude::*;
use diesel::r2d2::{Builder, ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

use tui::style::Color;

lazy_static::lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<SqliteConnection>> = {
        dotenv().ok();
        let db_file = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Builder::new()
            .max_size(5)
            .build(ConnectionManager::new(db_file))
            .expect("Failed to connect database")
    };
}

fn conn() -> PooledConnection<ConnectionManager<SqliteConnection>> {
    DB_POOL.get().expect("Failed to get connection")
}

pub struct DbUtils;

impl DbUtils {
    pub fn create_new_inventory(name: &str, color: i32) -> Inventory {
        use schema::inventory::dsl;
        let conn = conn();
        conn.transaction(|| {
            diesel::insert_into(dsl::inventory)
                .values((dsl::name.eq(name), dsl::color.eq(color)))
                .execute(&conn)
                .unwrap();
            dsl::inventory
                .find(sql("last_insert_rowid()"))
                .get_result(&conn)
        })
        .unwrap()
    }

    pub fn delete_inventory(id: i32) {
        use schema::inventory::dsl::inventory;
        use schema::tasks::dsl::{inventory_id as task_inv_id, tasks};
        use schema::tomatos::dsl::{inventory_id as tomatos_inv_id, tomatos};
        let conn = conn();
        diesel::delete(inventory.find(id)).execute(&conn).unwrap();
        diesel::delete(tasks.filter(task_inv_id.eq(id)))
            .execute(&conn)
            .unwrap();
        diesel::delete(tomatos.filter(tomatos_inv_id.eq(id)))
            .execute(&conn)
            .unwrap();
    }

    pub fn create_new_task(inventory_id: i32, name: &str, notes: Option<&str>) -> Task {
        let conn = conn();
        let create_at = chrono::Local::now().timestamp();
        use schema::tasks::dsl;
        conn.transaction(|| {
            diesel::insert_into(dsl::tasks)
                .values(TaskRow {
                    inventory_id,
                    name,
                    spent_minutes: 0,
                    create_at,
                    notes,
                })
                .execute(&conn)
                .unwrap();
            dsl::tasks
                .find(sql("last_insert_rowid()"))
                .get_result(&conn)
        })
        .unwrap()
    }

    pub fn delete_task(id: i32) {
        use schema::tasks::dsl::tasks;
        use schema::tomatos::dsl::{task_id, tomatos};
        let conn = conn();
        diesel::delete(tasks.find(id)).execute(&conn).unwrap();
        diesel::delete(tomatos.filter(task_id.eq(id)))
            .execute(&conn)
            .unwrap();
    }

    pub fn create_new_tomato(tomato: Tomato) {
        let conn = conn();
        diesel::insert_into(schema::tomatos::table)
            .values(tomato)
            .execute(&conn)
            .unwrap();
    }

    pub fn update_task_spent(task_id: i32, delta_spent: i64) {
        let conn = conn();
        use schema::tasks::dsl::*;
        diesel::update(tasks.find(task_id))
            .set(spent_minutes.eq(spent_minutes + delta_spent))
            .execute(&conn)
            .unwrap();
    }

    pub fn all_inventory() -> Vec<Inventory> {
        let conn = conn();
        schema::inventory::dsl::inventory
            .load::<Inventory>(&conn)
            .unwrap()
    }

    pub fn all_task_groupby(invs: &[Inventory]) -> Vec<Vec<Task>> {
        let conn = conn();
        Task::belonging_to(invs)
            .get_results::<Task>(&conn)
            .unwrap()
            .grouped_by(invs)
    }
}

const NORMAL: i32 = 0;
const INDEX: i32 = 1;
const RGB: i32 = 2;

#[derive(Default, Clone, Copy)]
pub struct DbColor(i32);

impl From<i32> for DbColor {
    fn from(i: i32) -> Self {
        DbColor(i)
    }
}

impl From<DbColor> for i32 {
    fn from(c: DbColor) -> Self {
        c.0
    }
}

impl From<DbColor> for Color {
    fn from(c: DbColor) -> Self {
        match c.0 >> 24 {
            NORMAL => match c.0 {
                0 => Color::Reset,
                1 => Color::Black,
                2 => Color::Red,
                3 => Color::Green,
                4 => Color::Yellow,
                5 => Color::Blue,
                6 => Color::Magenta,
                7 => Color::Cyan,
                8 => Color::Gray,
                9 => Color::DarkGray,
                10 => Color::LightRed,
                11 => Color::LightGreen,
                12 => Color::LightYellow,
                13 => Color::LightBlue,
                14 => Color::LightMagenta,
                15 => Color::LightCyan,
                16 => Color::White,
                _ => Color::Reset,
            },
            INDEX => Color::Indexed(c.0 as u8),
            RGB => Color::Rgb((c.0 >> 16) as u8, (c.0 >> 8) as u8, c.0 as u8),
            _ => Color::Reset,
        }
    }
}

impl From<Color> for DbColor {
    fn from(c: Color) -> Self {
        DbColor(match c {
            Color::Reset => 0,
            Color::Black => 1,
            Color::Red => 2,
            Color::Green => 3,
            Color::Yellow => 4,
            Color::Blue => 5,
            Color::Magenta => 6,
            Color::Cyan => 7,
            Color::Gray => 8,
            Color::DarkGray => 9,
            Color::LightRed => 10,
            Color::LightGreen => 11,
            Color::LightYellow => 12,
            Color::LightBlue => 13,
            Color::LightMagenta => 14,
            Color::LightCyan => 15,
            Color::White => 16,
            Color::Indexed(c) => INDEX << 24 | c as i32,
            Color::Rgb(r, g, b) => RGB << 24 | (r as i32) << 16 | (g as i32) << 8 | b as i32,
        })
    }
}
