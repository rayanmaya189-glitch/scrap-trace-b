use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use crate::models::ApiResponse;
use crate::utils::error::{AppError, AppResult};

/// Upload evidence file for dispute
pub async fn upload_evidence(
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
            
            let data = field.bytes().await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to read file data: {}", e))
            })?;
            
            // In production, upload to MinIO here
            // For now, we'll generate a presigned URL placeholder
            let file_id = Uuid::new_v4();
            let file_url = format!("/evidence/{}", file_id);
            
            // TODO: Implement MinIO upload
            // let minio_client = state.minio_client;
            // let bucket = "btrace-evidence";
            // let object_name = format!("disputes/{}/{}", chrono::Utc::now().format("%Y/%m/%d"), file_id);
            // minio_client.put_object(bucket, &object_name, &data).await?;
            // let presigned_url = minio_client.presigned_get_url(bucket, &object_name, 3600).await?;
            
            uploaded_files.push(serde_json::json!({
                "id": file_id,
                "filename": filename,
                "url": file_url,
                "size": data.len(),
                "content_type": field.content_type().map(|s| s.to_string()).unwrap_or_default()
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
            "Files uploaded successfully".to_string(),
        )),
    ))
}
