export type RuntimeEventEnvelope<T = unknown> = {
  type: string;
  payload: T;
};

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

export function isRuntimeEventEnvelope(
  value: unknown,
): value is RuntimeEventEnvelope {
  return (
    isObject(value) && typeof value.type === 'string' && 'payload' in value
  );
}

export function dispatchRuntimeEvent(data: RuntimeEventEnvelope) {
  window.dispatchEvent(
    new CustomEvent<RuntimeEventEnvelope>('sage:event', {
      detail: data,
    }),
  );

  window.dispatchEvent(
    new CustomEvent<RuntimeEventEnvelope>(`sage:event:${data.type}`, {
      detail: data,
    }),
  );
}

export function onRuntimeEventType<T>(
  type: string,
  handler: (event: T) => void,
): () => void {
  const listener = (event: Event) => {
    const custom = event as CustomEvent<RuntimeEventEnvelope<T>>;
    handler(custom.detail.payload);
  };

  window.addEventListener(`sage:event:${type}`, listener as EventListener);

  return () => {
    window.removeEventListener(`sage:event:${type}`, listener as EventListener);
  };
}
