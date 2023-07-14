# completions

![stability-badge](https://img.shields.io/badge/stability-stable-green?style=flat-square)

Das `completions`-Kommando erzeugt Dateien, die Anweisungen enthalten,
welche Argumente und Optionen des Toolkits für eine
[Shell](https://de.wikipedia.org/wiki/Shell_(Betriebssystem)) zur
Autovervollständigung verfügbar sind.

## Beschreibung

> **Hinweis:** Mit jeder neuen `pica`-Version können sich die Argumente
> und Optionen des _command-line interface_ (CLI) ändern. Daher ist ein
> regelmäßiges Updaten der Skripte zu empfehlen.

Nachfolgend werden exemplarisch die Befehle gezeigt, die für die
Einbindung in die jeweilige Shell nötig sind. Die Schritte sind vom
System sowie der Nutzereinstellung abhängig und müssen ggf. angepasst
werden.

Es werden folgende Shells unterstützt:

- [Bash](https://www.gnu.org/software/bash/),
- [Elvish](https://github.com/elves/elvish),
- [Fish](https://fishshell.com/),
- [Powershell](https://docs.microsoft.com/en-us/powershell/)
- und [ZSH](https://zsh.sourceforge.io/).

## Bash

```bash
$ mkdir -p ~/.local/share/bash-completion/completions
$ pica completions bash \
    -o  ~/.local/share/bash-completion/completions/pica
```

### Bash (macOS/Homebrew)

```bash
$ mkdir -p $(brew --prefix)/etc/bash_completion.d
$ pica completions bash \
    -o $(brew --prefix)/etc/bash_completion.d/pica.bash-completion
```

## Elvish

```bash
$ mkdir -p ~/.local/share/elvish/lib/completions
$ pica completions elvish -o ~/.local/share/elvish/lib/completions/pica.elv
$ echo "use completions/pica" >> ~/.elvish/rc.elv
```

## Fish

```bash
$ mkdir -p ~/.config/fish/completions
$ pica completions fish -o ~/.config/fish/completions/pica.fish
```

## Powershell

```bash
$ pica completions powershell \
     >> ${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
```

## ZSH

Der Pfad `~/.zfunc` muss in der Variable `$fpath` gesetzt sein, bevor
die Funktion `compinit` aufgerufen wird.

```bash
$ pica completions zsh -o ~/.zfunc/_pica
```
