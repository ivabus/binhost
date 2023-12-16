mod structs;
use structs::*;

use clap::Parser;
use rocket::figment::Figment;
use rocket::fs::NamedFile;
use std::collections::HashMap;
use std::time::Instant;

#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::response::content::RawText;

static mut BINS: Option<(HashMap<String, Bin>, Instant)> = None;

static WEB_SH: &str = include_str!("../web.sh");

#[cfg(feature = "sha256")]
static HASH_CALCULATION_SH: &str = r#"
if ! which sha256sum > /dev/null; then
  echo "No \`sha256sum\` command found, continuing without checking" 1>&2
else
  echo ":: Checking hashsum" 1>&2
  if ! ($DOWNLOAD_COMMAND {{EXTERNAL_ADDRESS}}/bin/$NAME/$(uname)/$(uname -m)/sha256 $OUTPUT_ARG - | sha256sum -c - > /dev/null); then
    echo "sha256 is invalid" 1>&2
    exit 255
  fi
fi
"#;
#[cfg(not(feature = "sha256"))]
static HASH_CALCULATION_SH: &str = "";
async fn reload_bins(bins: (&mut HashMap<String, Bin>, &mut Instant), args: &Args) {
	if (Instant::now() - *bins.1).as_secs() > args.refresh {
		*bins.0 = get_bins(args).await;
		*bins.1 = Instant::now();
	}
}

async fn get_bins(args: &Args) -> HashMap<String, Bin> {
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
			reload_bins((bins, time), &args).await;
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

#[get("/<bin>")]
async fn get_script(bin: &str) -> ScriptResponse {
	let args = Args::parse();
	unsafe {
		if let Some((bins, time)) = &mut BINS {
			reload_bins((bins, time), &args).await;
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

#[cfg(feature = "sha256")]
#[get("/bin/<bin>/<platform>/<arch>/sha256")]
async fn get_binary_hash(bin: &str, platform: &str, arch: &str) -> ScriptResponse {
	use rocket::tokio::io::AsyncReadExt;
	use sha2::digest::FixedOutput;
	use sha2::Digest;

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
		Ok(mut f) => {
			let mut hasher = sha2::Sha256::new();
			let mut contents: Vec<u8> = vec![];
			f.read_to_end(&mut contents).await.unwrap();
			hasher.update(contents);
			ScriptResponse::Text(RawText(format!(
				"{:x}  {}",
				hasher.finalize_fixed(),
				f.path().file_name().unwrap().to_str().unwrap()
			)))
		}
		Err(_) => ScriptResponse::Status(Status::BadRequest),
	}
}

#[cfg(not(feature = "sha256"))]
#[get("/bin/<_bin>/<_platform>/<_arch>/sha256")]
async fn get_binary_hash(_bin: &str, _platform: &str, _arch: &str) -> ScriptResponse {
	ScriptResponse::Status(Status::BadRequest)
}
#[launch]
async fn rocket() -> _ {
	let args = Args::parse();
	if !args.dir.exists() {
		eprintln!("Directory with binary files does not exist");
		std::process::exit(1);
	}

	unsafe {
		BINS = Some((get_bins(&args).await, Instant::now()));
	}

	let figment = Figment::from(rocket::Config::default())
		.merge(("ident", "BinHost"))
		.merge(("port", args.port))
		.merge(("address", args.address));
	rocket::custom(figment).mount("/", routes![index, get_script, get_binary, get_binary_hash])
}
