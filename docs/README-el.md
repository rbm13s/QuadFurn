<div align="center">

<img src="../assets/icon.jpeg" alt="QuadFurn CAD Logo" width="128" height="128" style="border-radius: 18px;" />

# QuadFurn CAD

**CAD για έπιπλα κουτιού από μοριοσανίδα — Σχεδιαστής 4 προβολών + Βελτιστοποίηση γκιλοτίνας κοπής πάνελ**

[![Build & Release](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml/badge.svg)](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml)
[![Downloads](https://img.shields.io/github/downloads/rbm13s/QuadFurn/total?logo=github&color=blue)](https://github.com/rbm13s/QuadFurn/releases)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe_0.29-blue)](https://github.com/emilk/egui)
[![Graphics](https://img.shields.io/badge/Renderer-wgpu-green)](https://wgpu.rs/)
[![Languages](https://img.shields.io/badge/Languages-37_Locales-purple)](#8-θέματα-και-τοπικοποίηση-σε-37-γλώσσες)
[![Platform](https://img.shields.io/badge/Platform-Windows_%7C_Linux_%7C_macOS_%7C_Web-0078D6)](#γρήγορη-εκκίνηση)
[![Web Demo](https://img.shields.io/badge/Online_CAD-Δοκιμή_στο_πρόγραμμα_περιήγησης-success?logo=webassembly)](https://rbm13s.github.io/QuadFurn/app/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)

<br/>

👉 **[🚀 Εκτελέστε το QuadFurn CAD στο πρόγραμμα περιήγησης (WebAssembly)](https://rbm13s.github.io/QuadFurn/app/)** • **[🌐 Επίσημος ιστότοπος](https://rbm13s.github.io/QuadFurn/el.html)**

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
**Ελληνικά** •
[简体中文](README-zh-cn.md) •
[繁體中文](README-zh-tw.md) •
[日本語](README-ja.md) •
[한국어](README-ko.md)

</div>

---

## Περιεχόμενα

- [Εισαγωγή](#εισαγωγή)
- [Στιγμιότυπα οθόνης](#στιγμιότυπα-οθόνης)
- [Βασικά χαρακτηριστικά](#βασικά-χαρακτηριστικά)
- [Λειτουργική τεκμηρίωση](#λειτουργική-τεκμηρίωση)
- [Χειρισμός και συντομεύσεις](#χειρισμός-και-συντομεύσεις)
- [Άδεια χρήσης](#άδεια-χρήσης)

---

## Εισαγωγή

Το **QuadFurn CAD** είναι μια εφαρμογή CAD υψηλής απόδοσης γραμμένη σε Rust για τον σχεδιασμό επίπλων κουτιού από μελαμίνη/μοριοσανίδα, MDF και κόντρα πλακέ, με αυτόματη αφαίρεση περιθωρίων σοκόρου (PVC/ABS) και βελτιστοποίηση κοπής πάνελ γκιλοτίνας 2D.

---

## Στιγμιότυπα οθόνης

### 1. Σχεδιαστής CAD 4 προβολών (Σκούρο θέμα)
Συγχρονισμένες 2D ορθογραφικές προβολές (**Κάτοψη XY**, **Πρόσοψη XZ**, **Πλάγια τομή YZ**) και **3D προβολή**:

![Σχεδιαστής CAD](screenshots/el/1designer.png)

### 2. Προδιαγραφές εξαρτημάτων
Πίνακας διαστάσεων μετά την αφαίρεση του σοκόρου:

![Προδιαγραφές](screenshots/el/2specs.png)

### 3. Χάρτες κοπής
Βέλτιστη τοποθέτηση σε τυποποιημένα φύλλα:

![Χάρτες κοπής](screenshots/el/3cutting.png)

### 4. Υλικά και σοκόρα
Διαχείριση παχών πάνελ, χρωμάτων και υφών (`PNG`, `JPG`, `WEBP`):

![Υλικά](screenshots/el/4materials.png)

### 5. Φωτεινό θέμα
Φωτεινό θέμα υψηλής αντίθεσης:

![Φωτεινό θέμα](screenshots/el/5lighttheme.png)

### 6. 37 ενσωματωμένες γλώσσες
Άμεση εναλλαγή μεταξύ 37 γλωσσών διεπαφής:

![37 γλώσσες](screenshots/el/6languages.png)

---

## Άδεια χρήσης

Το έργο διανέμεται υπό την ελεύθερη άδεια [MIT License](../LICENSE).
