# Kế hoạch: merge 12 commit upstream vào Brume

Dành cho người/agent thực thi. Viết để đọc một lần là làm được.

- Repo: **`Lynx-1ST/Brume`** — đã đổi tên từ `Lynx-1ST/Serein`, đã rời mạng lưới fork, độc lập.
- Upstream: `ViceVerse-cz/Serein`, nhánh `main`.
- Không còn workflow đồng bộ tự động. Merge **bằng tay**.

```powershell
git remote set-url origin https://github.com/Lynx-1ST/Brume.git
git remote add upstream https://github.com/ViceVerse-cz/Serein.git   # nếu chưa có
git fetch upstream
```

- Merge base: `058e3357`
- Upstream tip khi lập kế hoạch: `16b235fe`
- Upstream đổi **81 file**, fork đổi **263 file**, **42 file giao nhau**.

**Lấy cả 12 commit, không bỏ cái nào.** Lý do với commit `4faf0af0` nằm ở mục 2.2.

---

## 0. Việc đổi tên Brume đã xong — đọc trước khi merge

Toàn bộ việc đổi tên đã thực hiện, **kèm đường lui cho bản cũ ở mọi chỗ**:

| Nơi | Giá trị mới | Đường lui |
|---|---|---|
| `packaging/windows/installer.nsi` | `PRODUCT_ID "Brume"` | `LEGACY_PRODUCT_ID "Serein"` |
| `crates/platform/src/startup.rs` | `VALUE w!("Brume")` | `LEGACY_VALUE w!("Serein")` |
| `apps/desktop/src/updater.rs` | `brume-{tag}-…` | chấp nhận song song `serein-{tag}-…` |
| `apps/desktop/src/updater.rs` | URL `Lynx-1ST/Brume` | chấp nhận song song `Lynx-1ST/Serein` |
| `crates/local-store/src/lib.rs` | `brume-development` | di trú từ thư mục cũ |
| `.github/workflows/windows-release.yml` | publish tên `brume-*` | publish **kèm** `serein-*` |
| `.github/scripts/*.mjs` | `allowedRepositories` gồm cả hai tên | |

Đã kiểm trên bản phát hành thật `v0.1.0-nightly.20261124.88`: release mang **cả hai bộ tệp**
`brume-*` và `serein-*`, nên không bản cài nào bị kẹt.

**Hệ quả cho việc merge:** phía upstream vẫn viết "Serein" ở mọi chuỗi. Khi xung đột ở bất kỳ
chuỗi nào có tên sản phẩm, **luôn giữ "Brume"** của fork. Đừng để merge kéo tên cũ quay lại.

---

## 1. Danh sách 12 commit, theo thứ tự cũ → mới

