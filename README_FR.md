# Analyseur de commits Git

[English](README.md) · [中文](README_ZH.md) · [Español](README_ES.md)

Git Commit Analyzer est un plugin Git écrit en Rust qui exploite un modèle llama.cpp local pour analyser le diff déjà indexé et produire des messages de commit conformes à Git Flow. Le CLI résume automatiquement les gros fichiers, valide la structure de la réponse et fournit un message de secours déterministe en cas d'échec du modèle.

## Fonctionnalités

- **Inférence locale** : `llama_cpp_sys_2` exécute des modèles GGUF sans dépendre d'une API distante.
- **Résumé intelligent du diff** : les fichiers volumineux (lockfiles, artefacts) sont réduits à des résumés avant l'appel au modèle.
- **Respect de Git Flow** : vérifie la forme `<type>(<scope>): <subject>` et retente/échoue proprement si nécessaire.
- **CLI interactif** : vous pouvez accepter, éditer ou annuler le message proposé.
- **Prompts multilingues** : anglais (par défaut) et chinois simplifié.
- **Support multi-plateforme** : binaires pré-compilés pour macOS (Intel & Apple Silicon).

## Prérequis

- Git ≥ 2.30
- Un modèle GGUF local (le programme peut télécharger `unsloth/gemma-3-270m-it-GGUF` si aucun modèle n'est disponible)

## Installation

### Homebrew (Recommandé) - Installation binaire rapide

**Les utilisateurs macOS peuvent installer via Homebrew avec des binaires pré-compilés (pas de compilation Rust requise) :**

```bash
brew tap zh30/tap
brew install git-ca
```

Cela installe un binaire pré-compilé pour votre plateforme :
- **macOS** : Apple Silicon (M1/M2/M3/M4) et Intel (x86_64)

**Aucune chaîne d'outils Rust ou compilation requise !** Le binaire est automatiquement téléchargé depuis GitHub Releases.

**Note** : Les builds Linux sont temporairement désactivées en raison de problèmes de compilation. Les builds Windows sont disponibles via [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases) mais non distribuées via Homebrew.

### Installation manuelle

Téléchargez le binaire approprié pour votre plateforme depuis [Releases](https://github.com/zh30/git-commit-analyzer/releases) :

```bash
# macOS (Apple Silicon)
curl -L -o git-ca.tar.gz https://github.com/zh30/git-commit-analyzer/releases/download/v1.1.2/git-ca-1.1.2-apple-darwin-arm64.tar.gz
tar -xzf git-ca.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca
```

**Note** : Les builds Linux sont temporairement désactivées. Les builds Windows sont disponibles via [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases).

### Compilation depuis le source

Si vous préférez compiler depuis le source :

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
sudo cp target/release/git-ca /usr/local/bin/
```

### Script d'amorçage en une ligne

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

## Configuration initiale

Au premier lancement, le CLI exécute les étapes suivantes :

1. **Scan des modèles** dans les répertoires courants :
   - `./models` (répertoire projet)
   - `~/.cache/git-ca/models` (Linux/macOS)
   - `~/.local/share/git-ca/models` (Linux alternatif)
   - `~/Library/Application Support/git-ca/models` (macOS)

2. **Téléchargement automatique du modèle par défaut** (si aucun trouvé) :
   - Télécharge `unsloth/gemma-3-270m-it-GGUF` depuis Hugging Face
   - Le stocke dans `~/.cache/git-ca/models/`

3. **Demande de confirmation** si plusieurs modèles sont trouvés :
   ```bash
   git ca model  # Sélecteur de modèle interactif
   ```

## Utilisation

```bash
git add <fichiers>
git ca
```

À chaque invocation :

1. Le diff indexé est condensé (les fichiers volumineux apparaissent sous forme de résumé).
2. Le modèle llama.cpp génère un message de commit.
3. Si la réponse ne respecte pas Git Flow, une tentative plus stricte est effectuée ; à défaut, un message de secours déterministe est proposé.
4. Vous décidez d'**utiliser**, **éditer** ou **annuler** le message.

### Commandes de configuration

- `git ca model` — Sélecteur de modèle interactif
- `git ca language` — Choisir les prompts anglais ou chinois simplifié
- `git ca doctor` — Tester le chargement et l'inférence du modèle
- `git ca --version` — Afficher les informations de version

## Développement

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca      # tester contre les changements indexés
```

Fichiers principaux :
- `src/main.rs` : orchestration CLI, synthèse du diff, stratégie de repli
- `src/llama.rs` : encapsulation de la session llama.cpp

## Processus de publication

**Publication entièrement automatisée via GitHub Actions :**

1. Pousser un tag de version : `git tag v1.1.2 && git push origin v1.1.2`
2. GitHub Actions exécute automatiquement :
   - Compile les binaires pour macOS (Intel & Apple Silicon)
   - Crée une Release GitHub avec un changelog
   - Génère les checksums SHA256
   - **Met à jour automatiquement la formule Homebrew** avec les checksums des bottles
   - Pousse les mises à jour vers le dépôt `homebrew-tap`
3. Les utilisateurs peuvent immédiatement installer avec : `brew install git-ca`

**Note** : Les builds Linux sont temporairement désactivées en raison de problèmes de compilation. Les builds Windows sont disponibles via GitHub Releases mais non distribuées via Homebrew.

Voir [DEPLOY.md](DEPLOY.md) pour la documentation complète de publication.

## Plateformes supportées

- **macOS** : ✅ Apple Silicon (arm64) et Intel (x86_64) - Binaires pré-compilés via Homebrew
- **Linux** : ❌ Temporairement désactivé (problèmes de compilation)
- **Windows** : ⚠️ Disponible via GitHub Releases (pas via Homebrew)

## Contribution

Les contributions sont les bienvenues. Merci d'inclure :
- les sorties `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`,
- les mises à jour de la documentation (`README*.md`, `AGENTS.md`, `DEPLOY.md`) en cas de changement fonctionnel,
- une brève description de la vérification manuelle de `git ca` si applicable.

## Licence

Projet sous licence MIT. Consultez le fichier [LICENSE](LICENSE) pour plus d'informations.

## Remerciements

- La communauté Rust pour ses excellentes bibliothèques et outils
- L'équipe llama.cpp pour le moteur d'inférence locale efficace
