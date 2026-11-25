# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack

Pure Rust + Slint (`backend-winit`, `renderer-software`) targeting Windows 10/11 Pro as a single standalone zero-dependency `.exe` (with `+crt-static` MSVC linkage), plus native macOS preview mode with mock network adapters.

## Users

Non-technical end users on Windows 10/11 Pro (office/corporate staff) who occasionally need to assign a static IPv4 address, default gateway, corporate DNS servers, and Ethernet vs Wi-Fi priority without touching Windows Control Panel or `ncpa.cpl`.

## Product Purpose

Let an inexperienced Windows user reliably configure a static IPv4 address (and optionally adjust Wi-Fi vs Ethernet priority) in under 30 seconds across two clear Vietnamese-language steps, while protecting them from misconfiguring Subnet Mask, Gateway, DNS, virtual adapters, or inverted technical metric numbers.

## Positioning

A zero-install, single-file Vietnamese desktop utility that replaces multi-level Windows network dialogs with a guided 2-step flow: automatic Admin check, physical-only Ethernet (including USB Ethernet) & Wi-Fi discovery with Ethernet recommended by default, smart 4-octet IP input that auto-syncs Gateway, locked-by-default advanced fields with on-demand Edit buttons, and plain-language Wi-Fi/Ethernet priority selection that hides raw metric numbers completely.

## Operating Context

- Launched directly by double-clicking a standalone `.exe` on Windows 10/11 Pro workstations and laptops (including USB-C / USB Ethernet docks).
- Embeds `requireAdministrator` in the Windows manifest (with an automatic UAC relaunch fallback at startup) so Windows prompts UAC immediately on launch and opens directly into Step 1 without an intermediate reopen screen.
- Must run offline with zero external runtime dependencies (no .NET, no WebView2, no VC++ Redistributable, no GPU driver dependency).

## Capabilities and Constraints

- 100% Vietnamese UI copy with crisp diacritics (embedded Be Vietnam Pro font).
- Strict physical NIC filtering: show physical Ethernet (including USB Ethernet) and Wi-Fi only; hide virtual/VPN/Hyper-V/VMware/Bluetooth/Wi-Fi Direct adapters.
- Static IP only (no DHCP toggle/button on Screen 2).
- Default locked values: Subnet `255.255.255.0`, Gateway `<ip1>.<ip2>.<ip3>.1`, DNS `10.0.58.11` & `10.0.29.5`, each individually unlockable via a small Edit button.
- Priority accordion under checkbox `"Thay đổi ưu tiên Eth và Wifi"` displays everyday Vietnamese choices only (`Ưu tiên Wi-Fi` default, `Ưu tiên Ethernet`, `Mặc định Windows`) and maps internally to interface metrics (`10`/`50` or `automatic`).

## Product Principles

1. **Zero jargon on the primary path** — Hide technical concepts like interface metric numbers and virtual adapter clutter; surface plain Vietnamese intent.
2. **Safe by default, editable when needed** — Pre-fill and lock Subnet, Gateway, and DNS so casual users cannot accidentally break them, while keeping one-click unlock right beside each group.
3. **Calm, obvious two-step progress** — Always orient the user with clear step status, strong primary action hierarchy, and actionable Vietnamese feedback.
4. **Truly standalone reliability** — Single static binary, software rendering, zero external DLLs or runtimes, zero flashing console windows.

## Accessibility & Inclusion

- High-contrast light surface palette (WCAG AA+), Per-Monitor V2 DPI awareness on Windows 10/11 scaling (100%–200%), full keyboard navigation across 4-octet IP segments (auto-advance, Backspace back-step, clipboard paste).
