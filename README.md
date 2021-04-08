# narnia

narnia is a fast static webserver specifically designed for Tor hidden services. It's also able to spawn a Tor thread and expose itself as a hidden service, simply by setting an additional commandline option.

narnia is hosting a mirror of its own source code on http://3wisi2bfpxplne5wlwz4l5ucvsbaozbteaqnm62oxzmgwhb2qqxvsuyd.onion/.

```
# Start the http server and serve files in www/
narnia -B '[::1]:1337' -w www/
# Serve www/ and enable directory listing
narnia -B '[::1]:1337' -Lw www/
# Serve www/ on a unix domain socket
# The path needs to start with either . or /
narnia -B ./narnia.sock -w www/
# Serve www/ on a hidden service
# The hidden service address is in `data/hs/hostname`
narnia -D data/ -w www/
# Serve www/ but chroot into it beforehand, verbose logs
narnia -vv -B '[::1]:1337' -w / -C www/
```

## Comparison of http response headers

**narnia**
```
< HTTP/1.1 200 OK
< content-length: 1337
< accept-ranges: bytes
< date: Thu, 01 Jan 1970 00:00:00 GMT
< content-type: text/html; charset=utf-8
< x-content-type-options: nosniff
< referrer-policy: no-referrer
```

**onionshare**
```
< HTTP/1.0 200 OK
< Content-Type: text/html; charset=utf-8
< Content-Length: 1337
< X-Frame-Options: DENY
< X-Xss-Protection: 1; mode=block
< X-Content-Type-Options: nosniff
< Referrer-Policy: no-referrer
< Server: OnionShare
< Content-Security-Policy: default-src 'self'; style-src 'self'; script-src 'self'; img-src 'self' data:;
< Date: Mon, 05 Apr 2021 19:08:54 GMT
```

**nginx**
```
< HTTP/1.1 200 OK
< Server: nginx
< Date: Mon, 05 Apr 2021 19:04:42 GMT
< Content-Type: text/html
< Content-Length: 1337
< Last-Modified: Mon, 05 Apr 2021 19:04:33 GMT
< Connection: keep-alive
< ETag: "606b5f41-539"
< Accept-Ranges: bytes
```

## Static binary

**Linux**
```
sudo pacman -S musl
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl --features=vendored
strip target/x86_64-unknown-linux-musl/release/narnia
ldd target/x86_64-unknown-linux-musl/release/narnia
```

**Windows**
```
pacman -S mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu --features=vendored
x86_64-w64-mingw32-strip target/x86_64-pc-windows-gnu/release/narnia.exe
file target/x86_64-pc-windows-gnu/release/narnia.exe
```

## License

GPLv3+
