extern crate dirs;
use std::io::Write;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::cmp::min;

use openssh_keys::PublicKey;

use error::EnokeysError;
use scraper;
use USERNAME_REGEX;
use Destination;


pub fn handle_raw_submission(name: &str, pub_key: &str, destinations: &[Destination]) -> Result<(), EnokeysError> {
    for destination in destinations {
        let name = USERNAME_REGEX.replace_all(name, " ");
        let raw_storage_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&destination.raw_storage_file_name)?;
        writeln!(&raw_storage_file, "{} {}@raw", &pub_key, &name)?;
    }
    Ok(())
}

pub fn handle_submission(provider: &str, user_name: &str, name: &str, destinations: &[Destination]) -> Result<(), EnokeysError> {
    if provider.is_empty() || user_name.is_empty() {
        return Err(EnokeysError::InvalidData("username or service empty".to_string()));
    }
    let user_name = USERNAME_REGEX.replace_all(user_name, "");
    let name = USERNAME_REGEX.replace_all(name, " ");
    for destination in destinations {
        let mut storage_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&destination.providers_storage_file_name)?;
        let line = format!("# {} \n{}:{}\n", &name, provider, &user_name);
        println!("Adding entry:\n{}", &line);
        write!(storage_file, "{}", &line)?;
    }
    Ok(())
}

pub fn generate_authorized_key_files(destinations: &[Destination]) -> Result<(), EnokeysError> {
    for destination in destinations {
        let mut authorized_keys_file = File::create(&destination.authorized_keys_file_name)?;
        let mut storage_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&destination.providers_storage_file_name)?;
        let mut storage_file_content = String::new();

        // append deploy key
        let mut deploy_key = String::new();
        // TODO: User-configable ssh key
        let mut path = dirs::home_dir().unwrap();
        path.push(".ssh");
        path.push("id_ed25519.pub");
        println!("Loading SSH-pubkeyfile {:?}",path);
        if let Ok(mut deploy_key_file) = File::open(path) {
            deploy_key_file.read_to_string(&mut deploy_key)?;
            write!(authorized_keys_file, "{}", &deploy_key)?
        }


        // append raw keys
        let mut raw_keys = String::new();
        if let Ok(mut raw_keys_file) = File::open(&destination.raw_storage_file_name) {
            raw_keys_file.read_to_string(&mut raw_keys)?;
            match PublicKey::parse(&raw_keys) {
                Ok(key) => {
                    match &key.comment {
                        Some(ref comment) => {
                            let comment = USERNAME_REGEX.replace_all(&comment, " ");
                            let line = format!("{} {} {}\n", key.keytype(), base64::encode(&key.data()), &comment[0..min(comment.len(), 100)]);
                            write!(authorized_keys_file, "{}", &line)?
                        },
                        None => writeln!(authorized_keys_file, "{} {}", key.keytype(), base64::encode(&key.data()))?
                    }
                },
                Err(e) => println!("Failed to parse PublicKey: {:?}", e)
            }
        }

        // append keys from providers
        storage_file.read_to_string(&mut storage_file_content)?;
        for line in storage_file_content.split('\n').filter(|&i|!i.is_empty()).filter(|&s|&s[0..1]!="#") {
            let entry = line.split(':').collect::<Vec<&str>>();
            let user_keys = scraper::fetch(entry[1], entry[0])?;
            for key in user_keys {
                println!("parsing key: {}", &key);
                match PublicKey::parse(&key) {
                    Ok(key) => {
                        match &key.comment {
                            Some(ref comment) => {
                                let comment = USERNAME_REGEX.replace_all(&comment, " ");
                                let line = format!("{} {} {} ({}@{})\n", key.keytype(), base64::encode(&key.data()), &comment[0..min(comment.len(), 100)], entry[1], entry[0]);
                                write!(authorized_keys_file, "{}", &line)?
                            },
                            None => writeln!(authorized_keys_file, "{} {}", key.keytype(), base64::encode(&key.data()))?
                        }
                    },
                    Err(e) => println!("Failed to parse PublicKey: {:?}", e)
                }
            }
        }
    }
    Ok(())
}

pub fn load_deploy_keypair() -> Result<(), EnokeysError> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".ssh");
    path.push("id_ed25519");
    println!("Loading SSH-keyfile {:?}",path);
    // TODO: User-configable ssh key
    match File::open(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(EnokeysError::IOError(e))
    }
}
