import type { z } from "zod";
import { logger } from "./logger";
import { problemSchema } from "./schemas";

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly invalidFields: readonly string[];

  constructor(status: number, code: string, invalidFields: readonly string[] = []) {
    super(`API error ${status}: ${code}`);
    this.status = status;
    this.code = code;
    this.invalidFields = invalidFields;
  }
}

type Method = "GET" | "POST" | "PUT" | "DELETE";

async function send(method: Method, path: string, body?: unknown): Promise<Response> {
  const init: RequestInit = { method, credentials: "same-origin", headers: {} };
  if (body !== undefined) {
    init.headers = { "Content-Type": "application/json" };
    init.body = JSON.stringify(body);
  }
  logger.debug("api.request", { method, path });
  const response = await fetch(`/api/v1${path}`, init);
  logger.debug("api.response", { method, path, status: response.status });
  if (response.ok) {
    return response;
  }
  const problem = problemSchema.safeParse(await response.json().catch(() => null));
  if (!problem.success) {
    throw new ApiError(response.status, response.status >= 500 ? "internal" : "unknown");
  }
  throw new ApiError(response.status, problem.data.code, problem.data.invalidFields ?? []);
}

export async function request<T>(
  method: Method,
  path: string,
  schema: z.ZodType<T>,
  body?: unknown,
): Promise<T> {
  const response = await send(method, path, body);
  const parsed = schema.safeParse(await response.json());
  if (!parsed.success) {
    logger.error("api.unexpected_body", { method, path, issues: parsed.error.issues });
    throw new ApiError(response.status, "unexpected_response");
  }
  return parsed.data;
}

export async function requestEmpty(method: Method, path: string, body?: unknown): Promise<void> {
  await send(method, path, body);
}
