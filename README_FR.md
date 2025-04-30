# Analyseur de Commits Git

[English](README.md) | [中文](README_ZH.md)

Analyseur de Commits Git est un puissant plugin Git qui utilise l'IA pour générer automatiquement des messages de commit pertinents basés sur vos changements en attente. Il utilise Ollama pour analyser les différences git et proposer des messages de commit conformes au format Git Flow.

## Fonctionnalités

- Génération automatique de messages de commit conformes à Git Flow
- Propulsé par Ollama pour un traitement IA local
- Mode interactif permettant aux utilisateurs d'utiliser, de modifier ou d'annuler le message de commit proposé
- Compatibilité multi-plateformes (Linux, macOS, Windows)
- Personnalisable avec votre signature Git personnelle
- Support pour la sélection et la persistance des modèles

## Prérequis

- Git (version 2.0 ou ultérieure)
- Ollama installé et en cours d'exécution (https://ollama.com/download)
- Au moins un modèle de langage installé dans Ollama

## Installation

### Homebrew (macOS et Linux)

La façon la plus simple d'installer Git Commit Analyzer est via Homebrew :

```
brew tap zh30/tap
brew install git-ca
```

Après l'installation, vous pouvez immédiatement utiliser la commande `git ca`.

### Installation manuelle (Linux et macOS)

1. Clonez le dépôt :
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. Construisez le projet :
   ```
   cargo build --release
   ```

3. Créez un répertoire pour les plugins Git (s'il n'existe pas) :
   ```
   mkdir -p ~/.git-plugins
   ```

4. Copiez le binaire compilé dans le répertoire des plugins :
   ```
   cp target/release/git-ca ~/.git-plugins/
   ```

5. Ajoutez le répertoire des plugins à votre PATH. Ajoutez la ligne suivante à votre `~/.bashrc`, `~/.bash_profile`, ou `~/.zshrc` (selon votre shell) :
   ```
   export PATH="$HOME/.git-plugins:$PATH"
   ```

6. Rechargez votre configuration shell :
   ```
   source ~/.bashrc  # ou ~/.bash_profile, ou ~/.zshrc
   ```

### Windows - théoriquement possible

1. Clonez le dépôt :
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. Construisez le projet :
   ```
   cargo build --release
   ```

3. Créez un répertoire pour les plugins Git (s'il n'existe pas) :
   ```
   mkdir %USERPROFILE%\.git-plugins
   ```

4. Copiez le binaire compilé dans le répertoire des plugins :
   ```
   copy target\release\git-commit-analyzer.exe %USERPROFILE%\.git-plugins\
   ```

5. Ajoutez le répertoire des plugins à votre PATH :
   - Faites un clic droit sur 'Ce PC' ou 'Poste de travail' et sélectionnez 'Propriétés'
   - Cliquez sur 'Paramètres système avancés'
   - Cliquez sur 'Variables d'environnement'
   - Sous 'Variables système', trouvez et sélectionnez 'Path', puis cliquez sur 'Modifier'
   - Cliquez sur 'Nouveau' et ajoutez `%USERPROFILE%\.git-plugins`
   - Cliquez sur 'OK' pour fermer toutes les boîtes de dialogue

6. Redémarrez tous les invites de commande ouverts pour que les changements prennent effet.

## Comment utiliser

Après l'installation, vous pouvez utiliser Git Commit Analyzer dans n'importe quel dépôt Git :

1. Mettez en attente vos modifications dans votre dépôt Git (en utilisant la commande `git add`).
2. Exécutez la commande suivante :

   ```
   git ca
   ```

3. Si c'est la première fois que vous exécutez la commande, vous serez invité à sélectionner un modèle parmi vos modèles Ollama installés.
4. Le programme analysera vos modifications en attente et générera un message de commit suggéré.
5. Vous pouvez choisir d'utiliser le message suggéré, de le modifier ou d'annuler le commit.

Pour changer le modèle par défaut à tout moment, exécutez :

```
git ca model
```

## Contribution

Les contributions sont les bienvenues ! N'hésitez pas à soumettre une Pull Request.

## Licence

Ce projet est sous licence MIT - voir le fichier [LICENSE](LICENSE) pour plus de détails.

## Remerciements

- La communauté Rust pour fournir d'excellentes bibliothèques et outils
- Ollama pour fournir un support de modèle IA local 