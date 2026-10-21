<div align="center">

<img src="../assets/icon.jpeg" alt="QuadFurn CAD Logo" width="128" height="128" style="border-radius: 18px;" />

# QuadFurn CAD

**Կորպուսային կահույքի CAD ԼԴՍՊ-ի համար — 4-տեսադաշտային տարածական նախագծիչ + թերթային նյութերի գիլյոտինային կտրում**

[![Build & Release](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml/badge.svg)](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml)
[![Downloads](https://img.shields.io/github/downloads/rbm13s/QuadFurn/total?logo=github&color=blue)](https://github.com/rbm13s/QuadFurn/releases)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe_0.29-blue)](https://github.com/emilk/egui)
[![Graphics](https://img.shields.io/badge/Renderer-wgpu-green)](https://wgpu.rs/)
[![Languages](https://img.shields.io/badge/Languages-37_Locales-purple)](#8-ձևավորման-թեմաներ-և-լոկալիզացում-37-լեզվով)
[![Platform](https://img.shields.io/badge/Platform-Windows_%7C_Linux_%7C_macOS_%7C_Web-0078D6)](#արագ-մեկնարկ)
[![Web Demo](https://img.shields.io/badge/Օնլայն_CAD-Գործարկել_բրաուզերում-success?logo=webassembly)](https://rbm13s.github.io/QuadFurn/app/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)

<br/>

👉 **[🚀 Գործարկել QuadFurn CAD-ը բրաուզերում (WebAssembly)](https://rbm13s.github.io/QuadFurn/app/)** • **[🌐 Պաշտոնական կայք](https://rbm13s.github.io/QuadFurn/hy.html)**

<br/>

**Languages / Языки / Мови / Sprachen:**<br/>
[English](../README.md) •
[Русский](README-ru.md) •
[Українська](README-uk.md) •
[Беларуская](README-be.md) •
[Қазақша](README-kk.md) •
[ქართული](README-ka.md) •
**Հայերեն** •
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
[Български](README-bg.md) •
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

## Բովանդակություն

- [Ներածություն](#ներածություն)
- [Սքրինշոթեր և տեսողական ակնարկ](#սքրինշոթեր-և-տեսողական-ակնարկ)
- [Հիմնական հնարավորություններ](#հիմնական-հնարավորություններ)
- [Ֆունկցիոնալ փաստաթղթավորում](#ֆունկցիոնալ-փաստաթղթավորում)
- [Կառավարում և ստեղների համադրություններ](#կառավարում-և-ստեղների-համադրություններ)
- [Լիցենզիա](#լիցենզիա)

---

## Ներածություն

**QuadFurn CAD** — բարձր արտադրողականությամբ բազմապլատֆորմային CAD ծրագիր Rust-ով, որը հասանելի է ինչպես արագ աշխատասեղանի հավելված (Windows, Linux, macOS), այնպես էլ առանց տեղադրման WebAssembly (WASM) օնլայն տարբերակ անմիջապես ժամանակակից բրաուզերներում: Նախատեսված է ԼԴՍՊ-ից, ՄԴՖ-ից և նրբատախտակից կորպուսային կահույքի նախագծման, ավտոմատ մասնագրերի ստեղծման և 2D գիլյոտինային կտրման օպտիմալացման համար:

---

## Սքրինշոթեր և տեսողական ակնարկ

### 1. 4-տեսադաշտային CAD նախագծիչ (Մուգ թեմա)
Համաժամանակյա 2D պրոյեկցիաներ (**Վերևից XY**, **Առջևից XZ**, **Կողքից YZ**) և ապարատային **3D պատուհան**:

![4-տեսադաշտային CAD](screenshots/hy/1designer.png)

### 2. Դետալների մասնագիր
Խմբավորված աղյուսակ՝ եզրաշերտի հաստության նվազեցումից հետո չափսերով:

![Մասնագրի աղյուսակ](screenshots/hy/2specs.png)

### 3. Կտրման քարտեր
Օպտիմալ դասավորություն ստանդարտ թերթերի վրա:

![Կտրման քարտեր](screenshots/hy/3cutting.png)

### 4. Նյութեր և եզրաշերտեր
Հաստությունների, գույների և տեքստուրաների (`PNG`, `JPG`, `WEBP`) կառավարում:

![Նյութեր և եզրաշերտեր](screenshots/hy/4materials.png)

### 5. Բաց թեմա
Կոնտրաստային բաց թեմա:

![Բաց թեմա](screenshots/hy/5lighttheme.png)

### 6. 37 ներկառուցված լեզու
Արագ փոխարկում 37 լեզուների միջև **`🌐`** կոճակով:

![37 լեզու](screenshots/hy/6languages.png)

---

## Լիցենզիա

Ծրագիրը տարածվում է ազատ [MIT License](../LICENSE) արտոնագրով:
