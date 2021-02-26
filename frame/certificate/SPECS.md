# NRC-2: Manajemen Sertifikat

| **Author**   | Robin Syihab |
| ---:         |     :---      |
| **Status**   | Membutuhkan Komentar              |
| **Category** | Sertifikat                        |
| **Created**  | 2021-02-26                        |

## Daftar Isi

* [Abstraksi](#Abstraksi)
* [Motivasi](#Motivasi)
* [Spesifikasi](#Spesifikasi)
  * [Penerbitan](#Penerbitan)
  * [Pencabutan](#Pencabutan)
* [Implementasi](#Implementasi)
  * [Configuration](#Configuration)
  * [Types](#Types)
  * [Object](#Object)
  * [Event](#Event)
  * [Error](#Error)
  * [Storage](#Storage)
  * [Methods](#Methods)


## Abstraksi

Standar ini dibuat untuk mengimplementasikan mekanisme pembuatan, penerbitan, dan pencabutan sertifikat pada jaringan blockchain Nuchain.

## Motivasi

* Implementasi ini memungkinkan organisasi/instansi memberikan jaminan legitimasi dengan dasar bukti kopi atau non-kopi sertifikat secara digital dan ditandatangani secara digital yang diterbitkan di jaringan blockchain Nuchain.
* Penerbitan kopi atau non-kopi sertifikat secara digital pada jaringan blockchain membuat sertifikat tersebut abadi dan bisa dibuktikan eksistensinya dan keabsahannya oleh siapapun, kapanpun dan dimanapun.

## Spesifikasi

## Penerbitan

1. Sertifikat hanya bisa diterbitkan oleh organisasi yang terdaftar di Nuchain. Tentang organisasi bisa mereferensi pada [NRC-1](Organisasi).
2. Penerima sertifikat adalah akun pada Nuchain yang direpresentasikan dengan AccountId (Nuchain Address).
3. Sertifikat sifatnya unik per organisasi per jenis sertifikat per akun. Artinya satu penerima hanya bisa menerima satu jenis sertifikat yang sama oleh organisasi yang sama.

## Pencabutan

1. Sertifikat bisa dicabut oleh penerbit sertifikat dari penerima yang pernah menerimanyaa.

## Sifat Sertifikat

1. Sertifikat bisa memiliki batas waktu atau bisa juga selamanya tergantung kebijakan penerbit.
2. Sertifikat yang memiliki batas waktu dan telah expired lebih dari 3 bulan akan dihapus dari jaringan.
3. Sertifikat bisa dimusnahkan oleh penerima namun tidak dihapus, hanya ditandai sebagai telah dimusnahkan oleh penerima.
4. Sertifikat bisa memiliki lampiran data, sebagai contoh berupa citra/foto sertifikat offline, lampiran ini hanya berupa link/hash ke IPFS di mana data tersebut berada.

## Implementasi

### Configuration

* `ForceOrigin` - Operasi yang hanya boleh dilakukan oleh super user (sudo) atau secara konsensus.

### Types

* `OrgId` tipe ID untuk organisasi menggunakan unsigned 32 bit integer.
* `CertId` tipe ID untuk sertifikat menggunakan unsigned 64 bit integer.

### Object

Terdapat 3 jenis object dalam database:

* `OrgDetail` - berisi informasi detail organisasi: 
    * `name` - nama dari organisasi bertipe bytes (`Vec<u8>`).
    * `admin` - ID akun dari administrator berupa Nuchain Address.
    * `is_suspended` - penanda apakah organisasi dinonaktifkan atau tidak, berjenis boolean (`bool`).
* `CertDetail` - berisi informasi detail sertifikat:
    * `name` - nama dari sertifikat bertipe bytes (`Vec<u8>`).
    * `org_id` - ID dari organisasi yang menerbitkan bertipe `OrgId`.
* `OwnedCert` - object detail sertifikat yang dimiliki seseorang berisi informasi:
    * `owner` - ID pemilik (yang menerima) sertifikat, berupa `AccountId`.
    * `cert_id` - ID referensi dari sertifikat.
    * `date` - Waktu kapan sertifikat diterima.
    * `issued_by` - ID dari organisasi penerbit.
    * `signed_by` - Tanda tangan digital pejabat yang menerbitkan.
    * `notes` - Catatan yang bisa diisi oleh penerbit.
    * `attachment` - Data lampiran, bisa berupa link ke hash [IPFS](https://ipfs.io/) yang merujuk ke gambar/foto sertifikat offline apabila ada, atau bisa berupa data yang terelasi lainnya.

### Event

Berikut jenis-jenis event yang mungkin muncul selama operasi:

* `OrgAdded` - Ketika suatu organisasi baru diciptakan di dalam jaringan. Event ini berisi informasi ID organisai dan ID pemilik atau admin-nya.
* `CertAdded` - Ketika ada sertifikat baru dibuat di dalam jaringan. Event ini berisi informasi ID sertifikat dan ID organisasi.
* `CertIssued` - Ketika ada sertifikat baru diterbitkan dan diberikan kepada seseorang. Event ini berisi informasi ID sertifikat dan ID penerima sertifikatnya.

### Error

Berikut jenis-jenis error yang mungkin muncul selama operasi:

* `AlreadyExists` - ketika organisasi atau sertifikat sudah ada.
* `TooLong` - ketika nama organisasi terlalu panjang.
* `TooShort` - ketika nama organisasi terlalu pendek.

### Storage

Ada 4 object storage yang digunakan:

* `Organizations` dengan jenis `Map` digunakan sebagai registri penyimpanan informasi organisasi.
* `Certiicates` dengan jenis `Map` digunakan sebagai registri penyimpanan informasi sertifikat.
* `IssuedCertificates` dengan jenis `DoubleMap` digunakan sebagai registri penyimpanan informasi pemilik sertifikat.
* `OrgIdIndex` dengan jenis `Value` digunakan sebagai ID generator organisasi yang bersifat incremental.
* `CertIdIndex` dengan jenis `Value` digunakan sebagai ID generator sertifikat yang bersifat incremental.

### Methods

* `add_org` metode untuk menambahkan organisasi baru.
* `add_cert` metode untuk membuat sertifikat baru.
* `issue_cert` metode untuk menerbitkan sertifikat untuk seseorang.
* `revoke` metode untuk mencabut sertifikat yang telah diterbitkan untuk seseorang.
* `destroy` metode untuk memusnahkan sertifikat yang telah diterima oleh seseorang.
