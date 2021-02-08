use clap::{App, AppSettings, Arg};
use eyre::Result;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::fs;
use std::path::Path;
use std::time::Duration;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[\sa-zA-Z0-9]#([0-9]+)"#).unwrap());

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = App::new("Linker")
        .version(clap::crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Praveen Perera <praveen@avencera.com>")
        .about("\nAutomatically link CHANGELOG.MD PRs and Issues")
        .arg(
            Arg::with_name("file")
                .value_name("PATH")
                .help("A file  to run on")
                .index(1)
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let file_path = Path::new(matches.value_of("file").expect("Invalid PATH provided")).to_owned();

    let contents = fs::read_to_string(&file_path)?;

    let new_contents = RE
        .replace_all(&contents, |caps: &Captures| {
            let url = get_url(format!(
                "https://github.com/clux/kube-rs/issues/{}",
                caps[1].to_string()
            ));

            format!("[#{}]({})", caps[1].to_string(), url)
        })
        .to_string();

    Ok(fs::write(file_path, new_contents.as_bytes())?)
}

fn get_url(try_url: String) -> String {
    let mut retries = 1;
    let mut res = reqwest::blocking::get(&try_url).unwrap();

    while !res.status().is_success() {
        std::thread::sleep(Duration::from_millis(250 * retries));
        res = reqwest::blocking::get(&try_url).unwrap();
        retries = retries + 1;
    }

    res.url().to_string()
}
