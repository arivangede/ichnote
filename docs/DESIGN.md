# Ichnote — Dokumen Desain & Panduan Build

> **Status:** Draft v0.2 · **Nama:** Ichnote — *leave a trace.*
> Aplikasi pencatatan ringan ala Notion, local-first, open-source, dibangun dengan Rust + Tauri, dengan jendela cepat *always-on-top* dan sinkronisasi lewat folder lokal.

---

## 1. Ringkasan

Ichnote adalah aplikasi catatan desktop yang:

- **Local-first** — semua data tersimpan di perangkat (SQLite), aplikasi bekerja penuh tanpa internet.
- **Ringan** — Tauri (WebView bawaan OS), bukan Electron.
- **Punya jendela cepat** — jendela kecil *always-on-top* yang dipanggil dengan pintasan keyboard global, untuk melihat catatan yang di-pin sekaligus menulis catatan cepat.
- **Sinkron tanpa server** — sinkronisasi lewat folder yang sudah dimiliki pengguna (iCloud Drive, Dropbox, Google Drive for desktop, OneDrive, Syncthing). Biaya operasional pengembang: nol.
- **Kolaboratif secara aman** — isi catatan dan metadata memakai CRDT (Yjs/yrs), sehingga edit offline di banyak perangkat atau oleh banyak anggota tim tergabung otomatis tanpa konflik.
- **Open-source**, dengan ruang untuk layanan sinkronisasi berbayar di masa depan (model *open-core*).

### 1.1 Tujuan v1

1. Editor catatan berbasis blok (heading, paragraf, checklist, list, kode, kutipan).
2. Daftar catatan, folder, pin, Inbox, pencarian teks penuh.
3. Jendela cepat + pintasan global (tampilan pin + input catatan cepat).
4. Sinkronisasi folder antar perangkat milik sendiri dan antar anggota tim yang berbagi folder.
5. Target platform pertama: **macOS**. Kode tetap lintas platform agar versi Windows mudah menyusul.

### 1.2 Bukan tujuan v1

- Kolaborasi real-time (kursor orang lain, kehadiran online).
- Server / akun / login.
- Enkripsi file sinkron (dirancang agar bisa ditambahkan di v2 tanpa migrasi).
- Widget WidgetKit macOS, aplikasi mobile.
- Database/tabel ala Notion (property, view). Bisa dipertimbangkan setelah v1.

---

## 2. Prinsip Desain

1. **Data pengguna milik pengguna.** Format terbuka dan terdokumentasi; pengguna selalu bisa mengekspor dan berpindah.
2. **Siap sinkron sejak hari pertama.** UUIDv7 untuk semua ID, soft delete, semua perubahan lewat CRDT.
3. **Setiap perangkat hanya menulis ke foldernya sendiri.** Tidak pernah ada dua perangkat yang menulis file yang sama → tidak ada "conflicted copy".
4. **Transport sinkronisasi dapat diganti.** Folder hari ini; LAN, Google Drive API, PocketBase, atau layanan berbayar nanti, lewat satu trait `SyncTransport`.
5. **Kode inti tidak bergantung pada UI.** Logika data dan sinkronisasi ada di crate Rust terpisah yang bisa diuji tanpa Tauri.

---

## 3. Tech Stack

| Lapisan | Pilihan | Catatan |
|---|---|---|
| Shell aplikasi | **Tauri v2** | Lintas platform, WebView bawaan OS |
| Frontend | **React + TypeScript + Vite** | |
| Editor blok | **BlockNote** (berbasis TipTap/ProseMirror) | Dukungan Yjs bawaan |
| CRDT (frontend) | **Yjs** | |
| CRDT (backend) | **yrs** | Port Rust dari Yjs, format biner kompatibel |
| Database lokal | **SQLite** via `rusqlite` (feature `bundled`) | + FTS5 untuk pencarian |
| Migrasi DB | `rusqlite_migration` | |
| ID | `uuid` dengan feature `v7` | Terurut waktu |
| Pantau folder | `notify` (+ `notify-debouncer-full`) | |
| Checksum | `blake3` | |
| Async | `tokio` | |
| Serialisasi | `serde`, `serde_json` | |
| Error | `thiserror` (library), `anyhow` (aplikasi) | |
| Log | `tracing`, `tracing-subscriber` | |
| Pintasan global | `tauri-plugin-global-shortcut` | |
| Dialog pilih folder | `tauri-plugin-dialog` | |
| Satu instans | `tauri-plugin-single-instance` | |
| Auto-update (fase rilis) | `tauri-plugin-updater` | |
| **Nanti (v2+)** | `chacha20poly1305`, `argon2`, `ed25519-dalek`, `mdns-sd`, `keyring` | Enkripsi, tanda tangan, LAN, kredensial |

