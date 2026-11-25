# Design System — Easy IP Changer

## Mode & Visual Direction

- **Surface Mode:** Operate (guided 2-step desktop utility for non-technical Windows 10/11 Pro users).
- **Palette Strategy:** Committed Restraint — a deep authoritative Navy top rail (`#0F2744`) anchors the 2-step progress sequence over a calm Alabaster canvas (`#F4F6F9`), crisp white work surfaces (`#FFFFFF`), Cobalt primary actions (`#1565D8`), and Emerald recommendation/connected badges (`#0D7A5F` on `#E6F6F0`).

## Color Tokens (`ui/theme.slint`)

| Role | Token | Hex |
|---|---|---|
| Header background | `header-bg` | `#0F2744` |
| Header secondary text (hue-tinted) | `header-muted` | `#A9C5E8` |
| Header active step badge | `header-step-active` | `#60A5FA` |
| Canvas ground | `bg-canvas` | `#F4F6F9` |
| Card surface | `bg-surface` | `#FFFFFF` |
| Subtle context bar / pill | `bg-subtle` | `#EEF2F7` |
| Read-only locked input fill | `bg-readonly` | `#F7F9FC` |
| Selected card / radio tint | `bg-selected` | `#EFF6FF` |
| Primary action | `accent` / `accent-hover` / `accent-pressed` | `#1565D8` / `#0F53B8` / `#0B4194` |
| Primary text | `text-primary` | `#0F172A` |
| Secondary / muted text | `text-secondary` / `text-muted` | `#475569` / `#64748B` |
| Success / Recommended (`Nên chọn`) | `success` / `success-bg` / `success-border` | `#0D7A5F` / `#E6F6F0` / `#A7DFCA` |
| Warning (Admin shield) | `warning` / `warning-bg` / `warning-border` | `#B45309` / `#FEF3C7` / `#FCD34D` |
| Error / Validation | `danger` / `danger-bg` / `danger-border` | `#B91C1C` / `#FEF2F2` / `#FCA5A5` |

## Typography

- **Family:** Self-hosted **Be Vietnam Pro** (`Regular 400`, `Medium 500`, `SemiBold 600`, `Bold 700`) embedded directly inside the binary (`ui/fonts/`) for crisp Vietnamese diacritics on every Windows 10/11 Pro machine.
- **Scale:** Window title `16px` Bold, Screen headings `18–20px` Bold, Section headings `14.5px` Bold, IP octet inputs `17px` SemiBold, Form inputs `14px` Medium, Labels `13px` SemiBold, Helper text & badges `11–12.5px`.

## Iconography

- Authored monochrome SVG icon set (`ui/icons/`, `24×24` viewBox, uniform `1.8–2.2px` round stroke) tinted dynamically via Slint `colorize`: `ethernet`, `wifi`, `shield`, `edit`, `lock`, `check`, `chevron-down`, `refresh`, `alert`, `arrow-left`, `arrow-right`. Zero emoji or Unicode symbol substitutes.

## Layout & Components

- **Window dimensions:** `560 × 730 px` preferred (`520 × 650 px` minimum) so all 3 Step-2 sections (4-octet IP, locked Subnet/Gateway/DNS, and the `"Thay đổi ưu tiên Eth và Wifi"` checkbox) fit above the fold without scrolling, while expanding the accordion scrolls smoothly above an anchored `62px` footer.
- **No nested cards:** Flat white surfaces (`10px` radius, `1px` `#DCE3EC` border) separated by spacing or `1px` hairline dividers (`#EEF2F7`).
- **Motion:** Functional state transitions only (`140–180ms ease-out` on hover/focus/selection, smooth accordion height unfold and chevron rotation).
