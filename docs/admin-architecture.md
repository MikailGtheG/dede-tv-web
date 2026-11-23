# Dede TV Verwaltung: Architektur (P3)

Ziel: Ein Angehöriger ohne Linux-Kenntnisse richtet Sender, eigene Filme,
Sprache und Fernzugriff per Handy oder Laptop ein. Kein Terminal, keine
Pfade, keine Standardpasswörter.

## Bausteine

| Baustein | Läuft als | Aufgabe |
|---|---|---|
| `dede-tv-admin` (Rust, dieses Repo) | Systemnutzer `dede-admin`, ohne Login | HTTP-Oberfläche, Anmeldung, Formulare, Uploads |
| `dede-admin-apply` (Python, dede-tv-os) | root, nur per `systemd.path` ausgelöst | prüft und übernimmt Änderungsaufträge |
| Kodi / `dede-virtual-sync` | `dede` | wie bisher, liest nur die übernommene Konfiguration |

Der Webdienst hat keine Root-Rechte und kein sudo. Änderungen legt er als
JSON-Auftrag in `/var/lib/dede-tv-admin/requests/` ab. Eine Path-Unit
startet `dede-admin-apply`, das jeden Auftrag **erneut vollständig
validiert** (Schema, Längen, URL-Schemata, Ordnernamen) und dann schreibt:

- `/etc/dede-tv/iptv.conf` (M3U/EPG-Adresse)
- `/etc/dede-tv/virtual-channels.conf` (eigene Sender)
- `/etc/dede-tv/ui.conf` (Sprache) → Kodi-Einstellung `locale.language`
- `/etc/dede-tv/remote-access.conf` → nftables, Tailscale, WireGuard

Danach `systemctl restart dede-kodi` nur, wenn sich etwas für den
Fernseher geändert hat. Ergebnis und Fehlertext gehen als JSON zurück
nach `results/`, die Oberfläche zeigt sie in Klartext an.

Medien: `/srv/dede-tv/media` gehört `dede-admin:dede` (2750). Uploads
landen dort direkt; Ordner werden nur aus einer Auswahlliste gewählt oder
mit einem geprüften Namen (keine `/`, kein `..`, keine Steuerzeichen) neu
angelegt.

## Anmeldung

- Erstes Öffnen: Die Oberfläche fragt nach einem **Einrichtungscode**,
  den der Fernseher anzeigt (8 Ziffern, 30 min gültig, neu nach jedem
  Neustart, solange kein Passwort gesetzt ist). Danach wird ein Passwort
  (mind. 10 Zeichen) gesetzt. Es gibt kein Standardpasswort.
- Passwort-Hash: Argon2id. Sitzung: zufälliges 256-bit-Token im Cookie
  `HttpOnly; SameSite=Strict` (`Secure`, sobald HTTPS aktiv ist), 12 h.
- CSRF: Token pro Sitzung in jedem Formular, plus `Origin`-Prüfung.
- Sperre: nach 5 Fehlversuchen pro Adresse 15 min, global gedrosselt.
- Passwort vergessen: neuer Einrichtungscode über das Menü am Fernseher
  (nur mit Fernbedienung erreichbar, also nur vor Ort).
- Protokoll: jede Änderung mit Zeit, Quelle und Art im Journal, ohne
  Passwörter, Tokens oder IPTV-Adressen.

## Seiten (große Schrift, ein Schritt pro Seite)

1. **Start**: Status (Fernseher läuft, aktueller Sender, Netzwerk,
   Fernzugriff), große Knöpfe zu den Bereichen.
2. **Fernsehsender**: IPTV-Adresse eintragen und testen (nur http/https,
   keine Loopback-/Link-Local-Ziele).
3. **Eigene Sender**: Liste mit Nummer, Name, Filmanzahl, „Jetzt läuft“.
   „Neuer Sender“ → Name → Nummer (freie Nummern vorgeschlagen) → Ordner
   wählen oder neu anlegen → Filme hochladen → fertig.
4. **Filme**: Ordner ansehen, Filme hochladen und löschen.
5. **Sprache**: Deutsch, Türkçe, English. Gilt für Fernseher und
   Oberfläche. Ist **nur hier** umstellbar, nicht am Fernseher.
6. **Fernzugriff**: lokales Netz (an/aus), Tailscale (Anmeldelink
   anzeigen), WireGuard (Gerät hinzufügen → QR-Code fürs Handy).
7. **Hilfe & Diagnose**: Neustart Fernseher/Gerät, Diagnosepaket.

## Erreichbarkeit

`dede-tv-admin` lauscht auf Port 8080. nftables lässt ihn nur zu von:

- lokalem Netz (RFC 1918, IPv6 ULA/Link-Local), wenn aktiviert,
- `tailscale0`, wenn Tailscale aktiv ist,
- `wg0`, wenn WireGuard aktiv ist.

Kodi-JSON-RPC (9090), die Datenbank und Steuer-Sockets bleiben nur auf
Loopback und werden nie an den Browser durchgereicht.

## Sprachen

- Oberfläche: Meldungskataloge `de`, `tr`, `en` im Binärprogramm.
- Fernseher: Skin-Texte über `strings.po` (de_de, tr_tr, en_gb), Kodi
  über `locale.language`. Die Kodi-Sprachpakete `resource.language.de_de`
  und `resource.language.tr_tr` werden beim Image-Bau mit Prüfsumme
  eingebunden (Debian liefert nur en_gb).

## Umsetzungsschritte (je ein Draft-PR mit Tests)

1. EPG für eigene Sender (erledigt, dede-tv-os #212).
2. `dede-tv-admin`: Einrichtungscode, Passwort, Sitzungen, Startseite;
   `dede-admin-apply` mit Auftragsschema; Einbau ins Image; E2E per curl
   im QEMU-Gast.
3. Eigene Sender und Filme (Upload) über die Oberfläche.
4. Sprache DE/TR/EN (Oberfläche, Skin, Kodi).
5. IPTV-Adresse über die Oberfläche.
6. Fernzugriff: lokales Netz, Tailscale, WireGuard.
7. Diagnose und Neustart.