> Gunakan versi stabil terbaru setiap crate/paket saat memulai proyek, lalu kunci lewat `Cargo.lock` dan lockfile frontend.

---

## 4. Arsitektur

```
┌─────────────────────────── Ichnote.app ───────────────────────────┐
│                                                                    │
│  ┌─────────────────┐   ┌──────────────────┐                        │
│  │ Jendela utama   │   │ Jendela cepat    │   (WebView, React)     │
│  │ editor + daftar │   │ pin + quick note │                        │
│  └────────┬────────┘   └────────┬─────────┘                        │
│           │  Tauri commands / events (update Yjs biner)             │
│  ┌────────▼─────────────────────▼───────────────────────────────┐  │
│  │ src-tauri (lapisan aplikasi: commands, window, shortcut, tray)│  │
│  └────────┬───────────────────────────────┬─────────────────────┘  │
│  ┌────────▼────────┐             ┌────────▼────────┐               │
│  │ ichnote-core    │◀───────────▶│ ichnote-sync    │               │
│  │ model, SQLite,  │             │ SyncEngine,     │               │
│  │ dokumen yrs     │             │ FolderTransport │               │
│  └────────┬────────┘             └────────┬────────┘               │
└───────────┼───────────────────────────────┼────────────────────────┘
            ▼                               ▼
   ~/Library/Application           <Folder sinkron pilihan pengguna>
   Support/Ichnote/ichnote.db      (iCloud/Dropbox/GDrive/Syncthing)
```

### 4.1 Struktur repositori

```
ichnote/
├── Cargo.toml                 # Cargo workspace
├── crates/
│   ├── ichnote-core/          # model data, SQLite, migrasi, dokumen yrs
│   │   └── src/
│   │       ├── db.rs
│   │       ├── migrations/
│   │       ├── docs.rs        # load/apply/encode dokumen CRDT
│   │       ├── meta.rs        # dokumen meta workspace
│   │       ├── index.rs       # notes_index + FTS dari dokumen meta
│   │       └── ids.rs
│   └── ichnote-sync/          # mesin sinkronisasi
│       └── src/
│           ├── engine.rs      # SyncEngine: outbox → transport → apply
│           ├── transport.rs   # trait SyncTransport
│           ├── folder/        # FolderTransport
│           │   ├── layout.rs  # path & penamaan file
│           │   ├── format.rs  # encode/decode file biner
│           │   ├── writer.rs
│           │   ├── reader.rs
│           │   └── compact.rs
│           └── status.rs
├── apps/
│   └── desktop/
│       ├── src-tauri/         # Rust: commands, window, shortcut, tray
│       └── src/               # React: main window + quick window
│           ├── main/
│           ├── quick/
│           └── lib/ipc.ts
├── docs/
│   ├── sync-format.md         # spesifikasi format file (publik)
│   └── adr/                   # Architecture Decision Records
├── LICENSE
└── CONTRIBUTING.md
```

---

## 5. Model Data

### 5.1 Dua jenis dokumen CRDT

| Dokumen | Jumlah | Isi | Pemilik di runtime |
|---|---|---|---|
| **Meta workspace** | 1 per workspace | daftar catatan, judul, folder, pin, Inbox, tombstone hapus | Rust (`ichnote-core`) |
| **Isi catatan** | 1 per catatan | blok editor (struktur BlockNote) | Frontend (Yjs) dicerminkan ke Rust (yrs) |

Struktur dokumen meta (Yjs):

