# Đổi nhận diện: Serein → Brume

Trạng thái: **chờ duyệt**. Ngày 23/11/2026.

## Vì sao

Repo vừa rời mạng lưới fork của `ViceVerse-cz/Serein` và trở thành dự án độc lập. Chủ dự án
muốn nó **thôi mang bóng dáng bản fork của người khác**. Đây là lý do nhận diện, không phải vì
sản phẩm đã khác đi — phần mềm vẫn là client Discord viết bằng Rust/egui, trọng tâm Windows x64
và tiếng Việt.

Điều đó định hình toàn bộ phạm vi bên dưới: đổi những gì **người ta nhìn thấy**, giữ những gì
chỉ máy đọc.

## Mục tiêu

- Một cái tên và một bộ nhận diện không gợi tới Serein.
- Repo tìm thấy được trên GitHub — chính là thứ đã đánh đổi khi rời mạng lưới fork.
- README viết lại quanh tên mới.
- Logo đọc được ở 16 px và ở chế độ một màu.

## Không làm

- Không đổi `serein.exe`, thư mục dữ liệu `serein-development`, biến `SEREIN_DATA_DIR`, hay
  giá trị registry khởi động tên `Serein`.
- Không đổi tên repo trên GitHub ở giai đoạn này.
- Không quét toàn bộ 410 file có chữ "serein" trong lần này.

## Điều sẽ coi là thất bại

- Một bản đã cài mất đường cập nhật, mất tự khởi động, hoặc không đọc lại được cache cũ.
- Mark mới nhìn không ra gì ở 16 px hoặc khi bị tô một màu.
- Mất thông báo bản quyền của tác giả gốc, hoặc ngụ ý dự án gốc chứng thực cho bản này.

## Ràng buộc

| Ràng buộc | Nguồn | Hệ quả |
|---|---|---|
| Mark phải đọc được khi chỉ còn một màu | `serein-mark.svg` là bóng đặc, tô màu lúc vẽ; bản tray chỉ dùng kênh alpha | Loại mọi ý tưởng phụ thuộc gradient |
| Giữ thông báo bản quyền | MIT/Apache của dự án gốc | `LICENSE-*` và phần ghi công giữ nguyên |
| Không dùng chữ "Discord" trong tên | Hướng dẫn nhãn hiệu của Discord | Cũng tránh đuôi `-cord` |
| Updater đọc release từ `Lynx-1ST/Serein` | `apps/desktop/src/updater.rs` | Đổi tên repo sẽ dựa vào chuyển hướng của GitHub — chưa làm lần này |

## Các hướng đã cân nhắc

**Tên.** Đã tra mức trùng tên trên GitHub vì khả năng tìm thấy là lý do rời mạng lưới fork.
Từ tiếng Việt ít dùng trống nhất (`Suong` 299 repo, `BanMai` 44). Chủ dự án chọn **tên tiếng Anh**.
Trong nhóm tiếng Anh, các từ dễ đọc đều đụng dự án lớn: Solace 4.240, Vesper 3.292,
Petrichor 742 với một dự án 1.782★, Gossamer 215 với một dự án Go 454★.

**Brume** — 369 repo, kết quả đầu 119★ và khác lĩnh vực — được chọn. Nghĩa là màn sương mỏng,
cùng họ nghĩa với *serein* (mưa bụi từ trời quang) mà không phải bản sao; cả hai đều gốc Pháp,
nên giữ được chất của tên cũ bằng một từ khác hẳn. Một âm tiết, năm chữ cái, người Việt đọc được.

Các phương án bị loại: `Glisk` (21 repo, trống nhất, nhưng khó đọc và trông như từ bịa),
`Smirr` (gần nghĩa *serein* nhất, nhưng `rr` trông như gõ nhầm).

**Logo.** Ba hướng đã dựng thật và xem ở 112/48/24/16 px kèm bản một màu. Chọn hướng **A**.
Hướng B (bong bóng chat hiện đại hoá) giữ được di sản nhưng cái đuôi vỡ ở 16 px. Hướng C
(chỉ còn dải sương, bỏ nền) hợp gu phẳng nhưng biến mất ở 16 px và lọt thỏm trên thanh tác vụ.

## Quyết định

- Tên: **Brume**. Viết `brume` ở chỗ cần không dấu, **Brume** ở chỗ hiển thị.
- Logo: bốn dải sương **khoét rỗng** khỏi khối bo tròn `rx=60`, độ dài lệch nhau để đọc ra
  chuyển động trôi. Phẳng, không viền, không bóng đổ. Vì hình là phần khoét đi nên bóng đặc
  vẫn giữ nguyên ý nghĩa.
- Giữ nguyên gradient `#8BBEFF → #506DDC → #2D247C` và màu nhấn `#1a72e8`, nên giao diện
  không phải chỉnh lại và không phải kiểm tra lại độ tương phản.

## Những chỗ tôi tự quyết (chủ dự án chưa duyệt riêng)

- **Giữ định danh nội bộ.** Lý do: mục tiêu là nhận diện, mà exe, thư mục dữ liệu và khóa
  registry gần như vô hình — đổi chúng là trả giá thật (mất cập nhật, mất tự khởi động, mất
  cache) để được thứ không ai nhìn thấy.
- **Chia hai giai đoạn.** Giai đoạn 1 là bề mặt nhìn thấy, dưới đây. Giai đoạn 2, nếu muốn,
  mới quét nốt các chuỗi nội bộ như `"Serein Custom"` hay `"Serein contributors"`.

## Giai đoạn 1 — việc cụ thể

| Hạng mục | Tệp |
|---|---|
| Logo nguồn | `assets/brand/` — thêm `brume.svg`, `brume-flat.svg`, `brume-mark.svg`, viết lại `README.md` của thư mục |
| Icon đóng gói | `packaging/windows/Serein.ico` → dựng lại từ mark mới |
| Tiêu đề cửa sổ, tên ứng dụng | `apps/desktop/src/main.rs` |
| Bộ cài | `packaging/windows/installer.nsi` — tên sản phẩm, mục Start Menu |
| Tài nguyên Windows | `packaging/windows/serein.rc`, `apps/desktop/build.rs` |
| Ghi chú phát hành | `.github/workflows/windows-release.yml` — tiêu đề release |
| Tài liệu | `README.md`, `docs/windows-fork.md` |
| Dư âm fork | `apps/desktop/src/extensions.rs:25` — catalog extension đang tải từ `ViceVerse-cz/Serein` |

Mục cuối không thuộc nhận diện nhưng là dư âm fork có ý nghĩa chức năng: bản này vẫn lấy danh
mục extension từ repo gốc. Cần quyết định riêng — trỏ về repo này, hay cố ý giữ nguyên.

## Rủi ro

- Icon `.ico` nhiều kích thước phải dựng lại; chưa dựng được ở máy này nên phải kiểm bằng mắt
  sau khi CI đóng gói.
- Đổi tiêu đề release sẽ làm các bản release cũ và mới không cùng dạng tên.
- Chưa build được ở máy này, nên mọi thay đổi chuỗi chỉ được CI kiểm ở mức biên dịch.
