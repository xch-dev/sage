export function ApprovalMetaPill({ children }: { children: React.ReactNode }) {
  return (
    <span className='rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground'>
      {children}
    </span>
  );
}

export function ApprovalDetailRow({
  label,
  value,
  mono,
  breakAll,
}: {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
  breakAll?: boolean;
}) {
  return (
    <div className='grid grid-cols-[6rem_1fr] gap-3 text-sm'>
      <div className='text-xs font-medium uppercase tracking-wide text-muted-foreground'>
        {label}
      </div>
      <div
        className={[
          'min-w-0 text-foreground',
          mono ? 'font-mono text-xs' : '',
          breakAll ? 'break-all' : '',
        ].join(' ')}
      >
        {value}
      </div>
    </div>
  );
}
