#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate base64;
extern crate getopts;
extern crate regex;
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
extern crate openssh_keys;
extern crate reqwest;
extern crate ssh2;

mod deploy;
mod error;
mod scraper;
mod storage;

use error::EnokeysError;

use std::collections::HashMap;
use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use rocket::fairing::AdHoc;
use rocket::http::RawStr;
use rocket::request::{Form, FormError, FromFormValue};
use rocket::response::content;
use rocket::response::NamedFile;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use getopts::Options;
use regex::Regex;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"[^A-Za-z0-9\.@!\-_]").unwrap();
    static ref USER_DESTINATIONS_STORAGE_RAW: PathBuf = PathBuf::from("./data/user.raw");
    static ref USER_DESTINATIONS_STORAGE_PROVIDERS: PathBuf =
        PathBuf::from("./data/user.providers");
    static ref USER_DESTINATIONS_AUTHORIZED_KEYS: PathBuf =
        PathBuf::from("./keyfiles/user.authorized_keys");
    static ref ADMIN_DESTINATIONS_STORAGE_RAW: PathBuf = PathBuf::from("./data/admin.raw");
    static ref ADMIN_DESTINATIONS_STORAGE_PROVIDERS: PathBuf =
        PathBuf::from("./data/admin.providers");
    static ref ADMIN_DESTINATIONS_AUTHORIZED_KEYS: PathBuf =
        PathBuf::from("./keyfiles/admin.authorized_keys");
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
    port: u16,
}

pub struct Context {
    admin_destinations: Vec<Destination>,
    user_destinations: Vec<Destination>,
    admin_psk: String,
    user_psk: String,
}

#[derive(Debug, PartialEq)]
enum FormOption {
    GitHub,
    TubLab,
    GitLab,
    EnoLab,
    PubKey,
}

impl<'v> FromFormValue<'v> for FormOption {
    type Error = &'v RawStr;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        let variant = match v.as_str() {
            "GitHub" => FormOption::GitHub,
            "TubLab" => FormOption::TubLab,
            "EnoLab" => FormOption::EnoLab,
            "GitLab" => FormOption::GitLab,
            "PubKey" => FormOption::PubKey,
            _ => return Err(v),
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
    #[form(field = "tublabuser")]
    tublab_username: String,
    #[form(field = "gitlabuser")]
    gitlab_username: String,
    #[form(field = "enolabuser")]
    enolab_username: String,
    #[form(field = "sshpublic")]
    pub_key: String,
    authkey: String,
}

#[derive(Debug, FromForm)]
struct DeployInput {
    authkey: String,
}

#[post("/", data = "<form>")]
fn index_post(form: Result<Form<FormInput>, FormError>) -> Template {
    match form {
        Ok(form) => {
            let config = &*CONFIG.lock().unwrap();
            let admin = if form.authkey == config.admin_psk {
                true
            } else if &form.authkey == &config.user_psk {
                false
            } else {
                return Template::render("insert_result", &format!("Wrong authkey: {:?}", form));
            };
            println!("authkey {} admin={}", &form.authkey, admin);
            if form.radio == FormOption::GitHub {
                match storage::handle_submission("github", &form.github_username, &form.name, admin)
                {
                    Ok(_) => Template::render(
                        "insert_result",
                        &format!("Successfully added github user {:?}", &form.github_username),
                    ),
                    Err(e) => Template::render("insert_result", &format!("ERROR: {:?}", e)),
                }
            } else if form.radio == FormOption::TubLab {
                match storage::handle_submission("tublab", &form.tublab_username, &form.name, admin)
                {
                    Ok(_) => Template::render(
                        "insert_result",
                        &format!(
                            "Successfully added tubit gitlab user {:?}",
                            &form.tublab_username
                        ),
                    ),
                    Err(e) => Template::render("insert_result", &format!("ERROR: {:?}", e)),
                }
            } else if form.radio == FormOption::GitLab {
                match storage::handle_submission("gitlab", &form.gitlab_username, &form.name, admin)
                {
                    Ok(_) => Template::render(
                        "insert_result",
                        &format!(
                            "Successfully added gitlab.com user {:?}",
                            &form.gitlab_username
                        ),
                    ),
                    Err(e) => Template::render("insert_result", &format!("ERROR: {:?}", e)),
                }
            } else if form.radio == FormOption::EnoLab {
                match storage::handle_submission("enolab", &form.enolab_username, &form.name, admin)
                {
                    Ok(_) => Template::render(
                        "insert_result",
                        &format!(
                            "Successfully added enoflag gitlab user {:?}",
                            &form.enolab_username
                        ),
                    ),
                    Err(e) => Template::render("insert_result", &format!("ERROR: {:?}", e)),
                }
            } else if form.radio == FormOption::PubKey {
                match storage::handle_raw_submission(&form.name, &form.pub_key, admin) {
                    Ok(_) => Template::render(
                        "insert_result",
                        &format!("Successfully added raw pubkey{:?}", &form.pub_key),
                    ),
                    Err(e) => Template::render("insert_result", &format!("ERROR: {:?}", e)),
                }
            } else {
                Template::render("insert_result", &format!("ERROR: {:?}", form))
            }
        }
        Err(e) => Template::render("insert_result", &format!("Invalid form input: {:?}", e)),
    }
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
                return content::Html(format!("Wrong AUTHKEY: {:?}", form));
            };
            let admin_result = deploy::deploy(
                &config.admin_destinations,
                &ADMIN_DESTINATIONS_AUTHORIZED_KEYS,
            );
            let user_result = deploy::deploy(
                &config.user_destinations,
                &USER_DESTINATIONS_AUTHORIZED_KEYS,
            );
            format!(
                "deployed admin: {:?}\n<br/>\ndeployed user: {:?}",
                admin_result, user_result
            )
        }
        Err(e) => format!("Invalid form input: {:?}", e),
    })
}

