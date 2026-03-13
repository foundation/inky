"""
Inky — Transform email templates into email-safe HTML.

Powered by Rust via ctypes FFI.
"""

import ctypes
import json
import os
import platform
import sys

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

    # Configure function signatures
    _lib.inky_transform.argtypes = [ctypes.c_char_p]
    _lib.inky_transform.restype = ctypes.c_char_p

    _lib.inky_transform_with_columns.argtypes = [ctypes.c_char_p, ctypes.c_uint32]
    _lib.inky_transform_with_columns.restype = ctypes.c_char_p

    _lib.inky_transform_inline.argtypes = [ctypes.c_char_p]
    _lib.inky_transform_inline.restype = ctypes.c_char_p

    _lib.inky_migrate.argtypes = [ctypes.c_char_p]
    _lib.inky_migrate.restype = ctypes.c_char_p

    _lib.inky_migrate_with_details.argtypes = [ctypes.c_char_p]
    _lib.inky_migrate_with_details.restype = ctypes.c_char_p

    _lib.inky_validate.argtypes = [ctypes.c_char_p]
    _lib.inky_validate.restype = ctypes.c_char_p

    _lib.inky_version.argtypes = []
    _lib.inky_version.restype = ctypes.c_char_p

    _lib.inky_free.argtypes = [ctypes.c_char_p]
    _lib.inky_free.restype = None

    return _lib


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
        result = lib.inky_transform_with_columns(encoded, columns)
    else:
        result = lib.inky_transform(encoded)
    return result.decode("utf-8")


def transform_inline(html: str) -> str:
    """Transform Inky HTML and inline CSS from <style> blocks.

    Args:
        html: Inky template HTML with <style> blocks.

    Returns:
        Transformed HTML with CSS inlined.
    """
    lib = _get_lib()
    result = lib.inky_transform_inline(html.encode("utf-8"))
    return result.decode("utf-8")


def migrate(html: str) -> str:
    """Migrate v1 Inky syntax to v2.

    Args:
        html: v1 Inky template HTML.

    Returns:
        Migrated v2 HTML string.
    """
    lib = _get_lib()
    result = lib.inky_migrate(html.encode("utf-8"))
    return result.decode("utf-8")


def migrate_with_details(html: str) -> dict:
    """Migrate v1 syntax and return detailed results.

    Args:
        html: v1 Inky template HTML.

    Returns:
        Dict with 'html' (migrated HTML) and 'changes' (list of descriptions).
    """
    lib = _get_lib()
    result = lib.inky_migrate_with_details(html.encode("utf-8"))
    return json.loads(result.decode("utf-8"))


def validate(html: str) -> list:
    """Validate an Inky template and return diagnostics.

    Args:
        html: Inky template HTML.

    Returns:
        List of dicts with 'severity', 'rule', and 'message' fields.
    """
    lib = _get_lib()
    result = lib.inky_validate(html.encode("utf-8"))
    return json.loads(result.decode("utf-8"))


def version() -> str:
    """Get the Inky engine version.

    Returns:
        Version string (e.g. "2.0.0").
    """
    lib = _get_lib()
    result = lib.inky_version()
    return result.decode("utf-8")
