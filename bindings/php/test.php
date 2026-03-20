<?php

/**
 * Tests for the Inky PHP package.
 * Run: php test.php (after building libinky with `cargo build -p inky-ffi --release`)
 */

declare(strict_types=1);

require_once __DIR__ . '/src/Driver/DriverInterface.php';
require_once __DIR__ . '/src/Driver/FfiDriver.php';
require_once __DIR__ . '/src/Inky.php';

use Inky\Inky;

$passed = 0;
$failed = 0;

function assert_true(string $name, bool $condition, string $detail = ''): void
{
    global $passed, $failed;
    if ($condition) {
        $passed++;
    } else {
        $failed++;
        echo "  FAIL: {$name}\n";
        if ($detail) echo "        {$detail}\n";
    }
}

function assert_equal(string $name, string $actual, string $expected): void
{
    global $passed, $failed;
    if ($actual === $expected) {
        $passed++;
    } else {
        $failed++;
        echo "  FAIL: {$name}\n";
        echo "    expected: {$expected}\n";
        echo "    got:      {$actual}\n";
    }
}

// --- transform ---

echo "transform:\n";

$result = Inky::transform('<button href="#">Click</button>');
assert_true('button produces table', str_contains($result, 'class="button"'));
assert_true('button has href', str_contains($result, 'href="#"'));
assert_true('button has role=presentation', str_contains($result, 'role="presentation"'));

$result = Inky::transform('<container><row><column>Content</column></row></container>');
assert_true('full layout transforms', str_contains($result, 'class="container"'));
assert_true('row transforms', str_contains($result, 'class="row"'));
assert_true('column transforms', str_contains($result, 'columns'));

$result = Inky::transform('<button href="#" size="small" color="alert">Click</button>');
assert_true('v2 size attribute becomes class', str_contains($result, 'small'));
assert_true('v2 color attribute becomes class', str_contains($result, 'alert'));

$result = Inky::transform('<spacer height="20"></spacer>');
assert_true('spacer with v2 height', str_contains($result, 'height="20"'));

$result = Inky::transform('<divider></divider>');
assert_true('divider transforms', str_contains($result, 'class="divider"'));

$result = Inky::transform('<image src="hero.jpg" alt="Hero" width="600">');
assert_true('image transforms to img', str_contains($result, '<img'));
assert_true('image has width', str_contains($result, 'width="600"'));

$result = Inky::transform('<outlook><p>MSO only</p></outlook>');
assert_true('outlook conditional', str_contains($result, '<!--[if mso]>'));

$result = Inky::transform('<not-outlook><p>Modern</p></not-outlook>');
assert_true('not-outlook conditional', str_contains($result, '<!--[if !mso]><!-->'));

// --- transform with columns ---

echo "transform with columns:\n";

$result = Inky::transform('<column>Content</column>', 16);
assert_true('custom column count', str_contains($result, 'small-16') || str_contains($result, 'large-16'));

// --- transformInline ---

echo "transformInline:\n";

$result = Inky::transformInline('<html><head><style>.button { background: red; }</style></head><body><button href="#">Click</button></body></html>');
assert_true('inlines CSS', str_contains($result, 'background'));
assert_true('transforms components', str_contains($result, 'role="presentation"'));

// --- migrate ---

echo "migrate:\n";

$result = Inky::migrate('<columns large="6" small="12">Content</columns>');
assert_equal('columns to column', $result, '<column lg="6" sm="12">Content</column>');

$result = Inky::migrate('<h-line></h-line>');
assert_equal('h-line to divider', $result, '<divider></divider>');

$result = Inky::migrate('<spacer size="16"></spacer>');
assert_equal('spacer size to height', $result, '<spacer height="16"></spacer>');

// --- migrateWithDetails ---

echo "migrateWithDetails:\n";

$result = Inky::migrateWithDetails('<columns large="6">Content</columns>');
assert_true('returns array', is_array($result));
assert_true('has html field', isset($result['html']));
assert_true('has changes array', isset($result['changes']) && is_array($result['changes']));
assert_true('reports changes', count($result['changes']) > 0);
assert_true('html is migrated', str_contains($result['html'], '<column'));

// --- validate ---

echo "validate:\n";

$result = Inky::validate('<button>No href</button>');
assert_true('returns array', is_array($result));
$rules = array_column($result, 'rule');
assert_true('finds button href issue', in_array('button-no-href', $rules));

$result = Inky::validate('<img src="test.jpg">');
$rules = array_column($result, 'rule');
assert_true('finds missing alt', in_array('missing-alt', $rules));

$result = Inky::validate('<container><button href="#">OK</button></container>');
$errors = array_filter($result, fn($d) => $d['severity'] === 'error');
assert_true('valid template has no errors', count($errors) === 0);

// --- version ---

echo "version:\n";

$v = Inky::version();
assert_true('returns a string', is_string($v));
assert_true('looks like semver', (bool) preg_match('/^\d+\.\d+\.\d+/', $v));
assert_true('is 2.x', str_starts_with($v, '2.'));

// --- Summary ---

echo "\n";
if ($failed === 0) {
    echo "All {$passed} tests passed.\n";
} else {
    echo "{$passed} passed, {$failed} failed.\n";
    exit(1);
}
