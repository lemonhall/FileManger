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
// use tokio_util::codec::{BytesCodec, FramedRead};

const CHUNK_SIZE: u64 = 4 * 1024 * 1024; // 4MB

// Helper structure for API responses (can be more specific later)
#[derive(Debug, Serialize, Deserialize)]
struct BaiduApiResponse {
    errno: Option<i32>,
    request_id: Option<u64>,
    // Other fields depend on the specific API call
    // For precreate:
    uploadid: Option<String>,
    // For create:
    fs_id: Option<u64>,
    md5: Option<String>, // sometimes returned by pcssuperfile2 or create
    // We'll use serde_json::Value for flexibility for now or create specific structs
}

#[derive(Debug, Serialize, Deserialize)]
struct PrecreateResponse {
    errno: i32,
    path: Option<String>,
    uploadid: Option<String>,
    return_type: Option<i32>, // Typo in some docs? Should be rtype or type?
    block_list: Option<Vec<i32>>, // Or Vec<String> if it's a list of MD5s for precreate
    request_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateResponse {
    errno: i32,
    fs_id: u64,
    md5: String,
    server_filename: String,
    category: i32,
    path: String,
    size: u64,
    ctime: u64,
    mtime: u64,
    isdir: i32,
    request_id: u64,
}


pub struct BaiduUploader {
    client: Client,
    access_token: String,
}

impl BaiduUploader {
    pub fn new(access_token: String) -> Self {
        BaiduUploader {
            client: Client::new(),
            access_token,
        }
    }

    // Placeholder for the main upload function
    pub async fn upload_file(
        &self,
        local_file_path: &str,
        remote_path_dir: &str, // Directory on Baidu Pan, filename will be appended
    ) -> Result<Value> { // Returning serde_json::Value for now, can be a specific struct
        info!("Starting upload for {} to {}", local_file_path, remote_path_dir);

        let path = Path::new(local_file_path);
        if !path.is_file() {
            return Err(anyhow!("Local file not found: {}", local_file_path));
        }

        let file_name = path.file_name().ok_or_else(|| anyhow!("Invalid file path"))?.to_string_lossy().to_string();
        let full_remote_path = format!("{}/{}", remote_path_dir.trim_end_matches('/'), file_name);
        
        let file_size = std::fs::metadata(local_file_path)?.len();

        if file_size <= CHUNK_SIZE {
            self.upload_small_file(local_file_path, &full_remote_path, file_size).await
        } else {
            self.upload_large_file(local_file_path, &full_remote_path, file_size).await
        }
    }

