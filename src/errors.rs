use std::env::VarError;
use std::path::PathBuf;
use std::path::StripPrefixError;
use std::io::Error as IoError;
use std::num::ParseIntError;
use std::net::AddrParseError;
use std::str::Utf8Error;
use config::ConfigError;
use toml::ser::Error as TomlError;
use tera::Error as TeraError;
use serde_yaml::Error as YamlError;
use hyper::error::Error as HyperError;
use notify::Error as NotifyError;
use glob::PatternError;
use shellexpand::LookupError;

/// The error type used by this crate.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error")]
    Io(#[cause] IoError),

    #[fail(display = "Int Parse error")]
    IntParse(#[cause] ParseIntError),

    #[fail(display = "Address Parse error")]
    AddrParse(#[cause] AddrParseError),

    #[fail(display = "Theme template file encoding error")]
    ThemeTemplateEncodeing(#[cause] Utf8Error),

    #[fail(display = "Notify error")]
    Notify(#[cause] NotifyError),

    #[fail(display = "Glob pattern error")]
    Pattern(#[cause] PatternError),

    #[fail(display = "Config error")]
    Config(#[cause] ConfigError),

    #[fail(display = "Toml error")]
    Toml(#[cause] TomlError),

    #[fail(display = "Post head parse error, please use yaml grammar")]
    Yaml(#[cause] YamlError),

    #[fail(display = "Path expand error")]
    PathExpend(#[cause] LookupError<VarError>),

    #[fail(display = "Path strip prefix error")]
    PathStripPrefix(#[cause] StripPrefixError),

    #[fail(display = "Template error: {}", _0)]
    Template(String),
    // Template(#[cause] ::tera::Error),

    #[fail(display = "Server error")]
    Hyper(#[cause] HyperError),

    #[fail(display = "Argument error: {}", _0)]
    Argument(String),

    #[fail(display = "blog root directory {:?} already exists", _0)]
    RootDirExisted(PathBuf),

    #[fail(display = "blog theme {} not found", _0)]
    ThemeNotFound(String),

    #[fail(display = "blog theme {} in use, can not be deleted", _0)]
    ThemeInUse(String),

    #[fail(display = "post path {:?} format error: must be relative path without file extension", _0)]
    PostPathInvaild(PathBuf),

    #[fail(display = "post path {:?} already existed", _0)]
    PostPathExisted(PathBuf),

    #[fail(display = "post {:?} must has two parts: headers and body, splitted by first blank line", _0)]
    PostOnlyOnePart(PathBuf),

    #[fail(display = "post {:?} head part is empty", _0)]
    PostNoHead(PathBuf),

    #[fail(display = "post {:?} has not body part", _0)]
    PostNoBody(PathBuf),

    #[fail(display = "post {:?} head part format error", _0)]
    PostHead(PathBuf),
}

impl From<IoError> for Error {
     fn from(err: IoError) -> Error {
         Error::Io(err)
     }
}

impl From<ParseIntError> for Error {
     fn from(err: ParseIntError) -> Error {
         Error::IntParse(err)
     }
}

impl From<AddrParseError> for Error {
     fn from(err: AddrParseError) -> Error {
         Error::AddrParse(err)
     }
}

impl From<Utf8Error> for Error {
     fn from(err: Utf8Error) -> Error {
         Error::ThemeTemplateEncodeing(err)
     }
}

impl From<NotifyError> for Error {
     fn from(err: NotifyError) -> Error {
         Error::Notify(err)
     }
}

impl From<PatternError> for Error {
     fn from(err: PatternError) -> Error {
         Error::Pattern(err)
     }
}

impl From<ConfigError> for Error {
     fn from(err: ConfigError) -> Error {
         Error::Config(err)
     }
}

impl From<TomlError> for Error {
     fn from(err: TomlError) -> Error {
         Error::Toml(err)
     }
}

impl From<YamlError> for Error {
     fn from(err: YamlError) -> Error {
         Error::Yaml(err)
     }
}

impl From<LookupError<VarError>> for Error {
     fn from(err: LookupError<VarError>) -> Error {
         Error::PathExpend(err)
     }
}

impl From<StripPrefixError> for Error {
     fn from(err: StripPrefixError) -> Error {
         Error::PathStripPrefix(err)
     }
}

impl From<TeraError> for Error {
     fn from(err: TeraError) -> Error {
         Error::Template(err.description().to_string())
     }
}

impl From<HyperError> for Error {
     fn from(err: HyperError) -> Error {
         Error::Hyper(err)
     }
}

/// A specialized `Result` type where the error is hard-wired to [`Error`].
///
/// [`Error`]: enum.Error.html
pub type Result<T> = ::std::result::Result<T, Error>;
