#[macro_use]
extern crate clap;
extern crate exitfailure;
extern crate failure;
extern crate msgpack_simple;
extern crate quicli;
extern crate regex;
extern crate rpassword;
extern crate sodiumoxide;

use msgpack_simple::{MsgPack, MapElement};
use quicli::prelude::*;
use regex::Regex;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::pwhash;
use std::io::{self, Write};
use std::path::Path;

fn write_file (filename: &str, data: Vec<u8>, yflag: bool) -> Result<bool, exitfailure::ExitFailure> {
    if Path::new(filename).exists() && !yflag {
        let yes_regex = Regex::new(r"^(?i:y|yes|yep|yup|yeah|hell yes|1|true)$|^$").unwrap();
        let no_regex = Regex::new(r"^(?i:n|no|nope|nah|go away|0|false)$").unwrap();
        
        loop {
            let mut input = String::new();

            print!("File '{}' already exists. Overwrite? (Y/n) ", filename);
            io::stdout().flush()?;

            io::stdin().read_line(&mut input)?;
            
            if yes_regex.is_match(input.trim()) {
                break
            }

            if no_regex.is_match(input.trim()) {
                println!("Aborting");
                return Ok(false)
            }

            println!("Failed to recognize input");
        }
    }

    std::fs::write(filename, data).map_err(|_| failure::err_msg(format!("Failed to write file '{}'", filename)))?;
    Ok(true)
}

fn main() -> CliResult {
    let matches = clap_app!(Gibberish =>
        (version: "1.0")
        (author: "Ben Snow <apps@b3nsn0w.com>")
        (about: "Turns a file into gibberish and back again")
        (@arg degibberish: -d --degibberish "de-gibberishes a file previously gibberished")
        (@arg extension: -e --extension +takes_value "sets the extension (defaults to 'gibberish')")
        (@arg passphrase: -p --passphrase "sets a custom passphrase (interactive)")
        (@arg yes: -y --yes "does not ask before overwriting files")
        (@arg file: +required "Name of the file to gibberish") 
    ).get_matches();

    let filename = matches.value_of("file").unwrap();
    let file = std::fs::read(&filename).map_err(|_| failure::err_msg(format!("File '{}' not found", filename)))?;

    let re = Regex::new(r"^(.*)\.([^.]*)$").unwrap();
    let captures = re.captures(filename);

    let (orig_name, orig_ext) = match captures {
        Some(cap) => (String::from(&cap[1]), String::from(&cap[2])),
        None => (String::from(filename), String::from(""))
    };

    let extension = matches.value_of("extension").unwrap_or("gibberish");

    let yflag = matches.is_present("yes");

    if matches.is_present("degibberish") {
        let salt = pwhash::Salt::from_slice(&file[0 .. pwhash::SALTBYTES]).unwrap();
        let nonce = secretbox::Nonce::from_slice(&file[pwhash::SALTBYTES .. pwhash::SALTBYTES + secretbox::NONCEBYTES]).unwrap();
        let ciphertext = &file[pwhash::SALTBYTES + secretbox::NONCEBYTES ..];

        let password = if matches.is_present("passphrase") {
            let pw = rpassword::read_password_from_tty(Some("Passphrase: ")).unwrap();
            #[cfg(target_os = "windows")]
            println!();

            pw
        } else {
            orig_ext
        };

        let mut key = secretbox::Key([0; secretbox::KEYBYTES]);
        pwhash::derive_key(&mut key.0, password.as_bytes(), &salt, pwhash::OPSLIMIT_INTERACTIVE, pwhash::MEMLIMIT_INTERACTIVE).unwrap();

        let plaintext = secretbox::open(ciphertext, &nonce, &key).map_err(|_| failure::err_msg("Failed to decode gibberish"))?;
        let message = MsgPack::parse(&plaintext)?;

        let map = message.as_map()?;

        let mut extension = None;
        let mut content = None;

        for element in map {
            let key = element.key.as_string()?;
            if key == "extension" {
                extension = Some(element.value.as_string()?);
            } else if key == "file" {
                content = Some(element.value.as_binary()?);
            }
        }

        let target_filename = format!("{}.{}", &orig_name, extension.ok_or_else(|| failure::err_msg("Original file extension missing"))?);
        let data = content.ok_or_else(|| failure::err_msg("Original file content missing"))?;

        if write_file(&target_filename, data, yflag)? {
            println!("Decoded gibberish to {}", &target_filename);
        }
    } else {
        let salt = pwhash::gen_salt();
        let nonce = secretbox::gen_nonce();

        let password = if matches.is_present("passphrase") {
            loop {
                let password1 = rpassword::read_password_from_tty(Some("Passphrase: ")).unwrap();
                #[cfg(target_os = "windows")]
                println!();

                let password2 = rpassword::read_password_from_tty(Some("Confirm passphrase: ")).unwrap();
                #[cfg(target_os = "windows")]
                println!();

                if password1 == password2 {
                    break password1;
                }

                println!("Passphrases do not match, try again")
            }
        } else {
            String::from(extension)
        };

        let mut key = secretbox::Key([0; secretbox::KEYBYTES]);
        pwhash::derive_key(&mut key.0, password.as_bytes(), &salt, pwhash::OPSLIMIT_INTERACTIVE, pwhash::MEMLIMIT_INTERACTIVE).unwrap();

        let message = MsgPack::Map(vec![
            MapElement {
                key: MsgPack::String(String::from("extension")),
                value: MsgPack::String(orig_ext)
            },
            MapElement {
                key: MsgPack::String(String::from("file")),
                value: MsgPack::Binary(file)
            }
        ]);

        let ciphertext = secretbox::seal(&message.encode(), &nonce, &key);
        let target_filename = format!("{}.{}", &orig_name, extension);

        let mut data = Vec::with_capacity(ciphertext.len() + pwhash::SALTBYTES + secretbox::NONCEBYTES);
        data.extend_from_slice(&salt.0);
        data.extend_from_slice(&nonce.0);
        data.extend_from_slice(&ciphertext);

        if write_file(&target_filename, data, yflag)? {
            println!("Gibberish written to {}", &target_filename);
        }
    }

    Ok(())
}
