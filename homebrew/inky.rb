class Inky < Formula
  desc "Inky email templating engine"
  homepage "https://github.com/foundation/inky"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/foundation/inky/releases/download/v#{version}/inky-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_PLACEHOLDER_MACOS_ARM64"
    else
      url "https://github.com/foundation/inky/releases/download/v#{version}/inky-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_PLACEHOLDER_MACOS_X86_64"
    end
  end

  on_linux do
    url "https://github.com/foundation/inky/releases/download/v#{version}/inky-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "SHA256_PLACEHOLDER_LINUX_X86_64"
  end

  def install
    bin.install "inky"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/inky --version")
  end
end
