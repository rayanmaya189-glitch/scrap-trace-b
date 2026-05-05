use aws_sdk_s3::{Client, Config, Region, primitives::ByteStream};
use aws_credential_types::Credentials;
use std::time::Duration;
use tracing::{info, error};
use crate::utils::error::AppError;

pub struct MinioManager {
    pub client: Client,
    pub bucket: String,
}

impl MinioManager {
    pub fn new(endpoint: &str, access_key: &str, secret_key: &str, bucket: &str) -> Result<Self, AppError> {
        // Parse endpoint to extract region if present
        let region = Region::new("us-east-1"); // MinIO typically uses us-east-1
        
        // Create credentials
        let creds = Credentials::new(access_key, secret_key, None, None, "minio");
        
        // Configure S3 client for MinIO
        let config = Config::builder()
            .credentials_provider(creds)
            .region(region)
            .endpoint_url(endpoint)
            .force_path_style(true) // Required for MinIO
            .build();
        
        let client = Client::from_conf(config);
        
        info!("✅ MinIO client initialized for bucket: {}", bucket);
        
        Ok(MinioManager {
            client,
            bucket: bucket.to_string(),
        })
    }
    
    /// Upload file to MinIO
    pub async fn upload_file(
        &self,
        object_name: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, AppError> {
        let byte_stream = ByteStream::from(data);
        
        let result = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(object_name)
            .body(byte_stream)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("MinIO upload failed: {}", e)))?;
        
        info!("File uploaded to MinIO: {}/{}", self.bucket, object_name);
        
        // Return the object key (can be used to generate presigned URL)
        Ok(object_name.to_string())
    }
    
    /// Generate presigned URL for downloading a file
    pub async fn generate_presigned_url(
        &self,
        object_name: &str,
        expiry_seconds: u64,
    ) -> Result<String, AppError> {
        use aws_sdk_s3::presigning::PresigningConfig;
        use std::time::SystemTime;
        
        let presign_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(expiry_seconds))
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Presign config failed: {}", e)))?;
        
        let presigned_request = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(object_name)
            .presigned(presign_config)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Presign generation failed: {}", e)))?;
        
        Ok(presigned_request.uri().to_string())
    }
    
    /// Generate presigned URL for uploading (PUT)
    pub async fn generate_presigned_upload_url(
        &self,
        object_name: &str,
        content_type: &str,
        expiry_seconds: u64,
    ) -> Result<String, AppError> {
        use aws_sdk_s3::presigning::PresigningConfig;
        use std::time::SystemTime;
        
        let presign_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(expiry_seconds))
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Presign config failed: {}", e)))?;
        
        let presigned_request = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(object_name)
            .content_type(content_type)
            .presigned(presign_config)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Presign upload URL generation failed: {}", e)))?;
        
        Ok(presigned_request.uri().to_string())
    }
    
    /// Delete file from MinIO
    pub async fn delete_file(&self, object_name: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("MinIO delete failed: {}", e)))?;
        
        info!("File deleted from MinIO: {}/{}", self.bucket, object_name);
        Ok(())
    }
    
    /// Check if file exists
    pub async fn file_exists(&self, object_name: &str) -> Result<bool, AppError> {
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.into_service_error().is_not_found() {
                    Ok(false)
                } else {
                    Err(AppError::InternalServerError(format!("MinIO head_object failed: {}", e)))
                }
            }
        }
    }
    
    /// Get file metadata
    pub async fn get_file_metadata(
        &self,
        object_name: &str,
    ) -> Result<(u64, String), AppError> {
        let result = self.client
            .head_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("MinIO head_object failed: {}", e)))?;
        
        let size = result.content_length().unwrap_or(0) as u64;
        let content_type = result.content_type().unwrap_or("application/octet-stream").to_string();
        
        Ok((size, content_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires running MinIO instance
    async fn test_minio_upload() {
        let manager = MinioManager::new(
            "http://localhost:9000",
            "minioadmin",
            "minioadmin",
            "test-bucket",
        ).unwrap();
        
        let data = b"Hello, MinIO!".to_vec();
        let result = manager.upload_file("test.txt", data, "text/plain").await;
        
        assert!(result.is_ok());
    }
}
