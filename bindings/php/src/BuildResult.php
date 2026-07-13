<?php

declare(strict_types=1);

namespace Inky;

/**
 * Result of a full pipeline build.
 */
final class BuildResult
{
    /**
     * @param string[] $warnings Non-fatal notes (e.g. an unreadable linked SCSS file).
     */
    public function __construct(
        public readonly string $html,
        public readonly ?string $text,
        public readonly array $warnings,
    ) {
    }
}
