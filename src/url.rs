use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    sync::Arc,
};

use rustls as tls;

type Err = Box<dyn Error>;

#[derive(Debug)]
pub struct URLError {
    pub message: String,
}

impl Display for URLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for URLError {}

struct URLResponse {
    _version: String,
    _status: usize,
    _explanation: String,
    _headers: HashMap<String, String>,
    _body: String,
}

impl URLResponse {
    fn empty() -> Self {
        Self {
            _version: "".to_string(),
            _status: 0,
            _explanation: "".to_string(),
            _headers: HashMap::new(),
            _body: "".to_string(),
        }
    }
}

pub struct URL {
    // Original URL
    _url: String,
    scheme: String,
    host: String,
    path: String,
    port: Option<u16>,

    // Internal
    _response: URLResponse,
}

impl URL {
    pub fn new(url: String) -> Result<Self, Err> {
        let (scheme, rest) = url
            .split_once("://")
            .ok_or(Self::err("URL scheme missing"))?;
        let (mut host, path) = rest.split_once('/').map_or((rest, ""), |(h, p)| (h, p));

        let mut port = None;

        if host.contains(":") {
            let (_host, _port) = host.split_once(":").ok_or(Self::err("URL missing port"))?;
            host = _host;
            port = if _port.is_empty() {
                None
            } else {
                Some(_port.parse().unwrap())
            };
        }

        Ok(Self {
            scheme: scheme.to_string(),
            host: host.to_string(),
            path: format!("/{}", path),
            _url: url,
            port,

            _response: URLResponse::empty(),
        })
    }

    fn err(message: &str) -> URLError {
        URLError {
            message: message.to_string(),
        }
    }

    fn read_version_status_explanation<T>(&mut self, buffer: &mut BufReader<T>) -> Result<(), Err>
    where
        T: Read,
    {
        let mut vse_line = String::new();
        buffer.read_line(&mut vse_line)?;

        let vse_line_parts = vse_line.split(' ').collect::<Vec<&str>>();

        self._response._version = vse_line_parts[0].to_string();
        self._response._status = vse_line_parts[1].parse()?;
        self._response._explanation = vse_line_parts[2].replace("\r\n", "");

        Ok(())
    }

    fn read_headers<T>(&mut self, buffer: &mut BufReader<T>) -> Result<(), Err>
    where
        T: Read,
    {
        loop {
            let mut header_line = String::new();
            buffer.read_line(&mut header_line)?;

            if header_line == "\r\n" {
                break;
            }

            let (header_key, header_value) =
                header_line.split_once(':').ok_or("Error reading header")?;
            self._response._headers.insert(
                header_key.to_lowercase(),
                header_value.trim().to_lowercase(),
            );
        }
        Ok(())
    }

    fn read_body<T>(&mut self, buffer: &mut BufReader<T>) -> Result<(), Err>
    where
        T: Read,
    {
        buffer.read_to_string(&mut self._response._body)?;

        Ok(())
    }

    fn is_response_encoded(&self) -> bool {
        //  We do not support any compression algo
        self._response._headers.contains_key("transfer-encoding")
            || self._response._headers.contains_key("content-encoding")
    }

    fn get_port(&self) -> u16 {
        match self.port {
            Some(_port) => _port,
            None => {
                if self.is_https() {
                    443
                } else {
                    80
                }
            }
        }
    }

    fn is_https(&self) -> bool {
        self.scheme == "https"
    }

    fn create_conn(&self) -> TcpStream {
        TcpStream::connect((self.host.as_str(), self.get_port()))
            .expect("Could not connect to host")
    }

    fn http_request(&mut self) -> Result<&String, Err> {
        let mut socket_con = self.create_conn();

        write!(socket_con, "GET {} HTTP/1.0\r\n", self.path)?;
        write!(socket_con, "Host: {}\r\n", self.host)?;
        // When testing with google URL, user agent is required to return UTF-8 otherwise is ISO-8859-1
        write!(socket_con, "User-Agent: Mozilla/5.0\r\n")?;
        write!(socket_con, "\r\n")?;

        let mut buf = BufReader::new(socket_con);

        self.read_version_status_explanation(&mut buf)?;
        self.read_headers(&mut buf)?;

        if self.is_response_encoded() {
            return Err(Box::new(URLError {
                message: "Unsupported encodded content".to_string(),
            }));
        }

        self.read_body(&mut buf)?;

        Ok(&self._response._body)
    }

    fn https_request(&mut self) -> Result<&String, Err> {
        let mut sock = self.create_conn();
        let root_store = tls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };

        let mut config = tls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Allow using SSLKEYLOGFILE.
        config.key_log = Arc::new(tls::KeyLogFile::new());

        let server_name = self.host.clone().try_into().unwrap();

        let mut conn = tls::ClientConnection::new(Arc::new(config), server_name).unwrap();
        let mut socket_con = tls::Stream::new(&mut conn, &mut sock);

        write!(socket_con, "GET {} HTTP/1.0\r\n", self.path)?;
        write!(socket_con, "Host: {}\r\n", self.host)?;
        // When testing with google URL, user agent is required to return UTF-8 otherwise is ISO-8859-1
        write!(socket_con, "User-Agent: Mozilla/5.0\r\n")?;
        write!(socket_con, "\r\n")?;

        let mut buf = BufReader::new(socket_con);

        self.read_version_status_explanation(&mut buf)?;
        self.read_headers(&mut buf)?;

        if self.is_response_encoded() {
            return Err(Box::new(URLError {
                message: "Unsupported encodded content".to_string(),
            }));
        }

        self.read_body(&mut buf)?;

        Ok(&self._response._body)
    }

    pub fn request(&mut self) -> Result<&String, Err> {
        if self.is_https() {
            self.https_request()
        } else {
            self.http_request()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construct() {
        let url_result =
            URL::new("https://browser.engineering/examples/example1-simple.html".to_string());

        match url_result {
            Ok(url) => {
                assert_eq!(url.scheme, "https");
                assert_eq!(url.host, "browser.engineering");
                assert_eq!(url.path, "/examples/example1-simple.html");
            }
            _ => {}
        }
    }

    #[test]
    fn test_url_construct_fails_https() {
        let url = URL::new("https://browser.engineering/examples/example1-simple.html".to_string());

        assert!(url.is_ok());
    }

    #[test]
    fn test_response_data() {
        let mut url =
            URL::new("https://browser.engineering/examples/example1-simple.html".to_string())
                .unwrap();
        let response = url.request().unwrap();

        assert!(response.len() > 0);
    }

    #[test]
    fn test_response_returns_first_line() {
        let mut url =
            URL::new("https://browser.engineering/examples/example1-simple.html".to_string())
                .unwrap();
        url.request().unwrap();

        assert_eq!(url._response._version, "HTTP/1.1");
        assert_eq!(url._response._status, 200);
        assert_eq!(url._response._explanation, "OK");
    }

    #[test]
    fn test_response_headers() {
        let mut url =
            URL::new("https://browser.engineering/examples/example1-simple.html".to_string())
                .unwrap();
        url.request().unwrap();

        assert!(url._response._headers.len() > 0);
    }

    #[test]
    fn test_response_body() {
        let mut url =
            URL::new("https://browser.engineering/examples/example1-simple.html".to_string())
                .unwrap();
        url.request().unwrap();

        assert!(url._response._body.len() > 0);
    }

    #[test]
    fn test_http() {
        let mut url =
            URL::new("http://browser.engineering/examples/example1-simple.html".to_string())
                .unwrap();
        url.request().unwrap();

        assert!(url._response._body.len() > 0);
    }
}
