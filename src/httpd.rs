use crate::args::Args;
use crate::errors::*;
use crate::server::Bind;
use crate::utils;
use actix_files::NamedFile;
use actix_web::{
    get, http::header, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use std::borrow::Cow;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

const DIR_LIST_PADDING: usize = 50;

pub struct Config {
    web_root: String,
    list_directories: bool,
}

fn resolve_path_req(base: &str, req: &Path) -> Result<PathBuf> {
    let mut path = PathBuf::from(base);
    for comp in req.components() {
        match comp {
            Component::Prefix(_) => bail!("Invalid component: windows path prefix"),
            Component::RootDir => bail!("Invalid component: unix root directory"),
            Component::CurDir => bail!("Invalid component: current dir"),
            Component::ParentDir => bail!("Invalid component: parent dir"),
            Component::Normal(comp) => {
                path.push(comp);
            }
        }
    }
    Ok(path)
}

fn bad_request() -> HttpResponse {
    HttpResponse::BadRequest()
        .content_type("text/plain; charset=utf-8")
        .body("400 - bad request\n")
}

fn forbidden() -> HttpResponse {
    HttpResponse::Forbidden()
        .content_type("text/plain; charset=utf-8")
        .body("403 - forbidden\n")
}

fn not_found() -> HttpResponse {
    HttpResponse::NotFound()
        .content_type("text/plain; charset=utf-8")
        .body("404 - not found\n")
}

fn internal_error() -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type("text/plain; charset=utf-8")
        .body("500 - internal server error\n")
}

enum ResolvedPath<'a> {
    File(Cow<'a, Path>),
    ListDir(&'a Path),
    Forbidden,
    NotFound,
}

fn resolve_path_fs(path: &Path, list_directories: bool) -> ResolvedPath {
    if path.exists() {
        if path.is_dir() {
            let index_path = path.join("index.html");
            if index_path.exists() {
                ResolvedPath::File(Cow::Owned(index_path))
            } else if list_directories {
                ResolvedPath::ListDir(path)
            } else {
                ResolvedPath::Forbidden
            }
        } else {
            ResolvedPath::File(Cow::Borrowed(path))
        }
    } else {
        ResolvedPath::NotFound
    }
}

fn list_directory(full_path: &Path, req_path: &str) -> Result<String> {
    let escaped_req_path = htmlescape::encode_minimal(req_path);
    let mut buf = format!(
        r#"<html>
<head><title>Index of /{}</title></head>
<body>
<h1>Index of /{}</h1><hr><pre>
"#,
        escaped_req_path, escaped_req_path
    );

    if !req_path.is_empty() {
        buf.push_str("<a href=\"../\">../</a>\n");
    }

    let iter = fs::read_dir(&full_path).context("Failed to list directory")?;

    let mut listing = Vec::new();
    for entry in iter {
        let entry = entry.context("Failed to get directory entry")?;

        let file_name = entry.file_name();
        // skip filenames with invalid utf8
        if let Ok(file_name) = file_name.into_string() {
            let md = entry
                .metadata()
                .with_context(|| anyhow!("Failed to stat file: {:?}", entry.path()))?;
            listing.push((file_name, md.len()));
        }
    }

    listing.sort();

    for (file_name, size) in listing {
        let padding = DIR_LIST_PADDING.saturating_sub(file_name.len());
        let escaped_name = htmlescape::encode_attribute(&file_name);
        buf.push_str(&format!(
            "<a href=\"{}\">{}</a>{} {}\n",
            escaped_name,
            escaped_name,
            " ".repeat(padding),
            size
        ));
    }

    buf.push_str("</pre><hr></body>\n</html>\n");
    Ok(buf)
}

#[get("/{tail:.*}")]
async fn index(cfg: web::Data<Config>, req: HttpRequest) -> impl Responder {
    let req_path: PathBuf = req.match_info().query("tail").parse().unwrap();
    let path = match resolve_path_req(&cfg.web_root, &req_path) {
        Ok(path) => path,
        Err(err) => {
            debug!("Invalid request path: {:?} ({:#})", req_path, err);
            return bad_request();
        }
    };

    match resolve_path_fs(&path, cfg.list_directories) {
        ResolvedPath::File(path) => {
            let file = match NamedFile::open(&path) {
                Ok(file) => file,
                Err(err) => {
                    warn!("Failed to open file({:?}): {:#}", path, err);
                    return forbidden();
                }
            };

            let res = file
                .prefer_utf8(true)
                .disable_content_disposition()
                .use_etag(false)
                .use_last_modified(false)
                .into_response(&req);

            match res {
                Ok(res) => res,
                Err(err) => {
                    warn!("Failed to create http response for {:?}: {:#}", path, err);
                    internal_error()
                }
            }
        }
        ResolvedPath::ListDir(path) => {
            let req_path = match utils::path_to_string(req_path) {
                Ok(path) => path,
                Err(_) => return bad_request(),
            };

            // if req_path is not empty but doesn't end with /, redirect
            if !req_path.is_empty() && !req_path.ends_with('/') {
                return HttpResponse::Found()
                    .header(header::LOCATION, format!("/{}/", req_path))
                    .finish();
            }

            let listing = match list_directory(path, &req_path) {
                Ok(listing) => listing,
                Err(err) => {
                    warn!("Failed to list directory({:?}): {:#}", path, err);
                    return forbidden();
                }
            };
            HttpResponse::Ok()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(listing)
        }
        ResolvedPath::Forbidden => forbidden(),
        ResolvedPath::NotFound => not_found(),
    }
}

#[actix_web::main]
pub async fn run(args: Args, bind: Bind, web_root: String) -> Result<()> {
    let config = web::Data::new(Config {
        web_root,
        list_directories: args.list_directories,
    });
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                middleware::DefaultHeaders::new()
                    .header(header::DATE, "Thu, 01 Jan 1970 00:00:00 GMT")
                    .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                    .header(header::REFERRER_POLICY, "no-referrer"),
            )
            .wrap(middleware::Compress::default())
            .app_data(config.clone())
            .service(index)
    });

    let server = match bind {
        Bind::Tcp(tcp) => server.listen(tcp),
        #[cfg(unix)]
        Bind::Unix(uds) => server.listen_uds(uds),
    };

    server
        .context("Failed to setup server")?
        .run()
        .await
        .context("Failed to run http server")?;

    warn!("httpd thread has terminated");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("", "/var/www/"; "root")]
    #[test_case("index.html", "/var/www/index.html"; "index.html")]
    #[test_case("a/b/c", "/var/www/a/b/c"; "a b c file")]
    #[test_case("a/b/c/", "/var/www/a/b/c/"; "a b c dir")]
    #[test_case("a//b//c", "/var/www/a/b/c"; "multiple slash")]
    #[test_case("a/b/index.html", "/var/www/a/b/index.html"; "sub index.html")]
    #[test_case("C:\\\\", "/var/www/C:\\\\"; "windows prefix")]
    #[test_case("\\", "/var/www/\\"; "backslash")]
    fn test_valid_resolve_path_req(x: &str, y: &str) {
        let resolved = resolve_path_req("/var/www/", Path::new(x)).unwrap();
        assert_eq!(resolved, Path::new(y));
    }

    #[test_case("./index.html"; "dot slash index.html")]
    #[test_case("/"; "redundant slash")]
    #[test_case("."; "current directory")]
    #[test_case(".."; "parent")]
    #[test_case("a/b/../c"; "a b parent c")]
    fn test_invalid_resolve_path_req(x: &str) {
        let result = resolve_path_req("/var/www/", Path::new(x));
        assert!(result.is_err());
    }
}
