import { useState, useCallback, useEffect } from 'react';

interface UseDebounceOptions {
  delay?: number;
  leading?: boolean;
}

/**
 * Custom hook for debouncing values
 * Useful for search inputs, API calls, etc.
 */
export function useDebounce<T>(
  value: T,
  delay: number = 500,
  options: UseDebounceOptions = {}
): T {
  const { leading = false } = options;
  const [debouncedValue, setDebouncedValue] = useState<T>(value);
  const [isLeading, setIsLeading] = useState(leading);

  useEffect(() => {
    // Handle leading edge
    if (leading && isLeading) {
      setDebouncedValue(value);
      setIsLeading(false);
      return;
    }

    const handler = setTimeout(() => {
      setDebouncedValue(value);
      setIsLeading(leading);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay, leading, isLeading]);

  return debouncedValue;
}

/**
 * Custom hook for debouncing callback functions
 * Returns a debounced version of the callback
 */
export function useDebounceCallback<T extends (...args: any[]) => any>(
  callback: T,
  delay: number = 500
): T {
  const [timeoutId, setTimeoutId] = useState<NodeJS.Timeout | null>(null);

  const debouncedCallback = useCallback(
    (...args: Parameters<T>) => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }

      const newTimeoutId = setTimeout(() => {
        callback(...args);
      }, delay);

      setTimeoutId(newTimeoutId);
    },
    [callback, delay, timeoutId]
  ) as T;

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [timeoutId]);

  return debouncedCallback;
}
