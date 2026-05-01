import React, { useState, useCallback } from 'react';
import { Button } from './button';
import { Camera, SwitchCamera, X, Check } from 'lucide-react';
import { useCamera } from '../../hooks/useCamera';
import { cn } from '../../lib/utils';

export interface CameraCaptureProps {
  onCapture: (imageData: string) => void;
  onCancel?: () => void;
  className?: string;
  facingMode?: 'user' | 'environment';
}

/**
 * Camera capture component for taking photos
 * Used for slip photos, document capture, etc.
 */
export const CameraCapture: React.FC<CameraCaptureProps> = ({
  onCapture,
  onCancel,
  className,
  facingMode = 'environment',
}) => {
  const [capturedImage, setCapturedImage] = useState<string | null>(null);
  
  const {
    videoRef,
    canvasRef,
    isReady,
    error,
    startCamera,
    stopCamera,
    capturePhoto,
    hasPermission,
  } = useCamera({ facingMode });

  const handleStart = useCallback(async () => {
    await startCamera();
  }, [startCamera]);

  const handleCapture = useCallback(() => {
    const image = capturePhoto();
    if (image) {
      setCapturedImage(image);
      stopCamera();
    }
  }, [capturePhoto, stopCamera]);

  const handleRetake = useCallback(() => {
    setCapturedImage(null);
    startCamera();
  }, [startCamera]);

  const handleConfirm = useCallback(() => {
    if (capturedImage) {
      onCapture(capturedImage);
    }
  }, [capturedImage, onCapture]);

  const handleClose = useCallback(() => {
    stopCamera();
    setCapturedImage(null);
    onCancel?.();
  }, [stopCamera, onCancel]);

  React.useEffect(() => {
    handleStart();
    return () => {
      stopCamera();
    };
  }, []);

  return (
    <div className={cn('flex flex-col items-center justify-center p-4', className)}>
      {/* Header */}
      <div className="w-full flex justify-between items-center mb-4">
        <h3 className="text-lg font-semibold">
          {capturedImage ? 'Review Photo' : 'Take Photo'}
        </h3>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleClose}
          aria-label="Close camera"
        >
          <X className="h-5 w-5" />
        </Button>
      </div>

      {/* Camera Viewfinder / Captured Image */}
      <div className="relative w-full max-w-md aspect-[3/4] bg-black rounded-lg overflow-hidden">
        {!capturedImage ? (
          <>
            <video
              ref={videoRef}
              autoPlay
              playsInline
              muted
              className="w-full h-full object-cover"
            />
            <canvas ref={canvasRef} className="hidden" />
            
            {error && (
              <div className="absolute inset-0 flex items-center justify-center bg-black/80 text-white p-4 text-center">
                <div>
                  <p className="font-medium mb-2">{error}</p>
                  {!hasPermission && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={handleStart}
                      className="mt-2"
                    >
                      Try Again
                    </Button>
                  )}
                </div>
              </div>
            )}
          </>
        ) : (
          <img
            src={capturedImage}
            alt="Captured"
            className="w-full h-full object-contain"
          />
        )}
      </div>

      {/* Controls */}
      <div className="w-full max-w-md mt-6 flex justify-center gap-4">
        {!capturedImage ? (
          <>
            <Button
              variant="outline"
              size="icon"
              onClick={() => {
                stopCamera();
                useCamera({ facingMode: facingMode === 'user' ? 'environment' : 'user' });
              }}
              disabled={!isReady}
              aria-label="Switch camera"
            >
              <SwitchCamera className="h-5 w-5" />
            </Button>
            
            <Button
              size="lg"
              className="rounded-full h-16 w-16"
              onClick={handleCapture}
              disabled={!isReady || !!error}
              aria-label="Take photo"
            >
              <Camera className="h-8 w-8" />
            </Button>
            
            <div className="w-10" /> {/* Spacer for centering */}
          </>
        ) : (
          <>
            <Button
              variant="outline"
              onClick={handleRetake}
              aria-label="Retake photo"
            >
              Retake
            </Button>
            
            <Button
              onClick={handleConfirm}
              aria-label="Use this photo"
            >
              <Check className="h-4 w-4 mr-2" />
              Use Photo
            </Button>
          </>
        )}
      </div>

      {/* Instructions */}
      <p className="text-sm text-muted-foreground mt-4 text-center">
        {capturedImage
          ? 'Review your photo and confirm or retake'
          : 'Position the item in the frame and tap to capture'}
      </p>
    </div>
  );
};

/**
 * Simple camera button that opens the camera capture modal
 */
export interface CameraButtonProps {
  onCapture: (imageData: string) => void;
  className?: string;
  children?: React.ReactNode;
}

export const CameraButton: React.FC<CameraButtonProps> = ({
  onCapture,
  className,
  children,
}) => {
  const [isOpen, setIsOpen] = useState(false);

  if (!isOpen) {
    return (
      <>
        <Button
          variant="outline"
          className={className}
          onClick={() => setIsOpen(true)}
          aria-label="Open camera"
        >
          <Camera className="h-4 w-4 mr-2" />
          {children || 'Take Photo'}
        </Button>
        
        {/* Modal would be rendered here with a proper modal component */}
        {isOpen && (
          <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center">
            <CameraCapture
              onCapture={(image) => {
                onCapture(image);
                setIsOpen(false);
              }}
              onCancel={() => setIsOpen(false)}
            />
          </div>
        )}
      </>
    );
  }

  return null;
};
