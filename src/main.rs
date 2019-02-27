#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
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
use std::collections::HashMap;

use rocket::request::{Form, FromFormValue, FormError};
use rocket::response::NamedFile;
use rocket::http::RawStr;
use rocket::response::content;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;

use getopts::Options;
use regex::Regex;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"[^A-Za-z0-9\.@!\-_]").unwrap();
    static ref CONFIG: Mutex<Context> = Mutex::new(Context {
        admin_destinations: vec!(),
        user_destinations: vec!(),
        admin_psk: "default".to_string(),
        user_psk: "default".to_string()
    });
}

#[derive(Clone, Debug)]
pub struct Destination {
    address: String,
    userauth_agent: String,
    destination_name: String,
    authorized_keys_file_name: PathBuf,
    raw_storage_file_name: PathBuf,
    providers_storage_file_name: PathBuf,
    port: u16
}

pub struct Context {
    admin_destinations: Vec<Destination>,
    user_destinations: Vec<Destination>,
    admin_psk: String,
    user_psk: String
}

#[derive(Debug,PartialEq)]
enum FormOption {
    GitHub, Tubit, GitLab, EnoLab, PubKey
}

impl<'v> FromFormValue<'v> for FormOption {
    type Error = &'v RawStr;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        let variant = match v.as_str() {
            "GitHub" => FormOption::GitHub,
            "Tubit" => FormOption::Tubit,
            "EnoLab" => FormOption::EnoLab,
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
    #[form(field = "enolabuser")]
    enolab_username: String,
    #[form(field = "sshpublic")]
    pub_key: String,
    authkey: String
}

#[derive(Debug, FromForm)]
struct DeployInput {
    authkey: String
}

