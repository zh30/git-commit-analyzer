# Analyseur de commits Git

[English](README.md) · [中文](README_ZH.md) · [Español](README_ES.md)

Git Commit Analyzer est un plugin Git écrit en Rust qui exploite un modèle llama.cpp local pour analyser le diff déjà indexé et produire des messages de commit conformes à Git Flow. Le CLI résume automatiquement les gros fichiers, valide la structure de la réponse et fournit un message de secours déterministe en cas d’échec du modèle.

## Fonctionnalités

- **Inférence locale** : `llama_cpp_sys` exécute des modèles GGUF sans dépendre d’une API distante.
- **Résumé intelligent du diff** : les fichiers volumineux (lockfiles, artefacts) sont réduits à des résumés avant l’appel au modèle.
- **Respect de Git Flow** : vérifie la forme `<type>(<scope>) : <subject>` et retente/échoue proprement si nécessaire.
- **CLI interactif** : vous pouvez accepter, éditer ou annuler le message proposé.
- **Prompts multilingues** : anglais (par défaut) et chinois simplifié.
- **Contexte configurable** : ajustez la fenêtre de contexte llama via la configuration Git.

## Prérequis

- Git ≥ 2.30
- Chaîne d’outils Rust stable (`cargo`)
- Dépendances de compilation llama.cpp (cmake, compilateur C/C++, pilotes Metal/CUDA selon la plateforme)
- Un modèle GGUF local (le programme peut télécharger `unsloth/gemma-3-270m-it-GGUF` si aucun modèle n’est disponible)

## Installation

### Installation manuelle

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
mkdir -p ~/.git-plugins
cp target/release/git-ca ~/.git-plugins/
echo 'export PATH="$HOME/.git-plugins:$PATH"' >> ~/.bashrc    # adaptez selon votre shell
source ~/.bashrc
```

Au premier lancement, le CLI parcourt `./models`, `~/Library/Application Support/git-ca/models` et `~/.cache/git-ca/models`. S’il ne trouve aucun modèle, il propose de télécharger celui par défaut depuis Hugging Face.

### Homebrew (macOS / Linux)

```bash
brew tap zh30/tap
brew install git-ca
```

### Script d’amorçage

Un script optionnel (`install-git-ca.sh`) automatise la vérification des dépendances, la compilation et la mise à jour du PATH :

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

Inspectez le script avant exécution et assurez-vous qu’un modèle GGUF est disponible.

## Utilisation

```bash
git add <fichiers>
git ca
```

Lors de la première exécution, choisissez le chemin du modèle. À chaque invocation :

1. Le diff indexé est condensé (les fichiers volumineux apparaissent sous forme de résumé).
2. Le modèle llama.cpp génère un message de commit.
3. Si la réponse ne respecte pas Git Flow, une tentative plus stricte est effectuée ; à défaut, un message de secours (par ex. `chore(deps): update dependencies`) est proposé.
4. Vous décidez d’**utiliser**, **éditer** ou **annuler** le message.

### Configuration

- `git ca model` — sélectionne interactivement un modèle, stocké dans `commit-analyzer.model`.
- `git ca language` — bascule les prompts entre anglais et chinois, stocké dans `commit-analyzer.language`.
- `git config --global commit-analyzer.context 1024` — règle la longueur de contexte llama (512–8192). Le résumé du diff respecte automatiquement cette valeur.

## Développement

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca
```

Fichiers principaux :
- `src/main.rs` : orchestration CLI, synthèse du diff, stratégie de repli.
- `src/llama.rs` : encapsulation de la session llama.cpp.

## Contribution

Les contributions sont les bienvenues. Merci d’inclure :
- les sorties `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`,
- les mises à jour de la documentation (`README*.md`, `AGENTS.md`, `DEPLOY.md`) en cas de changement fonctionnel,
- un court descriptif des tests manuels `git ca` le cas échéant.

## Licence

Projet sous licence MIT. Consultez le fichier [LICENSE](LICENSE) pour plus d’informations.
