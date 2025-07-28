/*
 * Copyright © 2025 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { createLazyFileRoute } from '@tanstack/react-router'
import ForbiddenError from '@/features/errors/forbidden'

export const Route = createLazyFileRoute('/(errors)/403')({
  component: ForbiddenError,
})
