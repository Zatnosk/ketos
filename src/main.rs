#[macro_use]
extern crate warp;
extern crate openssl;
#[macro_use]
extern crate serde_json;

mod error;
mod actor;

use actor::Actor;

use warp::{Filter};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

fn webfinger(actors_ref: &Arc<Mutex<HashMap<String,Actor>>>) -> impl Fn(Vec<(String, String)>) -> Result<String, warp::reject::Rejection> + Clone {
	let actors_ref = Arc::clone(actors_ref);
	move |query: Vec<(String, String)>|{
		let actors = match actors_ref.lock(){
			Ok(actors) => actors,
			Err(poisoned) => poisoned.into_inner()
		};
		let mut actor = None;
		for (key,value) in query {
			if key == "resource" {
				let v : Vec<&str>= value.split(|c|c==':'||c=='@').collect();
				if v.len() == 3 && v[0] == "acct" && v[2]=="ap.zatnosk.dk" {
					actor = actors.get(v[1]);
				}
			}
		}
		match actor {
			Some(actor) => match actor.webfinger(){
				Ok(webfinger) => Ok(webfinger),
				Err(_) => Err(warp::reject())
			},
			None => Err(warp::reject())
		}
	}
}

fn actor_endpoint(actors_ref: &Arc<Mutex<HashMap<String,Actor>>>) -> impl Fn(String) -> Result<String, warp::reject::Rejection> + Clone {
	let actors_ref = Arc::clone(actors_ref);
	move |name: String| {
		let actors = match actors_ref.lock(){
			Ok(actors) => actors,
			Err(poisoned) => poisoned.into_inner()
		};
		match actors.get(&name){
			Some(actor) => match actor.json(){
				Ok(json) => Ok(json),
				Err(_) => Err(warp::reject())
			},
			None => Err(warp::reject())
		}
	}
}

fn main() {
	let actors = Arc::new(Mutex::new(HashMap::new()));
	let zatnosk = match Actor::load_from_file("zatnosk") {
		Ok(actor) => actor,
		Err(error) => {
			println!("Failed loading actor: {:?}", error);
			let actor = Actor::new("zatnosk").unwrap();
			actor.store_as_file().unwrap();
			actor
		}
	};
	actors.lock().unwrap().insert(String::from("zatnosk"), zatnosk);

	let webfinger = path!(".well-known"/"webfinger").and(warp::query::query()).and_then(webfinger(&actors));
	let actor_path = path!(String).and_then(actor_endpoint(&actors));

	let routes = warp::get2().and(
		webfinger
		.or(actor_path)
	);

	warp::serve(routes).run(([127,0,0,1],3000))
}
