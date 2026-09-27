class Cuv < Formula
  desc "Extremely fast, zero-dependency package manager and build driver for C/C++"
  homepage "https://github.com/ARASKOVA-labs/CUV"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/ARASKOVA-labs/CUV/releases/download/v#{version}/cuv-aarch64-apple-darwin.tar.gz"
    else
      url "https://github.com/ARASKOVA-labs/CUV/releases/download/v#{version}/cuv-x86_64-apple-darwin.tar.gz"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/ARASKOVA-labs/CUV/releases/download/v#{version}/cuv-aarch64-unknown-linux-gnu.tar.gz"
    else
      url "https://github.com/ARASKOVA-labs/CUV/releases/download/v#{version}/cuv-x86_64-unknown-linux-gnu.tar.gz"
    end
  end

  def install
    bin.install "cuv"
  end

  test do
    system "#{bin}/cuv", "--version"
    system "#{bin}/cuv", "info"
  end
end
