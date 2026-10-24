macro_rules! define_status_codes {
  ($($code: literal => $variant: ident: $message:literal),+ $(,)?) => {
    pub enum StatusCode {
      $($variant,)+

      Unknown(u16),
    }

    impl StatusCode {
      pub fn get_code(&self) -> u16 {
        match *self {
          $(Self::$variant => $code,)+
          Self::Unknown(code) => code,
        }
      }

      pub fn get_message(&self) -> &str {
        match *self {
          $(Self::$variant => $message,)+
          Self::Unknown(_) => "Unknown Status Code",
        }
      }
    }

    impl From<u16> for StatusCode {
      fn from(code: u16) -> Self {
        match code {
          $($code => Self::$variant,)+
          other => Self::Unknown(other),
        }
      }
    }

    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.get_code(), self.get_message())
      }
    }
  };
}

// unashamedly AI generated
define_status_codes! {
  100 => Status100: "Continue",
  101 => Status101: "Switching Protocols",
  102 => Status102: "Processing",
  103 => Status103: "Early Hints",

  200 => Status200: "OK",
  201 => Status201: "Created",
  202 => Status202: "Accepted",
  203 => Status203: "Non-Authoritative Information",
  204 => Status204: "No Content",
  205 => Status205: "Reset Content",
  206 => Status206: "Partial Content",
  207 => Status207: "Multi-Status",
  208 => Status208: "Already Reported",
  226 => Status226: "IM Used",

  300 => Status300: "Multiple Choices",
  301 => Status301: "Moved Permanently",
  302 => Status302: "Found",
  303 => Status303: "See Other",
  304 => Status304: "Not Modified",
  307 => Status307: "Temporary Redirect",
  308 => Status308: "Permanent Redirect",

  400 => Status400: "Bad Request",
  401 => Status401: "Unauthorized",
  402 => Status402: "Payment Required",
  403 => Status403: "Forbidden",
  404 => Status404: "Not Found",
  405 => Status405: "Method Not Allowed",
  406 => Status406: "Not Acceptable",
  407 => Status407: "Proxy Authentication Required",
  408 => Status408: "Request Timeout",
  409 => Status409: "Conflict",
  410 => Status410: "Gone",
  411 => Status411: "Length Required",
  412 => Status412: "Precondition Failed",
  413 => Status413: "Content Too Large",
  414 => Status414: "URI Too Long",
  415 => Status415: "Unsupported Media Type",
  416 => Status416: "Range Not Satisfiable",
  417 => Status417: "Expectation Failed",
  418 => Status418: "I'm a teapot",
  421 => Status421: "Misdirected Request",
  422 => Status422: "Unprocessable Content",
  423 => Status423: "Locked",
  424 => Status424: "Failed Dependency",
  425 => Status425: "Too Early",
  426 => Status426: "Upgrade Required",
  428 => Status428: "Precondition Required",
  429 => Status429: "Too Many Requests",
  431 => Status431: "Request Header Fields Too Large",
  451 => Status451: "Unavailable For Legal Reasons",

  500 => Status500: "Internal Server Error",
  501 => Status501: "Not Implemented",
  502 => Status502: "Bad Gateway",
  503 => Status503: "Service Unavailable",
  504 => Status504: "Gateway Timeout",
  505 => Status505: "HTTP Version Not Supported",
  506 => Status506: "Variant Also Negotiates",
  507 => Status507: "Insufficient Storage",
  508 => Status508: "Loop Detected",
  510 => Status510: "Not Extended",
  511 => Status511: "Network Authentication Required",
}