#[post("/", data = "<form>")]
fn index_post(form: Result<Form<FormInput>, FormError>) -> content::Html<String> {
    content::Html(match form {
        Ok(form) => {
            let config = &*CONFIG.lock().unwrap();
            let destinations = if form.authkey == config.admin_psk {
                &config.admin_destinations
            } else if &form.authkey == &config.user_psk {
                &config.user_destinations
            } else {
                return content::Html(format!("Wrong AUTHKEY: {:?}", form))
            };
            println!("authkey {} {:?}",&form.authkey, &destinations);
            if form.radio == FormOption::GitHub {
                match storage::handle_submission("github", &form.github_username, &form.name, destinations) {
                    Ok(_) => format!("<b>SUCCESS added github user {:?}</b>", &form.github_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if form.radio == FormOption::Tubit {
                match storage::handle_submission("tubit", &form.tubit_username, &form.name, destinations) {
                    Ok(_) => format!("<b>SUCCESS added tubit gitlab user {:?}</b>", &form.tubit_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if form.radio == FormOption::GitLab {
                match storage::handle_submission("gitlab", &form.gitlab_username, &form.name, destinations) {
                    Ok(_) => format!("<b>SUCCESS added gitlab.com user {:?}</b>", &form.gitlab_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if form.radio == FormOption::EnoLab {
                match storage::handle_submission("enolab", &form.enolab_username, &form.name, destinations) {
                    Ok(_) => format!("<b>SUCCESS added enoflag gitlab user {:?}</b>", &form.enolab_username),
                    Err(e) => format!("ERROR: {:?}", e)
                }
            } else if form.radio == FormOption::PubKey {
                match storage::handle_raw_submission(&form.name, &form.pub_key, destinations) {
                    Ok(_) => format!("<b>SUCCESS added raw pubkey {:?}</b>", &form.pub_key),
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
fn index_get() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[post("/deploy", data = "<form>")]
fn deploy_post(form: Result<Form<DeployInput>, FormError>) -> content::Html<String> {
    content::Html(match form {
        Ok(form) => {
            let config = &*CONFIG.lock().unwrap();
            if form.authkey != config.admin_psk {
                return content::Html(format!("Wrong AUTHKEY: {:?}", form))
            };
            let admin_result = deploy::deploy(&config.admin_destinations);
            let user_result = deploy::deploy(&config.user_destinations);
            format!("deployed admin: {:?}\n<br/>\ndeployed user: {:?}", admin_result, user_result)
        },
        Err(e) => format!("Invalid form input: {:?}", e)
    })
}

#[get("/deploy")]
fn deploy_get() -> Template {
    let config = &*CONFIG.lock().unwrap();
    storage::generate_authorized_key_files(&config.admin_destinations).unwrap();
    let mut context = HashMap::new();
    let destinations: Vec<String> = config.admin_destinations
        .iter()
        .map(|a| a.authorized_keys_file_name.to_str().unwrap().to_string()).collect();
    context.insert("destinations", destinations);
    Template::render("deploy", &context)
}

#[get("/favicon.ico")]
fn favicon() -> io::Result<NamedFile> {
    NamedFile::open("static/favicon.ico")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("n", "dry-run", "Do not push the generated authorized_key file");
    opts.optflag("h", "help", "Print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(e) => {
            println!("failed to parse cmd arguments ({})", e);
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

    let admin_env = match env::var("SERVER_ADMIN") {
        Ok(admin) => parse_destinations(&admin),
        Err(e) => {
            println!("Warning: SERVER_ADMIN not set. ({})", e);
            Ok(vec!())
        }
    };

    let user_env = match env::var("SERVER_USER") {
        Ok(admin) => parse_destinations(&admin),
        Err(e) => {
            println!("Warning: SERVER_USER not set. ({})", e);
            Ok(vec!())
        }
    };

    {
        let config = &mut *CONFIG.lock().unwrap();
        config.user_destinations = match user_env {
            Ok(user_env) => user_env.clone(),
            Err(e) => {
                println!("Could not parse SERVER_USER {:?}", e);
                return;
            }
        };

        config.admin_destinations = match admin_env {
            Ok(admin_env) => admin_env.clone(),
            Err(e) => {
                println!("Could not parse SERVER_ADMIN {:?}", e);
                return;
            }
        };

        config.admin_destinations.extend(config.user_destinations.iter().cloned());

        config.user_psk = env::var("PSK_USER").unwrap_or_else(|e|{
            println!("Warning: PSK_USER not set. {:?}", e);
            "default".to_string()
        });

        config.admin_psk = env::var("PSK_ADMIN").unwrap_or_else(|e|{
            println!("Warning: PSK_ADMIN not set. {:?}",e);
            "default".to_string()
        });

        storage::load_deploy_keypair().unwrap();
    }

    rocket::ignite()
        .mount("/static", StaticFiles::from("static"))
        .mount("/keyfiles", StaticFiles::from("keyfiles"))
        .mount("/", routes![index_post, index_get, deploy_get, deploy_post, favicon])
        .attach(Template::fairing())
        .launch();
}

fn parse_destinations(input: &str) -> Result<Vec<Destination>, EnokeysError> {
    if input == "" {
        return Ok(vec!())
    }
    let entries : Vec<&str> = input.split(",").collect();
    println!("{:?}",&entries);
    let mut destinations = vec!();
    for entry in entries {
        let split : Vec<&str>= entry.split('@').collect();
        let (userauth_agent, address) = match split.len() {
            2 => (split[0], split[1]),
            _ => return Err(EnokeysError::InvalidEnvironmentError)
        };
        let port = parse_port(address)?;
        let address = address.split(":").collect::<Vec<&str>>()[0];
        destinations.push(Destination {
            address: address.to_string(),
            userauth_agent: userauth_agent.to_string(),
            destination_name: format!("{}@{}:{}", &userauth_agent, &address, port),
            authorized_keys_file_name: PathBuf::from(format!("./keyfiles/{}@{}_{}.authorized_keys", &userauth_agent, &address, port)),
            raw_storage_file_name: PathBuf::from(format!("./data/{}@{}_{}.authorized_keys.raw", &userauth_agent, &address, port)),
            providers_storage_file_name: PathBuf::from(format!("./data/{}@{}_{}.authorized_keys.providers", &userauth_agent, &address, port)),
            port: port
        })
    }
    Ok(destinations)
}

fn parse_port(address: &str) -> Result<u16, EnokeysError> {
    let split = address.split(":").collect::<Vec<&str>>();
    if split.len() == 1 {
        return Ok(22);
    }
    return Ok(split[split.len()-1].parse::<u16>()?);
}
