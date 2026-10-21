<div align="center">

<img src="../assets/icon.jpeg" alt="QuadFurn CAD Logo" width="128" height="128" style="border-radius: 18px;" />

# QuadFurn CAD

**САПР за корпусна мебел от ПДЧ — 4-проекционен пространствен конструктор + гилотинно разкрояване на плочи**

[![Build & Release](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml/badge.svg)](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml)
[![Downloads](https://img.shields.io/github/downloads/rbm13s/QuadFurn/total?logo=github&color=blue)](https://github.com/rbm13s/QuadFurn/releases)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe_0.29-blue)](https://github.com/emilk/egui)
[![Graphics](https://img.shields.io/badge/Renderer-wgpu-green)](https://wgpu.rs/)
[![Languages](https://img.shields.io/badge/Languages-37_Locales-purple)](#8-теми-и-локализация-на-37-езика)
[![Platform](https://img.shields.io/badge/Platform-Windows_%7C_Linux_%7C_macOS_%7C_Web-0078D6)](#бърз-старт)
[![Web Demo](https://img.shields.io/badge/Онлайн_САПР-Стартиране_в_браузъра-success?logo=webassembly)](https://rbm13s.github.io/QuadFurn/app/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)

<br/>

👉 **[🚀 Стартиране на QuadFurn CAD директно в браузъра (WebAssembly)](https://rbm13s.github.io/QuadFurn/app/)** • **[🌐 Официален сайт](https://rbm13s.github.io/QuadFurn/bg.html)**

<br/>

**Languages / Языки / Мови / Sprachen:**<br/>
[English](../README.md) •
[Русский](README-ru.md) •
[Українська](README-uk.md) •
[Беларуская](README-be.md) •
[Қазақша](README-kk.md) •
[ქართული](README-ka.md) •
[Հայերեն](README-hy.md) •
[Azərbaycan](README-az.md) •
[Oʻzbekcha](README-uz.md) •
[Deutsch](README-de.md) •
[Français](README-fr.md) •
[Español](README-es.md) •
[Italiano](README-it.md) •
[Português](README-pt.md) •
[Polski](README-pl.md) •
[Čeština](README-cs.md) •
[Slovenčina](README-sk.md) •
[Slovenščina](README-sl.md) •
**Български** •
[Српски](README-sr.md) •
[Hrvatski](README-hr.md) •
[Nederlands](README-nl.md) •
[Dansk](README-da.md) •
[Norsk](README-no.md) •
[Svenska](README-sv.md) •
[Suomi](README-fi.md) •
[Eesti](README-et.md) •
[Latviešu](README-lv.md) •
[Lietuvių](README-lt.md) •
[Magyar](README-hu.md) •
[Română](README-ro.md) •
[Türkçe](README-tr.md) •
[Ελληνικά](README-el.md) •
[简体中文](README-zh-cn.md) •
[繁體中文](README-zh-tw.md) •
[日本語](README-ja.md) •
[한국어](README-ko.md)

</div>

---

## Съдържание

- [Въведение](#въведение)
- [Екранни снимки и визуален преглед](#екранни-снимки-и-визуален-преглед)
- [Основни възможности](#основни-възможности)
- [Функционална документация](#функционална-документация)
- [Управление и клавишни комбинации](#управление-и-клавишни-комбинации)
- [Лиценз](#лиценз)

---

## Въведение

**QuadFurn CAD** е високопроизводително междуплатформено CAD приложение, написано на Rust, налично както като бързо настолно приложение (Windows, Linux, macOS), така и като онлайн версия WebAssembly (WASM) директно в съвременни браузъри. Предназначено е за проектиране на корпусна мебел от ламинирано ПДЧ, МДФ, HDF и шперплат с автоматично генериране на спецификации и 2D оптимизация за гилотинно разкрояване.

---

## Екранни снимки и визуален преглед

### 1. 4-проекционен CAD конструктор (Тъмна тема)
Синхронни 2D проекции (**Отгоре XY**, **Отпред XZ**, **Отстрани YZ**) и хардуерен **3D изглед**:

![4-проекционен CAD](screenshots/bg/1designer.png)

### 2. Спецификация на детайлите
Таблица с размери след приспадане на канта:

![Спецификация](screenshots/bg/2specs.png)

### 3. Карти за гилотинно разкрояване
Оптимално подреждане върху стандартни плочи:

![Карти за разкрой](screenshots/bg/3cutting.png)

### 4. Материали и кантове
Управление на дебелини, декори и PVC/ABS кантове:

![Материали](screenshots/bg/4materials.png)

### 5. Светла тема
Контрастна светла тема:

![Светла тема](screenshots/bg/5lighttheme.png)

### 6. 37 вградени езика
Моментално превключване между 37 езика чрез **`🌐`**:

![37 езика](screenshots/bg/6languages.png)

---

## Лиценз

Проектът се разпространява под свободен [MIT License](../LICENSE) лиценз.
