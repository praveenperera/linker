use clap::{App, AppSettings, Arg};
use eyre::{bail, eyre, Result};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

static RE_ISSUES_PRS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"([\(\sa-zA-Z0-9])#([0-9]+)"#).unwrap());

static RE_CONTRIBUTORS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"@([A-Za-z0-9_-]+)([\s\n\)])"#).unwrap());

static RE_COMMITS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b[a-f0-9]{40}\b|\b[a-f0-9]{6,8}\b").unwrap());

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let mut valid_links: HashMap<String, (bool, String)> = HashMap::new();

    let matches = App::new("Linker")
        .version(clap::crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Praveen Perera <praveen@avencera.com>")
        .about("\nAutomatically link CHANGELOG.MD PRs, Issues and Contributors")
        .arg(
            Arg::with_name("file")
                .value_name("PATH")
                .help("A file to run on")
                .index(1)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("repo")
                .long("repo")
                .takes_value(true)
                .required(true)
                .help("The github repo of the project ex: avencera/rustywind"),
        )
        .get_matches();

    let repo = matches
        .value_of("repo")
        .unwrap()
        .trim_end()
        .trim_end_matches("/");

    if !reqwest::blocking::get(format!("https://github.com/{}", repo))
        .unwrap()
        .status()
        .is_success()
    {
        bail!("Invalid GitHub repo please try again with a valid one")
    }

    let file_path = Path::new(matches.value_of("file").expect("Invalid PATH provided")).to_owned();

    let contents = fs::read_to_string(&file_path)?;

    let new_contents = RE_ISSUES_PRS.replace_all(&contents, |caps: &Captures| {
        match get_url(
            format!("https://github.com/{}/issues/{}", &repo, &caps[2]),
            &mut valid_links,
        ) {
            Ok(url) => {
                debug!("Added link to PR/ISSUE: {}", &url);
                format!("{}[#{}]({})", &caps[1], &caps[2], url)
            }
            Err(e) => {
                error!("{:?}", e);
                caps[0].to_string()
            }
        }
    });

    let new_contents = RE_CONTRIBUTORS
        .replace_all(&new_contents, |caps: &Captures| {
            match get_url(format!("https://github.com/{}", &caps[1]), &mut valid_links) {
                Ok(url) => {
                    debug!("Added link to contributor: {}", &url);
                    format!(
                        "[@{}](https://github.com/{}){}",
                        &caps[1], &caps[1], &caps[2]
                    )
                }
                Err(e) => {
                    error!("{:?}", e);
                    caps[0].to_string()
                }
            }
        })
        .to_string();

    let new_contents = RE_COMMITS.replace_all(&new_contents, |caps: &Captures| {
        match get_url(
            format!("https://github.com/{}/commit/{}", &repo, &caps[0]),
            &mut valid_links,
        ) {
            Ok(url) => {
                debug!("Added link to commit: {}", &url);
                format!("[{}]({})", &caps[0], url)
            }
            Err(e) => {
                error!("{:?}", e);
                caps[0].to_string()
            }
        }
    });

    Ok(fs::write(file_path, new_contents.as_bytes())?)
}

fn get_url(try_url: String, valid_links: &mut HashMap<String, (bool, String)>) -> Result<String> {
    if let Some((true, actual_url)) = valid_links.get(&try_url) {
        debug!("{} was already found to be valid", &try_url);
        return Ok(actual_url.to_string());
    }

    let mut retries = 1;
    let mut res = reqwest::blocking::get(&try_url)?;

    while !res.status().is_success() && retries <= 15 {
        if res.status() == 404 {
            break;
        }

        info!(
            "Retrying for URL: {}, code: {}, retry number: {}",
            &try_url,
            res.status().to_string(),
            retries
        );
        std::thread::sleep(Duration::from_millis(2750 + (250 * retries)));
        res = reqwest::blocking::get(&try_url)?;
        retries += 1;
    }

    if res.status().is_success() {
        valid_links.insert(try_url, (true, res.url().to_string()));
        Ok(res.url().to_string())
    } else {
        valid_links.insert(try_url.clone(), (false, "".to_string()));
        Err(eyre!("Unable to get link for: {}", try_url))
    }
}
