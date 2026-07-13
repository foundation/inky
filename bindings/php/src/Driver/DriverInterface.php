<?php

declare(strict_types=1);

namespace Inky\Driver;

interface DriverInterface
{
    public static function isAvailable(): bool;

    public function transform(string $html): string;

    public function transformWithColumns(string $html, int $columns): string;

    public function transformInline(string $html): string;

    public function migrate(string $html): string;

    public function migrateWithDetails(string $html): array;

    public function validate(string $html): array;

    public function version(): string;

    /**
     * Run the full build pipeline. Returns the decoded JSON envelope.
     *
     * @return array{ok: bool, html?: string, text?: string, error?: string, warnings: string[]}
     */
    public function build(string $html, ?string $basePath, string $optionsJson): array;
}
