// SPDX-License-Identifier: MIT
// "Runner" is written in no_std Rust for the smaller executable size: ~49KiB (Darwin arm64) vs ~300KiB

#![no_main]
#![no_std]

use core::slice::from_raw_parts;
use ed25519_compact::{PublicKey, Signature};
use libc::{c_char, c_int, c_void, exit, open, printf, read, O_RDONLY};

// 1 KiB seems fine
const BUFFER_SIZE: usize = 1024;
const PUBKEY_LEN: usize = PublicKey::BYTES;
const SIGNATURE_LEN: usize = Signature::BYTES;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const c_char) -> i32 {
	printf("Starting runner\n\0".as_bytes().as_ptr() as *const c_char);

	let mut buff_public_key = [0_u8; PUBKEY_LEN];
	let a = 0;
	read(
		open("public_key\0".as_bytes().as_ptr() as *const c_char, O_RDONLY),
		buff_public_key.as_mut_ptr() as *mut c_void,
		PUBKEY_LEN,
	);
	let public_key = PublicKey::new(buff_public_key);

	let mut signature = [0_u8; SIGNATURE_LEN];
	read(
		open("signature\0".as_bytes().as_ptr() as *const c_char, O_RDONLY),
		signature.as_mut_ptr() as *mut c_void,
		SIGNATURE_LEN,
	);

	let arg = from_raw_parts(_argv, _argc as usize)[1]; // Getting path to binary
	let binary_fd = open(arg, O_RDONLY);
	let mut buffer = [0u8; BUFFER_SIZE];
	let mut state = public_key.verify_incremental(&Signature::new(signature)).unwrap();
	loop {
		let bytes_read: usize =
			read(binary_fd, buffer.as_mut_ptr() as *mut c_void, BUFFER_SIZE) as usize;
		state.absorb(&buffer[0..bytes_read]);
		if bytes_read != BUFFER_SIZE {
			break;
		}
	}

	printf("Signature: \0".as_bytes().as_ptr() as *const c_char);
	if state.verify().is_ok() {
		printf("OK\n\0".as_bytes().as_ptr() as *const c_char);
	} else {
		printf("Bad\n\0".as_bytes().as_ptr() as *const c_char);
		return 2;
	}
	0
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	use numtoa::NumToA;
	let mut buff = [0u8; 8];
	unsafe {
		printf("Panicked\0".as_bytes().as_ptr() as *const c_char);
		if let Some(location) = info.location() {
			printf(" at \0".as_bytes().as_ptr() as *const c_char);
			location.line().numtoa(10, &mut buff).iter().for_each(|ch| {
				libc::putchar(*ch as c_int);
			});
			printf(":\0".as_bytes().as_ptr() as *const c_char);
			location.column().numtoa(10, &mut buff).iter().for_each(|ch| {
				libc::putchar(*ch as c_int);
			});
			printf(" in \0".as_bytes().as_ptr() as *const c_char);
			location.file().chars().for_each(|ch| {
				libc::putchar(ch as c_int);
			});
		}
		libc::putchar(b'\n' as c_int);
		exit(3)
	}
}
