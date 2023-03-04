#![feature(proc_macro_hygiene, decl_macro)]

use rocket::http::Status;
use rocket_contrib::json::Json;
use rusqlite::Connection;
use serde::Serialize;
#[macro_use]
extern crate rocket;
#[derive(Serialize)]
struct TodoList {
    items: Vec<TodoItem>,
}
#[derive(Serialize)]
struct TodoItem {
    id: i64,
    item: String,
}
#[derive(Serialize)]
struct StatusMessage {
    message: String,
}
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
#[get("/todo")]
fn fetch_all_todo() -> Result<Json<TodoList>, Status> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_err) => {
            return Err(Status {
                code: 500,
                reason: "Error Connecting to database",
            });
        }
    };

    let mut statement = match db_connection.prepare("select id, item from todo_list;") {
        Ok(st) => st,
        Err(_) => {
            return Err(Status {
                code: 500,
                reason: "Error preparing database",
            });
        }
    };
    let results = statement.query_map([], |row| {
        Ok(TodoItem {
            id: row.get(0)?,
            item: row.get(1)?,
        })
    });
    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<TodoItem>> = rows.collect();

            match collection {
                Ok(items) => Ok(Json(TodoList { items })),
                Err(_err) => {
                    return Err(Status {
                        code: 500,
                        reason: "Error fetching todo items",
                    });
                }
            }
        }
        Err(_err) => {
            return Err(Status {
                code: 500,
                reason: "Error fetching todo items",
            });
        }
    }
}

#[post("/todo", format = "json", data = "<item>")]
fn create_todo(item: Json<String>) -> Result<Json<StatusMessage>, Status> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_err) => {
            return Err(Status {
                code: 500,
                reason: "Error Connecting to database",
            });
        }
    };

    let mut statement =
        match db_connection.prepare("insert into todo_list (id,item) values (null,$1);") {
            Ok(st) => st,
            Err(_) => {
                return Err(Status {
                    code: 500,
                    reason: "Error preparing database",
                });
            }
        };
    let results = statement.execute(&[&item.0]);
    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage {
            message: format!("{} rows inserted", rows_affected),
        })),
        Err(_) => {
            return Err(Status {
                code: 500,
                reason: "Error inserting",
            });
        }
    }
}

#[delete("/todo/<id>", format = "json")]
fn delete_todo(id: i64) -> Result<Json<StatusMessage>, Status> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_err) => {
            return Err(Status {
                code: 500,
                reason: "Error Connecting to database",
            });
        }
    };

    let mut statement = match db_connection.prepare("delete from todo_list where id = $1;") {
        Ok(st) => st,
        Err(_) => {
            return Err(Status {
                code: 500,
                reason: "Error preparing database",
            });
        }
    };
    let results = statement.execute(&[&id]);
    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage {
            message: format!("{} row deleted", rows_affected),
        })),
        Err(_) => {
            return Err(Status {
                code: 500,
                reason: "Error deleting",
            });
        }
    }
}

fn main() {
    {
        // Connect to database
        match Connection::open("data.sqlite") {
            Ok(conn) => {
                println!("Connected to database");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS users (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  email           TEXT NOT NULL,
                  password        TEXT NOT NULL
                  )",
                    [],
                )
                .unwrap();
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS todo_list (
              id              INTEGER PRIMARY KEY,
              item            VARCHAR(64) NOT NULL
            
              )",
                    [],
                )
                .unwrap();
            }
            Err(e) => {
                println!("Error connecting to database: {}", e);
            }
        }
    }
    rocket::ignite()
        .mount(
            "/",
            routes![index, fetch_all_todo, create_todo, delete_todo],
        )
        .launch();
}