#[get("/deploy")]
fn deploy_get() -> Template {
    let config = &*CONFIG.lock().unwrap();
    storage::generate_authorized_key_files().unwrap();
    let mut context = HashMap::new();
    let admin_destinations: Vec<String> = config
        .admin_destinations
        .iter()
        .map(|a| a.destination_name.to_string())
        .collect();
    let user_destinations: Vec<String> = config
        .user_destinations
        .iter()
        .map(|a| a.destination_name.to_string())
        .collect();
    context.insert("admin_destinations", admin_destinations);
    context.insert("user_destinations", user_destinations);
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
    opts.optflag("h", "help", "Print this help menu");
    opts.optopt(
        "a",
        "admin-servers",
        "Set the destinations (remote server) for the admin group",
        "ADMIN_SERVERS",
    );
    opts.optopt(
        "p",
        "admin-psk",
        "Set the pre-shared key to add keys the admin group",
        "ADMIN_PSK",
    );
    opts.optopt(
        "u",
        "user-servers",
        "Set the destinations (remote server) for the user group",
        "USER_SERVERS",
    );
    opts.optopt(
        "q",
        "user-psk",
        "Set the pre-shared key to add keys the user group",
        "USER_PSK",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("failed to parse cmd arguments ({})", e);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return;
    }

    {
        let config = &mut *CONFIG.lock().unwrap();

        let admin_env = match matches.opt_str("a") {
            Some(admin) => parse_destinations(&admin),
            None => {
                println!("Warning: No admin servers set");
                Ok(vec![])
            }
        };

        let user_env = match matches.opt_str("u") {
            Some(user) => parse_destinations(&user),
            None => {
                println!("Warning: No user servers set");
                Ok(vec![])
            }
        };

        config.user_destinations = match user_env {
            Ok(user_env) => user_env.clone(),
            Err(e) => {
                println!("Could not parse user servers: {:?}", e);
                return;
            }
        };

        config.admin_destinations = match admin_env {
            Ok(admin_env) => admin_env.clone(),
            Err(e) => {
                println!("Could not parse admin servers: {:?}", e);
                return;
            }
        };

        config
            .admin_destinations
            .extend(config.user_destinations.iter().cloned());
        println!(
            "admin destinations: {:?}",
            &config
                .admin_destinations
                .iter()
                .map(|d| d.destination_name.clone())
                .collect::<String>()
        );
        println!(
            "user destinations: {:?}",
            &config
                .user_destinations
                .iter()
                .map(|d| d.destination_name.clone())
                .collect::<String>()
        );

        config.user_psk = matches.opt_str("q").unwrap_or_else(|| {
            println!("Warning: User PSK not set.");
            "default".to_string()
        });

        config.admin_psk = matches.opt_str("p").unwrap_or_else(|| {
            println!("Warning: Admin PSK not set.");
            "default".to_string()
        });

        storage::load_deploy_keypair().unwrap();
    }

    rocket::ignite()
        .mount("/static", StaticFiles::from("static"))
        .mount("/keyfiles", StaticFiles::from("keyfiles"))
        .mount(
            "/",
            routes![index_post, index_get, deploy_get, deploy_post, favicon],
        )
        .attach(Template::fairing())
        .attach(AdHoc::on_response("Security Headers", |_, resp| {
            resp.adjoin_raw_header("X-XSS-Protection", "1; mode=block");
            resp.adjoin_raw_header("X-Frame-Options", "sameorigin");
            resp.adjoin_raw_header("Content-Security-Policy", "default-src 'self'");
            resp.adjoin_raw_header("X-Content-Type-Options", "nosniff"); // Disables content sniffing for older browsers
            resp.adjoin_raw_header("Referrer-Policy", "no-referrer-when-downgrade"); // won't send a referrer when going from https to http
        }))
        .launch();
}

fn parse_destinations(input: &str) -> Result<Vec<Destination>, EnokeysError> {
    if input == "" {
        return Ok(vec![]);
    }
    let entries: Vec<&str> = input.split(",").collect();
    let mut destinations = vec![];
    for entry in entries {
        let split: Vec<&str> = entry.split('@').collect();
        let (userauth_agent, address) = match split.len() {
            2 => (split[0], split[1]),
            _ => return Err(EnokeysError::InvalidEnvironmentError),
        };
        let port = parse_port(address)?;
        let address = address.split(":").collect::<Vec<&str>>()[0];
        destinations.push(Destination {
            address: address.to_string(),
            userauth_agent: userauth_agent.to_string(),
            destination_name: format!("{}@{}:{}", &userauth_agent, &address, port),
            port: port,
        })
    }
    Ok(destinations)
}

fn parse_port(address: &str) -> Result<u16, EnokeysError> {
    let split = address.split(":").collect::<Vec<&str>>();
    if split.len() == 1 {
        return Ok(22);
    }
    return Ok(split[split.len() - 1].parse::<u16>()?);
}
