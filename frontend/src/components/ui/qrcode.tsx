import React from 'react';
import { cn } from '../../lib/utils';

export interface QRCodeProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string;
  size?: number;
  bgColor?: string;
  fgColor?: string;
  level?: 'L' | 'M' | 'Q' | 'H';
  includeMargin?: boolean;
}

/**
 * QRCode component - renders a QR code using canvas
 * Note: For production, consider using a library like 'qrcode.react'
 */
export const QRCode: React.FC<QRCodeProps> = ({
  value,
  size = 200,
  bgColor = '#FFFFFF',
  fgColor = '#000000',
  level = 'M',
  includeMargin = false,
  className,
  ...props
}) => {
  const canvasRef = React.useRef<HTMLCanvasElement>(null);

  React.useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Simple QR code placeholder rendering
    // In production, use a proper QR code generation library
    ctx.fillStyle = bgColor;
    ctx.fillRect(0, 0, size, size);

    // Draw a simple pattern as placeholder
    ctx.fillStyle = fgColor;
    const cellSize = size / 21; // QR codes are 21x21 minimum
    
    // Draw position markers (the three corner squares)
    const drawMarker = (x: number, y: number) => {
      ctx.fillRect(x * cellSize, y * cellSize, 7 * cellSize, 7 * cellSize);
      ctx.fillStyle = bgColor;
      ctx.fillRect((x + 1) * cellSize, (y + 1) * cellSize, 5 * cellSize, 5 * cellSize);
      ctx.fillStyle = fgColor;
      ctx.fillRect((x + 2) * cellSize, (y + 2) * cellSize, 3 * cellSize, 3 * cellSize);
    };

    drawMarker(0, 0);
    drawMarker(14, 0);
    drawMarker(0, 14);

    // Add text label
    ctx.font = `${Math.min(size / 20, 12)}px monospace`;
    ctx.textAlign = 'center';
    ctx.fillText(value.slice(0, 20) + (value.length > 20 ? '...' : ''), size / 2, size - 10);

  }, [value, size, bgColor, fgColor]);

  return (
    <div
      className={cn('inline-block', className)}
      {...props}
    >
      <canvas
        ref={canvasRef}
        width={size}
        height={size}
        className="rounded-lg"
        aria-label={`QR Code: ${value}`}
      />
    </div>
  );
};

/**
 * Generate a downloadable QR code
 */
export function downloadQRCode(canvas: HTMLCanvasElement, filename: string = 'qrcode.png') {
  const link = document.createElement('a');
  link.download = filename;
  link.href = canvas.toDataURL('image/png');
  link.click();
}
