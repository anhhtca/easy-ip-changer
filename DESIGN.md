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
- **Scale:** Window title `14.5px` Bold, Screen headings `16–16.5px` Bold, Section headings `13px` Bold, IP octet inputs `15px` SemiBold, Form inputs `13px` Medium, Labels `12px` SemiBold, Helper text & badges `10–11.5px`.

## Iconography

- **App Icon (`assets/app.ico`, `assets/app-icon.png`):** Custom Windows 11 Fluent-style rounded tile in Deep Burgundy (`#6B0F26`) to Agribank Bordeaux (`#AE1C3F`) gradient featuring bold white `"IP"` lettering above Agribank Gold (`#FCD34D`) and Emerald Mint (`#34D399`) bidirectional swap arrows. Multi-resolution `.ico` (`16`, `24`, `32`, `48`, `64`, `128` px 32-bit BGRA DIB + `256` px PNG) embedded into the Windows `.exe` via `winresource`, paired with `assets/app-icon.png` for the Slint `Window` icon.
- **Header Brand Mark (`ui/icons/agribank-logo.svg`):** Vector Agribank square emblem (`28×28px`) displayed on the Agribank Bordeaux (`#AE1C3F`) top header rail beside the application title.
- **UI Icons:** Authored monochrome SVG icon set (`ui/icons/`, `24×24` viewBox, uniform `1.8–2.2px` round stroke) tinted dynamically via Slint `colorize`: `ethernet`, `wifi`, `edit`, `lock`, `check`, `chevron-down`, `refresh`, `alert`, `arrow-left`, `arrow-right`. Zero emoji or Unicode symbol substitutes.

## Layout & Components

- **Window dimensions:** `500 × 592 px` preferred (`460 × 552 px` minimum) so all 3 Step-2 sections (4-octet IP, locked Subnet/Gateway/DNS, and the `"Thay đổi ưu tiên Eth và Wifi"` checkbox) fit above the fold on standard Windows laptops (including 1366×768 and 1080p@150% scaling) without scrolling, while expanding the accordion scrolls smoothly above an anchored `48px` action footer and a persistent `24px` WinForms-style status bar footer (`Theme.bg-subtle`, `1px` top border).
- **Compact spacing & control tokens (`ui/theme.slint`):** 4px-base scale (`space-2xs: 2px`, `space-xs: 4px`, `space-sm: 6px`, `space-md: 8px`, `space-lg: 12px`, `space-xl: 16px` outer gutters); desktop-grade control heights (`control-height: 30px`, `octet-height: 34px`, `btn-height: 32px`, `btn-compact-height: 28px`, `edit-btn-height: 22px`).
- **No nested cards:** Flat white surfaces (`8px` radius, `1px` `#E6DFE2` border) separated by `8px` spacing or `1px` hairline dividers (`#F2EEF0`).
- **Motion:** Functional state transitions only (`140–180ms ease-out` on hover/focus/selection, smooth accordion height unfold and chevron rotation).
