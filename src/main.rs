use clap::{App, AppSettings, Arg};
use eyre::Result;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::fs;
use std::path::Path;
use std::time::Duration;

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"https://github.com/clux/kube-rs/issues/([0-9]+)"#).unwrap());

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = App::new("RustyWind")
        .version(clap::crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Praveen Perera <praveen@avencera.com>")
        .about("\nOrganize all your tailwind classes")
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
            let res = reqwest::blocking::get(&format!(
                "https://github.com/clux/kube-rs/issues/{}",
                caps[1].to_string()
            ))
            .unwrap();

            println!("STATUS: {:#?}, NUMBER: {}", res.status(), &caps[1]);

            if res.status().is_success() {
                std::thread::sleep(Duration::from_millis(1000));
                res.url().to_string()
            } else {
                std::thread::sleep(Duration::from_millis(2000));
                let res = reqwest::blocking::get(&format!(
                    "https://github.com/clux/kube-rs/issues/{}",
                    caps[1].to_string()
                ))
                .unwrap();
                res.url().to_string()
            }
        })
        .to_string();

    Ok(fs::write(file_path, new_contents.as_bytes())?)
}
