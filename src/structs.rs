use clap::Parser;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::content::RawText;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Parser, Clone, Debug)]
#[command(author, version)]
pub struct Args {
	/// Directory where binary files are contained
	#[arg(long, default_value = "bin")]
	pub dir: PathBuf,

	/// Refresh time (in secs)
	#[arg(long, default_value = "300")]
	pub refresh: u64,

	/// External address (url)
	#[arg(long, default_value = "http://127.0.0.1:8000")]
	pub url: String,

	/// Address to listen on
	#[arg(short, long, default_value = "127.0.0.1")]
	pub address: IpAddr,

	/// Port to listen on
	#[arg(short, long, default_value = "8000")]
	pub port: u16,
}

#[derive(Debug)]
pub struct Platform {
	pub system: String,
	pub arch: String,
}

#[derive(Debug)]
pub struct Bin {
	pub name: String,
	pub platforms: Vec<Platform>,
}

#[derive(Responder)]
pub enum ScriptResponse {
	Status(Status),
	Text(RawText<String>),
}

#[derive(Responder)]
pub enum BinaryResponse {
	Status(Status),
	Bin(NamedFile),
}
