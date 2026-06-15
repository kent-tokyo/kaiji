class Kaiji < Formula
  desc "High-performance CJK fuzzy match and text normalization engine"
  homepage "https://github.com/kent-tokyo/kaiji"
  version "0.1.0"
  license "MIT OR Apache-2.0"

  # TODO: Update sha256 checksums after publishing the first GitHub release.
  # Run: shasum -a 256 kaiji-macos-arm64
  on_macos do
    on_arm do
      url "https://github.com/kent-tokyo/kaiji/releases/download/v#{version}/kaiji-macos-arm64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"

      def install
        bin.install "kaiji-macos-arm64" => "kaiji"
      end
    end

    on_intel do
      url "https://github.com/kent-tokyo/kaiji/releases/download/v#{version}/kaiji-macos-x86_64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"

      def install
        bin.install "kaiji-macos-x86_64" => "kaiji"
      end
    end
  end

  test do
    assert_match "斉藤", shell_output("echo '齋藤' | #{bin}/kaiji normalize")
  end
end
