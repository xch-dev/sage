import { useCallback, useRef, useState } from 'react';

interface LongPressOptions {
  threshold?: number;
}

interface LongPressHandlers {
  onMouseDown: (event: React.MouseEvent | React.TouchEvent) => void;
  onTouchStart: (event: React.TouchEvent) => void;
  onMouseUp: (event: React.MouseEvent | React.TouchEvent) => void;
  onMouseLeave: (event: React.MouseEvent) => void;
  onTouchEnd: (event: React.TouchEvent) => void;
  onTouchMove: (event: React.TouchEvent) => void;
}

export const useLongPress = (
  onLongPress: (event: React.MouseEvent | React.TouchEvent) => void,
  onClick?: (event: React.MouseEvent | React.TouchEvent) => void,
  { threshold = 500 }: LongPressOptions = {},
): LongPressHandlers => {
  const [longPressTriggered, setLongPressTriggered] = useState(false);
  const timeout = useRef<NodeJS.Timeout>();
  const target = useRef<EventTarget>();
  const touchStart = useRef({ x: 0, y: 0 });

  const start = useCallback(
    (event: React.MouseEvent | React.TouchEvent) => {
      event.preventDefault();
      if (event.target === target.current) {
        return;
      }

      if (event.type === 'touchstart') {
        const touch = (event as React.TouchEvent).touches[0];
        touchStart.current = { x: touch.clientX, y: touch.clientY };
      }

      target.current = event.target;
      timeout.current = setTimeout(() => {
        onLongPress(event);
        setLongPressTriggered(true);
      }, threshold);
    },
    [onLongPress, threshold],
  );

  const clear = useCallback(
    (event: React.MouseEvent | React.TouchEvent, shouldTriggerClick = true) => {
      if (timeout.current) {
        clearTimeout(timeout.current);
      }

      if (event.type === 'touchend' && shouldTriggerClick) {
        const touch = (event as React.TouchEvent).changedTouches[0];
        const deltaX = Math.abs(touch.clientX - touchStart.current.x);
        const deltaY = Math.abs(touch.clientY - touchStart.current.y);

        if (deltaX > 10 || deltaY > 10) {
          shouldTriggerClick = false;
        }
      }

      if (shouldTriggerClick && !longPressTriggered && onClick) {
        event.preventDefault();
        onClick(event);
      }
      setLongPressTriggered(false);
      target.current = undefined;
    },
    [onClick, longPressTriggered],
  );

  const move = useCallback((event: React.TouchEvent) => {
    if (event.type === 'touchmove') {
      const touch = event.touches[0];
      const deltaX = Math.abs(touch.clientX - touchStart.current.x);
      const deltaY = Math.abs(touch.clientY - touchStart.current.y);

      if (deltaX > 10 || deltaY > 10) {
        if (timeout.current) {
          clearTimeout(timeout.current);
        }
        setLongPressTriggered(false);
        target.current = undefined;
      }
    }
  }, []);

  return {
    onMouseDown: start,
    onTouchStart: start,
    onMouseUp: clear,
    onMouseLeave: (e) => clear(e, false),
    onTouchEnd: (e) => {
      e.preventDefault();
      clear(e);
    },
    onTouchMove: move,
  };
};