    async fn upload_small_file(
        &self,
        local_file_path: &str,
        remote_path: &str,
        file_size: u64,
    ) -> Result<Value> {
        debug!("Uploading small file: {} to {}", local_file_path, remote_path);
        
        let mut file_content = Vec::new();
        File::open(local_file_path)?.read_to_end(&mut file_content)?;
        
        let file_md5 = format!("{:x}", md5::compute(&file_content));
        let block_list_json = serde_json::to_string(&vec![&file_md5])?;

        // 1. Precreate
        // API: http://pan.baidu.com/rest/2.0/xpan/file?method=precreate
        // Method: POST
        // Params: access_token, path, isdir=0, size, autoinit=1, block_list (JSON array of MD5s), rtype=3 (specific to some client types)
        let precreate_params = serde_json::json!({
            "path": remote_path,
            "isdir": 0,
            "size": file_size,
            "autoinit": 1, // auto init, for small files can be 1
            "block_list": block_list_json,
            "rtype": 3 // Or other rtype if needed, 3 often means complete overwrite/direct create
        });
        
        let precreate_url = "https://pan.baidu.com/rest/2.0/xpan/file";
        let precreate_resp = self.client.post(precreate_url)
            .query(&[("method", "precreate"), ("access_token", &self.access_token)])
            .form(&precreate_params) // Should be form for POST, or json if API expects JSON body
            .send()
            .await?;

        if !precreate_resp.status().is_success() {
            let error_text = precreate_resp.text().await?;
            error!("Precreate failed: {}", error_text);
            return Err(anyhow!("Precreate API call failed: {}", error_text));
        }

        let precreate_json: Value = precreate_resp.json().await?;
        debug!("Precreate response: {:?}", precreate_json);
        if precreate_json["errno"].as_i64().unwrap_or(-1) != 0 {
             return Err(anyhow!("Precreate failed with errno: {:?}", precreate_json));
        }
        let uploadid = precreate_json["uploadid"].as_str().ok_or_else(|| anyhow!("Missing uploadid in precreate response"))?.to_string();


        // 2. Upload (pcssuperfile2)
        // API: https://d.pcs.baidu.com/rest/2.0/pcs/superfile2?method=upload
        // Method: POST (multipart/form-data)
        // Params: access_token, partseq=0, path, uploadid, type=tmpfile
        // Body: file content
        
        let part = Part::bytes(file_content.clone()).file_name(Path::new(local_file_path).file_name().unwrap().to_string_lossy().into_owned());
        let form = Form::new().part("file", part);

        let pcs_url = format!("https://d.pcs.baidu.com/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&path={}&uploadid={}&partseq=0", 
            self.access_token, 
            urlencoding::encode(remote_path), // Path needs to be URL encoded
            uploadid
        );

        let upload_resp = self.client.post(&pcs_url)
            .multipart(form)
            .send()
            .await?;

        if !upload_resp.status().is_success() {
            let error_text = upload_resp.text().await?;
            error!("PCS Superfile2 upload failed: {}", error_text);
            return Err(anyhow!("PCS Superfile2 API call failed: {}", error_text));
        }
        let upload_json: Value = upload_resp.json().await?;
        debug!("PCS Superfile2 response: {:?}", upload_json);
        // PCS superfile2 might return an MD5 of the uploaded content or other status.
        // Python code checks: if 'md5' not in upload_response or not upload_response['md5']
        if upload_json.get("md5").and_then(Value::as_str).filter(|s| !s.is_empty()).is_none() {
             return Err(anyhow!("PCS Superfile2 response missing MD5 or it's empty: {:?}", upload_json));
        }


        // 3. Create
        // API: http://pan.baidu.com/rest/2.0/xpan/file?method=create
        // Method: POST
        // Params: access_token, path, isdir=0, size, uploadid, block_list (JSON array of MD5s), rtype=3
         let create_params = serde_json::json!({
            "path": remote_path,
            "isdir": 0,
            "size": file_size,
            "uploadid": uploadid,
            "block_list": block_list_json, // Same block_list as precreate for small files
            "rtype": 3
        });
        
        let create_url = "https://pan.baidu.com/rest/2.0/xpan/file";
        let create_resp = self.client.post(create_url)
            .query(&[("method", "create"), ("access_token", &self.access_token)])
            .form(&create_params) // Should be form for POST
            .send()
            .await?;

        if !create_resp.status().is_success() {
            let error_text = create_resp.text().await?;
            error!("Create file failed: {}", error_text);
            return Err(anyhow!("Create file API call failed: {}", error_text));
        }
        let create_json: Value = create_resp.json().await?;
        debug!("Create file response: {:?}", create_json);
        if create_json["errno"].as_i64().unwrap_or(-1) != 0 {
             return Err(anyhow!("Create file failed with errno: {:?}", create_json));
        }

        info!("Small file {} uploaded successfully to {}", local_file_path, remote_path);
        Ok(create_json)
    }

