#!/usr/bin/env python3
"""
MLX-powered Git commit message generator
Uses mlx-lm to generate commit messages locally on Apple Silicon
"""

import sys
import json
import argparse
from typing import Optional, Dict, Any

try:
    from mlx_lm import load, stream_generate
except ImportError:
    print("Error: mlx-lm not installed. Please install with: pip install mlx-lm", file=sys.stderr)
    sys.exit(1)

# Available MLX models for commit generation
AVAILABLE_MODELS = {
    "gemma-3-270m-it-6bit": "mlx-community/gemma-3-270m-it-6bit",
    "gemma-2b-it": "mlx-community/gemma-2b-it",
    "mistral-7b-instruct-v0.3-4bit": "mlx-community/Mistral-7B-Instruct-v0.3-4bit",
    "llama-3-8b-instruct-4bit": "mlx-community/Meta-Llama-3-8B-Instruct-4bit",
    "phi-3-mini-4k-instruct-4bit": "mlx-community/Phi-3-mini-4k-instruct-4bit",
}

DEFAULT_MODEL = "gemma-3-270m-it-6bit"

def build_commit_prompt(diff: str, language: str = "en") -> str:
    """Build a prompt for generating Git commit messages"""

    if language.lower() == "zh" or language.lower() == "chinese":
        return f"""分析这个 git diff 并提供一个遵循 Git Flow 格式的提交信息：

<类型>(<范围>): <主题>

<正文>

其中：
- <类型> 是以下之一：feat, fix, docs, style, refactor, test, chore
- <范围> 是可选的，表示受影响的模块
- <主题> 是命令式语气的简短描述
- <正文> 提供详细描述（可选）

重要指导原则：
1. 只选择一个最能代表变更主要目的的类型。
2. 将所有变更总结为一个简洁的主题行。
3. 不要在提交信息中包含正文或脚注。
4. 不要提及或引用任何问题编号。
5. 如果有多个不相关的变更，只关注最重要的变更。
6. **确保只生成一个提交信息。**
7. **提交信息的内容必须使用简体中文，包括主题和正文。**
8. **不允许使用英文，除了 Git Flow 格式的类型关键字（feat、fix、docs 等）。**

以下是要分析的 diff：

{diff}

你的任务：
1. 分析给定的 git diff。
2. **生成一个**严格遵循上述 Git Flow 格式的提交信息。
3. 确保你的回复**只**包含格式化的提交信息，不要有任何额外的解释或 markdown。
4. 提交信息**必须**以 <类型> 开头并遵循所示的确切结构。
5. **提交信息的内容（主题和正文）必须使用简体中文。**

记住：你的回复应该只包含中文的提交信息，不要有其他内容。"""

    else:
        return f"""Analyze this git diff and provide a **single** commit message following the Git Flow format:

<type>(<scope>): <subject>

<body>

Where:
- <type> is one of: feat, fix, docs, style, refactor, test, chore
- <scope> is optional and represents the module affected
- <subject> is a short description in the imperative mood
- <body> provides detailed description (optional)

Important guidelines:
1. Choose only ONE type that best represents the primary purpose of the changes.
2. Summarize ALL changes into a single, concise subject line.
3. Do not include a body or footer in the commit message.
4. Do not mention or reference any issue numbers.
5. Focus solely on the most significant change if there are multiple unrelated changes.
6. **Ensure that only one commit message is generated.**
7. **The commit message content must be written in English language.**
8. **Do not use any other languages except English for the content.**

Here's the diff to analyze:

{diff}

Your task:
1. Analyze the given git diff.
2. **Generate only one** commit message strictly following the Git Flow format described above.
3. Ensure your response contains **ONLY** the formatted commit message, without any additional explanations or markdown.
4. **The commit message content (subject and body) must be written in English.**

Remember: Your response should only include the English commit message, nothing else."""

def generate_commit_message(diff: str, model_name: str = DEFAULT_MODEL, language: str = "en",
                          max_tokens: int = 512, temperature: float = 0.7) -> str:
    """Generate a commit message using MLX"""

    if model_name not in AVAILABLE_MODELS:
        available = ", ".join(AVAILABLE_MODELS.keys())
        print(f"Error: Model '{model_name}' not available. Available models: {available}", file=sys.stderr)
        sys.exit(1)

    model_path = AVAILABLE_MODELS[model_name]

    try:
        # Load model and tokenizer
        print(f"Loading model: {model_path}", file=sys.stderr)
        model, tokenizer = load(model_path)

        # Build prompt
        prompt = build_commit_prompt(diff, language)

        # Generate response with streaming
        print("Generating commit message...", file=sys.stderr)
        response = ""

        for chunk in stream_generate(
            model,
            tokenizer,
            prompt=prompt,
            max_tokens=max_tokens,
            temperature=temperature,
            verbose=False
        ):
            response += chunk.text
            print(chunk.text, end="", flush=True)

        print()  # New line after generation
        return response.strip()

    except Exception as e:
        print(f"Error generating commit message: {e}", file=sys.stderr)
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="Generate Git commit messages using MLX")
    parser.add_argument("--diff", required=True, help="Git diff content")
    parser.add_argument("--model", default=DEFAULT_MODEL, help=f"Model to use (default: {DEFAULT_MODEL})")
    parser.add_argument("--language", default="en", choices=["en", "zh"], help="Output language")
    parser.add_argument("--max-tokens", type=int, default=512, help="Maximum tokens to generate")
    parser.add_argument("--temperature", type=float, default=0.7, help="Generation temperature")
    parser.add_argument("--list-models", action="store_true", help="List available models")

    args = parser.parse_args()

    if args.list_models:
        print("Available MLX models:")
        for name, path in AVAILABLE_MODELS.items():
            print(f"  {name}: {path}")
        return

    # Generate commit message
    result = generate_commit_message(
        diff=args.diff,
        model_name=args.model,
        language=args.language,
        max_tokens=args.max_tokens,
        temperature=args.temperature
    )

    # Output the result
    print(result)

if __name__ == "__main__":
    main()