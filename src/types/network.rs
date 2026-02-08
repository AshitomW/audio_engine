//! Network streaming types

use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

use crate::error::{AudioEngineError, Result};

/// Network Streaming Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NetworkProtocol {
    /// Realtime messaging protocol
    #[default]
    RTMP,
    /// HTTP Live streaming (input only)
    HLS,
    /// Real time transport protocol
    RTP,
}

impl NetworkProtocol {
    /// Returns the default port for this protocol
    #[must_use]
    pub const fn default_port(self) -> u16 {
        match self {
            Self::RTMP => 1935,
            Self::HLS => 80,
            Self::RTP => 5004,
        }
    }

    /// Returns the protocol scheme
    #[must_use]
    pub const fn scheme(self) -> &'static str {
        match self {
            Self::RTMP => "rtmp",
            Self::HLS => "https",
            Self::RTP => "rtp",
        }
    }
}

impl fmt::Display for NetworkProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RTMP => write!(f, "RTMP"),
            Self::HLS => write!(f, "HLS"),
            Self::RTP => write!(f, "RTP"),
        }
    }
}

impl FromStr for NetworkProtocol {
    type Err = AudioEngineError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "rtmp" => Ok(Self::RTMP),
            "hls" => Ok(Self::HLS),
            "rtp" => Ok(Self::RTP),
            _ => Err(AudioEngineError::InvalidStreamUrl {
                url: s.to_string(),
                reason: "Unknown protocol".to_string(),
            }),
        }
    }
}

/// Validated stream url
///
///
/// this type ensures urls are validated at parse time
/// and provides type safe access to url components.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamUrl {
    /// Original URL string
    raw: String,
    // Detected Protocol
    protocol: NetworkProtocol,
    /// Host (domain or IP)
    host: String,
    /// Port number
    port: u16,
    /// Path component
    path: String,
    /// Stream key (For RTMP)
    stream_key: Option<String>,
}

impl StreamUrl {
    /// Creates a new stream URL by parsing the given string
    ///
    /// # Errors
    /// Retusn an error if the url is malformed or uses an supported protocol
    pub fn parse(url: &str) -> Result<Self> {
        let url = url.trim();
        // Extract protocol
        let (protocol, rest) = if let Some(rest) = url.strip_prefix("rtmp://") {
            (NetworkProtocol::RTMP, rest)
        } else if let Some(rest) = url.strip_prefix("rtmps://") {
            (NetworkProtocol::RTMP, rest)
        } else if let Some(rest) = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
        {
            (NetworkProtocol::HLS, rest)
        } else if let Some(rest) = url.strip_prefix("rtp://") {
            (NetworkProtocol::RTP, rest)
        } else {
            return Err(AudioEngineError::InvalidStreamUrl {
                url: url.to_string(),
                reason: "Missing or unsupported protocol scheme".to_string(),
            });
        };

        let (host_port, path) = rest.split_once('/').unwrap_or((rest, ""));

        let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
            let port = p.parse().map_err(|_| AudioEngineError::InvalidStreamUrl {
                url: url.to_string(),
                reason: format!("Invalid port: {p}"),
            })?;
            (h.to_string(), port)
        } else {
            (host_port.to_string(), protocol.default_port())
        };

        if host.is_empty() {
            return Err(AudioEngineError::InvalidStreamUrl {
                url: url.to_string(),
                reason: "Empty Host".to_string(),
            });
        }

        let (path, stream_key) = if protocol == NetworkProtocol::RTMP {
            if let Some((p, key)) = path.rsplit_once('/') {
                (p.to_string(), Some(key.to_string()))
            } else if !path.is_empty() {
                (String::new(), Some(path.to_string()))
            } else {
                (path.to_string(), None)
            }
        } else {
            (path.to_string(), None)
        };

        Ok(Self {
            raw: url.to_string(),
            protocol,
            host,
            port,
            path,
            stream_key,
        })
    }

    /// Returns the original url string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Returns the protocol
    #[must_use]
    pub const fn protocol(&self) -> NetworkProtocol {
        self.protocol
    }

    /// Return the host
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the port
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the path
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the stream key for (RTMP)
    #[must_use]
    pub fn stream_key(&self) -> Option<&str> {
        self.stream_key.as_deref()
    }

    /// Attempts to resolve to a socket address.
    /// # Errors
    /// Returns an error if the host cannot be resolved
    pub fn to_socket_addr(&self) -> Result<SocketAddr> {
        use std::net::ToSocketAddrs;

        let addr_str = format!("{}:{}", self.host, self.port);
        addr_str
            .to_socket_addrs()
            .map_err(|e| AudioEngineError::NetworkConnection {
                message: format!("Failed to resolve {addr_str}: {e}"),
            })?
            .next()
            .ok_or_else(|| AudioEngineError::NetworkConnection {
                message: format!("No address found for {addr_str}"),
            })
    }
}

impl FromStr for StreamUrl {
    type Err = AudioEngineError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl fmt::Display for StreamUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// Stream bitrate in bits per second
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamBitrate(u32);
impl StreamBitrate {
    /// 128kbps low quality audio
    pub const KBPS_128: Self = Self(128_000);
    /// 192kbps medium quality audio
    pub const KBPS_192: Self = Self(192_000);
    /// 256kbps good quality audio
    pub const KBPS_256: Self = Self(256_000);
    /// 320kbps high quality audio
    pub const KBPS_320: Self = Self(320_000);

    /// Creates a new bitrate from bits per second
    #[must_use]
    pub const fn from_bps(bps: u32) -> Self {
        Self(bps)
    }

    /// Creates a new bitrate from kilbits per second
    #[must_use]
    pub const fn from_kbps(kbps: u32) -> Self {
        Self(kbps * 1000)
    }

    /// Retruns the bitrate in bits per second
    #[must_use]
    pub const fn as_bps(self) -> u32 {
        self.0
    }

    /// Returnss the bitrate in kilbits per second
    #[must_use]
    pub const fn as_kbps(self) -> u32 {
        self.0 / 1000
    }
}

impl Default for StreamBitrate {
    fn default() -> Self {
        Self::KBPS_192
    }
}

impl fmt::Display for StreamBitrate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} kbps", self.as_kbps())
    }
}
