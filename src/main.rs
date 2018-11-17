#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
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

use rocket::request::{Form, FromFormValue};
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
            storage_file_name: "default_destination.storage".to_string(),
            authorized_keys_file_name: "default_destination.authorized_keys".to_string()
        }
    });
}

pub struct Destination {
    address: String,
    userauth_agent: String,
    storage_file_name: String,
    authorized_keys_file_name: String
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
fn handle_post(form: Result<Form<FormInput>, Option<String>>) -> content::Html<String> {
    content::Html(match form {
        Ok(form) => {
            let config = &*CONFIG.lock().unwrap();
            let fin = form.get();
            if fin.radio == FormOption::GitHub {
                match storage::handle_submission("github", &fin.github_username, &fin.name, &config.default_destination) {
                    Ok(_) => format!("<b>SUCCESS added github user {:?}</b>", &fin.github_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if fin.radio == FormOption::Tubit {
                match storage::handle_submission("tubit", &fin.tubit_username, &fin.name, &config.default_destination) {
                    Ok(_) => format!("<b>SUCCESS added tubit user {:?}</b>", &fin.tubit_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if fin.radio == FormOption::GitLab {
                match storage::handle_submission("gitlab", &fin.gitlab_username, &fin.name, &config.default_destination) {
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

    rocket::ignite().mount("/", routes![static_files, index, handle_post]).launch();
}
