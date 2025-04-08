use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
};

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

    // Internal
    _response: URLResponse,
}

impl URL {
    pub fn new(url: String) -> Result<Self, Err> {
        let (scheme, rest) = url.split_once("://").ok_or("URL missing scheme")?;
        let (host, path) = rest.split_once('/').map_or((rest, ""), |(h, p)| (h, p));

        if scheme != "http" {
            return Err(Box::new(URLError {
                message: "HTTPS is not supported".to_string(),
            }));
        }

        Ok(Self {
            scheme: scheme.to_string(),
            host: host.to_string(),
            path: format!("/{}", path),
            _url: url,

            _response: URLResponse::empty(),
        })
    }

    fn read_version_status_explanation(
        &mut self,
        buffer: &mut BufReader<TcpStream>,
    ) -> Result<(), Err> {
        let mut vse_line = String::new();
        buffer.read_line(&mut vse_line)?;

        let vse_line_parts = vse_line.split(' ').collect::<Vec<&str>>();

        self._response._version = vse_line_parts[0].to_string();
        self._response._status = vse_line_parts[1].parse()?;
        self._response._explanation = vse_line_parts[2].replace("\r\n", "");

        Ok(())
    }

    fn read_headers(&mut self, buffer: &mut BufReader<TcpStream>) -> Result<(), Err> {
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

    fn read_body(&mut self, buffer: &mut BufReader<TcpStream>) -> Result<(), Err> {
        buffer.read_to_string(&mut self._response._body)?;

        Ok(())
    }

    fn is_response_encoded(&self) -> bool {
        //  We do not support any compression algo
        self._response._headers.contains_key("transfer-encoding")
            || self._response._headers.contains_key("content-encoding")
    }

    pub fn request(&mut self) -> Result<&String, Err> {
        let mut socket_con =
            TcpStream::connect((self.host.as_str(), 80 as u16)).expect("Could not connect to host");

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construct() {
        let url_result = URL::new("http://www.google.com/search?q=rust".to_string());

        match url_result {
            Ok(url) => {
                assert_eq!(url.scheme, "http");
                assert_eq!(url.host, "www.google.com");
                assert_eq!(url.path, "/search?q=rust");
            }
            _ => {}
        }
    }

    #[test]
    fn test_url_construct_fails_https() {
        let url = URL::new("https://www.google.com/search?q=rust".to_string());

        assert!(url.is_err());
    }

    #[test]
    fn test_response_data() {
        let mut url = URL::new("http://www.google.com/search?q=rust".to_string()).unwrap();
        let response = url.request().unwrap();

        assert!(response.len() > 0);
    }

    #[test]
    fn test_response_returns_first_line() {
        let mut url = URL::new("http://www.google.com/search?q=rust".to_string()).unwrap();
        url.request().unwrap();

        assert_eq!(url._response._version, "HTTP/1.0");
        assert_eq!(url._response._status, 200);
        assert_eq!(url._response._explanation, "OK");
    }

    #[test]
    fn test_response_headers() {
        let mut url = URL::new("http://www.google.com/search?q=rust".to_string()).unwrap();
        url.request().unwrap();

        assert!(url._response._headers.len() > 0);
    }

    #[test]
    fn test_response_body() {
        let mut url = URL::new("http://www.google.com/search?q=rust".to_string()).unwrap();
        url.request().unwrap();

        assert!(url._response._body.len() > 0);
    }
}
