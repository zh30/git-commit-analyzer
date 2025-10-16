# Analizador de commits Git

[English](README.md) · [中文](README_ZH.md) · [Français](README_FR.md)

Git Commit Analyzer es un plugin de Git escrito en Rust que aprovecha un modelo local de llama.cpp para analizar el diff preparado y generar mensajes de commit con formato Git Flow. El CLI resume automáticamente los cambios voluminosos, valida el formato devuelto por el modelo y ofrece mensajes deterministas de respaldo si la inferencia falla.

## Características

- **Inferencia local**: `llama_cpp_sys` ejecuta modelos GGUF sin depender de servicios remotos.
- **Resumen inteligente del diff**: los lockfiles y artefactos grandes se reducen a resúmenes antes de llamar al modelo.
- **Cumplimiento de Git Flow**: se comprueba `<type>(<scope>): <subject>`; si la respuesta no es válida, se reintenta o se devuelve un mensaje estándar.
- **CLI interactivo**: el usuario puede aceptar, editar o cancelar el mensaje sugerido.
- **Prompts multilingües**: inglés (predeterminado) y chino simplificado.
- **Contexto configurable**: ajuste la ventana de contexto de llama mediante configuración de Git.

## Requisitos

- Git ≥ 2.30
- Toolchain estable de Rust (`cargo`)
- Dependencias de compilación para llama.cpp (cmake, compilador C/C++, controladores Metal/CUDA según plataforma)
- Un modelo GGUF local (el programa puede descargar `unsloth/gemma-3-270m-it-GGUF` si no encuentra modelos)

## Instalación

### Instalación manual

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
mkdir -p ~/.git-plugins
cp target/release/git-ca ~/.git-plugins/
echo 'export PATH="$HOME/.git-plugins:$PATH"' >> ~/.bashrc   # adapte la ruta a su shell
source ~/.bashrc
```

En la primera ejecución el CLI busca modelos en `./models`, `~/Library/Application Support/git-ca/models` y `~/.cache/git-ca/models`. Si no encuentra ninguno, ofrece descargar el modelo predeterminado desde Hugging Face.

### Homebrew (macOS / Linux)

```bash
brew tap zh30/tap
brew install git-ca
```

### Script de arranque

Un script opcional (`install-git-ca.sh`) automatiza la comprobación de dependencias, la compilación y la actualización del PATH:

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

Revise el script antes de ejecutarlo y asegúrese de que dispone de un modelo GGUF accesible.

## Uso

```bash
git add <archivos>
git ca
```

Durante la primera ejecución se le pedirá seleccionar la ruta del modelo. En cada invocación:

1. El diff preparado se resume (los archivos grandes solo muestran un resumen).
2. El modelo llama.cpp genera el mensaje de commit.
3. Si el resultado no cumple Git Flow, se lanza un segundo intento más estricto; si todavía falla, se ofrece un mensaje de respaldo (por ejemplo `chore(deps): update dependencies`).
4. El usuario decide **usar**, **editar** o **cancelar** el mensaje.

### Configuración

- `git ca model` — selector interactivo de modelos; guarda la ruta en `commit-analyzer.model`.
- `git ca language` — alterna entre prompts en inglés y chino; guarda la preferencia en `commit-analyzer.language`.
- `git config --global commit-analyzer.context 1024` — ajusta la longitud de contexto (512–8192). El resumen del diff respeta automáticamente este valor.

## Desarrollo

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca
```

Archivos principales:
- `src/main.rs`: flujo del CLI, resumen del diff, generación de mensajes de respaldo.
- `src/llama.rs`: envoltorio minimalista sobre la sesión de llama.cpp.

## Contribución

Se aceptan Pull Requests. Incluya:
- resultados de `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test`,
- actualizaciones de documentación (`README*.md`, `AGENTS.md`, `DEPLOY.md`) cuando cambie el comportamiento,
- una breve nota sobre la verificación manual de `git ca` si aplica.

## Licencia

Proyecto con licencia MIT. Consulte el archivo [LICENSE](LICENSE) para más información.
