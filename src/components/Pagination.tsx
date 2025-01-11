import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { t } from '@lingui/core/macro';
import { Button } from './ui/button';
import { ChevronRightIcon, ChevronLeftIcon } from '@radix-ui/react-icons';

type PaginationProps = {
  page: number;
  total: number;
  setPage: (page: number) => void;
  pageSize: number;
  onPageSizeChange: (pageSize: number) => void;
};

export const Pagination = ({
  page,
  total,
  setPage,
  pageSize,
  onPageSizeChange,
}: PaginationProps) => {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const isFirstPage = page === 1;
  const isLastPage = page === totalPages;

  return (
    <div className='flex justify-between flex-wrap gap-2'>
      <div className='flex items-center justify-start gap-2'>
        <Button
          size='icon'
          variant='outline'
          onClick={() => setPage(Math.max(1, page - 1))}
          disabled={isFirstPage}
        >
          <ChevronLeftIcon className='h-4 w-4' />
        </Button>

        <Select
          onValueChange={(value) => setPage(parseInt(value))}
          defaultValue={page.toString()}
          value={page.toString()}
          disabled={totalPages === 1}
        >
          <SelectTrigger className='w-min text-sm'>
            <SelectValue placeholder={t`Page`}>
              {page}/{totalPages}
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            {Array.from({ length: totalPages }, (_, i) => (
              <SelectItem key={i} value={(i + 1).toString()}>
                {i + 1}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Button
          size='icon'
          variant='outline'
          onClick={() => setPage(Math.min(totalPages, page + 1))}
          disabled={isLastPage}
        >
          <ChevronRightIcon className='h-4 w-4' />
        </Button>
      </div>

      <Select
        onValueChange={(value) => {
          setPage(1);
          onPageSizeChange(parseInt(value));
        }}
        defaultValue={pageSize.toString()}
        value={pageSize.toString()}
      >
        <SelectTrigger className='w-min'>{pageSize}</SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem value='8'>8</SelectItem>
            <SelectItem value='16'>16</SelectItem>
            <SelectItem value='32'>32</SelectItem>
            <SelectItem value='64'>64</SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>
    </div>
  );
};
