## SETUP

## Methodik

Um dies zu erreichen, werden die zu sichernden Daten in 4-KiB große Blöcke aufgeteilt, nachdem sie mit einem AE-Schema verschl¨usselt wurden.
In diesem Fall verwenden wir RIV, da dieser als ”Nonce-Misuse” resistenter Algorithmus entworfen wurde. Zudem nutzen wir den Hashwert des Inhalts der resultieren- den Dateien als Namen.

Damit eine Ordnung hergestellt werden kann, die zur Rekonstruktion dieser Dateien genutzt wird, benutzen wir eine Eigenschaft der Blöcke einer Blockchain: An jeden Datenblock wird der Hash des vorigen Blockes angehängt.
Den Hash des ersten Blocks schreiben wir in die Metadaten. Da wir davon ausgehen müssen, dass diese Blöcke zufällig über mehrere Ordner hinweg gespeichert werden, hängen wir zusätzlich zum eigentlichen Hashwert noch den Hashwert des Pfades an.
Aufgrund der Charakteristik von Hashwerten, welche immer eine bestimmte Länge besitzen, wissen wir ab welcher Stelle wir den Pfad erwarten müssen.

Anschließend werden der originale Dateiname und die zu den Datenblöcken zugehörigen Hashwerte als Key-Value-Paar in einer Redis-Datenbank gespeichert.
Die Metadaten enthalten zusätzlich Informationen über die Nonce und eventuell eine Checksumme für die gesamte Datei, um diese nach vollständiger Rekonstruktion nochmals zu prüfen.

Um den Schlüssel zu generieren, der anfangs zur Verschlüsselung genutzt wird, verwenden wir scrypt, eine Key-Derivation-Funktion. Dabei wird das Login-Passwort zur Datenbank genutzt.
Außerdem muss die Sicherung des Schlüssels beachtet werden. Dies können wir durch eine ”Ver-XOR-ung” mit Passwort-Hash und Schlüssel erreichen.

Standardmäßig verwenden wir den SHA-256-Algorithmus zum Erzeugen von Hashwerten, da dieser bis dato ein weit verbreiteter und sicherer Standard ist. Abschließend, wenn die Kapazitäten ausreichen, würde ein entsprechendes User-Interface programmiert werden, um die Nutzung des Programms angenehmer und eventuell plattformübergreifend zu gestalten.