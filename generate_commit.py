#!/usr/bin/env python3
"""
MLX-powered Git commit message generator
Uses mlx-lm to generate commit messages locally on Apple Silicon
"""

import sys
import json
import argparse
from typing import Optional, Dict, Any, Tuple

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

def build_commit_prompts(diff: str, language: str = "en") -> Tuple[str, Tuple[str, str, str]]:
    """Return (system_instruction, (example_user, example_assistant, user_prompt))."""

    if language.lower() in {"zh", "chinese", "中文"}:
        system_prompt = (
            "你是一名资深化身的 Git 提交信息写手。"
            "请严格按照 Git Flow 格式输出：<类型>(<作用域>): <主题>。"
            "类型必须从 feat/fix/docs/style/refactor/test/chore 中选择，作用域可省略。"
            "主题保持命令式语气，使用简体中文描述内容，不要添加正文、脚注或额外解释。"
        )
        example_user = (
            "```diff\n"
            "diff --git a/src/main.rs b/src/main.rs\n"
            "@@ -1,2 +1,2 @@\n"
            "-println!(\"Hello\");\n"
            "+println!(\"Hello, world!\");\n"
            "```\n"
            "请生成提交信息。"
        )
        example_assistant = "feat(main): 改进问候语输出"
        user_prompt = (
            "以下是需要总结的 git diff，输出一条提交信息即可：\n"
            f"```diff\n{diff}\n```"
        )
    else:
        system_prompt = (
            "You are an expert release engineer who writes Git Flow commit messages."
            "Respond with exactly one line in the format <type>(<scope>): <subject>"
            " (scope optional). Use imperative voice, keep it under 72 characters,"
            " and do not include bodies, footers, or issue references."
        )
        example_user = (
            "```diff\n"
            "diff --git a/src/lib.rs b/src/lib.rs\n"
            "@@ -10,0 +11,3 @@\n"
            "+/// Greets the world\n"
            "+pub fn hello() {\n"
            "+    println!(\"hi\");\n"
            "+}\n"
            "```\n"
            "Generate the commit message."
        )
        example_assistant = "docs(lib): document hello helper"
        user_prompt = (
            "Here is the diff that needs a commit message:\n"
            f"```diff\n{diff}\n```"
        )

    return system_prompt, (example_user, example_assistant, user_prompt)

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

        # Build conversation prompts
        system_prompt, (example_user, example_assistant, user_prompt) = build_commit_prompts(
            diff, language
        )

        if hasattr(tokenizer, "apply_chat_template"):
            messages = [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": example_user},
                {"role": "assistant", "content": example_assistant},
                {"role": "user", "content": user_prompt},
            ]
            prompt = tokenizer.apply_chat_template(
                messages,
                add_generation_prompt=True,
            )
        else:
            prompt = (
                f"{system_prompt}\n\n"
                f"Example input:\n{example_user}\n\n"
                f"Example output:\n{example_assistant}\n\n"
                f"Current diff:\n{user_prompt}\n\nCommit message:"
            )

        # Generate response with streaming
        print("Generating commit message...", file=sys.stderr)
        response = ""

        generator = stream_generate(
            model,
            tokenizer,
            prompt=prompt,
            max_tokens=max_tokens,
        )

        for chunk in generator:
            response += chunk.text

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
