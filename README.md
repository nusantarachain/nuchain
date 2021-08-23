# NUCHAIN / RANTARA &middot; [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](#LICENSE) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/CONTRIBUTING.adoc)

Nuchain (Nusantara Chain) atau Rantara (Rantai Nusantara) adalah sarana untuk menuju Indonesia Society 5.0 melalui teknologi [blockchain](https://id.wikipedia.org/wiki/Rantai_blok).

## Motivasi

Nuchain diciptakan dengan tujuan untuk menyongsong Indonesia Society 5.0 dan Web 3.0 melalui sistem terdistribusi dan aman yang disebut dengan rantai blok (blockchain).

Mengapa blockchain? 

* Hampir semua aplikasi internet yang ada saat ini berjalan secara terpusat (centralized), salah satu semangat Web 3.0 adalah aplikasi terdistribusi. Blockchain yang secara alamiahnya adalah sistem terdistribusi bisa dijadikan fondasi untuk menuju era aplikasi internet terdistribusi.
* Keamanan, sistem blockchain diamankan menggunakan kriptografi di lapisan paling dasarnya, memberikan jaminan keamanan tinggi.
* Blockchain merupakan *trustless system* sangat bisa diandalkan untuk membangun aplikasi yang membutuhkan jaminan legitimasi tanpa perlu mempercayai satu sama lain.
* Nuchain bukan didesain untuk menjadi *crypto currency*, tetapi sebagai platform aplikasi terdistibusi (DApps) yang nantinya akan menjadi ekosistem baru di era teknologi digital yang membutuhkan keterbukaan, kejelasan, legitimasi, dan *good governance*.

## Roadmap

Nuchain akan dikembangan dalam **6 fase**:

### Fase 00: Inisiasi

Marupakan fase awal pengumpulan dukungan dan finalisasi desain sistem.

Pada fase ini sistem telah jalan di versi ujicoba, runtime versi 0.

### Fase 01: Genesis

Merupakan fase awal sistem Nuchain mulai dijalankan, pada fase ini Nuchain menggunakan mode PoA (Proof of Authority) sembari menunggu terkumpulnya jumlah validator independen yang cukup untuk nantinya akan beralih ke mode NPoS (Nominated Proof of Stake).

Pada fase ini pengguna awal bisa melakukan:

* **Klaim** token dari fase 00.
* **Stake** token dan mengajukan diri untuk menjadi validator atau nominator.

### Fase 02: NPoS

Pada masa ini jaringan Nuchain sudah tidak lagi menggunakan PoA, tetapi telah beralih ke NPoS (Nominated Proof of Stake) yang mana jaringan telah dijalankan oleh para validator independen secara terdesentralisasi.

Walaupun PoA telah digantikan dengan NPoS namun masih ada fitur *Sudo* yang digunakan oleh otoritas Nuchain untuk keperluan mengelola jaringan validator dan upgrade ke fase-fase berikutnya.

### Fase 03: Governance

Merupakan fase ujicoba konsep *governance* di Nuchain.

### Fase 04: Revolt

Fase revolusi penggantian sistem otoriter menjadi sistem demokrasi. Pada fase ini fitur *Sudo* akan dihapus, dan setiap keputusan selanjutnya akan ditentukan oleh dewan dan komite teknis.

Para anggota dewan dipilih oleh user, sementara komite teknis pada fase ini adalah para pengembang awal.

Fungsi-fungsi RPC atau Ekstrinsik strategis hanya bisa dieksekusi (dispatch) berdasarkan kesepakatan bersama dewan dan komite teknis menggunakan sistem voting.

### Fase 05: Pengembangan

Fase pengembangan untuk memaksimalkan pemanfaatan Nuchain dalam hal layanan publik dan good governance.

Pada fase ini fitur *smart contract* akan diaktifkan, dan user bisa mulai membuat DApps di atas jaringan Nuchain.

## Pengembangan

Saat ini Nuchain sedang dalam **Fase 01 - Genesis** dan sementara ini kehidupannya bisa dipantau melalui [Nuchain Dashboard](https://dashboard.nuchain.network).

Nuchain dikembangkan menggunakan [Substrate](https://substrate.dev) sumber terbuka dan siapapun bisa ikut join untuk mendukung perkembangan Nuchain.

## Instalasi

Instalasi ini dibutuhkan apabila ingin menjadi kontributor dengan menjalankan *node* yang akan bekerja sebagai validator atau observer.

Ada beberapa cara, yang pertama download pre-built binary dari halaman [Releases](https://github.com/nusantarachain/nuchain/releases), unduh sesuai dengan sistem operasi yang kamu gunakan.


### Docker

Cara tercepat bisa menggunakan Docker:

```bash
$ docker run --rm \
  -v '/var/data:/data' \
  -p '9933:9933' \
  -p '9944:9944' \
  -p '30333:30333' \
   --name nuchain anvie/nuchain:latest-alpine \
   nuchain --base-path=/data
```


Atau melakukan kompilasi sendiri dari kode sumber dengan mengikuti panduan sebagai berikut:

### Dari Kode Sumber

Nuchain membutuhkan beberapa dependensi untuk bisa melakukan kompilasi. Berikut adalah panduan untuk memasang dependensi pada setiap sistem operasi:

### MacOS

```bash
# Install Homebrew if necessary https://brew.sh/
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"

# Make sure Homebrew is up-to-date, install openssl and cmake
brew update
brew install openssl cmake
```

### Ubuntu/Debian

```bash
sudo apt update
# May prompt for location information
sudo apt install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl libz-dev
```

### Arch Linux

```bash
pacman -Syu --needed --noconfirm cmake gcc openssl-1.0 pkgconf git clang
export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0"
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
```

## Persiapan Lingkungan Kerja

Karena Nuchain ditulis menggunakan bahasa pemrograman Rust maka diperlukan beberapa komponen yang dibutuhkan berkaitan dengan Rust.

Gunakan rustup untuk memasang Rust:

```bash
# Install
curl https://sh.rustup.rs -sSf | sh
# Configure
source ~/.cargo/env
```

Nuchain menggunakan [WebAssembly](https://webassembly.org/) sebagai core on-chain runtime-nya dan sebagai VM untuk kontrak pintar (smart contract)-nya, sehingga diperlukan Wasm toolchain untuk Rust:

```bash
rustup default nightly-2021-01-29
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-01-29
```

**CATATAN**: Pada contoh di atas menggunakan Rust versi nightly build 2020-10-05 karena telah teruji bisa melakukan kompilasi dengan lancar, namun ini hanya sebagai contoh apabila pada kemudian hari ada perubahan di Rust nightly yang membuat kode Nuchain tidak lagi bisa dikompilasi (tidak kompatibel). Kamu bebas apabila mau menggunakan latest nightly.

```bash
git clone https://github.com/nusantarachain/nuchain.git
```

## Kompilasi

Untuk melakukan kompilasi cukup ketikkan:

```bash
make build
```

Output akan berada di `target/release/nuchain`.

## Menjalankan

Perintah berikut akan menjalankan Nuchain node dengan identitas node `node01` dan jalan secara lokal.

```bash
nuchain --name=node01
```

Apabila *node* ingin dijalankan sebagai validator maka tambahkan parameter `--validator`, contoh:

```bash
nuchain --validator --name=node01
```

Untuk lebih detailnya tentang cara menjadi validator bisa baca [menjadi validator](https://github.com/nusantarachain/nuchain/wiki/Menjadi-Validator).

## Glossary

* Validator - adalah node/komputer yang berjalan membuat blok dan memvalidasi setiap blok yang tercipta. Validator bekerja untuk mengamankan jaringan dan mendapatkan benefit dari *rewards* untuk setiap blok yang dibuatnya.
* ARA - Adalah kode unit native token atau bahan bakar yang diperlukan untuk menunjang keberlangsungan jaringan. ARA ini digunakan untuk mengamankan sistem dari serangan DoS dan memberikan *rewards* kepada para validator yang telah mengamankan jaringan.
* Nominator - adalah individu yang ikut berpartisipasi sebagai validator namun tidak dalam bentuk node/komputer tetapi dalam bentuk *stake* dengan meminjamkan ARA ke validator agar validator memiliki kesempatan besar terpilih sebagai pembuat blok di setiap putarannya. Nominator mendapatkan *rewards* sesuai dengan nilai yang di-*stake* dan prosentase bagi hasil yang ditetapkan oleh validator.
* DApps - adalah aplikasi yang dibuat untuk bisa jalan di atas jaringan blockchain.

## Kontribusi

Tertarik dengan proyek ini? Anda bisa ikut kontribusi dengan beberapa cara:

* [Menjadi validator](https://github.com/nusantarachain/nuchain/wiki/Menjadi-Validator).
* [Menjadi nominator](https://github.com/nusantarachain/nuchain/wiki/Menjadi-Nominator).
* Menjadi pengembang untuk peralatan pendukung Nuchain seperti mobile App, Block scanner, dll.
* Menjadi pengembang kontrak pintar (Smart Contract)/dApps, contoh bisal lihat [di sini](https://github.com/nusantarachain/onchain-apps).

## Dokumentasi

Untuk dokumentasi lebih lengkap baca [Nuchain Wiki](https://github.com/nusantarachain/nuchain/wiki).

## Komunitas

Bergabunglah dengan komunitas untuk diskusi tentang Nuchain melalui beberapa kanal berikut:

* Chat: [Element](https://app.element.io/#/room/!aYWUxhUvutqbMBQIsN:matrix.org), [Telegram](https://t.me/nusantarachain)
* Email: nusantarachain@gmail.com

## Contributions & Code of Conduct

Please follow the contributions guidelines as outlined in [`docs/CONTRIBUTING.adoc`](docs/CONTRIBUTING.adoc). In all communications and contributions, this project follows the [Contributor Covenant Code of Conduct](docs/CODE_OF_CONDUCT.md).

## Security

The security policy and procedures can be found in [`docs/SECURITY.md`](docs/SECURITY.md).

## License

Lisensi Nuchain mengikuti lisensi dari [Substrate](https://substrate.dev):

- Substrate Primitives (`sp-*`), Frame (`frame-*`) and the pallets (`pallets-*`), binaries (`/bin`) and all other utilities are licensed under [Apache 2.0](LICENSE-APACHE2).
- Substrate Client (`/client/*` / `sc-*`) is licensed under [GPL v3.0 with a classpath linking exception](LICENSE-GPL3).

The reason for the split-licensing is to ensure that for the vast majority of teams using Substrate to create feature-chains, then all changes can be made entirely in Apache2-licensed code, allowing teams full freedom over what and how they release and giving licensing clarity to commercial teams.

In the interests of the community, we require any deeper improvements made to Substrate's core logic (e.g. Substrate's internal consensus, crypto or database code) to be contributed back so everyone can benefit.
