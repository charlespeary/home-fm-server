use super::schema::songs;
use crate::db::DBResponse::SongExists;
use crate::song::{NewSong, Song};
use actix::{Actor, Handler, Message, SyncContext};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

fn get_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub struct DBExecutor {
    conn: SqliteConnection,
}

impl Actor for DBExecutor {
    type Context = SyncContext<Self>;
}

impl DBExecutor {
    pub fn new() -> Self {
        DBExecutor {
            conn: get_connection(),
        }
    }
}

enum DBAction {
    SaveSong { song: NewSong },
    GetRandomSong,
    CheckIfSongExists { song_name: String },
}

impl Message for DBAction {
    type Result = Result<DBResponse, DBError>;
}

enum DBResponse {
    SongExists,
    SongCreated,
    RandomSong { song: Song },
}

enum DBError {
    SongNotFound,
    SongCreationFailure,
    UnkownAction,
}

impl Handler<DBAction> for DBExecutor {
    type Result = Result<DBResponse, DBError>;
    fn handle(&mut self, msg: DBAction, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            DBAction::CheckIfSongExists { song_name } => {
                let exists = check_if_exsists(&self.conn, song_name);
                if exists {
                    Ok(DBResponse::SongExists)
                } else {
                    Err(DBError::SongNotFound)
                }
            }
            DBAction::GetRandomSong => {
                let song = get_random_song(&self.conn);
                Ok(DBResponse::RandomSong { song })
            }
            DBAction::SaveSong { song } => {
                let success = save_song(&self.conn, &song);
                if success {
                    Ok(DBResponse::SongCreated)
                } else {
                    Err(DBError::SongCreationFailure)
                }
            }
            _ => Err(DBError::UnkownAction),
        }
    }
}

fn get_random_song(conn: &SqliteConnection) -> Song {
    no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");
    songs::table
        .order(RANDOM)
        .limit(1)
        .first::<Song>(conn)
        .expect("unable to load posts")
}

fn save_song(conn: &SqliteConnection, song: &NewSong) -> bool {
    diesel::insert_into(songs::table)
        .values(song)
        .execute(conn)
        .unwrap()
        > 0
}

fn check_if_exists(conn: &SqliteConnection, song_name: String) -> bool {
    use super::schema::songs::dsl::name;
    songs::table
        .filter(name.eq(song_name))
        .first::<Song>(conn)
        .is_ok()
}
