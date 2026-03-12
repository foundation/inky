<?php

declare(strict_types=1);

namespace Inky;

use Inky\Driver\DriverInterface;
use Inky\Driver\FfiDriver;
use RuntimeException;

/**
 * Inky — Transform email templates into email-safe HTML.
 *
 * Auto-detects the best available driver:
 *   1. inky PHP extension (PECL) — recommended for production
 *   2. FFI driver — for development or self-managed servers
 */
class Inky
{
    private static ?DriverInterface $driver = null;

    /**
     * Transform Inky HTML into email-safe table markup.
     */
    public static function transform(string $html, int $columns = 12): string
    {
        $driver = self::getDriver();
        if ($columns !== 12) {
            return $driver->transformWithColumns($html, $columns);
        }
        return $driver->transform($html);
    }

    /**
     * Transform Inky HTML and inline CSS from <style> blocks.
     */
    public static function transformInline(string $html): string
    {
        return self::getDriver()->transformInline($html);
    }

    /**
     * Migrate v1 Inky syntax to v2.
     */
    public static function migrate(string $html): string
    {
        return self::getDriver()->migrate($html);
    }

    /**
     * Migrate v1 syntax and return detailed results.
     *
     * @return array{html: string, changes: string[]}
     */
    public static function migrateWithDetails(string $html): array
    {
        return self::getDriver()->migrateWithDetails($html);
    }

    /**
     * Validate an Inky template and return diagnostics.
     *
     * @return array<array{severity: string, rule: string, message: string}>
     */
    public static function validate(string $html): array
    {
        return self::getDriver()->validate($html);
    }

    /**
     * Get the Inky engine version.
     */
    public static function version(): string
    {
        return self::getDriver()->version();
    }

    /**
     * Get the active driver, auto-detecting if needed.
     */
    public static function getDriver(): DriverInterface
    {
        if (self::$driver !== null) {
            return self::$driver;
        }

        // Priority 1: PECL extension
        if (extension_loaded('inky')) {
            // Future: return new ExtensionDriver();
        }

        // Priority 2: FFI
        if (FfiDriver::isAvailable()) {
            self::$driver = new FfiDriver();
            return self::$driver;
        }

        throw new RuntimeException(
            "Inky requires either the 'inky' PHP extension (recommended) or the 'ffi' "
            . "extension with ffi.enable=true in php.ini. "
            . "See https://inky.email/php for setup instructions."
        );
    }

    /**
     * Set a custom driver instance.
     */
    public static function setDriver(DriverInterface $driver): void
    {
        self::$driver = $driver;
    }

    /**
     * Reset the driver (useful for testing).
     */
    public static function resetDriver(): void
    {
        self::$driver = null;
    }
}
