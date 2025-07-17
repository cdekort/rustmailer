import { Table } from '@tanstack/react-table'
import { Input } from '@/components/ui/input'
import { DataTableViewOptions } from '../components/data-table-view-options'

interface DataTableToolbarProps<TData> {
  table: Table<TData>
}

export function DataTableToolbar<TData>({
  table,
}: DataTableToolbarProps<TData>) {

  return (
    <div className='flex items-center justify-between'>
      <div className='flex flex-1 flex-col-reverse items-start gap-y-2 sm:flex-row sm:items-center sm:space-x-2'>
        <Input
          placeholder='Filter event hook...'
          value={(table.getState().globalFilter as string) ?? ''}
          onChange={(event) => {
            table.setGlobalFilter(event.target.value);
          }}
          className='h-8 w-80'
        />
      </div>
      <DataTableViewOptions table={table} />
    </div>
  )
}
