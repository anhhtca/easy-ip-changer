# Design System — Easy IP Changer

## Mode & Visual Direction

- **Surface Mode:** Operate (guided 2-step desktop utility for non-technical Windows 10/11 Pro users).
- **Palette Strategy:** Committed Restraint — an authoritative **Agribank Bordeaux** top rail (`#AE1C3F`) anchors the 2-step progress sequence over a calm warm Alabaster canvas (`#F7F5F6`), crisp white work surfaces (`#FFFFFF`), Agribank Bordeaux primary actions (`#AE1C3F`), and Emerald recommendation/connected badges (`#0D7A5F` on `#E6F6F0`).

## Color Tokens (`ui/theme.slint`)

| Role | Token | Hex |
|---|---|---|
| Header background | `header-bg` | `#AE1C3F` |
| Header secondary text (rose-tinted) | `header-muted` | `#FAD8E0` |
| Header active step badge | `header-step-active` | `#FFFFFF` |
| Canvas ground | `bg-canvas` | `#F7F5F6` |
| Card surface | `bg-surface` | `#FFFFFF` |
| Subtle context bar / pill | `bg-subtle` | `#F2EEF0` |
| Read-only locked input fill | `bg-readonly` | `#FAF8F9` |
| Selected card / radio tint | `bg-selected` | `#FDF2F5` |
| Primary action | `accent` / `accent-hover` / `accent-pressed` | `#AE1C3F` / `#951735` / `#7A122B` |
| Primary text | `text-primary` | `#1A1618` |
| Secondary / muted text | `text-secondary` / `text-muted` | `#52494D` / `#73686D` |
| Success / Recommended (`Nên chọn`) | `success` / `success-bg` / `success-border` | `#0D7A5F` / `#E6F6F0` / `#A7DFCA` |
| Warning (Admin shield) | `warning` / `warning-bg` / `warning-border` | `#B45309` / `#FEF3C7` / `#FCD34D` |
| Error / Validation | `danger` / `danger-bg` / `danger-border` | `#B91C1C` / `#FEF2F2` / `#FCA5A5` |

## Typography

- **Family:** Self-hosted **Be Vietnam Pro** (`Regular 400`, `Medium 500`, `SemiBold 600`, `Bold 700`) embedded directly inside the binary (`ui/fonts/`) for crisp Vietnamese diacritics on every Windows 10/11 Pro machine.
- **Scale:** Window title `16px` Bold, Screen headings `18–20px` Bold, Section headings `14.5px` Bold, IP octet inputs `17px` SemiBold, Form inputs `14px` Medium, Labels `13px` SemiBold, Helper text & badges `11–12.5px`.

## Iconography

- **App Icon (`assets/app.ico`, `assets/app-icon.png`):** Custom Windows 11 Fluent-style rounded tile in Deep Burgundy (`#6B0F26`) to Agribank Bordeaux (`#AE1C3F`) gradient featuring bold white `"IP"` lettering above Agribank Gold (`#FCD34D`) and Emerald Mint (`#34D399`) bidirectional swap arrows. Multi-resolution `.ico` (`16`, `24`, `32`, `48`, `64`, `128` px 32-bit BGRA DIB + `256` px PNG) embedded into the Windows `.exe` via `winresource`, paired with `assets/app-icon.png` for the Slint `Window` icon.
- **Header Brand Mark (`ui/icons/agribank-logo.svg`):** Vector Agribank square emblem (`34×34px`) displayed on the Agribank Bordeaux (`#AE1C3F`) top header rail beside the application title.
- **UI Icons:** Authored monochrome SVG icon set (`ui/icons/`, `24×24` viewBox, uniform `1.8–2.2px` round stroke) tinted dynamically via Slint `colorize`: `ethernet`, `wifi`, `shield`, `edit`, `lock`, `check`, `chevron-down`, `refresh`, `alert`, `arrow-left`, `arrow-right`. Zero emoji or Unicode symbol substitutes.

## Layout & Components

- **Window dimensions:** `560 × 730 px` preferred (`520 × 650 px` minimum) so all 3 Step-2 sections (4-octet IP, locked Subnet/Gateway/DNS, and the `"Thay đổi ưu tiên Eth và Wifi"` checkbox) fit above the fold without scrolling, while expanding the accordion scrolls smoothly above an anchored `62px` footer.
- **No nested cards:** Flat white surfaces (`10px` radius, `1px` `#E6DFE2` border) separated by spacing or `1px` hairline dividers (`#F2EEF0`).
- **Motion:** Functional state transitions only (`140–180ms ease-out` on hover/focus/selection, smooth accordion height unfold and chevron rotation).
