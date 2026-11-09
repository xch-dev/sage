export function NoUpdateBody({ name, onClose }: any) {
  return (
    <div className='space-y-4'>
      <h1 className='text-lg font-semibold'>App is up to date</h1>
      <p className='text-sm text-muted-foreground'>
        No installable update is available for {name}.
      </p>
      <div className='flex justify-end'>
        <button
          className='rounded-md border px-4 py-2 text-sm'
          onClick={onClose}
        >
          Close
        </button>
      </div>
    </div>
  );
}
