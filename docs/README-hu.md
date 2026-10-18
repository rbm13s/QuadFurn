<div align="center">

<img src="../assets/icon.jpeg" alt="QuadFurn CAD Logo" width="128" height="128" style="border-radius: 18px;" />

# QuadFurn CAD

**Bútoripari lap- és forgácslaptervező CAD — 4-nézetes térbeli szerkesztő + Guillotine lapszabászati optimalizáló**

[![Build & Release](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml/badge.svg)](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe_0.29-blue)](https://github.com/emilk/egui)
[![Graphics](https://img.shields.io/badge/Renderer-wgpu-green)](https://wgpu.rs/)
[![Languages](https://img.shields.io/badge/Languages-22_Locales-purple)](#8-vizuális-témák-és-lokalizáció-22-nyelven)
[![Platform](https://img.shields.io/badge/Platform-Windows_%7C_Linux_%7C_macOS-0078D6)](#első-lépések)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)

<br/>

**Languages / Языки / Мови / Sprachen:**<br/>
[English](../README.md) •
[Русский](README-ru.md) •
[Українська](README-uk.md) •
[Deutsch](README-de.md) •
[Polski](README-pl.md) •
[Español](README-es.md) •
[Français](README-fr.md) •
[Italiano](README-it.md) •
[Português](README-pt.md) •
[Čeština](README-cs.md) •
[Slovenčina](README-sk.md) •
[Slovenščina](README-sl.md) •
[Nederlands](README-nl.md) •
[Dansk](README-da.md) •
[Norsk](README-no.md) •
[Svenska](README-sv.md) •
**Magyar** •
[Română](README-ro.md) •
[Türkçe](README-tr.md) •
[简体中文](README-zh-cn.md) •
[日本語](README-ja.md) •
[한국어](README-ko.md)

</div>

---

## Tartalomjegyzék

- [Bevezetés](#bevezetés)
- [Képernyőképek és vizuális bemutató](#képernyőképek-és-vizuális-bemutató)
- [Főbb jellemzők](#főbb-jellemzők)
- [Funkcionális dokumentáció](#funkcionális-dokumentáció)
  - [1. 4-nézetes CAD szerkesztő (Hammer Editor + CorelDRAW munkamenet)](#1-4-nézetes-cad-szerkesztő-hammer-editor--coreldraw-munkamenet)
  - [2. Aritmetikai kifejezések a numerikus beviteli mezőkben](#2-aritmetikai-kifejezések-a-numerikus-beviteli-mezőkben)
  - [3. Élzárás és a nettó vágásméret automatikus levonása](#3-élzárás-és-a-nettó-vágásméret-automatikus-levonása)
  - [4. 2D guillotine lapszabászati optimalizálás (Cutting 3 stílusban)](#4-2d-guillotine-lapszabászati-optimalizálás-cutting-3-stílusban)
  - [5. Többlapos projektkezelés és projektek közötti vágólap](#5-többlapos-projektkezelés-és-projektek-közötti-vágólap)
  - [6. Gyártási útlevél exportálása (vektoros PDF, CSV, vágólap)](#6-gyártási-útlevél-exportálása-vektoros-pdf-csv-vágólap)
  - [7. Lapanyag-, textúra- és élzárókatalógus](#7-lapanyag--textúra--és-élzárókatalógus)
  - [8. Vizuális témák és lokalizáció 22 nyelven](#8-vizuális-témák-és-lokalizáció-22-nyelven)
- [Vezérlés és billentyűparancsok](#vezérlés-és-billentyűparancsok)
- [Első lépések](#első-lépések)
- [Beállítások és fájlformátumok](#beállítások-és-fájlformátumok)
- [Projektarchitektúra](#projektarchitektúra)
- [Licenc](#licenc)

---

## Bevezetés

A **QuadFurn CAD** egy villámgyors, natív asztali CAD alkalmazás Rust nyelven megírva, amelyet kifejezetten laminált bútorlapból (faforgácslap, MDF, HDF, rétegelt lemez) készülő korpuszbútorok tervezésére fejlesztettek, automatikus alkatrészjegyzék-készítéssel és 2D guillotine lapszabászati optimalizáló motorral kiegészítve.

A QuadFurn három elismert szoftver ergonomikus előnyeit ötvözi egyetlen munkafolyamatban:
1. **4-nézetes modellezés a Valve Hammer Editor stílusában**: Egyidejű szerkesztés három szinkronizált 2D merőleges vetületben (**Felülnézet XY — Alaprajz**, **Elölnézet XZ — Homlokzat**, **Oldalnézet YZ — Metszet**) gördítősávokkal, mágneses élillesztéssel és valós idejű **3D Körbeforgatás / Belső nézetes szabad repülés (`Z` Noclip)** ablakkal.
2. **Precíziós transzformációs sáv CorelDRAW stílusban**: Azonnali koordinátamegadás (`X`, `Y`, `Z`), készméret-állítás (`L × W × T`) és elforgatási szög (`∠Z`) beépített matematikai kifejezésértékelővel minden mezőben (pl. `600-4` vagy `720+18`).
3. **Élzárólevonás és fészkelés Cutting 3 stílusban**: Az ABS/PVC élzárók vastagságának automatikus levonása mind a 4 oldalon a lapszabász fűrészgéphez szükséges **nettó szabásméret** kiszámításához, kiegészítve egy guillotine-elvű 2D optimalizálóval, amely figyelembe veszi a szálirányt, a fűrészlap vágásvastagságát és a szélezési levágásokat.

---

## Képernyőképek és vizuális bemutató

### 1. 4-nézetes CAD szerkesztő (Sötét téma)
Összehangolt ortogonális vetületek (**Felülnézet XY**, **Elölnézet XZ**, **Oldalnézet YZ**) és textúrázott **3D nézet** perspektivikusan helyes UV-leképezéssel, 8-pontos méretező fogantyúkkal és a jobb oldali **Alkatrész-felügyelővel**.

![4-nézetes CAD szerkesztő](../assets/screenshots/1designer.png)

### 2. Alkatrészjegyzék (Specifikáció) és szabásméretek
Áttekinthető táblázat, amely bemutatja mind a **Készméreteket (`L × W`)**, mind a **Nettó szabásméreteket (`CUT L × W`)** az élzárók levonása után az egyes oldalakon (`L1`, `L2`, `W1`, `W2`).

![Alkatrészjegyzék](../assets/screenshots/2specs.png)

### 3. Guillotine lapszabászati terv
Optimalizált lapelrendezés szabványméretű lapokra (`2800 × 2070 mm` stb.) átmenő guillotine vágásokkal, fűrészlapvastagsággal, szélezéssel, színes élzárás-jelölőkkel és a hasznos hulladékok méretezett nyilvántartásával.

![Szabásterv](../assets/screenshots/3cutting.png)

### 4. Lapanyag- és élzárókatalógus
Lapvastagságok, 3D színek, egyéni felületi textúrák (`PNG`, `JPG`, `WEBP`), szálirány-zárak és ABS/PVC élzáró típusok kezelése.

![Anyagkatalógus](../assets/screenshots/4materials.png)

### 5. Világos téma támogatása
Modern, kontrasztos világos téma a sötét téma mellett, valamint az operativ rendszer témabeállításának automatikus követése.

![Világos téma](../assets/screenshots/5lighttheme.png)

### 6. 22 beépített nyelv (Felület és PDF export)
Azonnali nyelvváltás újraindítás nélkül a felső eszköztár **`🌐`** gombjával, az operációs rendszer nyelvének automatikus észlelése és többnyelvű PDF dokumentáció.

![22 nyelv](../assets/screenshots/6languages.png)

---

## Főbb jellemzők

- **Szinkronizált 4-nézetes munkaterület**: Szerkesszen a Felül (`XY`), Elöl (`XZ`), Oldalt (`YZ`) nézetekben, vagy nagyítson ki bármelyik ablakot egyetlen kattintással (`[Maximalizálás]` / `[4 nézet visszaállítása]`).
- **Hardveresen gyorsított 3D előnézet**: Körbeforgatás (`Jobb egérgomb`/`Középső gomb`) + Hammer stílusú szabad repülés (`Z gomb` + `WASD`/`Szóköz`/`Ctrl`) UV-textúrázással és pontos OBB sugárkövetéses kijelöléssel.
- **Mágneses illesztés és dinamikus hézagméretezés**: Automatikus rácshoz illesztés (`1`, `9`, `18`, `50 mm` vagy egyéni), mágneses tapadás a szomszédos elemek lapjaihoz és éleihez (`0 mm`), valamint a szomszédos alkatrészek valós távolságának milliméteres kijelzése.
- **Keretes kijelölés és csoportos mozgatás**: Rajzoljon kijelölő keretet bármely 2D nézetben több alkatrész egyszerre történő mozgatásához, duplázásához (`Ctrl+D`) vagy más bútorprojektekbe való átmásolásához (`Ctrl+C` / `Ctrl+V`).
- **Beépített számológép**: Minden numerikus beviteli mező fogad matematikai műveleteket, például `600+100`, `800-36`, `720/2` vagy `45+15`.
- **Automatikus élzárólevonás szabáskor**: Rendeljen ABS élzárót (`0.4 mm`, `0.5 mm`, `1.0 mm`, `2.0 mm` stb.) az alkatrész bármelyik 4 éléhez (`L1`, `L2`, `W1`, `W2`), és azonnal megjelenik a szabásméret:
  $$\text{Szabási hossz} = L_{\text{kész}} - E_{L1} - E_{L2}, \quad \text{Szabási szélesség} = W_{\text{kész}} - E_{W1} - E_{W2}$$
- **2D guillotine optimalizáló motor**: Többlapos szabászat a fa szálirányának, a fűrészlap vágásvastagságának és a 4 oldali szélezési levágásnak a figyelembevételével.
- **Teljes gyártási útlevél vektoros fekvő A4 PDF formátumban**: Egykattintásos exportálás az aktív nyelven:
  1. Részletes alkatrészjegyzék és készméretek
  2. 4-nézetes műszaki tervrajzok (`Felülnézet XY`, `3D Izometria`, `Elölnézet XZ`, `Oldalnézet YZ`) a befoglaló méretekkel
  3. Részletes guillotine lapszabászati tervek minden egyes táblára
  4. Lapanyag-összesítő és élzáró-hosszméret `+15 %` ráhagyással

---

## Funkcionális dokumentáció

### 1. 4-nézetes CAD szerkesztő (Hammer Editor + CorelDRAW munkamenet)

A **Szerkesztő** lap (`📐 1. Szerkesztő`) három fő területből áll:

1. **Felső eszköztárak**:
   - **Rajzeszközök**:
     - `↖ Kijelölés / Mozgatás (V)`: Kattintással kiválasztja az elemet, húzással mozgatja, vagy a 2D nézetekben lévő 8 fogantyúval átméretezi. Üres területen húzva téglalapos többszörös kijelölést indít.
     - `➕ Alkatrész rajzolása kerettel (B)`: Téglalap húzásával a 2D nézetekben (`Felülnézet XY`, `Elölnézet XZ` vagy `Oldalnézet YZ`) közvetlenül az adott síknak megfelelő lapot hoz létre.
     - `📏 Mérőszalag (M)`: Pontos távolságmérés ponttól pontig (`ΔX`, `ΔY`, `L`) milliméterben.
   - **Gyors beillesztés**: Gombok polc (`+ Polc (XY)`), oldalfal (`+ Oldallap (YZ)`) vagy ajtó/hátfal (`+ Ajtó/Hátfal (XZ)`) azonnali elhelyezésére.
   - **Rács és illesztés**: Rácslépés beállítása (`1`, `9`, `18`, `50 mm`), mágneses élillesztés és látható hézagméretezés.
   - **Kamerabeállítások**:
     - `🔍 Nézet középre igazítása`: A kamerát az origóra vagy a bútor középpontjára állítja.
     - `📐 Ablakhoz igazítás`: Kiszámítja a bútor külső méreteit és mind a 4 ablakban egyszerre a képernyőhöz igazítja a méretarányt.
   - **CorelDRAW transzformációs sáv**: Közvetlen koordináta-beírás `X`, `Y`, `Z`, alkatrészméretek `L`, `W`, `T`, zöld vágási címke `[Vágás: L × W mm]` és szögelforgatás `∠Z`.

2. **Négy szerkesztőablak**:
   - **2D ortogonális nézetek (`Felülnézet XY`, `Elölnézet XZ`, `Oldalnézet YZ`)** görgetősávokkal, milliméteres vonalzókkal és az élzárások színes sávjaival.
   - **3D nézet**: Körbeforgatás (`Jobb` vagy `Középső egérgomb`), zoom görgővel, 3D kijelölés és **Belső nézetes szabad repülés (Noclip)** (nyomja meg a **`Z`** gombot a 3D ablak felett, és repüljön a `WASD`, `Szóköz`, `Ctrl`/`Shift` billentyűkkel).

3. **Jobb oldali Alkatrész-felügyelő**:
   - Alkatrészek rendezése egyéni **Csoportokba** (*Korpusz*, *Ajtók*, *Fiókok*).
   - Láthatóság kapcsolója (`👁`) és szerkesztés elleni zárolás (`🔒`).
   - Lapanyag kiválasztása, tájolás, méretek (`L × W` megfordítási lehetőséggel `⇄`), szálirány rögzítése (`🌲 Szálirány rögzítése`), keresztirányú szál (`🔄 Keresztirányú szálfutás`) és interaktív 4-oldalas élzáró diagram.

---

### 2. Aritmetikai kifejezések a numerikus beviteli mezőkben

A program összes számmezője támogatja a matematikai műveleteket:
- **Összeadás és kivonás**: Egy `600` értékű mezőbe írja be, hogy `600+100` vagy `600-4`, majd nyomjon `Enter`-t — a mező tartalma azonnal `700`-ra vagy `596`-ra vált.
- **Szorzás, osztás és zárójelek**: Kifejezések, mint `(800-36)/2` vagy `18*2` automatikusan kiértékelődnek.
- A pont `.` és a vessző `,` egyaránt használható tizedesjelként, és a mértékegységeket (`mm`, `°`) a program automatikusan figyelmen kívül hagyja.

---

### 3. Élzárás és a nettó vágásméret automatikus levonása

- Minden alkatrésznek 4 éle van: **`L1`** (Hossz 1 / Elöl), **`L2`** (Hossz 2 / Hátul), **`W1`** (Szélesség 1 / Oldal 1) és **`W2`** (Szélesség 2 / Oldal 2).
- Ha egy `680 × 200 mm`-es alkatrészhez `2.0 mm`-es ABS élzárót rendel az `L1` élre és `0.5 mm`-eset az `L2`, `W1`, `W2` élekre, a program azonnal kiszámítja a **Nettó szabásméretet**:
  - $\text{Vágási hossz } L = 680 - 2.0 - 0.5 = 677.5\text{ mm}$
  - $\text{Vágási szélesség } W = 200 - 0.5 - 0.5 = 199.0\text{ mm}$
- Az élzárók jól megkülönböztethető színes sávként jelennek meg a 2D/3D vetületeken és a lapszabászati rajzokon is.

---

### 4. 2D guillotine lapszabászati optimalizálás (Cutting 3 stílusban)

A **Szabásterv** lap (`🪚 3. Szabásterv`) kiszámítja az alkatrészek legoptimálisabb elhelyezését az alapanyag lapokon:
- **Testreszabható fűrészelési paraméterek**:
  - **Fűrészlap vágási szélessége** (alapértelmezett `4.0 mm`).
  - **Lap szélezési levágásai** (`Bal`, `Jobb`, `Fent`, `Lent`, alapértelmezett `10 mm`) a gyári sérülések eltávolításához.
- **A fa szálirányának tiszteletben tartása**:
  - **Rögzített szálirányú (`🌲`) alkatrészek**: szigorúan párhuzamosak maradnak a lap hosszméretével (`Laphossz L`).
  - **Szabad szálirányú alkatrészek**: $90^\circ$-kal elforgathatók, amennyiben ez növeli az anyagkihozatalt.
- **Vizuális vágástérkép és hasznos maradékok**:
  - Minden lap megjeleníti a piros átmenő guillotine vágási vonalakat, az alkatrészszámokat, a vágásméreteket, az élzárókat és a méretezett felhasználható maradékokat.
  - Átfogó statisztika: szükséges lapok darabszáma, anyagkihozatali arány százalékban (%), összes vágáshossz és élzáró-szükséglet méterben.

---

### 5. Többlapos projektkezelés és projektek közötti vágólap

- **Kezdőlap és előzmények**: Új üres bútor készítése, indítás 2-ajtós kisszekrény sablonból, `.qfurn`/`.json` fájlok megnyitása vagy gyors választás a **10 legutóbb megnyitott projektből**.
- **Több bútor egyidejű tervezése**: Dolgozzon több elemen párhuzamos füleken (kattintson duplán a fül címére az átnevezéshez).
- **Másolás és beillesztés projektek között**: Jelöljön ki alkatrészeket az egyik fülön, nyomja meg a `Ctrl+C` kombinációt, lépjen át egy másik fülre (vagy egy másik nyitott QuadFurn ablakba), és nyomja meg a `Ctrl+V`-t. A célprojektben hiányzó anyagok és élzárók automatikusan létrejönnek.
- **Adatvesztés elleni védelem**: Módosított fül bezárásakor vagy a szoftverből való kilépéskor megerősítő ablak jelenik meg (`Mentés és kilépés`, `Kilépés mentés nélkül`, `Mégse`).

---

### 6. Gyártási útlevél exportálása (vektoros PDF, CSV, vágólap)

A **Specifikáció** lapon (`📋 2. Specifikáció`):
1. **`📄 Exportálás PDF-be` (Vektoros fekvő A4 gyártási útlevél)**:
   - **1. fejezet**: Teljes alkatrészjegyzék kész- és szabásméretekkel, élzárásokkal és száliránnyal.
   - **2. fejezet**: 4-nézetes műszaki tervrajzok (`Felülnézet XY`, `3D Izometria`, `Elölnézet XZ`, `Oldalnézet YZ`) külső méretekkel.
   - **3. fejezet**: Részletes guillotine lapszabászati tervek minden egyes felhasznált lapra.
   - **4. fejezet**: Lapanyag-összesítő és élzáró-fogyasztás (nettó folyóméter és $+15 \%$ tartalék).
   - A dokumentum az **aktuális nyelven** készül beágyazott TrueType/CID vektoros betűtípusokkal.
2. **`💾 Exportálás CSV-be (Excel)`**: UTF-8 BOM kódolású, pontosvesszővel `;` tagolt táblázat Microsoft Excelhez és LibreOffice-hoz.
3. **`📋 Vágási lista másolása`**: A szabásméreteket tabulátorral elválasztva a vágólapra másolja külső lapszabászati programokba (mint a Cutting 3) való beillesztéshez.

---

### 7. Lapanyag-, textúra- és élzárókatalógus

Az **Anyagok** lapon (`🎨 4. Anyagok`):
- Hozzon létre és szerkesszen **Lapanyagokat** (`Forgácslap 18 mm`, `MDF 16 mm`, `HDF 3 mm` stb.) RGB színekkel, nyers lapméretekkel (`L × W`) és alapértelmezett szálirány-beállítással.
- Töltsön be felületi textúrákat **`PNG`**, **`JPG`/`JPEG`** vagy **`WEBP`** formátumban a fotorealisztikus 3D látványhoz.
- **Hordozhatóság és automatikus textúra-keresés**: `.qfurn` fájl megnyitásakor a program automatikusan megkeresi a képeket:
  1. A projektfájlba közvetlenül beágyazva (Base64)
  2. Ugyanabban a mappában, ahol a `.qfurn` projektfájl található
  3. A projekt melletti `./textures/` almappában
  4. Relatív útvonal vagy eredeti abszolút útvonal szerint
- **Textúrák beágyazása a fájlba (`📦 Textúrák beágyazása a projektfájlba`)**: Kapcsolja be ezt a lehetőséget a felső menüben vagy az Anyagok lapon, hogy a képek Base64 formátumban közvetlenül a `.qfurn` fájlba kerüljenek. A projekt teljesen önállóvá válik, és bárhová hiányzó képek nélkül átvihető.
- **Grafikus kártya védelme**: A 60 MB-nál nagyobb fájlokat a program figyelmeztetéssel elutasítja, a túl nagy felbontású (> 4096 px) képeket pedig automatikusan leskálázza.
- **Élzáró típusok** kezelése vastagsággal és jelölőszínnel. Használatban lévő anyagok törlése megerősítést igényel.

---

### 8. Vizuális témák és lokalizáció 22 nyelven

- **Vizuális témák**: Egyszerű váltás a **🌙 Sötét**, **☀ Világos** és **🖥 Rendszer** témák között a felső sávban.
- **22 beépített nyelv**:
  `Magyar`, `English`, `Русский`, `Українська`, `Deutsch`, `Polski`, `Español`, `Français`, `Italiano`, `Português`, `Čeština`, `Slovenčina`, `Slovenščina`, `Nederlands`, `Dansk`, `Norsk`, `Svenska`, `Română`, `Türkçe`, `简体中文`, `日本語` és `한국어`.
- **Automatikus nyelvfelismerés az első indításkor**: Automatikusan felismeri és beállítja az operációs rendszer nyelvét. A beállításokat a `%APPDATA%\QuadFurn\settings.json` fájl tárolja.

---

## Vezérlés és billentyűparancsok

| Billentyű | Művelet |
| :--- | :--- |
| **`V`** | **Kijelölés és mozgatás** eszköz (8 fogantyúval és keretes kijelöléssel) |
| **`B`** | **Alkatrész rajzolása kerettel** eszköz a 2D nézetekben |
| **`M`** | **Mérőszalag** eszköz a 2D nézetekben |
| **Húzás `Bal egérgombbal` (2D)** | Elemek mozgatása, méretezése fogantyúval vagy kijelölő keret üres helyen |
| **`Shift + Bal egérgomb` (2D)** | Elem hozzáadása vagy elvétele a kijelölésből |
| **Húzás `Jobb` / `Középső gombbal` (2D)** | Nézet mozgatása (2D pásztázás) |
| **`Egérgörgő` (2D / 3D)** | Nagyítás / kicsinyítés a mutató pozíciójára |
| **Húzás `Jobb` / `Középső gombbal` (3D)** | 3D kamera körbeforgatása |
| **`Z` (a 3D ablak felett)** | **Belső nézetes szabad repülés (Noclip)** ki-/bekapcsolása |
| **`W` `A` `S` `D` / `Szóköz` / `Ctrl`** | Repülés előre/balra/hátra/jobbra/fel/le a 3D-ben (`Shift` gyorsít) |
| **`Ctrl + S`** | Aktuális bútor mentése (`.qfurn` / `.json`) |
| **`Ctrl + Z`** | Utolsó művelet visszavonása (akár 40 lépésig) |
| **`Ctrl + C` / `Ctrl + V`** | Kijelölt elemek másolása / beillesztése (fülek között is működik) |
| **`Ctrl + D`** | Kijelölt elemek megkettőzése |
| **`Delete` / `Backspace`** | Kijelölt elemek törlése |

---

## Első lépések

### Előre lefordított telepítőcsomagok (GitHub Releases)
Minden kiadás automatikusan generál futtatásra kész csomagokat a [GitHub Actions](https://github.com/rbm13s/QuadFurn/actions) segítségével:
- **Windows (`x86_64`)**: `QuadFurn-windows-x86_64.zip` — hordozható `QuadFurn.exe` beágyazott ikonnal.
- **Linux (`Debian 12+ / Ubuntu 22.04+ x86_64`)**:
  - `quadfurn_<verzió>_amd64.deb` — hivatalos Debian/Ubuntu csomag (telepíti a programot, a menübejegyzést és a rendszerikonokat):
    ```bash
    sudo apt install ./quadfurn_0.2.1_amd64.deb
    ```
  - `QuadFurn-linux-x86_64.tar.gz` — hordozható Linux archívum.
- **macOS (`Univerzális: Apple Silicon M1–M4 + Intel x86_64`)**: `QuadFurn-macos-universal.zip` — natív `QuadFurn.app` alkalmazás Retina ikonnal.

---

### Fordítás forráskódból

#### Rendszerkövetelmények
- [Rust Toolchain](https://rustup.rs/) (`stable`, Rust 2021 kiadás)
- Rendszerkönyvtárak **Linux (Debian / Ubuntu)** alatt:
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-dev libssl-dev \
    libgtk-3-dev libglib2.0-dev libatk1.0-dev libgdk-pixbuf-2.0-dev \
    libcairo2-dev libpango1.0-dev libgl1-mesa-dev fonts-dejavu-core fonts-noto-cjk
  ```
- **macOS**: Xcode parancssori eszközök (`xcode-select --install`)
- **Windows**: Windows 10 / 11 MSVC C++ fordítóeszközökkel

#### Parancsok a fordításhoz
```bash
# Futtatás fejlesztői módban
cargo run

# Optimalizált kiadási verzió fordítása
cargo build --release

# Egységtesztek futtatása
cargo test
```

---

## Beállítások és fájlformátumok

- **Felhasználói beállítások (`settings.json`)**:
  Automatikusan a platform szabványos konfigurációs mappájában tárolódik:
  - **Windows**: `%APPDATA%\QuadFurn\settings.json`
  - **Linux**: `~/.config/QuadFurn/settings.json`
  - **macOS**: `~/Library/Application Support/QuadFurn/settings.json`
  
  Megőrzi:
  - `theme_choice` (`Dark`, `Light`, `System`)
  - `language` (`Hu`, `En`, `Ru` stb.)
  - `recent_projects` (legfeljebb 10 legutóbb megnyitott fájl)
- **Projektfájlok (`.qfurn` / `.json`)**:
  Ember által olvasható UTF-8 formátumú JSON fájlok, amelyek tartalmazzák az összes alkatrészt, csoportot, anyagot, textúrát, élzárót és vágási beállítást. Bekapcsolt beágyazás esetén a képek Base64 formátumban kerülnek tárolásra.

---

## Projektarchitektúra

```text
QuadFurn/
├── .github/
│   └── workflows/
│       └── build.yml          # GitHub Actions munkafolyamatok (Linux .deb/.tar.gz, macOS .app, Windows .zip)
├── assets/
│   ├── icon.jpeg              # Alkalmazáslogó és forrásikon
│   └── screenshots/           # Képernyőképek a dokumentációhoz
├── docs/                      # 22 nyelvre lefordított dokumentáció
├── src/
│   ├── main.rs                # Belépési pont, lapfülek kezelése, kezdőoldal és gyorsbillentyűk
│   ├── model.rs               # Adatmodell (Project, PartBlock, SheetMaterial, EdgeBanding, OBB)
│   ├── viewport.rs            # 4-nézetes grafikus motor Hammer/CorelDRAW, 2D és 3D megjelenítés wgpu/painter
│   ├── snapping.rs            # Mágneses rács- és élillesztés, vetületi tengelyek
│   ├── cutter.rs              # 2D guillotine lapszabászat és élzárólevonás
│   ├── export.rs              # Vektoros fekvő A4 PDF exportáló és CSV exportáló
│   ├── ui_panels.rs           # CorelDRAW sáv, Alkatrész-felügyelő, Specifikáció, Szabásterv és Anyagok
│   └── i18n.rs                # Fordítási szótárak 22 nyelvre és operációs rendszer nyelvfelismerése
├── build.rs                   # Windows ICO ikon és PE erőforrás automatikus generálása
├── Cargo.toml                 # Csomagkonfiguráció és függőségek
└── LICENSE                    # MIT nyílt forráskódú licenc
```

---

## Licenc

Ez a projekt a [MIT Licenc](../LICENSE) alatt érhető el — szabadon használható, módosítható és terjeszthető kereskedelmi és magáncélú projektekben egyaránt.

