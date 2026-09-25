#!/bin/sh
# ctf.sh - prepare the tree for student deployment on a lab server.
#
# Removes the answer key and git metadata so students cannot cheat by
# finding the vulnerability writeups, working exploits, or git history:
#   - poc/              working exploit scripts
#   - vulnerabilities/  answer key
#   - .git              repository metadata / full history
#   - .gitignore        any .gitignore files in the tree
#
# Run from anywhere; it operates on the directory it lives in:
#   ./ctf.sh
set -e
ROOT=$(cd "$(dirname "$0")" && pwd)
cd "$ROOT"

for target in poc vulnerabilities .git; do
    if [ -e "$target" ]; then
        rm -rf "$target"
        echo "removed $target/"
    else
        echo "not present: $target/"
    fi
done

find . -name .gitignore -type f -print -delete

echo
echo "done - the tree no longer contains poc/, vulnerabilities/, .git or .gitignore files"
