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

## Building

### OpenBSD

You need to install rust, autoconf and automake. You're getting asked for a version, select the latest one and take note of the first two numbers of the version. You can look this up with `pkg_info` if you forget them. This example output is from OpenBSD 6.8.
```
$ doas pkg_add autoconf automake
quirks-3.442 signed on 2021-04-08T13:45:25Z
Ambiguous: choose package for autoconf
a	0: <None>
	1: autoconf-2.13p4
	2: autoconf-2.52p6
	3: autoconf-2.54p6
	4: autoconf-2.56p5
	5: autoconf-2.57p5
	6: autoconf-2.58p5
	7: autoconf-2.59p5
	8: autoconf-2.60p5
	9: autoconf-2.61p5
	10: autoconf-2.62p2
	11: autoconf-2.63p1
	12: autoconf-2.64p1
	13: autoconf-2.65p1
	14: autoconf-2.67p1
	15: autoconf-2.68p1
	16: autoconf-2.69p3
Your choice: 16
autoconf-2.69p3:metaauto-1.0p4: ok
autoconf-2.69p3: ok
Ambiguous: choose package for automake
a	0: <None>
	1: automake-1.4.6p5
	2: automake-1.8.5p9
	3: automake-1.9.6p12
	4: automake-1.10.3p9
	5: automake-1.11.6p3
	6: automake-1.12.6p2
	7: automake-1.13.4p2
	8: automake-1.14.1p1
	9: automake-1.15.1
	10: automake-1.16.2
Your choice: 10
automake-1.16.2: ok
```

Next, pass the versions to cargo build:
```bash
AUTOCONF_VERSION=2.69 AUTOMAKE_VERSION=1.16 cargo build
```

### Alpine

```
doas apk add make autoconf automake openssl-dev
```

## License

GPLv3+
