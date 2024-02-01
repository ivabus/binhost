// SPDX-License-Identifier: MIT

#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::time::Instant;

use clap::Parser;
use ed25519_compact::{KeyPair, Noise};
use rocket::figment::Figment;
use rocket::fs::{FileServer, NamedFile};
use rocket::http::Status;
use rocket::response::content::RawText;
use sha2::digest::FixedOutput;
use sha2::Digest;

use structs::*;

mod structs;

static mut BINS: Option<(HashMap<String, Bin>, Instant)> = None;
static mut MANIFEST: Option<Vec<u8>> = None;
static mut KEYPAIR: Option<ed25519_compact::KeyPair> = None;
static WEB_SH: &str = include_str!("../web.sh");

static HASH_CALCULATION_SH: &str = "";

fn reload_bins(bins: (&mut HashMap<String, Bin>, &mut Instant), args: &Args) {
	if (Instant::now() - *bins.1).as_secs() > args.refresh {
		*bins.0 = get_bins(args);
		*bins.1 = Instant::now();
	}
}

fn get_bins(args: &Args) -> HashMap<String, Bin> {
	let mut bins: HashMap<String, Bin> = HashMap::new();
	std::fs::read_dir(&args.dir).unwrap().for_each(|entry| {
		let en = entry.unwrap();
		if en.path().is_dir() {
			let mut bin: Bin = Bin {
				name: en.file_name().into_string().unwrap(),
				platforms: vec![],
			};
			std::fs::read_dir(en.path()).unwrap().for_each(|platform| {
				let plat = platform.unwrap();
				std::fs::read_dir(plat.path()).unwrap().for_each(|arch| {
					let ar = arch.unwrap();
					bin.platforms.push(Platform {
						system: plat.file_name().into_string().unwrap(),
						arch: ar.file_name().into_string().unwrap(),
					});
				});
			});
			bins.insert(bin.name.clone(), bin);
		}
	});
	bins
}

fn format_platform_list(bin: &Bin) -> String {
	let mut s = String::new();
	for i in &bin.platforms {
		s.push_str(&format!("{}-{}|", i.system, i.arch))
	}
	s.pop().unwrap();
	s
}

#[get("/")]
async fn index() -> RawText<String> {
	let args = Args::parse();
	let mut ret = String::new();
	unsafe {
		if let Some((bins, time)) = &mut BINS {
			reload_bins((bins, time), &args);
			if bins.is_empty() {
				return RawText(String::from("No binaries found"));
			}
			for (name, bin) in bins {
				ret.push_str(&format!(
					"- {} (platforms: {:?})\n",
					name,
					bin.platforms
						.iter()
						.map(|plat| format!("{}-{}", plat.system, plat.arch))
						.collect::<Vec<String>>()
				))
			}
		}
	}
	RawText(ret)
}

#[get("/runner/manifest")]
async fn get_manifest() -> Vec<u8> {
	unsafe {
		match &MANIFEST {
			Some(man) => man.clone(),
			_ => unreachable!(),
		}
	}
}
#[get("/<bin>")]
async fn get_script(bin: &str) -> ScriptResponse {
	let args = Args::parse();
	unsafe {
		if let Some((bins, time)) = &mut BINS {
			reload_bins((bins, time), &args);
			return match bins.get(bin) {
				None => ScriptResponse::Status(Status::NotFound),
				Some(bin) => {
					let mut script = String::from(WEB_SH);
					script = script
						.replace("{{HASH_CALCULATION}}", HASH_CALCULATION_SH)
						.replace("{{NAME}}", &bin.name)
						.replace("{{PLATFORM_LIST}}", &format_platform_list(bin))
						.replace("{{EXTERNAL_ADDRESS}}", &args.url);
					ScriptResponse::Text(RawText(script))
				}
			};
		}
	}
	ScriptResponse::Status(Status::NotFound)
}

#[get("/<bin>/platforms")]
async fn get_platforms(bin: &str) -> ScriptResponse {
	let args = Args::parse();
	unsafe {
		if let Some((bins, time)) = &mut BINS {
			reload_bins((bins, time), &args);
			return match bins.get(bin) {
				None => ScriptResponse::Status(Status::NotFound),
				Some(bin) => ScriptResponse::Text(RawText(format_platform_list(bin))),
			};
		}
	}
	ScriptResponse::Status(Status::NotFound)
}

#[get("/bin/<bin>/<platform>/<arch>")]
async fn get_binary(bin: &str, platform: &str, arch: &str) -> BinaryResponse {
	let args = Args::parse();
	let file = NamedFile::open(format!(
		"{}/{}/{}/{}/{}",
		args.dir.file_name().unwrap().to_str().unwrap(),
		bin,
		platform,
		arch,
		bin
	))
	.await;
	match file {
		Ok(f) => BinaryResponse::Bin(f),
		Err(_) => BinaryResponse::Status(Status::BadRequest),
	}
}

#[get("/bin/<bin>/<platform>/<arch>/sign")]
async fn get_binary_sign(bin: &str, platform: &str, arch: &str) -> SignResponse {
	let args = Args::parse();
	let file = match std::fs::read(format!(
		"{}/{}/{}/{}/{}",
		args.dir.file_name().unwrap().to_str().unwrap(),
		bin,
		platform,
		arch,
		bin
	)) {
		Ok(f) => f,
		Err(_) => return SignResponse::Status(Status::BadRequest),
	};
	let keypair = unsafe {
		match &KEYPAIR {
			Some(pair) => pair,
			_ => unreachable!(),
		}
	};
	SignResponse::Bin(keypair.sk.sign(file.as_slice(), Some(Noise::generate())).as_slice().to_vec())
}
#[launch]
async fn rocket() -> _ {
	let args = Args::parse();
	if !args.dir.exists() {
		eprintln!("Directory with binary files does not exist");
		std::process::exit(1);
	}

	unsafe {
		BINS = Some((get_bins(&args), Instant::now()));
		println!("Generating keypair");
		let kp = KeyPair::generate();
		KEYPAIR = Some(kp.clone());
		println!(
			"Keypair generated. Public key: {}",
			&kp.pk.iter().map(|x| format!("{:x}", x)).collect::<Vec<String>>().join("")
		);
		println!("Generating manifest");
		let mut manifest: Vec<u8> = Vec::new();
		let mut bin_pub_key: Vec<u8> = kp.pk.to_vec();
		manifest.append(&mut bin_pub_key);
		let mut runners = 0;

		for element in std::fs::read_dir(args.runners_dir).unwrap() {
			let en = element.unwrap();
			if en.path().is_file() {
				let mut hasher = sha2::Sha256::new();
				hasher.update(std::fs::read(en.path()).unwrap().as_slice());
				let mut contents = Vec::from(
					format!(
						"{:x}  {}\n",
						hasher.finalize_fixed(),
						en.path().file_name().unwrap().to_str().unwrap()
					)
					.as_bytes(),
				);
				runners += 1;
				manifest.append(&mut contents);
			}
		}
		let mut hasher = sha2::Sha256::new();
		hasher.update(&manifest);
		println!(
			"Manifest generated with {} runners and SHA256: {:x}",
			runners,
			hasher.finalize_fixed()
		);
		MANIFEST = Some(manifest)
	};

	let figment = Figment::from(rocket::Config::default())
		.merge(("ident", "Binhost"))
		.merge(("port", args.port))
		.merge(("address", args.address));
	rocket::custom(figment)
		.mount(
			"/",
			routes![index, get_manifest, get_script, get_platforms, get_binary, get_binary_sign],
		)
		.mount("/runner", FileServer::from("runners"))
}
