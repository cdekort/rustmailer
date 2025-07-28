/*
 * Copyright © 2025 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { DotsHorizontalIcon } from '@radix-ui/react-icons'
import { Row } from '@tanstack/react-table'
import { IconTrash } from '@tabler/icons-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useTasksContext } from '../context'
import { EmailTask } from '../data/schema'
import { MessageSquareMore } from 'lucide-react'

interface EmailTableRowActionsProps {
  row: Row<EmailTask>
}

export function EmailTableRowActions({ row }: EmailTableRowActionsProps) {
  const { setOpen, setCurrentEmailRow } = useTasksContext()
  return (
    <>
      <DropdownMenu modal={false}>
        <DropdownMenuTrigger asChild>
          <Button
            variant='ghost'
            className='flex h-8 w-8 p-0 data-[state=open]:bg-muted'
          >
            <DotsHorizontalIcon className='h-4 w-4' />
            <span className='sr-only'>Open menu</span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align='end' className='w-[160px]'>
          <DropdownMenuItem
            onClick={() => {
              setCurrentEmailRow(row.original)
              setOpen('detail')
            }}
          >
            Detail
            <DropdownMenuShortcut>
              <MessageSquareMore size={16} />
            </DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => {
              setCurrentEmailRow(row.original)
              setOpen('delete')
            }}
            className='!text-red-500'
          >
            Remove
            <DropdownMenuShortcut>
              <IconTrash size={16} />
            </DropdownMenuShortcut>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </>
  )
}