    async fn upload_large_file(
        &self,
        local_file_path: &str,
        remote_path: &str,
        file_size: u64,
    ) -> Result<Value> {
        debug!("Uploading large file: {} to {} (size: {})", local_file_path, remote_path, file_size);

        let mut md5_list: Vec<String> = Vec::new();
        let mut file = File::open(local_file_path)?;
        let mut buffer = vec![0; CHUNK_SIZE as usize]; // Read in chunks

        // Calculate MD5 for all chunks first
        let num_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
        debug!("Number of chunks: {}", num_chunks);

        for i in 0..num_chunks {
            let bytes_to_read = if i == num_chunks - 1 {
                (file_size % CHUNK_SIZE) as usize
            } else {
                CHUNK_SIZE as usize
            };
             // Ensure buffer is correct size for last chunk if smaller
            let current_chunk_size = if bytes_to_read == 0 && file_size > 0 && i == num_chunks -1 { // handles case where last chunk is exactly CHUNK_SIZE
                CHUNK_SIZE as usize
            } else if bytes_to_read == 0 && file_size == 0 { // empty file
                0
            } else if bytes_to_read == 0 && i < num_chunks -1 { // should not happen if CHUNK_SIZE > 0
                 CHUNK_SIZE as usize
            }
            else {
                bytes_to_read
            };

            if current_chunk_size == 0 && file_size > 0 { // If last chunk was exactly CHUNK_SIZE, bytes_to_read would be 0 from modulo
                 let mut chunk_data = vec![0; CHUNK_SIZE as usize];
                 file.read_exact(&mut chunk_data)?;
                 let chunk_md5 = format!("{:x}", md5::compute(&chunk_data));
                 md5_list.push(chunk_md5);
            } else if current_chunk_size > 0 {
                let mut chunk_data = vec![0; current_chunk_size];
                file.read_exact(&mut chunk_data)?;
                let chunk_md5 = format!("{:x}", md5::compute(&chunk_data));
                md5_list.push(chunk_md5);
            }
        }
        
        let block_list_json = serde_json::to_string(&md5_list)?;
        debug!("Block list for large file: {}", block_list_json);

        // 1. Precreate
        let precreate_params = serde_json::json!({
            "path": remote_path,
            "isdir": 0,
            "size": file_size,
            "autoinit": 1, // autoinit=1 even for large files based on Python code's precreate
            "block_list": block_list_json, // This is the list of chunk MD5s
            "rtype": 3
        });
        
        let precreate_url = "https://pan.baidu.com/rest/2.0/xpan/file";
        let precreate_resp = self.client.post(precreate_url)
            .query(&[("method", "precreate"), ("access_token", &self.access_token)])
            .form(&precreate_params)
            .send()
            .await?;
        
        if !precreate_resp.status().is_success() {
            let error_text = precreate_resp.text().await?;
            error!("Large file precreate failed: {}", error_text);
            return Err(anyhow!("Large file precreate API call failed: {}", error_text));
        }

        let precreate_json: Value = precreate_resp.json().await?;
        debug!("Large file precreate response: {:?}", precreate_json);
        if precreate_json["errno"].as_i64().unwrap_or(-1) != 0 {
             return Err(anyhow!("Large file precreate failed with errno: {:?}", precreate_json));
        }
        let uploadid = precreate_json["uploadid"].as_str().ok_or_else(|| anyhow!("Missing uploadid in large file precreate response"))?.to_string();


        // 2. Upload Chunks (pcssuperfile2)
        // Re-open file for chunked reading for upload
        let mut file_for_upload = TokioFile::open(local_file_path).await?;

        for (i, _chunk_md5) in md5_list.iter().enumerate() {
            let part_seq = i.to_string();
            
            // Seek to the correct position for the current chunk
            // TokioFile does not directly support seek, so we might need to read sequentially
            // or use std::fs::File and wrap it for async read if precise seeking is needed for retries.
            // For a simple sequential upload:
            let mut chunk_buffer = vec![0u8; CHUNK_SIZE as usize]; // Standard chunk size
            let bytes_read = file_for_upload.read(&mut chunk_buffer).await?;

            if bytes_read == 0 { // Should not happen if md5_list is correct
                break;
            }
            let actual_chunk_data = &chunk_buffer[..bytes_read];


            let file_part = Part::bytes(actual_chunk_data.to_vec()) // Clone data for Part
                .file_name(Path::new(local_file_path).file_name().unwrap().to_string_lossy().into_owned()) // Use original filename
                .mime_str("application/octet-stream")?; // Set appropriate MIME type

            let form = Form::new().part("file", file_part);
            
            let pcs_url = format!("https://d.pcs.baidu.com/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&path={}&uploadid={}&partseq={}", 
                self.access_token, 
                urlencoding::encode(remote_path), 
                uploadid,
                part_seq
            );

            debug!("Uploading chunk {} of {}", i + 1, num_chunks);
            let upload_resp = self.client.post(&pcs_url)
                .multipart(form)
                .send()
                .await?;

            if !upload_resp.status().is_success() {
                let error_text = upload_resp.text().await?;
                error!("PCS Superfile2 chunk upload failed for part {}: {}", part_seq, error_text);
                return Err(anyhow!("PCS Superfile2 chunk API call failed for part {}: {}", part_seq, error_text));
            }
            let upload_json: Value = upload_resp.json().await?;
            debug!("PCS Superfile2 chunk {} response: {:?}", part_seq, upload_json);
            // Check for MD5 or error in response if API provides it per chunk
             if upload_json.get("md5").and_then(Value::as_str).filter(|s| !s.is_empty()).is_none() {
                // Some APIs might not return MD5 for every chunk, but might return an error code
                // The Python code example implies we only care if it *doesn't* return MD5, meaning it might be an error.
                // Or, it might only return an overall MD5 at the 'create' step.
                // For now, we'll be strict like the Python code for small files.
                // If chunk upload has a specific error field, check that.
                // For now, if no MD5, assume it *could* be an issue if not success.
                // The python code checks: if 'md5' not in upload_response or not upload_response['md5']
                // This check might be too strict for chunks if the API doesn't return MD5 per chunk.
                // Let's assume for now it should return an MD5 or a clear error.
                // If the overall create step later verifies the full file MD5, this might be less critical.
                warn!("PCS Superfile2 chunk response for part {} missing MD5 or it's empty: {:?}. Proceeding, but final create step is crucial.", part_seq, upload_json);
            }
        }

        // 3. Create (Merge Chunks)
        let create_params = serde_json::json!({
            "path": remote_path,
            "isdir": 0,
            "size": file_size,
            "uploadid": uploadid,
            "block_list": block_list_json, // Full list of chunk MD5s
            "rtype": 3
        });
        
        let create_url = "https://pan.baidu.com/rest/2.0/xpan/file";
        let create_resp = self.client.post(create_url)
            .query(&[("method", "create"), ("access_token", &self.access_token)])
            .form(&create_params)
            .send()
            .await?;

        if !create_resp.status().is_success() {
            let error_text = create_resp.text().await?;
            error!("Large file create (merge) failed: {}", error_text);
            return Err(anyhow!("Large file create (merge) API call failed: {}", error_text));
        }
        let create_json: Value = create_resp.json().await?;
        debug!("Large file create (merge) response: {:?}", create_json);
         if create_json["errno"].as_i64().unwrap_or(-1) != 0 {
             return Err(anyhow!("Large file create (merge) failed with errno: {:?}", create_json));
        }
        
        info!("Large file {} uploaded successfully to {}", local_file_path, remote_path);
        Ok(create_json)
    }
}

// Basic test function (not a unit test, just for illustration)
// To use this, you'd need to set up an async runtime like tokio.
// #[tokio::main]
// async fn main_test() -> Result<()> {
//     // Configure logging (e.g., env_logger::init())
//     let token = std::env::var("BAIDU_NETDISK_ACCESS_TOKEN")
//         .map_err(|_| anyhow!("BAIDU_NETDISK_ACCESS_TOKEN not set"))?;
//     let uploader = BaiduUploader::new(token);

//     // Create a dummy small file
//     let small_file_path = "./dummy_small_file.txt";
//     std::fs::write(small_file_path, "Hello Baidu Netdisk!")?;
    
//     // Create a dummy large file (e.g., > 4MB)
//     let large_file_path = "./dummy_large_file.bin";
//     let mut large_file = std::fs::File::create(large_file_path)?;
//     let data_chunk = vec![0u8; 1024 * 1024]; // 1MB
//     for _ in 0..5 { // Create a 5MB file
//         use std::io::Write;
//         large_file.write_all(&data_chunk)?;
//     }
//     drop(large_file);


//     match uploader.upload_file(small_file_path, "/来自：mcp_server_rust_test").await {
//         Ok(resp) => info!("Small file upload success: {:?}", resp),
//         Err(e) => error!("Small file upload error: {:?}", e),
//     }

//     match uploader.upload_file(large_file_path, "/来自：mcp_server_rust_test").await {
//         Ok(resp) => info!("Large file upload success: {:?}", resp),
//         Err(e) => error!("Large file upload error: {:?}", e),
//     }
    
//     // Cleanup dummy files
//     std::fs::remove_file(small_file_path)?;
//     std::fs::remove_file(large_file_path)?;

//     Ok(())
// } 