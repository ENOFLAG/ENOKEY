#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate fs2;
extern crate getopts;
extern crate base64;
extern crate regex;
#[macro_use] extern crate lazy_static;
extern crate openssh_keys;
extern crate reqwest;
extern crate ssh2;

mod scraper;
mod error;
mod deploy;

use error::EnokeysError;

use std::env;
use std::io;
use std::fs::OpenOptions;
use std::sync::Mutex;
use std::io::{Read,Write};
use std::cmp::min;
use std::path::Path;
use std::path::PathBuf;

use rocket::request::{Form, FromFormValue};
use rocket::response::NamedFile;
use rocket::http::RawStr;
use rocket::response::content;

use getopts::Options;
use openssh_keys::PublicKey;
use fs2::FileExt;
use regex::Regex;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"[^A-Za-z0-9\.@!-_]").unwrap();
    static ref CONFIG: Mutex<Config> = Mutex::new(Config {
        filename: None,
        authorized_keys: None
    });
}

pub struct Config {
    filename: Option<String>,
    authorized_keys: Option<String>
}

#[derive(Debug,PartialEq)]
enum FormOption {
    GitHub, Tubit, GitLab, PubKey
}

impl<'v> FromFormValue<'v> for FormOption {
    type Error = &'v RawStr;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        let variant = match v.as_str() {
            "GitHub" => FormOption::GitHub,
            "Tubit" => FormOption::Tubit,
            "GitLab" => FormOption::GitLab,
            "PubKey" => FormOption::PubKey,
            _ => return Err(v)
        };
        print!("{:?}",variant);
        Ok(variant)
    }
}

#[derive(Debug, FromForm)]
struct FormInput {
    name: String,
    #[form(field = "type")]
    radio: FormOption,
    #[form(field = "githubuser")]
    github_username: String,
    #[form(field = "tubituser")]
    tubit_username: String,
    #[form(field = "gitlabuser")]
    gitlab_username: String,
    #[form(field = "sshpublic")]
    pub_key: String,
}

fn write_authorized_keys(cfg: &Config) -> Result<(), EnokeysError>{
    if let Some(ref filename) = cfg.filename {
        if let Some(ref authfile) = cfg.authorized_keys {
            let mut inputstoragefile = OpenOptions::new().read(true).open(&filename)?;
            let mut authorized_keysfile = OpenOptions::new().write(true).read(true).open(&authfile)?;
            inputstoragefile.lock_exclusive()?;
            authorized_keysfile.lock_exclusive()?;
            let mut file_contents = String::new();
            inputstoragefile.read_to_string(&mut file_contents)?;
            println!("this is file_contents: {}", file_contents);
            // TODO: Retain comment and add it to the pubkey comment
            for s in file_contents.split("\n").filter(|&i|!i.is_empty()).filter(|&s|&s[0..1]!="#") {
                let foo = s.split(":").collect::<Vec<&str>>();
                let user_keys = match foo[0] {
                    "github" => scraper::fetch_github(foo[1])?,
                    "gitlab" => scraper::fetch_gitlab(foo[1])?,
                    "enolab" => scraper::fetch_enolab(foo[1])?,
                    "tublab" => scraper::fetch_tublab(foo[1])?,
                    _=> return Err(EnokeysError::InvalidProviderError(foo[0].to_string()))
                };
                for key2 in user_keys {
                    match PublicKey::parse(&key2) {
                        Ok(key) => {
                            match &key.comment {
                                &Some(ref comment) => {
                                    let comment = USERNAME_REGEX.replace_all(&comment, " ");
                                    write!(authorized_keysfile, "{} {} {}\n", key.keytype(), base64::encode(&key.data()), &comment[0..min(comment.len(), 100)])?
                                },
                                &None => write!(authorized_keysfile, "{} {}\n", key.keytype(), base64::encode(&key.data()))?
                            }
                        },
                        Err(e) => println!("{:?}", e)
                    }
                }
            }
            inputstoragefile.unlock()?;
            let mut auth_contents = String::new();
            authorized_keysfile.read_to_string(&mut auth_contents)?;
            authorized_keysfile.unlock()?;
            deploy::deploy(cfg, &auth_contents);
        }
    }
    Ok(())
}

fn write_service_to_file(service: &str, username: &str, name: &str) -> Result<(), EnokeysError> {
    if service.is_empty() || username.is_empty() {
        return Err(EnokeysError::InvalidData("username or service empty".to_string()));
    }
    let cname = USERNAME_REGEX.replace_all(name, " ");
    let cuser = USERNAME_REGEX.replace_all(username, "");
    let cfg = &CONFIG.lock().unwrap();
    if let Some(ref filename) = cfg.filename {
        let mut file = OpenOptions::new().append(true).open(&filename)?;
        file.lock_exclusive()?;
        write!(file, "# {} \n{}:{}\n", cname, service, cuser)?;
        file.unlock()?
    }
    println!{"writeauthkey_s"};
    write_authorized_keys(&cfg)?;
    Ok(())
}

