use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;
use tokio_util::codec::{BytesCodec, FramedRead};

const CHUNK_SIZE: u64 = 4 * 1024 * 1024; // 4MB
// ... existing code ... 