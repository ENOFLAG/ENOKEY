#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
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
mod storage;

use error::EnokeysError;

use std::env;
use std::io;
use std::sync::Mutex;
use std::path::Path;
use std::path::PathBuf;

use rocket::request::{Form, FromFormValue, FormError};
use rocket::response::NamedFile;
use rocket::http::RawStr;
use rocket::response::content;

use getopts::Options;
use regex::Regex;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"[^A-Za-z0-9\.@!-_]").unwrap();
    static ref CONFIG: Mutex<Context> = Mutex::new(Context {
        default_destination: Destination {
            address: "localhost:22".to_string(),
            userauth_agent: "root".to_string(),
            destination_name: "default_destination".to_string(),
        }
    });
}

pub struct Destination {
    address: String,
    userauth_agent: String,
    destination_name: String
}

pub struct Context {
    default_destination: Destination
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

#[post("/", data = "<form>")]
fn handle_post(form: Result<Form<FormInput>, FormError>) -> content::Html<String> {
    content::Html(match form {
        Ok(form) => {
            let config = &*CONFIG.lock().unwrap();
            if form.radio == FormOption::GitHub {
                match storage::handle_submission("github", &form.github_username, &form.name, &config.default_destination) {
                    Ok(_) => format!("<b>SUCCESS added github user {:?}</b>", &form.github_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if form.radio == FormOption::Tubit {
                match storage::handle_submission("tubit", &form.tubit_username, &form.name, &config.default_destination) {
                    Ok(_) => format!("<b>SUCCESS added tubit user {:?}</b>", &form.tubit_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if form.radio == FormOption::GitLab {
                match storage::handle_submission("gitlab", &form.gitlab_username, &form.name, &config.default_destination) {
                    Ok(_) => format!("<b>SUCCESS added gitlab user {:?}</b>", &form.gitlab_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if form.radio == FormOption::PubKey {
                match storage::handle_raw_submission(&form.name, &form.pub_key, &config.default_destination) {
                    Ok(_) => format!("<b>SUCCESS added raw pubkey {}", &form.pub_key),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else {
                format!("ERROR: {:?}", form)
            }
        },
        Err(e) => format!("Invalid form input: {:?}", e)
    })
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/favicon.ico")]
fn favicon() -> io::Result<NamedFile> {
    NamedFile::open("static/favicon.ico")
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
    opts.optflag("n", "dry-run", "Do not push the generated authorized_key file");
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

    if matches.opt_present("n") {
        eprintln!("dry mode is currently not supported");
        std::process::exit(1);
    }

    rocket::ignite().mount("/", routes![static_files, index, handle_post, favicon]).launch();
}
