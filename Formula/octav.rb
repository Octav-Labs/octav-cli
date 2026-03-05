# typed: false
# frozen_string_literal: true

# Homebrew formula for Octav CLI
# To use: brew install Octav-Labs/tap/octav
#
# This file is a template. Copy it to the homebrew-tap repo at:
#   https://github.com/Octav-Labs/homebrew-tap/Formula/octav.rb
# and update the version, URLs, and sha256 hashes for each release.

class Octav < Formula
  desc "CLI for the Octav crypto portfolio API"
  homepage "https://github.com/Octav-Labs/octav-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Octav-Labs/octav-cli/releases/download/v#{version}/octav-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_DARWIN_SHA256"
    else
      url "https://github.com/Octav-Labs/octav-cli/releases/download/v#{version}/octav-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_DARWIN_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Octav-Labs/octav-cli/releases/download/v#{version}/octav-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_LINUX_SHA256"
    else
      url "https://github.com/Octav-Labs/octav-cli/releases/download/v#{version}/octav-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_X86_64_LINUX_SHA256"
    end
  end

  def install
    bin.install "octav"
  end

  test do
    assert_match "CLI for the Octav crypto portfolio API", shell_output("#{bin}/octav --help")
  end
end
