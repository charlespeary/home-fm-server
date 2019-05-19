use super::schema::songs;
use crate::song::{NewSong, Song};
use actix::{Actor, Context, Handler, Message};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use diesel::result::Error as DieselError;
use diesel::sqlite::SqliteConnection;

pub type Conn = SqliteConnection;
pub type SqlPool = Pool<ConnectionManager<Conn>>;
pub type PooledConn = PooledConnection<ConnectionManager<Conn>>;

/// Create new connection pool to the database.
pub fn new_pool<S: Into<String>>(database_url: S) -> Result<SqlPool, ()> {
    let manager = ConnectionManager::<Conn>::new(database_url.into());
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to the db");
    Ok(pool)
}

/// Struct holding connection to the database.
pub struct DBExecutor {
    conn: SqlPool,
}

impl Actor for DBExecutor {
    type Context = Context<Self>;
}

impl DBExecutor {
    pub fn new(conn: SqlPool) -> Self {
        DBExecutor { conn }
    }

    fn get_conn(&mut self) -> PooledConn {
        self.conn.get().unwrap()
    }
}

/// Get random song from db with nsfw set to false.
pub struct GetRandomSong;
impl Message for GetRandomSong {
    type Result = Result<Song, DieselError>;
}

impl Handler<GetRandomSong> for DBExecutor {
    type Result = Result<Song, DieselError>;

    fn handle(&mut self, msg: GetRandomSong, ctx: &mut Self::Context) -> Self::Result {
        get_random_song(&self.get_conn())
    }
}

/// Check if song is already saved in the database.
/// If song exists then it is returned, otherwise return DieselError.
pub struct CheckSongExistence {
    pub song_name: String,
    pub artists: String,
}

impl Message for CheckSongExistence {
    type Result = Result<Song, DieselError>;
}

impl Handler<CheckSongExistence> for DBExecutor {
    type Result = Result<Song, DieselError>;
    fn handle(&mut self, msg: CheckSongExistence, ctx: &mut Self::Context) -> Self::Result {
        get_song(&self.get_conn(), msg.song_name, msg.artists)
    }
}

/// Save new song in database.
pub struct SaveSong {
    pub song: NewSong,
}

impl Message for SaveSong {
    type Result = Result<Song, DieselError>;
}

impl Handler<SaveSong> for DBExecutor {
    type Result = Result<Song, DieselError>;

    fn handle(&mut self, msg: SaveSong, ctx: &mut Self::Context) -> Self::Result {
        save_song(&self.get_conn(), &msg.song)
    }
}

/// Get all of the available songs from the database.
pub struct GetAllSongs;
impl Message for GetAllSongs {
    type Result = Result<Vec<Song>, DieselError>;
}

impl Handler<GetAllSongs> for DBExecutor {
    type Result = Result<Vec<Song>, DieselError>;

    fn handle(&mut self, msg: GetAllSongs, ctx: &mut Self::Context) -> Self::Result {
        get_all_songs(&self.get_conn())
    }
}

/// Toggle nsfw of song with given id.
pub struct ToggleSongNsfw {
    pub id: i32,
    pub is_nsfw: bool,
}

impl Message for ToggleSongNsfw {
    type Result = Result<Song, DieselError>;
}

impl Handler<ToggleSongNsfw> for DBExecutor {
    type Result = Result<Song, DieselError>;

    fn handle(&mut self, msg: ToggleSongNsfw, ctx: &mut Self::Context) -> Self::Result {
        toggle_song_nsfw(&self.get_conn(), msg.id, msg.is_nsfw)
    }
}

/// Delete song with given id.
pub struct DeleteSong {
    pub song_id: i32,
}

impl Message for DeleteSong {
    type Result = Result<Song, DieselError>;
}

impl Handler<DeleteSong> for DBExecutor {
    type Result = Result<Song, DieselError>;

    fn handle(&mut self, msg: DeleteSong, ctx: &mut Self::Context) -> Self::Result {
        delete_song(&self.get_conn(), msg.song_id)
    }
}
/// Returns random song from db with nsfw set to false.
fn get_random_song(conn: &PooledConn) -> Result<Song, DieselError> {
    use super::schema::songs::dsl::nsfw;

    no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");
    songs::table
        .filter(nsfw.eq(false))
        .order(RANDOM)
        .limit(1)
        .first::<Song>(conn)
}

/// Saves song in database.
fn save_song(conn: &PooledConn, song: &NewSong) -> Result<Song, DieselError> {
    diesel::insert_into(songs::table).values(song).execute(conn);
    get_song(conn, song.name.clone(), song.artists.clone())
}

/// Returns song with given id from database.
fn get_song(
    conn: &PooledConn,
    song_name: String,
    song_artists: String,
) -> Result<Song, DieselError> {
    use super::schema::songs::dsl::{artists, name};
    songs::table
        .filter(name.eq(song_name).and(artists.eq(song_artists)))
        .first::<Song>(conn)
}

/// Returns all available songs from database.
fn get_all_songs(conn: &PooledConn) -> Result<Vec<Song>, DieselError> {
    songs::table.load::<Song>(conn)
}

/// Toggles song's nsfw.
fn toggle_song_nsfw(conn: &PooledConn, song_id: i32, is_nsfw: bool) -> Result<Song, DieselError> {
    use super::schema::songs::dsl::{id, nsfw};
    diesel::update(songs::table.filter(id.eq(song_id)))
        .set(nsfw.eq(is_nsfw))
        .execute(conn);
    songs::table
        .filter(id.eq(song_id))
        .limit(1)
        .first::<Song>(conn)
}

/// Deletes song from database.
fn delete_song(conn: &PooledConn, song_id: i32) -> Result<Song, DieselError> {
    use super::schema::songs::dsl::id;
    let song = songs::table
        .filter(id.eq(song_id))
        .limit(1)
        .first::<Song>(conn)?;
    diesel::delete(songs::table.filter(id.eq(song_id))).execute(conn);
    std::fs::remove_file(&song.path);
    Ok(song)
}
