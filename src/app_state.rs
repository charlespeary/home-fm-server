use actix_web::{HttpRequest, HttpResponse, Json, Result, State};
use serde::ser::{SerializeStruct, Serializer};
// to avoid name shadowing it's named as MSerialize, M stands for Makro
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
	pub current_song: Arc<Mutex<String>>,
}

impl Serialize for AppState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct("AppState", 2)?;
		let current_song = self.current_song.as_ref().lock().unwrap().clone();
		state.serialize_field("current_song", &current_song)?;
		state.end()
	}
}

pub fn get_app_state(req: &HttpRequest<AppState>) -> Result<Json<AppState>> {
	Ok(Json(req.state().clone()))
}

#[derive(Serialize)]
pub struct RequestResponse {
	success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Song {
	name: String,
}

// TODO: Make state shared across all threads
pub fn set_current_song(song: Json<Song>, state: State<AppState>) -> Result<Json<RequestResponse>> {
	*(state.current_song.lock().unwrap()) = song.name.to_string();
	println!("current song: {}", state.current_song.lock().unwrap());
	Ok(Json(RequestResponse { success: true }))
}
