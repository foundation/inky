"""
Inky — Transform email templates into email-safe HTML.

Powered by Rust via ctypes FFI.
"""

from __future__ import annotations

import ctypes
import json
import os
import platform
import sys
from dataclasses import dataclass

__version__ = "2.0.0"

_lib = None


def _find_library():
    """Find the libinky shared library."""
    system = platform.system()
    if system == "Darwin":
        name = "libinky.dylib"
    elif system == "Windows":
        name = "inky.dll"
    else:
        name = "libinky.so"

    # Check paths relative to this package (development layout)
    pkg_dir = os.path.dirname(os.path.abspath(__file__))
    candidates = [
        # Development: cargo build output
        os.path.join(pkg_dir, "..", "..", "..", "..", "target", "release", name),
        os.path.join(pkg_dir, "..", "..", "..", "..", "target", "debug", name),
        # Bundled with package
        os.path.join(pkg_dir, name),
        # System paths
        os.path.join("/usr/local/lib", name),
        os.path.join("/usr/lib", name),
    ]

    for path in candidates:
        resolved = os.path.normpath(path)
        if os.path.exists(resolved):
            return resolved

    return None


def _get_lib():
    """Get or initialize the shared library handle."""
    global _lib
    if _lib is not None:
        return _lib

    lib_path = _find_library()
    if lib_path is None:
        raise RuntimeError(
            "Could not find libinky shared library. "
            "Build it with: cargo build -p inky-ffi --release"
        )

    _lib = ctypes.CDLL(lib_path)

    # All inky_* functions return an owned char* that must be freed with
    # inky_free. restype MUST be c_void_p (not c_char_p): ctypes converts
    # c_char_p results to bytes and discards the pointer, which would make
    # the string impossible to free (a leak on every call).
    _lib.inky_transform.argtypes = [ctypes.c_char_p]
    _lib.inky_transform.restype = ctypes.c_void_p

    _lib.inky_transform_with_columns.argtypes = [ctypes.c_char_p, ctypes.c_uint32]
    _lib.inky_transform_with_columns.restype = ctypes.c_void_p

    _lib.inky_transform_inline.argtypes = [ctypes.c_char_p]
    _lib.inky_transform_inline.restype = ctypes.c_void_p

    _lib.inky_migrate.argtypes = [ctypes.c_char_p]
    _lib.inky_migrate.restype = ctypes.c_void_p

    _lib.inky_migrate_with_details.argtypes = [ctypes.c_char_p]
    _lib.inky_migrate_with_details.restype = ctypes.c_void_p

    _lib.inky_validate.argtypes = [ctypes.c_char_p]
    _lib.inky_validate.restype = ctypes.c_void_p

    _lib.inky_version.argtypes = []
    _lib.inky_version.restype = ctypes.c_void_p

    _lib.inky_build.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
    _lib.inky_build.restype = ctypes.c_void_p

    _lib.inky_free.argtypes = [ctypes.c_void_p]
    _lib.inky_free.restype = None

    return _lib


class InkyError(RuntimeError):
    """Raised when the native inky library reports an error."""


class InkyBuildError(RuntimeError):
    """Raised when the full build pipeline fails.

    Attributes:
        warnings: Non-fatal notes collected before the failure.
    """

    def __init__(self, message: str, warnings: list):
        super().__init__(message)
        self.warnings = warnings


@dataclass
class BuildResult:
    """Result of a full pipeline build."""

    html: str
    text: str | None
    warnings: list


def _call_str(fn, *args) -> str:
    """Call a native char*-returning function; copy, free, and decode it."""
    lib = _get_lib()
    ptr = fn(*args)
    if not ptr:
        raise InkyError("inky native call failed (null result)")
    try:
        return ctypes.string_at(ptr).decode("utf-8")
    finally:
        lib.inky_free(ptr)


def transform(html: str, columns: int = 12) -> str:
    """Transform Inky HTML into email-safe table markup.

    Args:
        html: Inky template HTML.
        columns: Number of grid columns (default: 12).

    Returns:
        Transformed HTML string.
    """
    lib = _get_lib()
    encoded = html.encode("utf-8")
    if columns != 12:
        return _call_str(lib.inky_transform_with_columns, encoded, columns)
    return _call_str(lib.inky_transform, encoded)


def transform_inline(html: str) -> str:
    """Transform Inky HTML and inline CSS from <style> blocks.

    Args:
        html: Inky template HTML with <style> blocks.

    Returns:
        Transformed HTML with CSS inlined.
    """
    lib = _get_lib()
    return _call_str(lib.inky_transform_inline, html.encode("utf-8"))


def migrate(html: str) -> str:
    """Migrate v1 Inky syntax to v2.

    Args:
        html: v1 Inky template HTML.

    Returns:
        Migrated v2 HTML string.
    """
    lib = _get_lib()
    return _call_str(lib.inky_migrate, html.encode("utf-8"))


def migrate_with_details(html: str) -> dict:
    """Migrate v1 syntax and return detailed results.

    Args:
        html: v1 Inky template HTML.

    Returns:
        Dict with 'html' (migrated HTML) and 'changes' (list of descriptions).
    """
    lib = _get_lib()
    return json.loads(_call_str(lib.inky_migrate_with_details, html.encode("utf-8")))


def validate(html: str) -> list:
    """Validate an Inky template and return diagnostics.

    Args:
        html: Inky template HTML.

    Returns:
        List of dicts with 'severity', 'rule', and 'message' fields.
    """
    lib = _get_lib()
    return json.loads(_call_str(lib.inky_validate, html.encode("utf-8")))


def version() -> str:
    """Get the Inky engine version.

    Returns:
        Version string (e.g. "2.0.0").
    """
    lib = _get_lib()
    return _call_str(lib.inky_version)


def build(html: str, base_path: str | None = None, **options) -> BuildResult:
    """Run the full build pipeline: layouts, includes, custom components,
    data merge, framework SCSS, component transform, CSS inlining, and
    output cleanup — identical to `inky build`.

    Args:
        html: Inky template HTML.
        base_path: Directory used to resolve layouts, includes, custom
            components, and linked SCSS/CSS (default: None).
        **options: inline_css (bool), framework_css (bool),
            components_dir (str), columns (int), hybrid (bool),
            bulletproof_buttons (bool), plain_text (bool), data (dict of
            merge variables).

    Returns:
        BuildResult with html, text (or None), and warnings.

    Raises:
        InkyBuildError: If the pipeline fails. Carries `.warnings`.
    """
    lib = _get_lib()
    encoded_html = html.encode("utf-8")
    encoded_base = base_path.encode("utf-8") if base_path is not None else None
    options_json = json.dumps(options).encode("utf-8")

    ptr = lib.inky_build(encoded_html, encoded_base, options_json)
    if not ptr:
        raise InkyError("inky native call failed (null result)")
    try:
        envelope = json.loads(ctypes.string_at(ptr).decode("utf-8"))
    finally:
        lib.inky_free(ptr)

    warnings = envelope.get("warnings", [])
    if not envelope.get("ok"):
        raise InkyBuildError(envelope.get("error", "unknown build error"), warnings)

    return BuildResult(html=envelope["html"], text=envelope.get("text"), warnings=warnings)
