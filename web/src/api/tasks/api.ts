/*
 * Copyright © 2025 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import axiosInstance from "@/api/axiosInstance";
import { PaginatedResponse } from "..";
import { EmailTask, EventHookTask, TaskStatus } from "@/features/tasks/data/schema";

export const list_email_tasks = async (pageIndex: number, pageSize: number, status: TaskStatus | 'all') => {
    const baseUrl = `/api/v1/send-email-tasks?page=${pageIndex + 1}&page_size=${pageSize}&desc=true`;
    const url = status !== 'all' ? `${baseUrl}&status=${status}` : baseUrl;
    const response = await axiosInstance.get<PaginatedResponse<EmailTask>>(url);
    return response.data;
};

export const delete_email_task = async (id: number) => {
    const response = await axiosInstance.delete(`/api/v1/send-email-task/${id}`);
    return response.data;
}

export const list_hook_tasks = async (pageIndex: number, pageSize: number, status: TaskStatus | 'all') => {
    const baseUrl = `/api/v1/hook-tasks?page=${pageIndex + 1}&page_size=${pageSize}&desc=true`;
    const url = status !== 'all' ? `${baseUrl}&status=${status}` : baseUrl;
    const response = await axiosInstance.get<PaginatedResponse<EventHookTask>>(url);
    return response.data;
};

export const delete_hook_task = async (id: number) => {
    const response = await axiosInstance.delete(`/api/v1/hook-task/${id}`);
    return response.data;
} 