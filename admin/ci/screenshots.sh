#!/usr/bin/env bash
# Screenshots of every console page with example data, for the docs.
# Needs: a release build, curl, chromium. Prints "DEDE_ADMIN_SHOT: <name> <base64 png>".
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"
mkdir -p "$work/state" "$work/run" "$work/media/dede-film" "$work/media/cocuk" "$work/html"
printf '[Dede Film]\nnumber = 5\nfolder = dede-film\n\n[Çocuk Kanalı]\nnumber = 8\nfolder = cocuk\n' > "$work/vc.conf"
printf 'WG_ENDPOINT=haus.example.org:51820\nWG_DEVICE=10.77.0.2 key= Handy Mikail\nWG_DEVICE=10.77.0.3 key= Laptop\n' > "$work/remote.conf"
printf 'UPDATE_STATE=available\nUPDATE_CURRENT=0.1.0-test.1\nUPDATE_AVAILABLE=0.2.0\nUPDATE_CHECKED=2026-11-28T09:12\n' > "$work/update.conf"
printf 'TS_STATE=Running\nTS_ADDRESS=100.101.102.103\n' > "$work/tailscale.conf"
printf 'M3U_URL=https://iptv.example.com/get.php?token=geheim\n' > "$work/iptv.conf"
truncate -s 1450M "$work/media/dede-film/Die Feuerzangenbowle (1944).mp4"
truncate -s 980M "$work/media/dede-film/Emil und die Detektive.mkv"
truncate -s 1210M "$work/media/dede-film/Hababam Sınıfı.mp4"
port=18099
export DEDE_ADMIN_LISTEN="127.0.0.1:$port" DEDE_ADMIN_STATE="$work/state" DEDE_ADMIN_RUNTIME="$work/run" \
  DEDE_UI_CONFIG="$work/ui.conf" DEDE_CHANNELS_CONFIG="$work/vc.conf" DEDE_MEDIA_ROOT="$work/media" \
  DEDE_IPTV_CONFIG="$work/iptv.conf" DEDE_REMOTE_CONFIG="$work/remote.conf" DEDE_TS_CONFIG="$work/tailscale.conf" DEDE_UPDATE_STATE="$work/update.conf" DEDE_VERSION_FILE="$work/version"
"$here/target/release/dede-tv-admin" 2>/dev/null & pid=$!
trap 'kill $pid 2>/dev/null; rm -rf "$work"' EXIT
url="http://127.0.0.1:$port"
for _ in $(seq 50); do curl -fs "$url/setup" -o /dev/null && break; sleep 0.2; done
jar="$work/jar"
browser=$(command -v chromium || command -v chromium-browser || command -v google-chrome)

lang=de-DE
shot() { # name path [width height]
  local name=$1 path=$2 w=${3:-430} h=${4:-1000}
  curl -fs -b "$jar" "$url$path" -o "$work/html/$name.html"
  sed -i "s#href=\"/style.css\"#href=\"file://$here/static/style.css\"#" "$work/html/$name.html"
  LANGUAGE="${lang%%-*}" "$browser" --headless=new --no-sandbox --lang="$lang" --accept-lang="$lang" --disable-gpu --hide-scrollbars \
    --force-device-scale-factor=2 --window-size="$w,$h" \
    --screenshot="$work/$name.png" "file://$work/html/$name.html" >/dev/null 2>&1
  echo "DEDE_ADMIN_SHOT: $name $(base64 -w0 "$work/$name.png")"
}

curl -fs -c "$jar" "$url/setup" -o /dev/null
shot setup /setup 430 1000
form=$(awk '/dede_form/{print $7}' "$jar")
code=$(cat "$work/run/setup-code")
curl -fs -b "$jar" -c "$jar" -H "Origin: $url" --data-urlencode "form=$form" --data-urlencode "code=$code" \
  --data-urlencode "password=Beispiel-Passwort-1" --data-urlencode "repeat=Beispiel-Passwort-1" "$url/setup" -o /dev/null
csrf=$(curl -fs -b "$jar" "$url/" | grep -o 'name="csrf" value="[0-9a-f]*"' | head -1 | cut -d'"' -f4)
shot home / 430 800
shot channels /channels 430 900
shot channel-new /channels/new 430 900
shot channel-films /channels/5 430 1500
loc=$(curl -fs -b "$jar" -H "Origin: $url" --data "csrf=$csrf&back=/channels/5" -o /dev/null -w '%{redirect_url}' "$url/apply")
shot applying "${loc#$url}" 430 700
id=$(echo "$loc" | grep -o '/apply/[0-9a-f]*' | cut -d/ -f3)
mkdir -p "$work/state/results"; echo '{"ok":true}' > "$work/state/results/$id.json"
shot applied "${loc#$url}" 430 700
shot iptv /iptv 430 1100
shot language /language 430 900
shot remote /remote 430 1900
loc=$(curl -fs -b "$jar" -H "Origin: $url" --data "csrf=$csrf&name=Tablet&endpoint=haus.example.org" -o /dev/null -w '%{redirect_url}' "$url/remote/add")
id=$(echo "$loc" | grep -o '/apply/[0-9a-f]*' | cut -d/ -f3)
peer='[Interface]\nPrivateKey = BEISPIELbeispielBEISPIELbeispielBEISPIEL0=\nAddress = 10.77.0.4/32\n\n[Peer]\nPublicKey = SERVERserverSERVERserverSERVERserverSERVE0=\nEndpoint = haus.example.org:51820\nAllowedIPs = 10.77.0.1/32\nPersistentKeepalive = 25\n'
echo "{\"ok\":true,\"peer\":\"$peer\"}" > "$work/state/results/$id.json"
shot remote-device "${loc#$url}" 430 1500
loc=$(curl -fs -b "$jar" -H "Origin: $url" --data "csrf=$csrf" -o /dev/null -w '%{redirect_url}' "$url/remote/tailscale/login")
id=$(echo "$loc" | grep -o '/apply/[0-9a-f]*' | cut -d/ -f3)
echo '{"ok":true,"login":"https://login.tailscale.com/a/1a2b3c4d5e6f"}' > "$work/state/results/$id.json"
shot tailscale-login "${loc#$url}" 430 1100
shot updates /updates 430 1100
shot system /system 430 1300
loc=$(curl -fs -b "$jar" -H "Origin: $url" --data "csrf=$csrf" -o /dev/null -w '%{redirect_url}' "$url/system/diagnostics")
id=$(echo "$loc" | grep -o '/apply/[0-9a-f]*' | cut -d/ -f3)
echo '{"ok":true,"report":{"units":{"tv":"active","guard":"active","console":"active","tailscale":"active","wireguard":"inactive"},"failed":[],"uptime_s":273840,"media_free_mb":118272,"media_total_mb":235520,"system":"Debian GNU/Linux 13 (trixie)"}}' > "$work/state/results/$id.json"
shot diagnostics "${loc#$url}" 430 1100
printf 'LANGUAGE=tr\n' > "$work/ui.conf"
lang=tr-TR
shot home-tr / 430 800
shot channel-films-tr /channels/5 430 1500
curl -fs -b "$jar" -c "$jar" -H "Origin: $url" --data "csrf=$csrf" "$url/logout" -o /dev/null
printf 'LANGUAGE=de\n' > "$work/ui.conf"
lang=de-DE
curl -fs -c "$jar" "$url/login" -o /dev/null
shot login /login 430 700
