import { useState, useCallback } from 'react';

interface UseFileUploadOptions {
  accept?: string;
  maxSize?: number; // in bytes
  multiple?: boolean;
}

interface FileWithPreview extends File {
  preview: string;
  id: string;
}

interface UseFileUploadReturn {
  files: FileWithPreview[];
  isUploading: boolean;
  error: string | null;
  progress: number;
  selectFiles: (files: FileList | null) => void;
  removeFile: (fileId: string) => void;
  clearFiles: () => void;
  uploadProgress: Record<string, number>;
}

/**
 * Custom hook for file selection and upload management
 * Supports image previews, size validation, and progress tracking
 */
export function useFileUpload(options: UseFileUploadOptions = {}): UseFileUploadReturn {
  const {
    accept,
    maxSize = 10 * 1024 * 1024, // 10MB default
    multiple = true,
  } = options;

  const [files, setFiles] = useState<FileWithPreview[]>([]);
  const [isUploading, setIsUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [uploadProgress, setUploadProgress] = useState<Record<string, number>>({});

  const validateFile = (file: File): string | null => {
    // Check file type
    if (accept) {
      const acceptedTypes = accept.split(',').map(type => type.trim());
      const fileType = file.type;
      const fileExtension = '.' + file.name.split('.').pop()?.toLowerCase();
      
      const isAccepted = acceptedTypes.some(type => {
        if (type.startsWith('.')) {
          return fileExtension === type.toLowerCase();
        }
        if (type.endsWith('/*')) {
          const baseType = type.split('/')[0];
          return fileType.startsWith(baseType + '/');
        }
        return fileType === type;
      });

      if (!isAccepted) {
        return `File type "${file.type}" is not accepted. Accepted types: ${accept}`;
      }
    }

    // Check file size
    if (file.size > maxSize) {
      const maxSizeMB = (maxSize / (1024 * 1024)).toFixed(2);
      return `File "${file.name}" exceeds maximum size of ${maxSizeMB}MB`;
    }

    return null;
  };

  const selectFiles = useCallback((fileList: FileList | null) => {
    if (!fileList || fileList.length === 0) {
      return;
    }

    setError(null);
    const newFiles: FileWithPreview[] = [];
    const errors: string[] = [];

    Array.from(fileList).forEach(file => {
      const validationError = validateFile(file);
      
      if (validationError) {
        errors.push(validationError);
        return;
      }

      // Create preview for images
      let preview = '';
      if (file.type.startsWith('image/')) {
        preview = URL.createObjectURL(file);
      }

      const fileWithPreview: FileWithPreview = {
        ...file,
        preview,
        id: `${file.name}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      };

      newFiles.push(fileWithPreview);
    });

    if (errors.length > 0) {
      setError(errors.join('\n'));
    }

    if (multiple) {
      setFiles(prev => [...prev, ...newFiles]);
    } else {
      // For single file, keep only the first valid file
      setFiles(newFiles.slice(0, 1));
    }
  }, [accept, maxSize, multiple]);

  const removeFile = useCallback((fileId: string) => {
    setFiles(prev => {
      const fileToRemove = prev.find(f => f.id === fileId);
      if (fileToRemove?.preview) {
        URL.revokeObjectURL(fileToRemove.preview);
      }
      return prev.filter(f => f.id !== fileId);
    });
    
    setUploadProgress(prev => {
      const newProgress = { ...prev };
      delete newProgress[fileId];
      return newProgress;
    });
  }, []);

  const clearFiles = useCallback(() => {
    files.forEach(file => {
      if (file.preview) {
        URL.revokeObjectURL(file.preview);
      }
    });
    setFiles([]);
    setError(null);
    setProgress(0);
    setUploadProgress({});
  }, [files]);

  // Cleanup on unmount
  useState(() => {
    return () => {
      clearFiles();
    };
  });

  return {
    files,
    isUploading,
    error,
    progress,
    selectFiles,
    removeFile,
    clearFiles,
    uploadProgress,
  };
}

/**
 * Simulate file upload with progress
 * Replace with actual upload logic
 */
export async function simulateFileUpload(
  file: FileWithPreview,
  onProgress: (progress: number) => void
): Promise<string> {
  return new Promise((resolve, reject) => {
    let progress = 0;
    const interval = setInterval(() => {
      progress += Math.random() * 20;
      if (progress >= 100) {
        progress = 100;
        clearInterval(interval);
        resolve(`https://example.com/uploads/${file.name}`);
      }
      onProgress(progress);
    }, 200);
  });
}
