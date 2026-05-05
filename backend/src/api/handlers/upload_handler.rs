use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::services::MinioManager;
use crate::models::ApiResponse;
use crate::utils::error::{AppError, AppResult};

/// Application state for upload routes
pub type UploadState = Arc<MinioManager>;

/// Upload evidence file for dispute
pub async fn upload_evidence(
    State(minio_manager): State<UploadState>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    let mut uploaded_files = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("unknown").to_string();
        
        if name == "file" {
            let filename = field.file_name().map(|s| s.to_string())
                .unwrap_or_else(|| format!("evidence_{}.dat", Uuid::new_v4()));
            
            let content_type = field.content_type().map(|s| s.to_string()).unwrap_or_else(|| "application/octet-stream".to_string());
            
            let data = field.bytes().await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to read file data: {}", e))
            })?;
            
            // Generate unique object key for MinIO
            let file_id = Uuid::new_v4();
            let date_prefix = chrono::Utc::now().format("%Y/%m/%d");
            let object_key = format!("disputes/{}/{}", date_prefix, file_id);
            
            // Upload to MinIO
            let uploaded_key = minio_manager
                .upload_file(&object_key, data.to_vec(), &content_type)
                .await?;
            
            // Generate presigned URL for access (valid for 7 days)
            let presigned_url = minio_manager
                .generate_presigned_url(&uploaded_key, 604800) // 7 days in seconds
                .await?;
            
            uploaded_files.push(serde_json::json!({
                "id": file_id,
                "filename": filename,
                "url": presigned_url,
                "object_key": uploaded_key,
                "size": data.len(),
                "content_type": content_type,
                "uploaded_at": chrono::Utc::now().to_rfc3339()
            }));
        }
    }
    
    if uploaded_files.is_empty() {
        return Err(AppError::BadRequest("No files uploaded".to_string()));
    }
    
    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            serde_json::json!({
                "files": uploaded_files,
                "count": uploaded_files.len()
            }),
            "Files uploaded successfully to secure storage".to_string(),
        )),
    ))
}
