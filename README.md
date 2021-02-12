# Nuchain &middot; [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](#LICENSE) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/CONTRIBUTING.adoc)

Blockchain untuk Nusantara menuju Indonesia Society 5.0.

Nuchain diciptakan dengan tujuan untuk menyongsong era Web 3.0 ([Web 3.0 Vision](https://web3.foundation/about/)) dan membangun Indonesia Society 5.0 melalui kehebatan sistem terdistribusi dan aman yang disebut dengan rantai blok (Blockchain).

Nuchain dikembangkan menggunakan [Substrate](https://substrate.dev) sumber terbuka dan siapapun bisa ikut join untuk mendukung perkembangan Nuchain.

## Instalasi

Instalasi ini dibutuhkan apabila ingin menjadi kontributor dengan menjalankan *node* yang akan bekerja sebagai validator atau observer.

Ada beberapa cara, yang pertama download pre-built binary dari halaman [Releases](https://github.com/nusantarachain/nuchain/releases), unduh sesuai dengan sistem operasi yang kamu gunakan.

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

Karena Nuchain ditulis menggunakan bahasa pemrograman Rust maka dipelukan beberapa komponen yang dibutuhkan berkaitan dengan Rust.

Gunakan rustup untuk memasang Rust:

```bash
# Install
curl https://sh.rustup.rs -sSf | sh
# Configure
source ~/.cargo/env
```

Nuchain menggunakan [WebAssembly](https://webassembly.org/) sebagai core on-chain runtime-nya dan sebagai VM untuk kontrak pintar (smart contract)-nya, sehingga diperlukan Wasm toolchain untuk Rust:

```bash
rustup install nightly-2020-10-05
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-05
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

Perintah berikut akan menjalankan Nuchain node dengan identitas node `unsiq-node01` dan jalan secara lokal.

```bash
nuchain --base-path=/var/nuchain --name=unsiq-node01
```

Untuk jalan dan terhubung dengan node-node lainnya di luar sana, maka perlu ditambahkan parameter `--bootnodes`:

```bash
nuchain --base-path=/var/nuchain --name=unsiq-node01 --bootnodes=/ip4/<OTHER-NODE-IP>/tcp/30333/p2p/<ID-NODE>
```

`<OTHER-NODE-IP>` adalah IP dari node lain yang ingin digunakan sebagai titik masuk awal.
`<ID-NODE>` adalah ID dari node yang akan dijadikan sebagai pintu awal masuknya node kamu ke dalam jaringan utama Nuchain (mainnet).

## Komunitas

Bergabunglah dengan komunitas untuk diskusi tentang Nuchain melalui beberapa kanal berikut:


Chat: [Element](https://app.element.io/#/room/!aYWUxhUvutqbMBQIsN:matrix.org)
Email: nusantarachain@gmail.com


## Contributions & Code of Conduct

Please follow the contributions guidelines as outlined in [`docs/CONTRIBUTING.adoc`](docs/CONTRIBUTING.adoc). In all communications and contributions, this project follows the [Contributor Covenant Code of Conduct](docs/CODE_OF_CONDUCT.md).

## Security

The security policy and procedures can be found in [`docs/SECURITY.md`](docs/SECURITY.md).

## License

Lisensi Nuchain mengikuti lisensi dari [Substrate](https://substrate.dev).

- Substrate Primitives (`sp-*`), Frame (`frame-*`) and the pallets (`pallets-*`), binaries (`/bin`) and all other utilities are licensed under [Apache 2.0](LICENSE-APACHE2).
- Substrate Client (`/client/*` / `sc-*`) is licensed under [GPL v3.0 with a classpath linking exception](LICENSE-GPL3).

The reason for the split-licensing is to ensure that for the vast majority of teams using Substrate to create feature-chains, then all changes can be made entirely in Apache2-licensed code, allowing teams full freedom over what and how they release and giving licensing clarity to commercial teams.

In the interests of the community, we require any deeper improvements made to Substrate's core logic (e.g. Substrate's internal consensus, crypto or database code) to be contributed back so everyone can benefit.
