extern crate wallpaper;
extern crate ureq;
extern crate chrono;

extern crate serde;
extern crate serde_json;
extern crate toml;

pub mod error;

use std::io;
use std::fs::File;

use chrono::{Utc, Datelike, NaiveDate};

use serde::{Deserialize, Serialize};

use error::{APIError, Error, Result};

const CONFIG_FN: &'static str = "config.toml";

#[derive(Deserialize)]
struct Config
{
    pub api_key: String,
    pub apod_api_url: String,
    pub wallpaper_dir: String
}

// Don't want to read from them, but we do need them to exist for deserialization to be complete
#[allow(dead_code)]
#[derive(Deserialize, Serialize)]
struct APOD
{
    copyright: Option<String>,
    date: String,
    explanation: String,
    // Might not exist...
    pub hdurl: Option<String>,
    media_type: String,
    service_version: String,
    title: String,
    pub url: String,
}

fn get_file_extension(string: &str) -> String
{
    let idx = string.rfind('.').unwrap();
    return string[idx+1..string.len()].to_string();
}


pub fn get_wallpaper_url<T>(apod_api_url: &str, api_key: &str, date: &T) -> Result<String>
where T: Datelike + std::fmt::Display
{
    let url = format!("{}?api_key={}&date={}", apod_api_url, api_key, date);
    let resp = ureq::get(&url).call();  
    
    if !resp.ok()
    {
        return Err(Error::API(APIError {
            status_code : resp.status(),
            url : url.to_string()
        }));
    }

    let apod: APOD = serde_json::from_str(&resp.into_string()?)?;

    match apod.hdurl
    {
        Some(hdurl) => {
            // It seems like if hdurl is None, then we actually have a video.
            return Ok(hdurl);
        },
        None => {
            return Ok(apod.url);
        }
    }
}

pub fn get_wallpaper_dir<T>(wallpaper_url: &str, wallpaper_dir: &str, date: T) -> String
where T: Datelike
{
    let wallpaper_ext = get_file_extension(wallpaper_url);
    let fname = format!("{}_{}_{}.{}", date.day(), date.month(), date.year(), wallpaper_ext);
    let mut wallpaper_dir = wallpaper_dir.to_string();
    wallpaper_dir.push_str(&fname);

    return wallpaper_dir;
}

pub fn download_wallpaper(url: &str, wallpaper_dir: &str) -> Result<()>
{
    // Should probably make this return a Result instead...
    let resp = ureq::get(&url).call();  
    
    if !resp.ok()
    {
        return Err(Error::API(APIError {
            status_code : resp.status(),
            url : url.to_string()
        }));
    }

    let mut out = File::create(wallpaper_dir)?;
    io::copy(&mut resp.into_reader(), &mut out)?;
    Ok(())
}

fn read_config(config_fn: &str) -> Result<Config>
{
    use std::io::Read;
    let mut file = File::open(config_fn)?;

    let mut string = String::new();
    file.read_to_string(&mut string)?;

    let config = toml::from_str(&string)?;

    Ok(config)
}

fn wallpaper_main() -> Result<()>
{
    let args: Vec<String> = std::env::args().collect();
    let date: chrono::NaiveDate;
    
    if args.len() == 1
    {
        let today = Utc::today(); 
        date = NaiveDate::from_ymd(today.year(), today.month(), today.day());
    } else
    {
        date = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d").expect("Invalid date provided. Please use YYYY-MM-DD format.");
    }

    let config = read_config(CONFIG_FN)?;
    
    // let archive_url = get_archive_url(date).unwrap();
    let wallpaper_url = get_wallpaper_url(&config.apod_api_url, &config.api_key, &date)?;
    wallpaper::set_from_url(&wallpaper_url).unwrap();

    // Should I pass this by reference?
    let wallpaper_dir = get_wallpaper_dir(&wallpaper_url, &config.wallpaper_dir, date);

    download_wallpaper(&wallpaper_url, &wallpaper_dir).unwrap();

    Ok(())
}

fn main() -> Result<()>
{
    // cache_main()
    wallpaper_main()
}