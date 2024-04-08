use clap::{arg, Args, Subcommand};
use home;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const DEFAULT_JWT_FILE: &str = ".agama/agama-jwt";
const DEFAULT_AUTH_URL: &str = "http://localhost:3000/api/authenticate";
const DEFAULT_FILE_MODE: u32 = 0o600;

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Login with defined server. Result is JWT stored locally and made available to
    /// further use. Password can be provided by commandline option, from a file or it fallbacks
    /// into an interactive prompt.
    Login(LoginArgs),
    /// Release currently stored JWT
    Logout,
    /// Prints currently stored JWT to stdout
    Show,
}

/// Main entry point called from agama CLI main loop
pub async fn run(subcommand: AuthCommands) -> anyhow::Result<()> {
    match subcommand {
        AuthCommands::Login(options) => login(LoginArgs::proceed(options).password()?).await,
        AuthCommands::Logout => logout(),
        AuthCommands::Show => show(),
    }
}

/// Reads stored token and returns it
fn jwt() -> anyhow::Result<String> {
    if let Some(file) = jwt_file() {
        if let Ok(token) = read_line_from_file(&file.as_path()) {
            return Ok(token);
        }
    }

    Err(anyhow::anyhow!("Authentication token not available"))
}

/// Stores user provided configuration for login command
#[derive(Args, Debug)]
pub struct LoginArgs {
    #[arg(long, short = 'p')]
    password: Option<String>,
    #[arg(long, short = 'f')]
    file: Option<PathBuf>,
}

impl LoginArgs {
    /// Transforms user provided options into internal representation
    /// See Credentials trait
    fn proceed(options: LoginArgs) -> Box<dyn Credentials> {
        if let Some(password) = options.password {
            Box::new(KnownCredentials { password })
        } else if let Some(path) = options.file {
            Box::new(FileCredentials { path })
        } else {
            Box::new(MissingCredentials {})
        }
    }
}

/// Placeholder for no configuration provided by user
struct MissingCredentials;

/// Stores whatever is needed for reading credentials from a file
struct FileCredentials {
    path: PathBuf,
}

/// Stores credentials as provided by the user directly
struct KnownCredentials {
    password: String,
}

/// Transforms credentials from user's input into format used internaly
trait Credentials {
    fn password(&self) -> io::Result<String>;
}

impl Credentials for KnownCredentials {
    fn password(&self) -> io::Result<String> {
        Ok(self.password.clone())
    }
}

impl Credentials for FileCredentials {
    fn password(&self) -> io::Result<String> {
        read_line_from_file(&self.path.as_path())
    }
}

impl Credentials for MissingCredentials {
    fn password(&self) -> io::Result<String> {
        let password = read_credential("Password".to_string())?;

        Ok(password)
    }
}

/// Path to file where JWT is stored
fn jwt_file() -> Option<PathBuf> {
    Some(home::home_dir()?.join(DEFAULT_JWT_FILE))
}

/// Reads first line from given file
fn read_line_from_file(path: &Path) -> io::Result<String> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Cannot find the file containing the credentials.",
        ));
    }

    if let Ok(file) = File::open(&path) {
        // cares only of first line, take everything. No comments
        // or something like that supported
        let raw = BufReader::new(file).lines().next();

        if let Some(line) = raw {
            return line;
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to open the file",
    ))
}

/// Asks user to provide a line of input. Displays a prompt.
fn read_credential(caption: String) -> io::Result<String> {
    let mut cred = String::new();

    println!("{}: ", caption);

    io::stdin().read_line(&mut cred)?;
    if cred.pop().is_none() || cred.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to read {}", caption),
        ));
    }

    Ok(cred)
}

/// Sets the archive owner to root:root. Also sets the file permissions to read/write for the
/// owner only.
fn set_file_permissions(file: &Path) -> io::Result<()> {
    let attr = fs::metadata(file)?;
    let mut permissions = attr.permissions();

    // set the file file permissions to -rw-------
    permissions.set_mode(DEFAULT_FILE_MODE);
    fs::set_permissions(file, permissions)?;

    Ok(())
}

/// Necessary http request header for authenticate
fn authenticate_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers
}

/// Query web server for JWT
async fn get_jwt(url: String, password: String) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .headers(authenticate_headers())
        .body(format!("{{\"password\": \"{}\"}}", password))
        .send()
        .await?;
    let body = response
        .json::<std::collections::HashMap<String, String>>()
        .await?;
    let value = body.get(&"token".to_string());

    if let Some(token) = value {
        return Ok(token.clone());
    }

    Err(anyhow::anyhow!("Failed to get authentication token"))
}

/// Logs into the installation web server and stores JWT for later use.
async fn login(password: String) -> anyhow::Result<()> {
    // 1) ask web server for JWT
    let res = get_jwt(DEFAULT_AUTH_URL.to_string(), password).await?;

    // 2) if successful store the JWT for later use
    if let Some(path) = jwt_file() {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        } else {
            return Err(anyhow::anyhow!("Cannot store the authentication token"));
        }

        fs::write(path.as_path(), res)?;
        set_file_permissions(path.as_path())?;
    }

    Ok(())
}

/// Releases JWT
fn logout() -> anyhow::Result<()> {
    let path = jwt_file();

    if !&path.clone().is_some_and(|p| p.exists()) {
        // mask if the file with the JWT doesn't exist (most probably no login before logout)
        return Ok(());
    }

    // panicking is right thing to do if expect fails, becase it was already checked twice that
    // the path exists
    let file = path.expect("Cannot locate stored JWT");

    Ok(fs::remove_file(file)?)
}

/// Shows stored JWT on stdout
fn show() -> anyhow::Result<()> {
    // we do not care if jwt() fails or not. If there is something to print, show it otherwise
    // stay silent
    if let Ok(token) = jwt() {
        println!("{}", token);
    }

    Ok(())
}
