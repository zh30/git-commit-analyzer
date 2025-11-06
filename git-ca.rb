class GitCa < Formula
  desc "AI-powered Git plugin for generating meaningful commit messages"
  homepage "https://github.com/zh30/git-commit-analyzer"
  url "https://github.com/zh30/git-commit-analyzer/archive/refs/tags/v2.0.9.tar.gz"
  sha256 "6f1536f5a364f2052bffa01be7e42b4b53cf4281bb2548d21884a47263f8c7d8"
  license "MIT"
  head "https://github.com/zh30/git-commit-analyzer.git", branch: "main"

  # Bottle support for pre-built binaries
  bottle do
    root_url "https://github.com/zh30/git-commit-analyzer/releases/download/v#{version}"
    sha256 cellar: :any_skip_relocate, arm64_sequoia: "4a0a692d07e26f3808ff8d5cc17dc8e03c97eae98eb2637fd9fa725a78a81e94"
    sha256 cellar: :any_skip_relocate, x86_64_sequoia: "a3869b7eaa30dd6c90c326ca0aef8fc49e89a6b8e63eab224c704b242d0c3e43"
    # Note: Linux builds disabled due to compilation issues
    #       Windows builds available via GitHub Releases but not distributed via Homebrew
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