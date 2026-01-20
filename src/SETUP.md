# crypt_split - Setup und Benutzung

## Voraussetzungen

### Systemanforderungen


- Rust (Edition 2021 oder neuer)
- Docker und Docker Compose
- Redis (hier via Dockerimage)

### Installation von Rust

Falls Rust noch nicht installiert ist:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```


## Installation

### 1. Repository klonen

```bash
git clone git@github.com:cyanhudini/crypt_split.git
cd split_hash_crypt_distr
```

### 2. Redis-Datenbank starten

Die Redis-Datenbank wird über Docker Compose bereitgestellt:

```bash
docker-compose up -d
```

Redis Commander ist als Weboberfläche auf dem Port 8081 erreichbar

### 3. Umgebungsvariablen konfigurieren


```bash
REDIS_URL=redis://127.0.0.1:6379
CONFIG_PATH=local_cloud.json
```

### 4. Cloud-Pfade konfigurieren

Um die Speicherorte für die Chunks festzulegen, müssen die Pfade in der local_cloud.json angepasst werden

Beispiel:
```json
{
    "local_cloud_paths" : [
        "/split_hash_crypt_distr/test/cloud1",
        "/split_hash_crypt_distr/test/cloud2",
        "/split_hash_crypt_distr/test/cloud3"
    ]
}
```

### 5. Projekt kompilieren

```bash
cargo build --release
```

Die ausfühhrbare Datei befindet sich unter `target/release/crypt_split`.


## Benutzung

### Grundlegende Befehle

```bash
crypt_split <BEFEHL> [OPTIONEN]
```

### Verfügbare Befehle

#### init - Schlüssel initialisieren

Initialisiert einen neuen Master-Schlüssel. Dieser Schritt muss vor der ersten Nutzung ausgeführt werden.

```bash
crypt_split init
```

Dieses Passwort wird zum Entsperren des Schlüssel verwendet. Falls dieses Passwort verloren geht, gehen somit auch die gesichterten Daten verloren.

#### encrypt - Datei verschlüsseln und aufteilen

Verschlüsselt eine Datei und teilt sie in Chunks auf:

```bash
crypt_split encrypt -i <EINGABEDATEI> [-o <AUSGABEPFAD>]
```

Optionen:
- `-i, --input-file`: Pfad zur zu verschlüsselnden Datei (erforderlich)
- `-o, --output-path`: Ausgabeverzeichnis für die Chunks (Standard: ./chunks)

Beispiel:

```bash
crypt_split encrypt -i dokument.pdf -o ./chunks
```

#### distr - Chunks verteilen

Verteilt die verschlüsselten Chunks auf die konfigurierten Speicherorte:

```bash
crypt_split distr -c <CHUNKS_PFAD> -f <DATEINAME>
```

Optionen:
- `-c, --chunks-path`: Pfad zum Chunks-Verzeichnis
- `-f, --file-name`: Name der zu verteilenden Datei

Beispiel:

```bash
crypt_split distr -c ./chunks -f dokument.pdf
```

#### encrypt-then-distribute (WIP)

Eine Verkettung von Encrypt und Distribute, damit beide Befehle nicht einzeln eingegeben werden müssen. 


```bash
crypt_split encrypt-then-distribute 
```

#### list - Gespeicherte Dateien auflisten (WIP)

Listet alle in der Datenbank gespeicherten Dateien auf:

```bash
crypt_split list
```

#### reconstruct - Datei wiederherstellen (WIP)

Rekonstruiert eine zuvor verschlüsselte und verteilte Datei:

```bash
crypt_split reconstruct
```

#### delete - Datei löschen (WIP)

Löscht eine gespeicherte Datei und ihre Chunks:

```bash
crypt_split delete
```

## Fehlerbehebung