| # | Commit | Nội dung | File | Đụng fork | Rủi ro |
|---|---|---|---|---|---|
| 1 | `471f37e6` | fix(downloads): thử lại khi kết nối tải bị reset (#1109) | 1 | 0 | — |
| 2 | `ebdbf4ac` | fix(attachments): gắn sao cho webp động như GIF (#1105) | 3 | 3 | **cao** |
| 3 | `e2523b17` | fix(video): nhận tệp WebM trên Linux (#1074) | 3 | 0 | — |
| 4 | `276fd9eb` | docs: ghi hạn mức 256 KiB cho ô kết quả pin (#1102) | 1 | 1 | thấp |
| 5 | `4faf0af0` | fix(voice): giữ định dạng đầu ra AirPods trên macOS (#1104) | 2 | 2 | **cao** |
| 6 | `ab3d9c27` | fix(packaging): bỏ sàn phiên bản theo máy build cho Arch (#1106) | 3 | 0 | — |
| 7 | `475315fc` | fix(composer): giữ điểm hoàn tác của bản nháp (#1108) | 6 | 2 | trung bình |
| 8 | `3f7da1d4` | feat(ui): thêm bạn vào nhóm DM có sẵn (#1100) | 27 | 22 | **cao** |
| 9 | `62b5ea14` | feat(timeline): hiện tag máy chủ cạnh tên người gửi (#1073) | 14 | 9 | **cao** |
| 10 | `d98f16fa` | feat(ui): bật/tắt spoiler cho tệp đã đính (#1096) | 25 | 21 | **cao** |
| 11 | `8b2c993b` | fix(extensions): đưa cuộc gọi đang hoạt động lên đầu danh sách | 2 | 2 | **rất cao** |
| 12 | `16b235fe` | perf(video): trình khung hình không qua một lượt UI (#1113) | 25 | 8 | **cao** |

---

## 2. Năm chỗ nguy hiểm, kèm lý do cụ thể

Đây là phần một agent bắt đầu từ đầu sẽ không thấy. Mỗi mục đều đã kiểm trong mã nguồn.

### 2.1 `8b2c993b` — rủi ro cao nhất, dù chỉ đụng 2 file

Nó sửa `apps/desktop/src/extension_app.rs` và `crates/ui/src/voice.rs`. Đó **đúng hai file**
chứa một chỗ fork cố ý khác upstream.

Trong `crates/ui/src/voice.rs`, hàm `resolve_member_with` của fork có thêm một bộ lọc:

```rust
let name = member
    .and_then(|m| m.nick.as_deref())
    .filter(|nick| !nick.is_empty())          // <- CHỈ fork có dòng này
    .or_else(|| user.map(|user| state.user_display_name(user)))
    .unwrap_or("Participant");
```

Upstream không có `.filter(...)`: biệt danh rỗng thì để trống tên. Fork quyết định biệt danh
rỗng phải rơi về tên hiển thị.

Test `extension_app::tests::voice_roster_snapshot_carries_names_flags_and_speaking` trong
`apps/desktop/src/extension_app.rs` (quanh dòng 1510) đã được fork sửa cho khớp. Upstream vẫn
giữ assert cũ.

**Bắt buộc:** giữ hành vi của fork và giữ assert của fork. Lấy nhầm bản upstream thì CI đỏ với
`assertion left == right failed: an empty nickname is left out`.

### 2.2 `4faf0af0` — GIỮ, dù tiêu đề nói macOS

Commit này gồm hai nửa khác hẳn nhau:

- **Nửa macOS**, trong `crates/platform/src/audio.rs`, là code chết trên Windows:
  `cfg!(target_os = "windows") || (cfg!(target_os = "macos") && …)` — vế đầu đã `true`.
- **Nửa còn lại, trong `crates/discord-voice/src/audio.rs`, không bị `cfg` bọc và chạm
  Windows**: nó đảo thứ tự, mở luồng mic **trước** luồng loa, với chú thích *"Starting a
  Bluetooth microphone can switch the paired speaker to its call format."* Trên Windows đó
  chính là cú chuyển A2DP sang HFP khiến tai nghe Bluetooth tụt xuống mono khi mở mic.

Lý do thứ hai để giữ: `discord-voice/src/audio.rs` đã mang toàn bộ phần viết lại stereo của
fork. Càng để nó lệch khỏi upstream thì mọi lần merge sau càng khó.

Khi giải xung đột ở file này, **không được đánh mất** những thứ sau của fork:

- `Playback::pull()` và `sample_at()` trả về **cặp** `(f32, f32)`; hai kênh dùng chung
  `phase`/`step`. Tách pha riêng là hỏng hình ảnh âm thanh.
- Trường `right: Option<Frame>` trong `Playback`.
- Chỗ ghi ra thiết bị tính `mono = (left + right) * 0.5` và nạp bản trộn đó cho bộ khử tiếng vọng.
- `device_level` và `device_level_db()` — mức tín hiệu **thô từ thiết bị**, dùng cho cảnh báo
  mic câm, khác với `preview_level` là mức sau khi lọc tạp âm.

### 2.3 `16b235fe` — đổi đường trình khung hình

Thêm phụ thuộc vào `crates/platform/Cargo.toml`, sửa `Cargo.lock`, `crates/ui/src/video.rs`,
`apps/desktop/src/watch.rs`, `apps/desktop/src/voice.rs`.

Fork có thay đổi riêng trong `apps/desktop/src/voice.rs`:
- `ui.voice_no_input_detected` lấy từ `live.audio.device_level_db()`, **không phải**
  `preview_level_db()`.
- `ui.voice_suppression_fell_back |= live.audio.suppression_fell_back();`

Giải xung đột theo kiểu **giữ cả hai phía**, không chọn một.

### 2.4 `62b5ea14` đụng `crates/local-store/src/lib.rs`

File này vừa đổi thư mục dữ liệu sang `brume-development` kèm mã di trú. Upstream sửa cùng file
cho tính năng tag máy chủ. Giữ cả hai: nhận phần tag máy chủ của upstream, giữ nguyên phần tên
thư mục và di trú của fork.

### 2.5 Mười khóa ngôn ngữ mới

Upstream thêm 10 khóa vào `en-US` (2.790 → 2.800):

```
attachments-mark-spoiler
attachments-remove-spoiler
attachments-spoiler-label
reading-chat-reading-settings-show-server-tags
reading-chat-reading-settings-show-server-tags-description
reading-chat-reading-settings-show-server-tags-hidden-everywhere
group-add-title
group-add-subtitle
group-add-full
group-add-confirm
```

Upstream có kèm bản dịch cho cả 13 ngôn ngữ, gồm `vi` và `ko`. Hai việc phải làm:

1. **CI bắt buộc `vi` và `ko` phủ 100% so với `en-US`.** Thiếu một khóa là đỏ.
2. Catalog `vi` của fork là bản **viết lại**, thuật ngữ đã rà theo ngữ cảnh Discord. Phải đọc
   lại 10 chuỗi `vi` upstream mang sang xem có khớp giọng văn của fork không, đừng nhận nguyên xi.

Kiểm bằng script:

```powershell
function K($p){ (Get-Content $p -Encoding utf8) | Where-Object { $_ -match '^[a-zA-Z][a-zA-Z0-9_-]*\s*=' } | ForEach-Object { ($_ -split '=')[0].Trim() } }
$en=K "crates\ui\locales\en-US\main.ftl"; $vi=K "crates\ui\locales\vi\main.ftl"; $ko=K "crates\ui\locales\ko\main.ftl"
"vi thieu: " + (@(Compare-Object $en $vi | Where-Object SideIndicator -eq '<=').Count)
"ko thieu: " + (@(Compare-Object $en $ko | Where-Object SideIndicator -eq '<=').Count)
```

---

## 3. Những thay đổi riêng của fork phải giữ nguyên

Khi gặp xung đột ở các file dưới đây, mặc định là **giữ cả hai phía**, không chọn một.

| File | Thứ fork thêm |
|---|---|
| `crates/ui/src/voice.rs` | lọc biệt danh rỗng (2.1); hai `Select::selected_text` dùng `owned_control_text` |
| `apps/desktop/src/extension_app.rs` | assert của fork trong test roster |
| `crates/discord-voice/src/audio.rs` | stereo `(f32, f32)`, `right`, `device_level` (2.2) |
| `crates/discord-voice/src/audio/echo.rs` | bộ dò RNNoise chỉ để lấy `speech_probability` |
| `crates/discord-voice/src/presentation.rs` | `PlaybackFrame.right` |
| `crates/discord-voice/src/stream_playout.rs` | `HEADROOM`, cắt tồn dư khi tick trễ thay vì xóa sạch |
| `crates/discord-voice/src/stream_audio_receive.rs` | giải mã stereo, tách kênh xen kẽ |
| `crates/ui/src/avatars.rs` | `paints_downscaled`, `ensure_texture`, `texture_budget` |
| `crates/model/src/cache_preferences.rs` | `texture_mib` và `TextureLimit` |
| `crates/ui/src/i18n/builtin.rs` | ánh xạ chuỗi tiếng Anh dựng sẵn sang khóa catalog |
| `crates/ui/src/design.rs` | `radio_row` dùng `owned_control_text` |
| `apps/desktop/src/voice.rs` | cảnh báo mic đo từ `device_level_db()` |
| `crates/local-store/src/lib.rs` | `brume-development` và mã di trú thư mục |
| `crates/platform/src/startup.rs` | `VALUE`/`LEGACY_VALUE` cho tự khởi động |
| `apps/desktop/src/updater.rs` | `asset_name()` và URL chấp nhận cả hai tên |
| `packaging/windows/installer.nsi` | `PRODUCT_ID`/`LEGACY_PRODUCT_ID` |
| `README.md`, `docs/windows-fork.md` | bản tiếng Việt, **không nhận bản upstream** |

Và quy tắc bao trùm: **mọi chuỗi có tên sản phẩm đều giữ "Brume"**.

---

## 4. Cách làm

Khuyến nghị: **một PR cho cả 12 commit**. Tách nhỏ không giúp gì vì các commit dựa lên nhau và
cùng đụng một nhóm file.

```powershell
git checkout main
git pull
git checkout -b chore/sync-upstream-20261124
git merge upstream/main
```

Giải xung đột theo mục 2 và 3. Sau đó:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo xtask check      # bien dich + clippy -D warnings + test toan workspace
```

Rồi mở PR vào `main`.

**Lưu ý về CI:** commit `d71fc8ef` đã **tắt CI tự chạy khi push lên `main`**. Kiểm lại
`.github/workflows/ci.yml` xem trigger còn gì; nhiều khả năng phải mở PR hoặc chạy tay trong
tab Actions thì CI mới chạy. Đừng đẩy thẳng lên `main` rồi tưởng CI sẽ kiểm giúp.

---

## 5. Những điều cần biết về môi trường

- Máy chủ dự án **không build được** (thiếu linker MSVC). Mọi kiểm chứng đều qua CI. Nếu máy
  người thực thi build được thì chạy `cargo xtask check` tại chỗ sẽ nhanh hơn nhiều.
- CI chạy `cargo clippy --workspace --all-targets --locked -- -D warnings`, tức biên dịch cả
  test với mọi cảnh báo coi là lỗi. Lỗi hay gặp: kiểu số chưa xác định khi gọi
  `is_multiple_of`, và `chunks_exact` với kích thước hằng (clippy đòi `as_chunks`).
- Test `recovery_probes_a_ready_socket_and_resumes_only_when_it_is_silent` trong
  `crates/discord-gateway` **chập chờn**: nó mở websocket TCP thật với đồng hồ thật và hỏng khi
  máy tải nặng, báo `ResetWithoutClosingHandshake`. Đỏ một lần thì chạy lại, đừng sửa test.
- CI **không** chạy bước publish release, nên lỗi trong khối ghi chú phát hành của
  `windows-release.yml` chỉ lộ ra lúc phát hành thật.
- Thay đổi trong `windows-release.yml` chỉ có hiệu lực từ bản phát hành **kế tiếp**, vì bước
  publish chạy định nghĩa workflow lấy từ nhánh mặc định tại thời điểm kích hoạt.

---

## 6. Một quyết định còn bỏ ngỏ

`apps/desktop/src/extensions.rs` vẫn tải danh mục extension từ
`https://raw.githubusercontent.com/ViceVerse-cz/Serein/main/extensions/catalog.json`.

Repo đã tách và đã đổi tên nhưng vẫn ăn danh mục extension của upstream. Đây là dư âm fork có ý
nghĩa chức năng, không phải chuyện thương hiệu. Hai lựa chọn: trỏ về repo này (phải tự duy trì
danh mục) hoặc giữ nguyên (tiếp tục phụ thuộc). Chủ dự án chưa quyết. Trong mã đã có chú thích
đánh dấu chỗ này.
