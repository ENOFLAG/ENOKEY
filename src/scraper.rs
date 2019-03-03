use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use error::EnokeysError;

const GITHUB: &str = "github";
const GITLAB: &str = "gitlab";
const TUBLAB: &str = "tublab";
const ENOLAB: &str = "enolab";

pub fn fetch(user: &str, provider: &str) -> Result<Vec<String>, EnokeysError> {
    let url = get_url(&user, &provider)?;
    if let Some(keys) = fetch_from_cache(&user, &provider) {
        Ok(keys)
    } else {
        let mut res = reqwest::get(&url)?;
        if res.status() != 200 {
            return Err(EnokeysError::InvalidProviderResponse);
        }
        let keys = res
            .text()?
            .split('\n')
            .filter(|&i| !i.is_empty())
            .map(|s| format!("{} {}@{}", s, user, &provider))
            .collect::<Vec<String>>();
        save_to_cache(&user, &provider, &keys);
        Ok(keys)
    }
}

fn get_url(user: &str, provider: &str) -> Result<String, EnokeysError> {
    match provider {
        GITHUB => Ok(format!("https://www.github.com/{}.keys", &user)),
        GITLAB => Ok(format!("https://www.gitlab.com/{}.keys", &user)),
        TUBLAB => Ok(format!("https://gitlab.tubit.tu-berlin.de/{}.keys", &user)),
        ENOLAB => Ok(format!("https://gitlab.enoflag.de/{}.keys", &user)),
        x => Err(EnokeysError::InvalidProviderError(x.to_owned())),
    }
}

fn save_to_cache(user: &str, provider: &str, keys: &[String]) {
    println!("Saving keys of {}@{} to cache", &user, &provider);
    fs::create_dir_all(format!("./.enocache/{}", &provider)).unwrap();
    if let Ok(mut file) = File::create(format!("./.enocache/{}/{}", &provider, &user)) {
        for key in keys {
            file.write_all(key.as_bytes()).unwrap();
            file.write_all("\n".as_bytes()).unwrap();
        }
    } else {
        eprintln!("Could not create cache file")
    }
}

fn fetch_from_cache(user: &str, provider: &str) -> Option<Vec<String>> {
    if let Ok(mut file) = File::open(format!("./.enocache/{}/{}", &provider, &user)) {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        Some(
            content
                .split('\n')
                .filter(|&i| !i.is_empty())
                .map(|s| s.to_owned())
                .collect::<Vec<String>>(),
        )
    } else {
        None
    }
}
