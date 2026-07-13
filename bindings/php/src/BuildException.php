<?php

declare(strict_types=1);

namespace Inky;

use RuntimeException;

/**
 * A pipeline build failure. `$warnings` holds the non-fatal notes collected
 * before the failure.
 */
final class BuildException extends RuntimeException
{
    /**
     * @param string[] $warnings
     */
    public function __construct(string $message, public readonly array $warnings = [])
    {
        parent::__construct($message);
    }
}
