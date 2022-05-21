# certstream-rust

Small funemployment Rust project to build a CLI program to read the websocket stream from <https://certstream.calidog.io> and log all certificate domains to a RocksDB database, plus a small utility to dump the gathered domains from RocksDB.

## Building

```
git clone git@github.com:hrbrmstr/certstream-rust
cargo build --release 
```

## Installing

```
cargo install --git https://github.com/hrbrmstr/certstream-rust
```

## Read from CertStream websocket

```
USAGE:
    certstream [OPTIONS] --dbpath <DBPATH>

OPTIONS:
    -d, --dbpath <DBPATH>        
    -h, --help                   Print help information
    -p, --patience <PATIENCE>    [default: 5]
    -s, --server <SERVER>        [default: wss://certstream.calidog.io/]
    -V, --version                Print version information
```

e.g.
```
certstream --dbpath=~/Data/cert_doms.1 # kill or ^C to stop
```

## See the domains

```
USAGE:
    dumpdoms --dbpath <DBPATH>
```

e.g.
```
dumpdoms --dbpath=~/Data/cert_doms.1 | tail -30
zyw017777.direct.quickconnect.to
zywolapki.polmedia.webd.pl
zyx.dzign4u.com
zyxus.keenetic.pro
zyy.aob.mybluehost.me
zyy.kqv.mybluehost.me
zyyid.muo.cc
zyzfnh.net
zyzyxllc.com
zz188.xyz
zz3wd.com
zzalale.com
zzb.cx
zzbolt.org
zzdzkbuxtcwxiru62jefplj2dy.ap-southeast-2.es.amazonaws.com
zzekasd.xyz
zzgljy.com
zznptabez1jijw5z.myfritz.net
zzsmarketing.com
zzx.admin.thinker.vc
zzx.api.thinker.vc
zzx.oauth.thinker.vc
zzx52.direct.quickconnect.to
zzyhomv.shop
zzyzxvape.synology.me
zzz.com.ar
zzz.org.ua
zzz.zzz.org.ua
zzz4ncfxemkkuulse6ctkwzrau.eu-west-3.es.amazonaws.com
zzzzbw.cn
```
