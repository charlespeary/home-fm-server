use diesel::prelude::*;
use diesel::sqlite::{SqliteConnection, Sqlite};
use actix::{Actor, SyncContext};
use std::env;
use dotenv::dotenv;


fn get_connection() -> SqliteConnection{
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
}

pub struct DBExecutor{
    con : SqliteConnection
}

impl Actor for DBExecutor{
    type Context = SyncContext<Self>;
}


impl DBExecutor{
    pub fn new() -> Self{
        DBExecutor{
            con:get_connection()
        }
    }
}