import React, { useRef, useCallback } from 'react';
import { cn } from '../../lib/utils';

export interface FileUploadProps extends React.HTMLAttributes<HTMLDivElement> {
  accept?: string;
  maxSize?: number;
  multiple?: boolean;
  onFilesSelected?: (files: File[]) => void;
  onError?: (error: string) => void;
  className?: string;
}

export const FileUpload = React.forwardRef<HTMLDivElement, FileUploadProps>(
  ({
    accept,
    maxSize = 10 * 1024 * 1024, // 10MB
    multiple = true,
    onFilesSelected,
    onError,
    className,
    children,
    ...props
  }, ref) => {
    const inputRef = useRef<HTMLInputElement>(null);

    const validateFile = useCallback((file: File): string | null => {
      if (accept) {
        const acceptedTypes = accept.split(',').map(t => t.trim());
        const fileType = file.type;
        const fileExtension = '.' + file.name.split('.').pop()?.toLowerCase();
        
        const isAccepted = acceptedTypes.some(type => {
          if (type.startsWith('.')) return fileExtension === type.toLowerCase();
          if (type.endsWith('/*')) return fileType.startsWith(type.split('/')[0] + '/');
          return fileType === type;
        });

        if (!isAccepted) {
          return `File type "${file.type}" is not accepted`;
        }
      }

      if (file.size > maxSize) {
        const maxSizeMB = (maxSize / (1024 * 1024)).toFixed(2);
        return `File "${file.name}" exceeds ${maxSizeMB}MB`;
      }

      return null;
    }, [accept, maxSize]);

    const handleDrop = useCallback((e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      const files = Array.from(e.dataTransfer.files);
      processFiles(files);
    }, [onFilesSelected, onError, validateFile]);

    const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();
    }, []);

    const handleDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();
    }, []);

    const processFiles = useCallback((files: File[]) => {
      const validFiles: File[] = [];
      const errors: string[] = [];

      files.forEach(file => {
        const error = validateFile(file);
        if (error) {
          errors.push(error);
        } else {
          validFiles.push(file);
        }
      });

      if (errors.length > 0 && onError) {
        onError(errors.join('\n'));
      }

      if (validFiles.length > 0 && onFilesSelected) {
        onFilesSelected(multiple ? validFiles : validFiles.slice(0, 1));
      }
    }, [onFilesSelected, onError, validateFile, multiple]);

    const handleClick = useCallback(() => {
      inputRef.current?.click();
    }, []);

    const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files;
      if (files) {
        processFiles(Array.from(files));
      }
      // Reset input to allow selecting same file again
      e.target.value = '';
    }, [processFiles]);

    return (
      <div
        ref={ref}
        className={cn(
          'flex flex-col items-center justify-center rounded-lg border-2 border-dashed border-muted-foreground/25 p-8 transition-colors hover:border-muted-foreground/50 hover:bg-muted/25 cursor-pointer',
          className
        )}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onClick={handleClick}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            handleClick();
          }
        }}
        aria-label="File upload area. Click or drag files here."
        {...props}
      >
        <input
          ref={inputRef}
          type="file"
          className="hidden"
          accept={accept}
          multiple={multiple}
          onChange={handleInputChange}
          aria-hidden="true"
        />
        
        {children || (
          <>
            <svg
              className="h-12 w-12 text-muted-foreground mb-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
              />
            </svg>
            <p className="text-sm text-muted-foreground">
              Drag and drop files here, or click to select
            </p>
            <p className="text-xs text-muted-foreground mt-1">
              {accept ? `Accepted: ${accept}` : 'Any file type'} • Max size: {(maxSize / (1024 * 1024)).toFixed(1)}MB
            </p>
          </>
        )}
      </div>
    );
  }
);
FileUpload.displayName = 'FileUpload';