fn write_key_to_file(key: PublicKey, name: &str) -> Result<PublicKey, EnokeysError> {
    let cname = USERNAME_REGEX.replace_all(name, " ");
    let cfg = &CONFIG.lock().unwrap();
    if let Some(ref filename) = cfg.filename {
        /* Despite OpenOption's append doc stating "This option, when true, means that writes will
         * append to a file instead of overwriting previous contents. Note that setting .write(true).append(true)
         * has the same effect as setting only .append(true)" we have to open the file with either write or read
         * permissions. If we don't, file.lock_exclusive will fail on windows because the handle was not actually
         * opened with the GENERIC_READ or GENERIC_WRITE access right.
         * https://msdn.microsoft.com/en-us/library/windows/desktop/aa365203(v=vs.85).aspx
         */
        let mut file = OpenOptions::new().read(true).append(true).open(&filename)?;
        file.lock_exclusive()?;
        match &key.comment {
            &Some(ref comment) => {
                let comment = USERNAME_REGEX.replace_all(&comment, " ");
                write!(file, "pubkey:{} {} {} ({})\n", key.keytype(), base64::encode(&key.data()), &comment[0..min(comment.len(), 100)], cname)?
            },
            &None => write!(file, "pubkey:{} {} ({})\n", key.keytype(), base64::encode(&key.data()), cname)?,
        }
        file.unlock()?
    }
    println!{"writeauthkey_k"};
    write_authorized_keys(&cfg)?;
    Ok(key)
}

#[post("/", data = "<form>")]
fn handle_post(form: Result<Form<FormInput>, Option<String>>) -> content::Html<String> {
    content::Html(match form {
        Ok(form) => {
            let fin = form.get();
            if fin.radio == FormOption::PubKey {
                match PublicKey::parse(&fin.pub_key) {
                    Ok(key) => {
                        match write_key_to_file(key, &fin.name) {
                            Ok(k) => format!("<b>SUCCESS added public key: {:?} for user: {}</b>", k, &fin.name),
                            Err(e) => format!("{:?}", e)
                        }
                    },
                    Err(e) => format!("{:?}", e)
                }
            } else if fin.radio == FormOption::GitHub {
                match write_service_to_file("github", &fin.github_username, &fin.name) {
                    Ok(_) => format!("<b>SUCCESS added github user {:?}</b>", &fin.github_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if fin.radio == FormOption::Tubit {
                match write_service_to_file("tubit", &fin.tubit_username, &fin.name) {
                    Ok(_) => format!("<b>SUCCESS added tubit user {:?}</b>", &fin.tubit_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if fin.radio == FormOption::GitLab {
                match write_service_to_file("gitlab", &fin.gitlab_username, &fin.name) {
                    Ok(_) => format!("<b>SUCCESS added gitlab user {:?}</b>", &fin.gitlab_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else {
                format!("ERROR: {:?}", form.get())
            }
        },
        Err(Some(f)) => format!("Invalid form input: {}", f),
        Err(None) => "Form input was invalid UTF8.".to_string(),
    })
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/static/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    let allowed_files = vec!(
        "css/bootstrap.min.css",
        "css/bootstrap.min.css.map",
        "css/style.css"
    );

    if let Some(file) = file.to_str() {
        if allowed_files.contains(&file) {
            return NamedFile::open(Path::new("static/").join(file)).ok();
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("s", "storage", "The output ENOKEYS storage file", "ENOKEYS.storage");
    opts.optopt("o", "authorized_keys", "The output authorized_keys file", "authorized_keys");
    opts.optflag("n", "dry-run", "Do not write to cfg file");
    opts.optflag("h", "help", "Print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            println!("failed to parse cmd arguments ({})", f);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return;
    }

    if !matches.opt_present("n") {
        let filename = if let Some(filename) = matches.opt_str("s") {
             filename
        } else {
            "ENOKEYS.storage".to_string()
        };
        CONFIG.lock().unwrap().filename = Some(filename.clone());
        if !OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .is_ok() {
            println!("could not create storage file");
            return;
        }
        let authfile = if let Some(authfile) = matches.opt_str("o") {
            authfile
        } else {
            "authorized_keys".to_string()
        };
        CONFIG.lock().unwrap().authorized_keys = Some(authfile.clone());
        if !OpenOptions::new()
            .write(true)
            .create(true)
            .open(authfile)
            .is_ok() {
            println!("could not create authorized_keys file");
            return;
        }
    }

    rocket::ignite().mount("/", routes![static_files, index, handle_post]).launch();
}
