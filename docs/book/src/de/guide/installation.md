# Installation

Das Toolkit _pica-rs_ kann unter Linux, macOS und Windows verwendet werden. Für diese
Betriebssysteme stehen unterschiedliche Pakete zum
[Download auf GitHub](https://github.com/deutsche-nationalbibliothek/pica-rs/releases)
bereit.

## Debian & Ubuntu

Beginnend mit der Version `0.10.0` stehen für die Linux-Distributionen
[Debian](https://www.debian.org/) sowie [Ubuntu](https://ubuntu.com/) fertige
`DEB`-Pakete zum Download bereit. Diese können mit folgendem Kommando installiert
werden:

```bash
$ dpkg -i pica_0.10.0-1_amd64.deb
```

## RedHat & SUSE & CentOS

Beginnend mit der Version `0.10.0` stehen für die Linux-Distributionen
[RedHat](https://www.redhat.com/), [SUSE](https://www.suse.com/) und
[CentOS](https://www.centos.org/) stehen fertige `RPM` zum Download bereit. Diese können
mit folgendem Kommando installiert werden:

```bash
$ rpm -i pica-0.10.0-1.x86_64.rpm
```

Für CentOS 7 steht ein spezielles `RPM`-Paket bereit (`pica-0.10.0-glibc2.17-1.x86_64.rpm`),
dass die [GNU C Library (glibc)](https://www.gnu.org/software/libc) in der Version 2.17
verwendet.

## Binary-Releases

Für die Betriebssysteme Linux, macOS und Windows stehen mit jeder neuen Version Binaries zum
Download zur Verfügung. Die Archive (`*.tar.gz` oder `*.zip`) enthalten das `pica`-Programm,
das für die jeweilige Architektur gebaut wurde und das ohne eine Installation direkt genutzt
werden kann.

Folgende Architekturen werden unterstützt:

| Zielarchitektur          | Bemerkung                                  |
|--------------------------|--------------------------------------------|
| x86_64-unknown-linux-gnu | 64-bit Linux (kernel 2.6.32+, glibc 2.11+) |
| x86_64-apple-darwin      | 64-bit macOS (10.7+, Lion+)                |
| x86_64-pc-windows-gnu    | 64-bit MinGW (Windows 7+)                  |
| i686-pc-windows-msvc     | 32-bit MSVC (Windows 7+)                   |

Für die Zielarchitektur `x86_64-unknown-linux-gnu` steht noch ein weiteres Binary für
Betriebssysteme, die die [GNU C Library (glib)](https://www.gnu.org/software/libc)
in der Version 2.17 verwenden, bereit.

## Installation aus den Quellen

Das Projekt lässt sich auch direkt aus den Quellen kompilieren. Hierfür wird eine
aktuelle [Rust](https://www.rust-lang.org/)-Version (`>= 1.58`) mit dem Paketmanager
`cargo` benötigt.

Der aktuelle Entwicklungsstand lässt sich wie folgt installieren:

```bash
$ git clone https://github.com/deutsche-nationalbibliothek/pica-rs.git
$ cd pica-rs
$ cargo build --release
```

Das fertige Release-Binary befindet sich im Verzeichnis `target/release/pica`.

Wenn die Quelle nicht benötigt werden, kann das Projekt auch direkt über den Paketmanager
`cargo` installiert werden:

```bash
# Installation des aktuellen Entwicklungsversion
$ cargo install --git https://github.com/deutsche-nationalbibliothek/pica-rs \
     --branch main pica

# Installation der Version 0.13.0
$ cargo install --git https://github.com/deutsche-nationalbibliothek/pica-rs \
      --tag v0.13.0 pica

# Installation des Entwicklungszweigs "feat1"
$ cargo install --git https://github.com/deutsche-nationalbibliothek/pica-rs \
      --branch feat1 pica
```

Das fertige Programm befindet sich dann im Verzeichnis `.cargo/bin`.
