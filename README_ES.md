# Analizador de Commits Git

[English](README.md) | [中文](README_ZH.md) | [Français](README_FR.md)

Analizador de Commits Git es un potente plugin de Git que utiliza IA para generar automáticamente mensajes de commit significativos basados en tus cambios preparados. Utiliza Ollama para analizar diferencias git y proponer mensajes de commit siguiendo el formato Git Flow.

## Características

- Generación automática de mensajes de commit que cumplen con Git Flow
- Funciona con Ollama para procesamiento de IA local
- Modo interactivo que permite a los usuarios usar, editar o cancelar el mensaje de commit propuesto
- Compatibilidad multiplataforma (Linux, macOS, Windows)
- Personalizable con tu firma Git personal
- Soporte para selección y persistencia de modelos

## Requisitos previos

- Git (versión 2.0 o posterior)
- Ollama instalado y en ejecución (https://ollama.com/download)
- Al menos un modelo de lenguaje instalado en Ollama

## Instalación

### Homebrew (macOS y Linux)

La forma más fácil de instalar Git Commit Analyzer es a través de Homebrew:

```
brew tap zh30/tap
brew install git-ca
```

Después de la instalación, puede usar inmediatamente el comando `git ca`.

### Instalación manual (Linux y macOS)

1. Clonar el repositorio:
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. Construir el proyecto:
   ```
   cargo build --release
   ```

3. Crear un directorio para los plugins de Git (si no existe):
   ```
   mkdir -p ~/.git-plugins
   ```

4. Copiar el binario compilado al directorio de plugins:
   ```
   cp target/release/git-ca ~/.git-plugins/
   ```

5. Añadir el directorio de plugins a su PATH. Añada la siguiente línea a su `~/.bashrc`, `~/.bash_profile`, o `~/.zshrc` (dependiendo de su shell):
   ```
   export PATH="$HOME/.git-plugins:$PATH"
   ```

6. Recargar la configuración de su shell:
   ```
   source ~/.bashrc  # o ~/.bash_profile, o ~/.zshrc
   ```

### Windows - teóricamente posible

1. Clonar el repositorio:
   ```
   git clone https://github.com/zh30/git-commit-analyzer.git
   cd git-commit-analyzer
   ```

2. Construir el proyecto:
   ```
   cargo build --release
   ```

3. Crear un directorio para los plugins de Git (si no existe):
   ```
   mkdir %USERPROFILE%\.git-plugins
   ```

4. Copiar el binario compilado al directorio de plugins:
   ```
   copy target\release\git-commit-analyzer.exe %USERPROFILE%\.git-plugins\
   ```

5. Añadir el directorio de plugins a su PATH:
   - Haga clic derecho en 'Este PC' o 'Mi PC' y seleccione 'Propiedades'
   - Haga clic en 'Configuración avanzada del sistema'
   - Haga clic en 'Variables de entorno'
   - En 'Variables del sistema', busque y seleccione 'Path', luego haga clic en 'Editar'
   - Haga clic en 'Nuevo' y añada `%USERPROFILE%\.git-plugins`
   - Haga clic en 'Aceptar' para cerrar todos los cuadros de diálogo

6. Reinicie cualquier símbolo del sistema abierto para que los cambios surtan efecto.

## Cómo usar

Después de la instalación, puede utilizar Git Commit Analyzer en cualquier repositorio Git:

1. Prepare sus cambios en su repositorio Git (utilizando el comando `git add`).
2. Ejecute el siguiente comando:

   ```
   git ca
   ```

3. Si es la primera vez que ejecuta el comando, se le pedirá que seleccione un modelo de sus modelos Ollama instalados.
4. El programa analizará sus cambios preparados y generará un mensaje de commit sugerido.
5. Puede elegir usar el mensaje sugerido, editarlo o cancelar el commit.

Para cambiar el modelo predeterminado en cualquier momento, ejecute:

```
git ca model
```

## Contribución

¡Las contribuciones son bienvenidas! No dude en enviar una Pull Request.

## Licencia

Este proyecto está licenciado bajo la Licencia MIT - consulte el archivo [LICENSE](LICENSE) para más detalles.

## Agradecimientos

- A la comunidad de Rust por proporcionar excelentes bibliotecas y herramientas
- A Ollama por proporcionar soporte para modelos de IA locales 