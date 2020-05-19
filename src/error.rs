#[derive(Debug)]
pub struct APIError
{
    pub url: String,
    pub status_code: u16
}

#[derive(Debug)]
pub enum Error
{
    API(APIError),
    JSON(serde_json::Error),
    TOML(toml::de::Error),
    IOError(std::io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_json::Error> for Error
{
    fn from(from: serde_json::Error) -> Self
    {
        Error::JSON(from)
    }
}

impl From<toml::de::Error> for Error
{
    fn from(from: toml::de::Error) -> Self
    {
        Error::TOML(from)
    }
}

impl From<std::io::Error> for Error
{
    fn from(from: std::io::Error) -> Self
    {
        Error::IOError(from)
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
            Self::IOError(err) => {
                write!(fmt, "{}", err)?;
            }
        }

        Ok(())
    }
}
