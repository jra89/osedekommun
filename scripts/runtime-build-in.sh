#!/bin/bash
# Runs INSIDE the old-glibc container started by scripts/build-runtime.sh.
# Rebuilds nginx and PHP with the same options as the original builds and
# installs them into /work/runtime.
set -e
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends \
    build-essential make wget ca-certificates pkg-config \
    libpcre3-dev zlib1g-dev libssl-dev

cd /tmp

# --- nginx 1.28.3 (same configure flags as before) ---
wget -q https://nginx.org/download/nginx-1.28.3.tar.gz
tar xzf nginx-1.28.3.tar.gz
cd nginx-1.28.3
./configure --prefix=/opt/nginx --sbin-path=sbin/nginx \
    --conf-path=conf/nginx.conf --with-http_ssl_module
make -j"$(nproc)"
make install
cd /tmp

# --- PHP 8.3.17 (minimal static build, same options as the original) ---
wget -q https://www.php.net/distributions/php-8.3.17.tar.gz
tar xzf php-8.3.17.tar.gz
cd php-8.3.17
./configure --prefix=/opt/php --disable-all \
    --enable-cli --enable-cgi --enable-fpm \
    --enable-pdo --with-pdo-mysql=mysqlnd \
    --with-mysqli=mysqlnd --enable-mysqlnd \
    --enable-session --enable-filter --enable-ctype \
    --enable-hash --enable-json --enable-tokenizer \
    --with-config-file-path=/opt/php/etc
make -j"$(nproc)"
make install

# --- install into the project tree ---
rm -rf /work/runtime/nginx /work/runtime/php
cp -a /opt/nginx /work/runtime/nginx
cp -a /opt/php /work/runtime/php
# bundle nginx shared-library deps, as before
mkdir -p /work/runtime/nginx/extra-libs
cp -a /usr/lib/x86_64-linux-gnu/libpcre2-8.so.0* /work/runtime/nginx/extra-libs/
cp -a /usr/lib/x86_64-linux-gnu/libz.so.1* /work/runtime/nginx/extra-libs/

# --- verify everything links against this container's glibc ---
echo "=== ldd check (no 'not found' allowed) ==="
if ldd /work/runtime/nginx/sbin/nginx | grep -qi 'not found'; then exit 1; fi
if ldd /work/runtime/php/sbin/php-fpm | grep -qi 'not found'; then exit 1; fi
/work/runtime/nginx/sbin/nginx -v 2>&1
/work/runtime/php/sbin/php-fpm -v 2>&1 | head -1
echo BUILD-DONE
