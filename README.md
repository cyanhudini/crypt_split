
# Motivation

Aufgrund stetig steigender Cyber Angriffe [Mad22] auf technische Infrastruktur, wird es immer wichtiger Wege zu finden eigene Daten zu sichern und vor fremden Augen zu schützen, hierbei
spielen Cloud Dienste eine zentrale Rolle. Zwar vermarkten namhafte Cloud-Anbieter ihre Lösungen als vertraulich, indem sie die zu speichernden Daten verschlüsseln.
Allerdings ergibt sich aus den Nutzungsbedingungen, dass diese Anbieter (z.B. Apple) immer einen Master- Key besitzen und damit potenziell persönliche und
vertrauliche Informationen einsehen können. In dieser Bachelorarbeit wird eine Lösung präsentiert, die es ermöglicht, Daten gezielt in verschlüsselte Blöcke aufzuteilen
und über mehrere Clouds hinweg zu verteilen. Dadurch wird eine Rekonstruktion dieser Dateien ohne eine Datenbank, die als Karte dient, unmäglich.

Um dies zu erreichen, werden die zu sichernden Daten in 4-KiB große Blöcke aufgeteilt, nachdem sie mit einem AE-Schema verschlüsselt wurden.
In diesem Fall verwenden wir RIV, da dieser als ”Nonce-Misuse” resistenter Algorithmus entworfen wurde. Zudem nutzen wir den Hashwert des Inhalts der resultierenden Dateien als Namen.

Damit eine Ordnung hergestellt werden kann, die zur Rekonstruktion dieser Dateien genutzt wird, benutzen wir eine Eigenschaft der Blöcke einer Blockchain: An jeden Datenblock wird der Hash des vorigen Blockes angehängt.
Den Hash des ersten Blocks schreiben wir in die Metadaten. Da wir davon ausgehen müssen, dass diese Blöcke zufällig über mehrere Ordner hinweg gespeichert werden, hängen wir zusätzlich zum eigentlichen Hashwert noch den Hashwert des Pfades an.
Aufgrund der Charakteristik von Hashwerten, welche immer eine bestimmte Länge besitzen, wissen wir ab welcher Stelle wir den Pfad erwarten müssen.

Anschließend werden der originale Dateiname und die zu den Datenblöcken zugehörigen Hashwerte als Key-Value-Paar in einer Redis-Datenbank gespeichert.
Die Metadaten enthalten zusätzlich Informationen über die Nonce und eventuell eine Checksumme für die gesamte Datei, um diese nach vollständiger Rekonstruktion nochmals zu prüfen.

Um den Schlüssel zu generieren, der anfangs zur Verschlüsselung genutzt wird, verwenden wir scrypt, eine Key-Derivation-Funktion. Dabei wird das Login-Passwort zur Datenbank genutzt.
Außerdem muss die Sicherung des Schlüssels beachtet werden. Dies können wir durch eine ”Ver-XOR-ung” mit Passwort-Hash und Schlüssel erreichen.

Standardmäßig verwenden wir den SHA-256-Algorithmus zum Erzeugen von Hashwerten, da dieser bis dato ein weit verbreiteter und sicherer Standard ist. Abschließend, wenn die Kapazitäten ausreichen, würde ein entsprechendes User-Interface programmiert werden, um die Nutzung des Programms angenehmer und eventuell plattformübergreifend zu gestalten.


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
