import { useEffect } from 'react';

export function useIntersectionObserver(
  elementRef: React.RefObject<Element>,
  callback: IntersectionObserverCallback,
  options: IntersectionObserverInit = { threshold: 0 },
) {
  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    const observer = new IntersectionObserver(callback, options);
    observer.observe(element);

    return () => {
      observer.disconnect();
    };
  }, [elementRef, callback, options]);
}