```
Y.Doc (id = workspace_id)
├── Y.Map "workspace" → { name }
├── Y.Map "notes"     → note_id → Y.Map {
│                          title, folder_id, pinned, in_inbox,
│                          created_at, updated_at, deleted_at }
└── Y.Map "folders"   → folder_id → Y.Map { name, parent_id, deleted_at }
```

Menghapus catatan = mengisi `deleted_at` (tombstone), bukan menghapus entri. Ini memastikan penghapusan tersebar ke semua perangkat.

### 5.2 Skema SQLite

Lokasi: direktori data aplikasi per OS (`~/Library/Application Support/Ichnote/` di macOS, `%APPDATA%\Ichnote\` di Windows).

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- Pengaturan & identitas perangkat (key 'device_id' dibuat saat pertama dijalankan)
CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE workspaces (
  id          TEXT PRIMARY KEY,          -- UUIDv7
  name        TEXT NOT NULL,
  sync_path   TEXT,                      -- NULL = tidak disinkron
  created_at  INTEGER NOT NULL,          -- unix ms
  updated_at  INTEGER NOT NULL
);

-- Semua dokumen CRDT (meta & isi catatan)
CREATE TABLE docs (
  id            TEXT PRIMARY KEY,        -- untuk meta: = workspace_id
  workspace_id  TEXT NOT NULL REFERENCES workspaces(id),
  kind          INTEGER NOT NULL,        -- 0 = meta, 1 = note
  state         BLOB NOT NULL,           -- state yrs ter-encode (v1)
  updated_at    INTEGER NOT NULL
);

-- Indeks turunan dari dokumen meta. Bisa dibangun ulang kapan saja.
CREATE TABLE notes_index (
  id            TEXT PRIMARY KEY,
  workspace_id  TEXT NOT NULL,
  title         TEXT NOT NULL DEFAULT '',
  folder_id     TEXT,
  pinned        INTEGER NOT NULL DEFAULT 0,
  in_inbox      INTEGER NOT NULL DEFAULT 0,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  deleted_at    INTEGER
);
CREATE INDEX idx_notes_ws      ON notes_index(workspace_id, deleted_at, updated_at DESC);
CREATE INDEX idx_notes_pinned  ON notes_index(workspace_id, pinned) WHERE deleted_at IS NULL;

-- Pencarian teks penuh (diisi dari teks polos isi catatan)
CREATE VIRTUAL TABLE notes_fts USING fts5(
  note_id UNINDEXED, title, body, tokenize = 'unicode61 remove_diacritics 2'
);

-- Outbox: update lokal yang belum ditulis ke transport (lokal saja, tidak disinkron)
CREATE TABLE outbox (
  seq           INTEGER PRIMARY KEY AUTOINCREMENT,
  workspace_id  TEXT NOT NULL,
  doc_id        TEXT NOT NULL,
  doc_kind      INTEGER NOT NULL,
  payload       BLOB NOT NULL,           -- update yrs v1
  created_at    INTEGER NOT NULL
);

-- Nomor urut file berikutnya yang akan ditulis perangkat ini per workspace
CREATE TABLE sync_out_state (
  workspace_id  TEXT PRIMARY KEY,
  next_seq      INTEGER NOT NULL DEFAULT 1,
  last_compact  INTEGER
);

-- Sampai mana perangkat ini sudah membaca file dari perangkat lain
CREATE TABLE sync_cursor (
  workspace_id    TEXT NOT NULL,
  source_device   TEXT NOT NULL,
  last_seq        INTEGER NOT NULL,
  PRIMARY KEY (workspace_id, source_device)
);

-- Perangkat yang dikenal di workspace (dari device.json)
CREATE TABLE devices (
  workspace_id  TEXT NOT NULL,
  device_id     TEXT NOT NULL,
  name          TEXT NOT NULL,
  platform      TEXT NOT NULL,
  last_seen_at  INTEGER,
  PRIMARY KEY (workspace_id, device_id)
);
```

> **Optimasi nanti:** menyimpan `state` utuh setiap perubahan bisa mahal untuk dokumen besar. Alternatifnya, simpan log update per dokumen dan gabungkan berkala. Untuk v1, simpan state utuh dengan debounce (mis. 1–2 detik) sudah cukup.

### 5.3 Alur data editor

1. Saat catatan dibuka, frontend memanggil `load_doc(doc_id)` → menerima state biner → `Y.applyUpdate(ydoc, state)` → BlockNote terikat ke `ydoc`.
2. Setiap perubahan lokal (`ydoc.on('update')`, origin bukan remote) dikirim ke Rust lewat `apply_local_update(doc_id, update)`.
3. Rust menerapkan update ke dokumen yrs, menyimpan state, memperbarui FTS, dan memasukkan update ke `outbox`.
4. Update dari perangkat lain diterapkan Rust lalu dikirim ke frontend lewat event `doc-remote-update`; frontend menerapkannya dengan origin `"remote"` supaya tidak dikirim balik.

> **Tip performa IPC:** jangan kirim `Vec<u8>` sebagai array JSON. Gunakan dukungan payload biner mentah di Tauri v2 (`tauri::ipc::Request`/`Response`) atau base64 sebagai jalan pintas awal.

> **Catatan cepat** dibuat di frontend jendela cepat (yang juga punya Yjs + skema BlockNote), bukan di Rust. Membangun struktur Yjs yang kompatibel dengan BlockNote dari Rust rawan salah format.

---

## 6. Jendela Cepat (Quick Window)

### 6.1 Perilaku

- Dipanggil/disembunyikan dengan pintasan global. Default: `Option+Shift+N` (macOS) / `Alt+Shift+N` (Windows). Bisa diubah di pengaturan.
- Selalu di atas jendela lain, tanpa title bar, tidak muncul di Dock/taskbar.
- **Bagian atas:** tampilan yang dipilih pengguna — catatan yang di-pin (satu catatan atau daftar pin), termasuk checklist yang bisa dicentang langsung.
- **Bagian bawah:** kolom input catatan cepat. `Enter` menyimpan ke **Inbox** (`in_inbox = 1`), `Shift+Enter` baris baru.
- `Esc` menyembunyikan jendela. Opsi pengaturan: sembunyikan saat kehilangan fokus.
- Posisi dan ukuran terakhir diingat.
- Ikon tray/menu bar: buka jendela utama, tampilkan jendela cepat, keluar.

### 6.2 Konfigurasi Tauri (`tauri.conf.json`, potongan)

```json
{
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      { "label": "main", "url": "/", "width": 1100, "height": 750 },
      {
        "label": "quick",
        "url": "/quick",
        "width": 420,
        "height": 480,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "visibleOnAllWorkspaces": true,
        "resizable": true,
        "center": true
      }
    ]
  }
}
```

> `macOSPrivateApi` diperlukan untuk jendela transparan di macOS, tetapi tidak diizinkan di Mac App Store. Untuk v1 distribusi langsung (DMG ber-notarisasi) sehingga aman dipakai.

### 6.3 Pintasan global (Rust)

```rust
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn toggle_quick(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("quick") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _sc, ev| {
                    if ev.state() == ShortcutState::Pressed {
                        toggle_quick(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let sc = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyN);
            app.global_shortcut().register(sc)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("gagal menjalankan Ichnote");
}
```

> Jika ingin perilaku persis seperti Spotlight di atas aplikasi fullscreen macOS, pertimbangkan plugin komunitas yang mengubah jendela menjadi `NSPanel`. Evaluasi setelah v1 berjalan.

---

## 7. Sinkronisasi Folder

### 7.1 Konsep

- Pengguna memilih folder sinkron per workspace (mis. di dalam Dropbox atau Google Drive).
- **Setiap perangkat hanya menulis ke `devices/<device_id>/` miliknya.** Perangkat lain hanya membaca.
- Isi file = kumpulan update CRDT. Karena update CRDT idempoten dan komutatif, urutan kedatangan dan duplikasi tidak masalah.
- Berbagi dengan tim = membagikan folder lewat layanan penyimpanan (mis. share folder Google Drive dengan akses Editor). Akses Viewer otomatis menjadi read-only karena anggota tidak bisa membuat folder perangkatnya.

### 7.2 Tata letak folder

```
<FolderSinkron>/Ichnote/
└── <workspace_id>/
    ├── workspace.json                  # ditulis sekali oleh pembuat, tidak pernah diubah
    └── devices/
        ├── <device_id_A>/
        │   ├── device.json             # hanya ditulis oleh A
        │   ├── snap-000000000042.ichn
        │   ├── upd-000000000043.ichn
        │   └── upd-000000000044.ichn
        └── <device_id_B>/
            ├── device.json
            └── upd-000000000007.ichn
```

`workspace.json`:

```json
{
  "format": "ichnote-workspace",
  "format_version": 1,
  "workspace_id": "0192f3c4-...",
  "created_at": 1763450000000,
  "created_by_device": "0192f3c1-..."
}
```

Nama workspace **tidak** disimpan di sini (karena file ini tidak boleh diubah); nama ada di dokumen meta CRDT.

`device.json`:

```json
{ "device_id": "0192f3c1-...", "name": "MacBook Andi", "platform": "macos", "app_version": "0.1.0" }
```

### 7.3 Format file `.ichn` (versi 1)

Semua integer little-endian.

**Header (40 byte)**

| Offset | Ukuran | Field | Keterangan |
|---|---|---|---|
| 0 | 4 | `magic` | `b"ICHN"` |
| 4 | 2 | `version` | `1` |
| 6 | 2 | `flags` | bit0 = terenkripsi, bit1 = terkompresi. v1: `0` |
| 8 | 16 | `device_id` | UUID biner perangkat penulis |
| 24 | 8 | `seq` | nomor urut file (sama dengan nama file) |
| 32 | 4 | `record_count` | jumlah record |
| 36 | 1 | `kind` | 0 = update, 1 = snapshot |
| 37 | 3 | `reserved` | `0` |

**Record × `record_count`**

| Ukuran | Field | Keterangan |
|---|---|---|
| 16 | `doc_id` | UUID biner dokumen |
| 1 | `doc_kind` | 0 = meta, 1 = note |
| 4 | `payload_len` | panjang payload |
| `payload_len` | `payload` | update yrs (encoding v1) |

**Footer (32 byte):** `blake3(header ‖ records)`.

Aturan pembaca:
- Abaikan file yang namanya tidak cocok pola `^(upd|snap)-\d{12}\.ichn$` (termasuk `.tmp`, "conflicted copy", placeholder `.icloud`).
- Tolak (dan coba lagi nanti) file dengan magic salah, panjang tidak cocok, atau checksum salah — biasanya karena file belum selesai tersinkron.
- File dengan `version` lebih tinggi dari yang didukung: lewati dan tampilkan pesan "perbarui aplikasi".

Spesifikasi ini juga dipublikasikan di `docs/sync-format.md` agar pihak lain bisa membuat alat yang kompatibel.

### 7.4 Trait transport

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

pub struct UpdateRecord {
    pub doc_id: Uuid,
    pub doc_kind: DocKind,
    pub payload: Vec<u8>,
}

pub struct IncomingBatch {
    pub source_device: Uuid,
    pub seq: u64,
    pub records: Vec<UpdateRecord>,
}

#[async_trait]
pub trait SyncTransport: Send + Sync {
    /// Tulis satu batch update milik perangkat ini. Mengembalikan seq file.
    async fn push(&self, workspace: Uuid, records: Vec<UpdateRecord>) -> anyhow::Result<u64>;

    /// Ambil batch dari perangkat lain yang lebih baru dari cursor masing-masing.
    async fn pull(
        &self,
        workspace: Uuid,
        cursors: &HashMap<Uuid, u64>,
    ) -> anyhow::Result<Vec<IncomingBatch>>;

    /// Ringkas update lama perangkat ini menjadi snapshot.
    async fn compact(&self, workspace: Uuid, snapshot: Vec<UpdateRecord>) -> anyhow::Result<()>;
}
```

`FolderTransport` adalah implementasi pertama. Transport lain (LAN, Google Drive API, PocketBase, layanan berbayar) mengimplementasikan trait yang sama.

### 7.5 Alur tulis

1. Perubahan lokal masuk `outbox`.
2. **Debounce**: flush saat tidak ada perubahan selama 2 detik, atau paling lambat tiap 10 detik saat terus mengetik, atau saat catatan ditutup / aplikasi keluar.
3. Gabungkan update per dokumen (`yrs::merge_updates_v1`) supaya satu file tidak berisi ratusan potongan kecil.
4. Tulis ke `upd-<seq>.ichn.tmp` → `fsync` → rename ke `upd-<seq>.ichn` → `next_seq += 1` → hapus baris outbox terkait. Semua dalam urutan ini supaya crash di tengah tidak menghilangkan data.

### 7.6 Alur baca

1. Pemicu: event dari `notify` (debounce 1 detik) **dan** pemindaian berkala tiap 30 detik (event folder cloud tidak selalu andal), serta saat aplikasi dibuka / kembali dari sleep.
2. Untuk setiap `devices/<id>/` selain milik sendiri: daftar file dengan `seq > sync_cursor.last_seq`, urutkan naik.
3. Validasi file (7.3). File yang gagal validasi menghentikan pembacaan perangkat itu untuk siklus ini (jangan melompati seq).
4. Terapkan setiap record ke dokumen yrs terkait → simpan → perbarui `notes_index`/FTS → kirim event ke frontend.
5. Perbarui `sync_cursor` setelah seluruh file berhasil diterapkan (dalam satu transaksi SQLite).
6. Baca `device.json` baru untuk memperbarui tabel `devices`.

**Gap seq** (mis. cursor 30, file tersedia 42 ke atas) adalah normal setelah compaction perangkat lain: snapshot berisi semuanya. Pembaca cukup memproses file yang ada dengan seq lebih besar dari cursor.

### 7.7 Compaction

- Pemicu: perangkat punya lebih dari 200 file update sendiri, atau sudah lebih dari 24 jam sejak compaction terakhir dan ada update baru.
- Tulis `snap-<seq>.ichn` berisi **state lengkap** semua dokumen workspace sebagaimana diketahui perangkat ini (`encode_state_as_update_v1`), dengan `seq` lebih besar dari semua file sebelumnya.
- Setelah snapshot berhasil ditulis, hapus file `upd-*` dan `snap-*` **milik sendiri** yang seq-nya lebih kecil. Tidak pernah menyentuh folder perangkat lain.

### 7.8 Bergabung ke workspace / perangkat baru

1. Pengguna memilih "Gabung workspace" → pilih folder `Ichnote/<workspace_id>/`.
2. Aplikasi memvalidasi `workspace.json`, membuat baris `workspaces`, membuat `devices/<device_id>/device.json`.
3. Baca semua snapshot + update dari semua perangkat (cursor kosong) → bangun dokumen dan indeks lokal.

### 7.9 Kasus khusus yang harus ditangani

| Kasus | Penanganan |
|---|---|
| File online-only (iCloud *Optimize Storage*, GDrive mode *stream*, Dropbox/OneDrive online-only) | Deteksi file placeholder/tidak terbaca; tampilkan peringatan + instruksi "jadikan folder selalu tersedia offline" |
| Folder sinkron hilang / drive dilepas | Status sinkron "tidak tersedia"; aplikasi tetap jalan lokal, outbox menumpuk dan dikirim saat folder kembali |
| Dua instans aplikasi pada satu perangkat | `tauri-plugin-single-instance`: instans kedua hanya memfokuskan jendela instans pertama |
| Folder perangkat lain dihapus orang | Tidak merusak data lokal (data sudah tergabung); tampilkan perangkat sebagai "hilang" |
| Jam perangkat salah | Tidak berpengaruh pada penggabungan (CRDT tidak bergantung waktu); `updated_at` hanya untuk tampilan |
| Folder dibagikan di Google Drive "Shared with me" | Panduan onboarding: *Add shortcut to Drive* atau gunakan Shared Drive agar folder muncul di disk lokal |
| Sandbox Mac App Store (nanti) | Simpan *security-scoped bookmark* untuk folder sinkron |

### 7.10 Status sinkron di UI

Ikon kecil di sidebar dan jendela cepat: *Tersinkron*, *Menyinkron…*, *Menunggu folder*, *Ada masalah (klik untuk detail)*. Halaman pengaturan menampilkan daftar perangkat dan kapan terakhir terlihat.

---

## 8. API Tauri (Commands & Events)

**Commands**

| Command | Fungsi |
|---|---|
| `list_workspaces()` | Daftar workspace |
| `create_workspace(name)` | Membuat workspace + dokumen meta |
| `set_sync_folder(workspace_id, path)` | Mengaktifkan sinkronisasi folder |
| `join_workspace(path)` | Bergabung ke workspace dari folder |
| `list_notes(workspace_id, filter)` | filter: semua / folder / pinned / inbox / sampah |
| `create_note(workspace_id, folder_id?)` | Mengembalikan `note_id` |
| `load_doc(doc_id)` | State biner dokumen |
| `apply_local_update(doc_id, update)` | Menerapkan update dari editor |
| `update_note_meta(note_id, patch)` | Judul, pin, folder, inbox |
| `delete_note(note_id)` / `restore_note(note_id)` | Soft delete |
| `search(workspace_id, query)` | FTS5 |
| `sync_now(workspace_id)` | Paksa siklus sinkron |
| `sync_status(workspace_id)` | Status + daftar perangkat |
| `get_settings()` / `set_setting(key, value)` | Termasuk pintasan keyboard |

**Events (Rust → frontend)**

| Event | Payload |
|---|---|
| `doc-remote-update` | `{ doc_id, update }` |
| `notes-changed` | `{ workspace_id }` |
| `sync-status` | `{ workspace_id, state, detail? }` |

---

## 9. Roadmap

### Fase 0 — Fondasi
- [ ] Amankan nama: organisasi GitHub, domain, crates.io/npm
- [ ] Cargo workspace + aplikasi Tauri v2 + React/Vite
- [ ] CI (GitHub Actions): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, lint & typecheck frontend, build macOS
- [ ] Pilih lisensi, `CONTRIBUTING.md`, keputusan CLA (lihat bagian 12)

### Fase 1 — Aplikasi lokal
- [ ] Migrasi SQLite + `device_id`
- [ ] Dokumen meta & isi catatan (yrs ↔ Yjs)
- [ ] Editor BlockNote, daftar catatan, folder, pin, Inbox, sampah
- [ ] Pencarian FTS5
- [ ] Ekspor ke Markdown (per catatan & seluruh workspace)

**Selesai bila:** bisa dipakai sehari-hari di satu perangkat tanpa kehilangan data setelah restart/crash.

### Fase 2 — Jendela cepat
- [ ] Jendela `quick` + pintasan global yang bisa diubah
- [ ] Tampilan pin + checklist interaktif + input catatan cepat ke Inbox
- [ ] Ikon tray/menu bar, single-instance

**Selesai bila:** dari aplikasi apa pun, pintasan → ketik → Enter → catatan tersimpan dalam < 1 detik.

### Fase 3 — Sinkronisasi folder
- [ ] Format `.ichn` + parser + tes fuzz
- [ ] `FolderTransport`: tulis, baca, compaction
- [ ] SyncEngine: outbox, cursor, watcher + polling
- [ ] UI: pilih folder, gabung workspace, status sinkron, daftar perangkat
- [ ] Panduan pengguna berbagi folder (Google Drive, Dropbox, iCloud, Syncthing)

**Selesai bila:** dua perangkat + satu anggota tim mengedit catatan yang sama saat offline, lalu online, dan semua berakhir dengan isi identik.

### Fase 4 — Rilis macOS
- [ ] Code signing + notarisasi (Apple Developer Program)
- [ ] Auto-update (`tauri-plugin-updater`) via GitHub Releases
- [ ] Situs/README, dokumentasi format data

### Fase 5+ — Lanjutan
- [ ] Versi Windows (uji WebView2, pintasan, jalur folder)
- [ ] Enkripsi file sinkron (Argon2 + XChaCha20-Poly1305, flag bit0)
- [ ] Tanda tangan update per perangkat (Ed25519)
- [ ] Sinkronisasi LAN peer-to-peer (mDNS)
- [ ] Transport Google Drive API (`drive.file` + opsi kredensial sendiri)
- [ ] Server self-host (PocketBase) / layanan berbayar dengan kolaborasi real-time
- [ ] Widget macOS (WidgetKit, Swift) membaca data via App Group

---

## 10. Strategi Pengujian

- **Unit test** `ichnote-core`: migrasi, apply update, rebuild indeks dari dokumen meta.
- **Tes konvergensi** `ichnote-sync`: simulasikan 2–4 "perangkat" dengan database terpisah dan satu folder sementara (`tempfile`). Lakukan operasi acak (buat, edit, hapus, pindah folder) di setiap perangkat, jalankan sinkron dalam urutan acak, lalu pastikan semua state identik. Gunakan `proptest` untuk menghasilkan skenario.
- **Tes ketahanan**: file setengah tertulis, checksum salah, file `.tmp`, file asing, gap seq, folder hilang di tengah sinkron, crash di antara tulis dan rename.
- **Fuzz** parser `.ichn` dengan `cargo-fuzz`.
- **Uji manual matriks layanan**: iCloud Drive, Dropbox, Google Drive for desktop (mode stream & mirror), OneDrive, Syncthing — termasuk skenario berbagi folder antar akun.

---

## 11. Risiko & Mitigasi

| Risiko | Mitigasi |
|---|---|
| Struktur Yjs BlockNote berubah antar versi | Kunci versi BlockNote; uji migrasi sebelum upgrade; simpan versi skema editor di dokumen meta |
| Folder cloud lambat/tidak andal | CRDT + cursor membuat keterlambatan aman; UI status yang jujur |
| Jumlah file membengkak | Debounce + merge update + compaction |
| Anggota tim menghapus folder perangkat lain | Data lokal tetap utuh; v2: tanda tangan Ed25519 untuk deteksi |
| Kehilangan data karena bug sinkron | Backup otomatis harian database lokal (simpan 7 terakhir); ekspor Markdown |
| Kebijakan `macOSPrivateApi` | Distribusi di luar App Store untuk v1 |

---

## 12. Lisensi & Model Komersial

- **Open-core:** aplikasi, format data, dan sinkronisasi folder sepenuhnya open-source. Produk berbayar di masa depan berupa layanan terkelola (sinkron instan, kolaborasi real-time, manajemen tim, backup), bukan fitur aplikasi yang dikunci.
- **Pilihan lisensi:**
  - *MIT/Apache-2.0* — paling bebas; pihak lain boleh membuat layanan pesaing dari kode ini.
  - *AGPL-3.0* — pihak yang menjalankan versi modifikasi sebagai layanan wajib membuka kodenya.
  - Banyak proyek: klien MIT/Apache, server AGPL atau tertutup.
- **CLA/DCO:** jika ingin tetap bisa melisensikan ulang (mis. dual licensing), pasang CLA sejak kontribusi eksternal pertama.
- **Janji ke pengguna:** format data terbuka dan terdokumentasi; pengguna selalu bisa kembali ke sinkronisasi folder gratis.

---

## 13. Nama & Identitas

**Ichnote** (dibaca *"ik-nout"*), tagline: *leave a trace.*

- **ichno-** — dari bahasa Yunani *ichnos*, "jejak kaki" (akar kata *ichnology*, ilmu tentang jejak dan jejak fosil): catatan sebagai jejak pikiran yang tertinggal dan tersinkron di banyak perangkat.
- **Ich** — "saya" dalam bahasa Jerman: *Ich + note* = "catatan saya", selaras dengan prinsip bahwa data milik pengguna.
- **note** — catatan.

Penamaan teknis:

| Hal | Nilai |
|---|---|
| Crate | `ichnote-core`, `ichnote-sync` |
| Folder sinkron | `<FolderSinkron>/Ichnote/` |
| Ekstensi file sinkron | `.ichn` |
| Magic bytes | `ICHN` |
| Bundle identifier (contoh) | `app.ichnote.desktop` |

Sebelum rilis publik, amankan organisasi GitHub, nama crate/npm, dan domain (`ichnote.app` / `.dev` / `.com`), serta periksa PDKI dan WIPO Global Brand Database.
