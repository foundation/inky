<?php

declare(strict_types=1);

namespace Inky\Driver;

use FFI;
use RuntimeException;

class FfiDriver implements DriverInterface
{
    private FFI $ffi;

    public function __construct(?string $libPath = null, ?string $headerPath = null)
    {
        if (!self::isAvailable()) {
            throw new RuntimeException(
                'FFI extension is not available or not enabled. '
                . 'Set ffi.enable=true in php.ini for development, '
                . 'or use ffi.enable=preload with opcache.preload for production.'
            );
        }

        $headerPath = $headerPath ?? self::findHeader();
        $libPath = $libPath ?? self::findLibrary();

        if ($headerPath === null) {
            throw new RuntimeException('Could not find inky.h header file.');
        }
        if ($libPath === null) {
            throw new RuntimeException(
                'Could not find libinky shared library. '
                . 'Build it with: cargo build -p inky-ffi --release'
            );
        }

        $this->ffi = FFI::cdef(file_get_contents($headerPath), $libPath);
    }

    public static function isAvailable(): bool
    {
        return extension_loaded('ffi')
            && in_array(ini_get('ffi.enable'), ['1', 'true', 'preload'], true);
    }

    public function transform(string $html): string
    {
        return $this->callAndFree('inky_transform', $html);
    }

    public function transformWithColumns(string $html, int $columns): string
    {
        $ptr = $this->ffi->inky_transform_with_columns($html, $columns);
        $result = FFI::string($ptr);
        $this->ffi->inky_free($ptr);
        return $result;
    }

    public function transformInline(string $html): string
    {
        return $this->callAndFree('inky_transform_inline', $html);
    }

    public function migrate(string $html): string
    {
        return $this->callAndFree('inky_migrate', $html);
    }

    public function migrateWithDetails(string $html): array
    {
        $json = $this->callAndFree('inky_migrate_with_details', $html);
        return json_decode($json, true, 512, JSON_THROW_ON_ERROR);
    }

    public function validate(string $html): array
    {
        $json = $this->callAndFree('inky_validate', $html);
        return json_decode($json, true, 512, JSON_THROW_ON_ERROR);
    }

    public function version(): string
    {
        $ptr = $this->ffi->inky_version();
        $result = FFI::string($ptr);
        $this->ffi->inky_free($ptr);
        return $result;
    }

    /**
     * Call an FFI function that takes a single string arg and returns a string.
     */
    private function callAndFree(string $fn, string $input): string
    {
        $ptr = $this->ffi->{$fn}($input);
        $result = FFI::string($ptr);
        $this->ffi->inky_free($ptr);
        return $result;
    }

    private static function findHeader(): ?string
    {
        $candidates = [
            __DIR__ . '/../stubs/inky.h',
            __DIR__ . '/../../stubs/inky.h',
        ];
        foreach ($candidates as $path) {
            if (file_exists($path)) {
                return realpath($path);
            }
        }
        return null;
    }

    private static function findLibrary(): ?string
    {
        $ext = PHP_OS_FAMILY === 'Darwin' ? 'dylib' : (PHP_OS_FAMILY === 'Windows' ? 'dll' : 'so');
        $name = PHP_OS_FAMILY === 'Windows' ? 'inky.dll' : "libinky.{$ext}";

        $candidates = [
            // Development: cargo build output
            __DIR__ . "/../../../../target/release/{$name}",
            __DIR__ . "/../../../../target/debug/{$name}",
            // System-installed
            "/usr/local/lib/{$name}",
            "/usr/lib/{$name}",
        ];

        foreach ($candidates as $path) {
            if (file_exists($path)) {
                return realpath($path);
            }
        }
        return null;
    }
}
