import { useEffect, useRef } from 'react';

export function useIntersectionObserver(
  elementRef: React.RefObject<Element>,
  callback: IntersectionObserverCallback,
  options: IntersectionObserverInit = { threshold: 0 },
) {
  // Store callback in a ref to avoid re-creating observer on every callback change
  const callbackRef = useRef(callback);

  useEffect(() => {
    callbackRef.current = callback;
  }, [callback]);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    const observer = new IntersectionObserver(
      (entries, observer) => callbackRef.current(entries, observer),
      options,
    );
    observer.observe(element);

    return () => {
      observer.disconnect();
    };
    // Only re-create observer if element or options change
  }, [
    elementRef,
    options.threshold,
    options.root,
    options.rootMargin,
    options,
  ]);
}
