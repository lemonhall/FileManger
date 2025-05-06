use anyhow::{anyhow, Result};
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;

// --- Response Structs ---
// Based on python model names and typical API responses. Needs verification.

#[derive(Debug, Deserialize, Serialize)] // Added Serialize for potential use
pub struct QuotaResponse {
    pub errno: i32,
    pub request_id: u64,
    pub total: u64, // Total space in bytes
    pub used: u64,  // Used space in bytes
    // pub free: Option<u64>, // Sometimes APIs provide free space directly
    // pub expire: Option<bool>, // Might be related to checkexpire
}

#[derive(Debug, Deserialize, Serialize)] // Added Serialize for potential use
pub struct UserInfoResponse {
    pub errno: i32,
    pub request_id: String,
    pub baidu_name: String,
    pub netdisk_name: String,
    pub avatar_url: String,
    pub vip_type: i32, // Type of VIP (e.g., 0 for non-VIP, 1, 2 for different levels)
    pub uk: u64, // <-- Added User ID field
}

// --- Main Struct ---

pub struct BaiduUserInfo {
    client: Client,
    access_token: String,
}

impl BaiduUserInfo {
    pub fn new(access_token: String) -> Self {
        BaiduUserInfo {
            client: Client::new(), // Consider sharing a client if used alongside BaiduUploader
            access_token,
        }
    }

    // --- API Methods ---

    /// Get user quota information.
    /// Corresponds to Python sdk's apiquota
    pub async fn get_quota(
        &self,
        checkexpire: Option<i32>, // Optional param like in Python sdk
        checkfree: Option<i32>,   // Optional param like in Python sdk
    ) -> Result<QuotaResponse> {
        let base_url = "https://pan.baidu.com/api/quota";
        let mut query_params = vec![("openapi", "xpansdk"), ("access_token", &self.access_token)];
        
        // Use owned strings for optional params to satisfy lifetime requirements if needed
        let checkexpire_str = checkexpire.map(|v| v.to_string());
        let checkfree_str = checkfree.map(|v| v.to_string());

        if let Some(ref val) = checkexpire_str {
            query_params.push(("checkexpire", val));
        }
        if let Some(ref val) = checkfree_str {
            query_params.push(("checkfree", val));
        }

        debug!("Requesting quota with params: {:?}", query_params);

        let resp = self.client.get(base_url)
            .query(&query_params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_else(|_| "Failed to get error body".to_string());
            error!("Get quota API call failed with status {}: {}", status, text);
            return Err(anyhow!("Get quota API call failed with status {}: {}", status, text));
        }

        let quota_resp: QuotaResponse = resp.json().await?;
        debug!("Get quota response: {:?}", quota_resp);

        if quota_resp.errno != 0 {
            error!("Get quota failed with errno: {}", quota_resp.errno);
            // Consider returning the whole response even on error if needed
            return Err(anyhow!("Get quota failed with errno: {}", quota_resp.errno));
        }

        Ok(quota_resp)
    }

    /// Get user basic information.
    /// Corresponds to Python sdk's xpannasuinfo
    pub async fn get_user_info(&self) -> Result<UserInfoResponse> {
        let base_url = "https://pan.baidu.com/rest/2.0/xpan/nas";
        let query_params = [
            ("method", "uinfo"), 
            ("openapi", "xpansdk"), 
            ("access_token", &self.access_token)
        ];

        debug!("Requesting user info...");

        let resp = self.client.get(base_url)
            .query(&query_params)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            // Keep existing error handling for non-success status codes
            let text = resp.text().await.unwrap_or_else(|_| "Failed to get error body".to_string());
            error!("Get user info API call failed with status {}: {}", status, text);
            return Err(anyhow!("Get user info API call failed with status {}: {}", status, text));
        }

        // Read the response body as text first for debugging
        let raw_body = resp.text().await?;
        debug!("Raw user info response body: {}", raw_body);

        // Now try to deserialize the raw body
        let user_info_resp: UserInfoResponse = match serde_json::from_str(&raw_body) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to decode user info JSON: {}. Raw body was: {}", e, raw_body);
                return Err(anyhow!("error decoding response body: {}", e)); // Match the error message seen by user
            }
        };

        // Check Baidu's internal error number AFTER successful decoding
        if user_info_resp.errno != 0 {
            error!("Get user info failed with errno: {}", user_info_resp.errno);
            // Consider returning the error with the decoded struct if partial info is useful
            return Err(anyhow!("Get user info failed with errno: {}", user_info_resp.errno));
        }

        Ok(user_info_resp)
    }
} 