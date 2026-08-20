export function PartialUpdateBody({ header, error }: any) {
  return (
    <div className='space-y-4'>
      <h1 className='text-lg font-semibold'>Update cannot be installed</h1>

      <div className='text-sm text-muted-foreground'>
        {header.name} requires unsupported manifest features.
      </div>

      <pre className='max-h-48 overflow-auto rounded-xl bg-muted p-3 text-xs whitespace-pre-wrap'>
        {error}
      </pre>
    </div>
  );
}
