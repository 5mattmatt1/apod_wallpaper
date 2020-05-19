#![feature(step_trait)]

extern crate serde_rusqlite;
// serde_rusqlite = "0.24.0"

pub mod datetimeex;
use datetimeex::NaiveDateEx;

use rusqlite::Connection;
use rusqlite::NO_PARAMS;

#[derive(Debug)]
pub enum Error
{
    API(APIError),
    JSON(serde_json::Error),
    TOML(toml::de::Error),
    Sqlite(rusqlite::Error),
    SerdeSqlite(serde_rusqlite::Error),
    IOError(std::io::Error)
}

impl From<serde_rusqlite::Error> for Error
{
    fn from(from: serde_rusqlite::Error) -> Self
    {
        Error::SerdeSqlite(from)
    }
}

impl std::fmt::Display for Error
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self
        {
            Self::API(api_error) => {
                write!(fmt, "Got status code {} from {}.", api_error.status_code, api_error.url)?;
            },
            Self::JSON(err) => {
                write!(fmt, "{}", err)?;
            },
            Self::TOML(err) => {
                write!(fmt, "{}", err)?;
            },
            Self::Sqlite(err) => {
                write!(fmt, "{}", err)?;
            },
            Self::SerdeSqlite(err) => {
                write!(fmt, "{}", err)?;
            },
            Self::IOError(err) => {
                write!(fmt, "{}", err)?;
            }
        }

        Ok(())
    }
}


fn add_apod(conn: &rusqlite::Connection, apod: &APOD) -> Result<()>
{
    conn.execute_named("INSERT INTO apod (
            copyright,
            date,
            explanation,
            hdurl,
            media_type,
            service_version,
            title,
            url
        ) VALUES (
            :copyright,
            :date,
            :explanation,
            :hdurl,
            :media_type,
            :service_version,
            :title,
            :url
        )", 
        &serde_rusqlite::to_params_named(apod)?.to_slice()
    )?;

    Ok(())
}

fn get_apod<T>(conn: &rusqlite::Connection, date: &T) -> Result<Option<APOD>>
where T: Datelike + std::fmt::Display
{
    // deserializing using query_and_then() and from_row_with_columns(), better performance than from_row()
    let mut statement = conn.prepare("SELECT * FROM apod").unwrap();
    let columns = serde_rusqlite::columns_from_statement(&statement);
    let rows = statement.query_and_then(NO_PARAMS, |row| 
        serde_rusqlite::from_row_with_columns::<APOD>(row, &columns)
    )?;

    for result in rows
    {
        let row: APOD = result?;
        if row.date == format!("{}", date)
        {
            return Ok(Some(row)); 
        }
    }

    Ok(None)
}

pub fn init_sqlite(db_name: &str) -> Result<Connection>
{
    // Database migrations are for chumps
    let conn = Connection::open(db_name)?;

    conn.execute(
        "create table if not exists apod (
             id integer primary key,
             copyright text,
             date text not null unique,
             explanation text not null,
             hdurl text,
             media_type text not null,
             service_version text not null,
             title text not null,
             url text not null
         )",
        NO_PARAMS,
    )?;

    Ok(conn)
}

pub fn get_wallpaper_url<T>(conn: &rusqlite::Connection, apod_api_url: &str, api_key: &str, date: &T) -> Result<String>
where T: Datelike + std::fmt::Display
{
    // Need to create a proper error type for this app
    let apod: APOD;
    match get_apod(conn, date)?
    {
        Some(db_apod) => {
            apod = db_apod;
        },
        None => {
            let url = format!("{}?api_key={}&date={}", apod_api_url, api_key, date);
            let resp = ureq::get(&url).call();  
            
            if !resp.ok()
            {
                return Err(Error::API(APIError {
                    status_code : resp.status(),
                    url : url.to_string()
                }));
            }

            let api_apod: APOD = serde_json::from_str(&resp.into_string()?)?;
            add_apod(conn, &api_apod)?;
            apod = api_apod;
        }
    }

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

fn cache_date_range(conn: &rusqlite::Connection, config: &Config, start_date: NaiveDateEx, end_date: NaiveDateEx) -> Result<()>
{
    for date_ex in start_date..end_date
    {
        let date: chrono::NaiveDate = date_ex.into();
        println!("Date: {}", date);
        get_wallpaper_url(&conn, &config.apod_api_url, &config.api_key, &date)?;
    }

    Ok(())
}

fn cache_main() -> Result<()>
{
    let args: Vec<String> = std::env::args().collect();

    let start_date = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d").expect("Invalid date provided. Please use YYYY-MM-DD format.");
    let end_date = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d").expect("Invalid date provided. Please use YYYY-MM-DD format.");

    let config = read_config(CONFIG_FN)?;
    let conn = init_sqlite(&config.db_name)?;

    let start_date_ex = NaiveDateEx::from(start_date);
    let end_date_ex = NaiveDateEx::from(end_date);
    cache_date_range(&conn, &config, start_date_ex, end_date_ex)
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
    let conn = init_sqlite(&config.db_name)?;
    
    // let archive_url = get_archive_url(date).unwrap();
    let wallpaper_url = get_wallpaper_url(&conn, &config.apod_api_url, &config.api_key, &date)?;
    wallpaper::set_from_url(&wallpaper_url).unwrap();

    // Should I pass this by reference?
    let wallpaper_dir = get_wallpaper_dir(&wallpaper_url, &config.wallpaper_dir, date);

    download_wallpaper(&wallpaper_url, &wallpaper_dir).unwrap();

    Ok(())
}