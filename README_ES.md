# Analizador de commits Git

[English](README.md) · [中文](README_ZH.md) · [Français](README_FR.md)

Git Commit Analyzer es un plugin de Git escrito en Rust que aprovecha un modelo local de llama.cpp para analizar el diff preparado y generar mensajes de commit con formato Git Flow. El CLI resume automáticamente los cambios voluminosos, valida el formato devuelto por el modelo y ofrece mensajes deterministas de respaldo si la inferencia falla.

## Características

- **Inferencia local**: `llama_cpp_sys_2` ejecuta modelos GGUF sin depender de servicios remotos.
- **Resumen inteligente del diff**: los lockfiles y artefactos grandes se reducen a resúmenes antes de llamar al modelo.
- **Cumplimiento de Git Flow**: se comprueba `<type>(<scope>): <subject>`; si la respuesta no es válida, se reintenta o se devuelve un mensaje estándar.
- **CLI interactivo**: el usuario puede aceptar, editar o cancelar el mensaje sugerido.
- **Prompts multilingües**: inglés (predeterminado) y chino simplificado.
- **Soporte multiplataforma**: binarios precompilados para macOS (Intel & Apple Silicon).

## Requisitos

- Git ≥ 2.30
- Un modelo GGUF local (el programa puede descargar `unsloth/gemma-3-270m-it-GGUF` si no encuentra modelos)

## Instalación

### Homebrew (Recomendado) - Instalación binaria rápida

**Los usuarios de macOS pueden instalar vía Homebrew con binarios precompilados (no se requiere compilación Rust):**

```bash
brew tap zh30/tap
brew install git-ca
```

Esto instala un binario precompilado para tu plataforma:
- **macOS**: Apple Silicon (M1/M2/M3/M4) e Intel (x86_64)

**¡No se requiere cadena de herramientas Rust o compilación!** El binario se descarga automáticamente desde GitHub Releases.

**Nota**: Las compilaciones de Linux están temporalmente deshabilitadas debido a problemas de compilación. Las compilaciones de Windows están disponibles vía [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases) pero no se distribuyen vía Homebrew.

### Instalación manual

Descarga el binario apropiado para tu plataforma desde [Releases](https://github.com/zh30/git-commit-analyzer/releases):

```bash
# macOS (Apple Silicon)
curl -L -o git-ca.tar.gz https://github.com/zh30/git-commit-analyzer/releases/download/v1.1.2/git-ca-1.1.2-apple-darwin-arm64.tar.gz
tar -xzf git-ca.tar.gz
sudo mv git-ca /usr/local/bin/
chmod +x /usr/local/bin/git-ca
```

**Nota**: Las compilaciones de Linux están temporalmente deshabilitadas. Las compilaciones de Windows están disponibles vía [GitHub Releases](https://github.com/zh30/git-commit-analyzer/releases).

### Compilar desde el código fuente

Si prefieres compilar desde el código fuente:

```bash
git clone https://github.com/zh30/git-commit-analyzer.git
cd git-commit-analyzer
cargo build --release
sudo cp target/release/git-ca /usr/local/bin/
```

### Script de arranque en una línea

```bash
bash -c "$(curl -fsSL https://sh.zhanghe.dev/install-git-ca.sh)"
```

## Configuración inicial

En la primera ejecución, el CLI ejecuta los siguientes pasos:

1. **Busca modelos** en directorios comunes:
   - `./models` (directorio del proyecto)
   - `~/.cache/git-ca/models` (Linux/macOS)
   - `~/.local/share/git-ca/models` (Linux alternativo)
   - `~/Library/Application Support/git-ca/models` (macOS)

2. **Descarga automáticamente el modelo por defecto** (si no se encuentra ninguno):
   - Descarga `unsloth/gemma-3-270m-it-GGUF` desde Hugging Face
   - Lo almacena en `~/.cache/git-ca/models/`

3. **Solicita confirmación** si se encuentran múltiples modelos:
   ```bash
   git ca model  # Selector interactivo de modelos
   ```

## Uso

```bash
git add <archivos>
git ca
```

En cada invocación:

1. El diff preparado se resume (los archivos grandes solo muestran un resumen).
2. El modelo llama.cpp genera el mensaje de commit.
3. Si el resultado no cumple Git Flow, se lanza un segundo intento más estricto; si todavía falla, se ofrece un mensaje de respaldo determinista.
4. El usuario decide **usar**, **editar** o **cancelar** el mensaje.

### Comandos de configuración

- `git ca model` — Selector interactivo de modelos
- `git ca language` — Elegir prompts en inglés o chino simplificado
- `git ca doctor` — Probar carga e inferencia del modelo
- `git ca --version` — Mostrar información de versión

## Desarrollo

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- git ca      # probar contra cambios preparados
```

Archivos principales:
- `src/main.rs`: flujo del CLI, resumen del diff, generación de mensajes de respaldo
- `src/llama.rs`: envoltorio sobre la sesión de llama.cpp

## Proceso de publicación

**Publicación completamente automatizada vía GitHub Actions:**

1. Crear un tag de versión: `git tag v1.1.2 && git push origin v1.1.2`
2. GitHub Actions ejecuta automáticamente:
   - Compila binarios para macOS (Intel & Apple Silicon)
   - Crea una Release de GitHub con changelog
   - Genera checksums SHA256
   - **Actualiza automáticamente la fórmula de Homebrew** con los checksums de las botellas
   - Empuja actualizaciones al repositorio `homebrew-tap`
3. Los usuarios pueden instalar inmediatamente con: `brew install git-ca`

**Nota**: Las compilaciones de Linux están temporalmente deshabilitadas debido a problemas de compilación. Las compilaciones de Windows están disponibles vía GitHub Releases pero no se distribuyen vía Homebrew.

Ver [DEPLOY.md](DEPLOY.md) para la documentación completa de publicación.

## Plataformas soportadas

- **macOS**: ✅ Apple Silicon (arm64) e Intel (x86_64) - Binarios precompilados vía Homebrew
- **Linux**: ❌ Temporalmente deshabilitado (problemas de compilación)
- **Windows**: ⚠️ Disponible vía GitHub Releases (no vía Homebrew)

## Contribución

Se aceptan Pull Requests. Incluye:
- resultados de `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test`,
- actualizaciones de documentación (`README*.md`, `AGENTS.md`, `DEPLOY.md`) cuando cambie el comportamiento,
- una breve nota sobre la verificación manual de `git ca` si aplica.

## Licencia

Proyecto con licencia MIT. Consulta el archivo [LICENSE](LICENSE) para más información.

## Agradecimientos

- La comunidad Rust por sus excelentes bibliotecas y herramientas
- El equipo de llama.cpp por el eficiente motor de inferencia local
