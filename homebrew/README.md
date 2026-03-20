# Homebrew Formula Template

This directory contains the Homebrew formula template for Inky.

## Setting Up the Tap

To publish Inky via Homebrew, create a `foundation/homebrew-inky` repository on GitHub and copy the formula there:

```
homebrew-inky/
  Formula/
    inky.rb
```

## Installation

```sh
brew tap foundation/inky
brew install inky
```

## Release Workflow

The release workflow automatically updates the formula in the tap repository with the correct version, download URLs, and SHA256 checksums. The placeholder values in this template (`VERSION_PLACEHOLDER`, `SHA256_PLACEHOLDER_*`) are replaced during the release process.
