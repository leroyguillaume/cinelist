const LEVELS = ["debug", "info", "warn", "error"] as const;

export type LogLevel = (typeof LEVELS)[number];
export type LogFields = Record<string, unknown>;

function configuredLevel(): LogLevel {
  const value: unknown = import.meta.env.VITE_LOG_LEVEL;
  return LEVELS.find((level) => level === value) ?? (import.meta.env.DEV ? "debug" : "warn");
}

const threshold = LEVELS.indexOf(configuredLevel());

function emit(level: LogLevel, event: string, fields: LogFields): void {
  if (LEVELS.indexOf(level) < threshold) {
    return;
  }
  const record = { level, event, ...fields };
  // The browser console is the only sink a static front has.
  // biome-ignore lint/suspicious/noConsole: this module is the configured logger.
  console[level](record);
}

export const logger = {
  debug: (event: string, fields: LogFields = {}) => emit("debug", event, fields),
  info: (event: string, fields: LogFields = {}) => emit("info", event, fields),
  warn: (event: string, fields: LogFields = {}) => emit("warn", event, fields),
  error: (event: string, fields: LogFields = {}) => emit("error", event, fields),
};
