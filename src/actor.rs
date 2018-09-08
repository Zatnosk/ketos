extern crate openssl;

use openssl::rsa::Rsa;

use std::str;
use std::path::Path;
use std::fs;
use std::fs::File;
use serde_json;
use serde_json::Value;
use error::{Error, ErrorKind};


pub struct Actor {
	name: String,
	key_pair: Rsa<openssl::pkey::Private>
}

impl Actor {
	pub fn new(name: &str) -> Result<Actor, Error> {
		Ok(Actor{name:name.to_string(), key_pair:Rsa::generate(2048)?})
	}

	fn public_pem(&self) -> Result<String, Error> {
		match self.key_pair.public_key_to_pem() {
			Ok(pem) => match str::from_utf8(pem.as_slice()) {
				Ok(pem_str) => Ok(pem_str.to_string()),
				Err(_) => Err(Error::empty(ErrorKind::Security))
			},
			Err(_) => Err(Error::empty(ErrorKind::Security))
		}
	}

	fn private_pem(&self) -> Result<String, Error> {
		match self.key_pair.private_key_to_pem() {
			Ok(pem) => match str::from_utf8(pem.as_slice()) {
				Ok(pem_str) => Ok(pem_str.to_string()),
				Err(_) => Err(Error::empty(ErrorKind::Security))
			},
			Err(_) => Err(Error::empty(ErrorKind::Security))
		}
	}

	pub fn json(&self) -> Result<String, Error> {
		Ok(serde_json::to_string_pretty(&json!({
			"@context": [
				"https://www.w3.org/ns/activitystreams",
				"https://w3id.org/security/v1"
			],
			"id": format!("https://ap.zatnosk.dk/{}",self.name),
			"type": "Person",
			"preferredUsername": self.name,
			"inbox": "https://ap.zatnosk.dk/inbox",

			"publicKey": {
				"id": format!("https://ap.zatnosk.dk/{}#main-key", self.name),
				"owner": format!("https://ap.zatnosk.dk/{}", self.name),
				"publicKeyPem": self.public_pem()?
			}
		}))?)
	}

	pub fn webfinger(&self) -> Result<String, Error> {
		Ok(serde_json::to_string_pretty(&json!({
			"subject": format!("acct:{}@ap.zatnosk.dk", self.name),
			"links":[
				{
					"rel": "self",
					"type": "application/activitypub+json",
					"href": format!("https://ap.zatnosk.dk/{}", self.name)
				}
			]
		}))?)
	}

	pub fn store_as_file(&self) -> Result<(), Error>{
		let path = Path::new("./data/actors");
		if !path.exists() {
			fs::create_dir_all(path)?;
		}
		let actor_path = path.join(format!("{}.json",self.name));
		let json = serde_json::to_string_pretty(&json!({
			"name": self.name,
			"privateKeyPem": self.private_pem()?
		}))?;
		fs::write(actor_path.as_path(), json)?;
		Ok(())
	}

	pub fn load_from_file(name: &str) -> Result<Actor, Error> {
		let path = format!("./data/actors/{}.json",name);
		let file = File::open(Path::new(&path))?;
		let json: Value = serde_json::from_reader(file)?;
		let name = match json.get("name") {
			Some(value) => match value.as_str(){
				Some(value) => value.to_string(),
				None => return Err(Error::empty(ErrorKind::InvalidJson))
			},
			None => return Err(Error::empty(ErrorKind::InvalidJson))
		};
		let private_key_pem = match json.get("privateKeyPem"){
			Some(value) => match value.as_str(){
				Some(value) => value,
				None => return Err(Error::empty(ErrorKind::InvalidJson))
			},
			None => return Err(Error::empty(ErrorKind::InvalidJson))
		};
		Ok(Actor{
			name: name,
			key_pair: Rsa::private_key_from_pem(private_key_pem.as_bytes())?
		})
	}
}
