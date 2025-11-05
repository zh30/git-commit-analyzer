class GitCa < Formula
  desc "AI-powered Git plugin for generating meaningful commit messages"
  homepage "https://github.com/zh30/git-commit-analyzer"
  url "https://github.com/zh30/git-commit-analyzer/archive/refs/tags/v1.1.2.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_CHECKSUM"
  license "MIT"
  head "https://github.com/zh30/git-commit-analyzer.git", branch: "main"

  # Bottle support for pre-built binaries
  bottle do
    root_url "https://github.com/zh30/git-commit-analyzer/releases/download/v#{version}"
    sha256 cellar: :any_skip_relocate, arm64_sequoia: "REPLACE_WITH_ARM64_MACOS_SHA256"
    sha256 cellar: :any_skip_relocate, x86_64_sequoia: "REPLACE_WITH_X86_64_MACOS_SHA256"
    sha256 cellar: :any_skip_relocate, arm64_linux: "REPLACE_WITH_ARM64_LINUX_SHA256"
    sha256 cellar: :any_skip_relocate, x86_64_linux: "REPLACE_WITH_X86_64_LINUX_SHA256"
  end

  def install
    bin.install "git-ca"
  end

  def caveats
    <<~EOS
      To use git-ca, you need a local GGUF model (llama.cpp format).

      The tool will automatically download the default model
      (unsloth/gemma-3-270m-it-GGUF) on first run, or you can:
        - Place GGUF files in ./models directory
        - Place GGUF files in ~/.cache/git-ca/models directory
        - Run 'git ca model' to select a model manually

      To set up a default model, run:
        git ca model

      Note: git-ca uses local llama.cpp inference (no remote API calls).
    EOS
  end

  test do
    # Test to verify that the binary is installed correctly
    assert_match version.to_s, shell_output("#{bin}/git-ca --version")
  end
end 