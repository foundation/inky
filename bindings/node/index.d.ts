export interface TransformOptions {
  /** Number of grid columns (default: 12) */
  columns?: number;
}

export interface ValidateOptions {
  /** Number of grid columns (default: 12) */
  columns?: number;
}

export interface Diagnostic {
  severity: "warning" | "error";
  rule: string;
  message: string;
}

export interface MigrateResult {
  html: string;
  changes: string[];
}

/** Transform Inky HTML into email-safe table markup. */
export function transform(html: string, options?: TransformOptions): string;

/** Migrate v1 Inky syntax to v2. */
export function migrate(html: string): string;

/** Migrate v1 syntax and return detailed results. */
export function migrateWithDetails(html: string): MigrateResult;

/** Validate an Inky template and return diagnostics. */
export function validate(
  html: string,
  options?: ValidateOptions
): Diagnostic[];

/** Get the Inky version. */
export function version(): string;
