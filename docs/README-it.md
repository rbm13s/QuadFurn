<div align="center">

<img src="../assets/icon.jpeg" alt="QuadFurn CAD Logo" width="128" height="128" style="border-radius: 18px;" />

# QuadFurn CAD

**CAD per mobili in pannelli e truciolare — Progettista spaziale a 4 viste + Ottimizzatore di taglio lastre a ghigliottina**

[![Build & Release](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml/badge.svg)](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml)
[![Downloads](https://img.shields.io/github/downloads/rbm13s/QuadFurn/total?logo=github&color=blue)](https://github.com/rbm13s/QuadFurn/releases)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe_0.29-blue)](https://github.com/emilk/egui)
[![Graphics](https://img.shields.io/badge/Renderer-wgpu-green)](https://wgpu.rs/)
[![Languages](https://img.shields.io/badge/Languages-22_Locales-purple)](#8-temi-visivi-e-localizzazione-in-22-lingue)
[![Platform](https://img.shields.io/badge/Platform-Windows_%7C_Linux_%7C_macOS-0078D6)](#guida-introduttiva)
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
**Italiano** •
[Português](README-pt.md) •
[Čeština](README-cs.md) •
[Slovenčina](README-sk.md) •
[Slovenščina](README-sl.md) •
[Nederlands](README-nl.md) •
[Dansk](README-da.md) •
[Norsk](README-no.md) •
[Svenska](README-sv.md) •
[Magyar](README-hu.md) •
[Română](README-ro.md) •
[Türkçe](README-tr.md) •
[简体中文](README-zh-cn.md) •
[日本語](README-ja.md) •
[한국어](README-ko.md)

</div>

---

## Indice dei contenuti

- [Introduzione](#introduzione)
- [Schermate e panoramica visiva](#schermate-e-panoramica-visiva)
- [Caratteristiche principali](#caratteristiche-principali)
- [Documentazione funzionale](#documentazione-funzionale)
  - [1. Progettista CAD a 4 viste (flusso di lavoro Hammer Editor + CorelDRAW)](#1-progettista-cad-a-4-viste-flusso-di-lavoro-hammer-editor--coreldraw)
  - [2. Espressioni aritmetiche nei campi numerici](#2-espressioni-aritmetiche-nei-campi-numerici)
  - [3. Bordatura e detrazione automatica della misura netta di taglio](#3-bordatura-e-detrazione-automatica-della-misura-netta-di-taglio)
  - [4. Ottimizzazione di taglio lastre 2D a ghigliottina (stile Cutting 3)](#4-ottimizzazione-di-taglio-lastre-2d-a-ghigliottina-stile-cutting-3)
  - [5. Schede multi-progetto e appunti tra progetti](#5-schede-multi-progetto-e-appunti-tra-progetti)
  - [6. Esportazione del passaporto di produzione (PDF vettoriale, CSV, appunti)](#6-esportazione-del-passaporto-di-produzione-pdf-vettoriale-csv-appunti)
  - [7. Catalogo materiali, texture e bordature](#7-catalogo-materiali-texture-e-bordature)
  - [8. Temi visivi e localizzazione in 22 lingue](#8-temi-visivi-e-localizzazione-in-22-lingue)
- [Controlli e scorciatoie da tastiera](#controlli-e-scorciatoie-da-tastiera)
- [Guida introduttiva](#guida-introduttiva)
- [Impostazioni e formati di file](#impostazioni-e-formati-di-file)
- [Architettura del progetto](#architettura-del-progetto)
- [Licenza](#licenza)

---

## Introduzione

**QuadFurn CAD** è un'applicazione desktop CAD ad alte prestazioni scritta in Rust, progettata per l'ideazione e costruzione di mobili in pannelli (truciolare nobilitato/melaminico, MDF, HDF, compensato), con distinta base automatica e ottimizzazione di taglio bidimensionale a ghigliottina.

QuadFurn unisce l'ergonomia di tre celebri strumenti in un flusso di lavoro armonioso:
1. **Modellazione a 4 viste in stile Valve Hammer Editor**: Modifica sincronizzata attraverso tre proiezioni ortografiche 2D (**Dall'alto XY — Pianta**, **Frontale XZ — Prospetto**, **Laterale YZ — Sezione**) con barre di scorrimento Hammer, snap magnetico ai bordi e una vista **3D Orbit / Volo libero in prima persona (`Z` Noclip)** in tempo reale.
2. **Barra di trasformazione di precisione in stile CorelDRAW**: Inserimento numerico immediato delle coordinate (`X`, `Y`, `Z`), dimensioni complessive (`L × W × T`) e angolo di rotazione orizzontale (`∠Z`), con calcolatrice matematica integrata direttamente nei campi (es. digitando `600-4` o `720+18`).
3. **Detrazione bordi e nesting in stile Cutting 3**: Sottrazione automatica dello spessore del bordo in PVC/ABS da tutti e 4 i lati per calcolare le esatte **misure nette di taglio alla sega**, abbinata a un algoritmo di taglio 2D a ghigliottina che rispetta la venatura del legno, lo spessore della lama e i rifili perimetrali.

---

## Schermate e panoramica visiva

### 1. Progettista CAD a 4 viste (Tema Scuro)
Proiezioni ortografiche simultanee (**Dall'alto XY**, **Frontale XZ**, **Laterale YZ**) e **Vista 3D** con mappatura texture con correzione prospettica, gizmo a 8 maniglie di ridimensionamento e **Ispettore elementi** a destra.

![Progettista CAD a 4 viste](../assets/screenshots/1designer.png)

### 2. Distinta base e specifiche
Tabella riassuntiva che visualizza sia le **Dimensioni finite (`L × W`)** che le **Dimensioni nette di taglio (`CUT L × W`)** dopo la detrazione dei bordi su ciascun lato (`L1`, `L2`, `W1`, `W2`).

![Tabella delle specifiche](../assets/screenshots/2specs.png)

### 3. Schema di taglio lastre a ghigliottina
Ottimizzazione del taglio su lastre standard (`2800 × 2070 mm`, ecc.) con tagli passanti a ghigliottina, tolleranza lama, margini di rifilo, indicatori visivi di bordatura e mappa dei ritagli riutilizzabili.

![Schema di taglio](../assets/screenshots/3cutting.png)

### 4. Catalogo materiali e bordature
Gestione degli spessori dei pannelli, colori 3D, texture personalizzate (`PNG`, `JPG`, `WEBP`), vincoli di venatura e tipi di bordo in PVC/ABS.

![Catalogo materiali e bordature](../assets/screenshots/4materials.png)

### 5. Supporto per Tema Chiaro
Tema chiaro nitido ad alto contrasto affiancato al Tema Scuro e alla sincronizzazione automatica con le impostazioni di sistema.

![Interfaccia con Tema Chiaro](../assets/screenshots/5lighttheme.png)

### 6. 22 lingue integrate (Interfaccia ed esportazione PDF)
Cambio istantaneo della lingua senza riavvio tramite il selettore **`🌐`** nella barra superiore, rilevamento automatico della lingua del sistema operativo al primo avvio ed esportazioni PDF multilingue.

![22 lingue disponibili](../assets/screenshots/6languages.png)

---

## Caratteristiche principali

- **Area di lavoro sincronizzata a 4 viste**: Modifica nelle viste Dall'alto (`XY`), Frontale (`XZ`), Laterale (`YZ`), oppure massimizza qualsiasi vista con un solo clic (`[Massimizza]` / `[Ripristina 4 viste]`).
- **Anteprima 3D accelerata da GPU**: Telecamera orbitale (`Tasto destro`/`Tasto centrale`) + modalità volo libero in prima persona in stile Hammer (`tasto Z` + `WASD`/`Spazio`/`Ctrl`) con rendering delle texture e raycasting OBB preciso.
- **Snap magnetico e quote dinamiche delle distanze**: Aggancio automatico al passo della griglia (`1`, `9`, `18`, `50 mm` o personalizzato), snap magnetico a spigoli e facce vicine (`0 mm`), e indicazione in tempo reale della distanza dalle parti adiacenti.
- **Selezione rettangolare e trasformazione di gruppo**: Traccia un riquadro di selezione in qualsiasi vista 2D per selezionare più parti, spostarle assieme, duplicarle (`Ctrl+D`) o copiarle/incollarle tra diversi progetti di arredo (`Ctrl+C` / `Ctrl+V`).
- **Calcolatrice matematica integrata**: Ogni casella numerica supporta espressioni aritmetiche quali `600+100`, `800-36`, `720/2` o `45+15`.
- **Detrazione automatica dei bordi per il taglio**: Assegna bordi in PVC/ABS (`0.4 mm`, `0.5 mm`, `1.0 mm`, `2.0 mm`, ecc.) a ciascuno dei 4 lati (`L1`, `L2`, `W1`, `W2`) e calcola subito la dimensione di taglio:
  $$\text{Lunghezza taglio} = L_{\text{finito}} - E_{L1} - E_{L2}, \quad \text{Larghezza taglio} = W_{\text{finito}} - E_{W1} - E_{W2}$$
- **Motore di nesting 2D a ghigliottina**: Calcolo di taglio su lastre multiple rispettando l'orientamento della venatura del legno, spessore della lama e rifilo su 4 lati.
- **Passaporto di produzione completo in PDF vettoriale A4 Orizzontale**: Esportazione con un clic nella lingua attiva contenente:
  1. Distinta base completa dei pezzi
  2. Prospetti tecnici a 4 viste (`Dall'alto XY`, `Assonometria 3D`, `Frontale XZ`, `Laterale YZ`)
  3. Schemi di taglio a ghigliottina per ogni singola lastra
  4. Riepilogo lastre e consumo lineare di bordatura con maggiorazione sfrido del `+15%`

---

## Documentazione funzionale

### 1. Progettista CAD a 4 viste (flusso di lavoro Hammer Editor + CorelDRAW)

La scheda **Progettista** (`📐 1. Progettista`) include tre aree principali:

1. **Barre degli strumenti superiori**:
   - **Strumenti di disegno**:
     - `↖ Seleziona / Sposta (V)`: Clicca su un elemento per selezionarlo, trascina per spostarlo, o manipola le 8 maniglie di ridimensionamento nelle viste 2D. Trascina sull'area vuota per disegnare un riquadro di selezione multipla.
     - `➕ Crea elemento con riquadro (B)`: Traccia un rettangolo in una vista 2D (`Dall'alto XY`, `Frontale XZ` o `Laterale YZ`) per inserire un pannello orientato secondo quel piano.
     - `📏 Metro a nastro (M)`: Misura distanze punto-punto esatte (`ΔX`, `ΔY`, `L`) in millimetri.
   - **Inserimento rapido**: Pulsanti per aggiungere con un clic ripiani (`+ Ripiano (XY)`), montanti (`+ Montante (YZ)`) o frontali/schienali (`+ Frontale/Schienale (XZ)`).
   - **Griglia e snap**: Scelta del passo della griglia (`1`, `9`, `18`, `50 mm`), snap magnetico ai bordi delle parti adiacenti e quote dinamiche dei giochi.
   - **Inquadratura telecamera**:
     - `🔍 Centra vista`: Riporta la vista al centro del mobile/origine.
     - `📐 Adatta alla finestra`: Calcola l'ingombro totale del modello e adatta la scala in tutti i 4 riquadri contemporaneamente.
   - **Barra di trasformazione CorelDRAW**: Visualizzazione e modifica diretta delle coordinate `X`, `Y`, `Z`, dimensioni `L`, `W`, `T`, etichetta verde di taglio `[Taglio: L × W mm]` e rotazione `∠Z`.

2. **Riquadri a 4 viste**:
   - **Viste ortografiche 2D (`Dall'alto XY`, `Frontale XZ`, `Laterale YZ`)** dotate di barre di scorrimento interattive, righelli millimetrici ed evidenziazione a colori dei bordi applicati.
   - **Vista 3D**: Rotazione orbitale (`Tasto destro` o `Tasto centrale`), zoom con rotellina, selezione 3D e **Modalità Volo Libero in Prima Persona** (premi **`Z`** sopra la vista 3D per volare con i tasti `WASD`, `Spazio`, `Ctrl`/`Maiusc`).

3. **Ispettore elementi laterale destro**:
   - Raggruppamento delle parti in **Gruppi** personalizzati (*Scocca*, *Ante*, *Cassetti*).
   - Attivazione/disattivazione di visibilità (`👁`) e blocco modifiche (`🔒`).
   - Scelta del materiale, orientamento, dimensioni (`L × W` con scambio rapido `⇄`), blocco venatura (`🌲 Blocca direzione venatura`), venatura trasversale (`🔄 Venatura trasversale`) e schema interattivo perimetrale dei 4 bordi.

---

### 2. Espressioni aritmetiche nei campi numerici

Tutti i campi di input numerici supportano formule aritmetiche:
- **Addizioni e sottrazioni**: Digita `600+100` o `600-4` in una casella contenente `600` e premi `Invio` — il campo calcolerà all'istante `700` o `596`.
- **Moltiplicazioni, divisioni e parentesi**: Espressioni come `(800-36)/2` o `18*2` vengono valutate automaticamente.
- Sia il punto `.` che la virgola `,` sono accettati come separatori decimali, e i suffissi delle unità di misura (`mm`, `°`) vengono ignorati senza generare errori.

---

### 3. Bordatura e detrazione automatica della misura netta di taglio

- Ciascun pannello presenta 4 lati: **`L1`** (Lunghezza 1 / Anteriore), **`L2`** (Lunghezza 2 / Posteriore), **`W1`** (Larghezza 1 / Estremità 1) e **`W2`** (Larghezza 2 / Estremità 2).
- Applicando ad esempio un bordo PVC da `2.0 mm` su `L1` e da `0.5 mm` su `L2`, `W1`, `W2` su una parte da `680 × 200 mm`, il software calcola in automatico la **Misura netta di taglio**:
  - $\text{Taglio } L = 680 - 2.0 - 0.5 = 677.5\text{ mm}$
  - $\text{Taglio } W = 200 - 0.5 - 0.5 = 199.0\text{ mm}$
- I bordi vengono evidenziati con strisce colorate sia nei prospetti 2D/3D che negli schemi di taglio delle lastre.

---

### 4. Ottimizzazione di taglio lastre 2D a ghigliottina (stile Cutting 3)

La scheda **Schema di taglio** (`🪚 3. Schema di taglio`) ottimizza la disposizione di tutte le parti sui pannelli:
- **Parametri di taglio configurabili**:
  - **Spessore della lama** (predefinito `4.0 mm`).
  - **Margini di rifilo perimetrale** (`Sinistra`, `Destra`, `Superiore`, `Inferiore`, predefinito `10 mm`) per eliminare sbeccature o bordi grezzi di fabbrica.
- **Rispetto dell'orientamento della venatura**:
  - Parti con **Venatura bloccata (`🌲`)**: mantenute rigorosamente allineate alla lunghezza della lastra (`Lunghezza L`).
  - Parti con **Venatura libera**: ruotabili di $90^\circ$ se ciò migliora lo sfruttamento superficiale.
- **Mappa visiva dei tagli e sfridi riutilizzabili**:
  - Visualizzazione di ogni lastra con linee di taglio a ghigliottina in rosso, numerazione dei pezzi, dimensioni di taglio, codici bordo e ritagli riutilizzabili etichettati con misure.
  - Statistiche dettagliate: numero di lastre necessarie, rendimento percentuale (%), metri lineari di taglio e consumo bordi.

---

### 5. Schede multi-progetto e appunti tra progetti

- **Pagina iniziale e progetti recenti**: Creazione di un nuovo mobile vuoto, avvio da modello predefinito di mobile a 2 ante, apertura di file `.qfurn`/`.json` o selezione rapida tra gli **ultimi 10 progetti aperti**.
- **Schede di lavoro multiple**: Lavora su più elementi d'arredo contemporaneamente (fai doppio clic sul titolo di una scheda per rinominarlo).
- **Copia e incolla tra progetti**: Seleziona le parti in una scheda, premi `Ctrl+C`, passa a un'altra scheda (o a un'altra istanza di QuadFurn) e premi `Ctrl+V`. Eventuali materiali o bordi mancanti vengono creati o associati automaticamente nel progetto di destinazione.
- **Protezione contro modifiche non salvate**: Alla chiusura di una scheda modificata o all'uscita dal programma appare una finestra di conferma (`Salva ed esci`, `Esci senza salvare`, `Annulla`).

---

### 6. Esportazione del passaporto di produzione (PDF vettoriale, CSV, appunti)

Dalla scheda **Distinta base** (`📋 2. Distinta base`):
1. **`📄 Esporta in PDF` (Passaporto di produzione A4 Orizzontale vettoriale)**:
   - **Sezione 1**: Distinta base dettagliata (dimensioni finite e di taglio, bordi, orientamento venatura).
   - **Sezione 2**: Tavole grafiche del progettista a 4 viste (`Dall'alto XY`, `Assonometria 3D`, `Frontale XZ`, `Laterale YZ`) con ingombri massimi.
   - **Sezione 3**: Schemi completi di taglio a ghigliottina per ogni singola lastra.
   - **Sezione 4**: Riepilogo materiali e consumo lineare bordature (metri netti e $+15\%$ di sfrido).
   - Documento generato interamente nella **lingua attiva dell'interfaccia** con font TrueType/CID vettoriali incorporati.
2. **`💾 Esporta in CSV (Excel)`**: File foglio di calcolo UTF-8 con BOM e delimitatore `;`, pronto all'apertura diretta in Microsoft Excel o LibreOffice.
3. **`📋 Copia distinta di taglio`**: Copia negli appunti una tabella con separatori di tabulazione, ideale per essere incollata in software di taglio esterni (come Cutting 3).

---

### 7. Catalogo materiali, texture e bordature

Nella scheda **Materiali** (`🎨 4. Materiali`):
- Aggiungi o modifica i **Materiali delle lastre** (`Truciolare 18 mm`, `MDF 16 mm`, `HDF 3 mm`, ecc.) con colori RGB, dimensioni della lastra grezza (`L × W`) e vincolo di venatura predefinito.
- Carica texture superficiali in formato **`PNG`**, **`JPG`/`JPEG`** o **`WEBP`** per un rendering 3D fotorealistico.
- **Portabilità e ricerca intelligente delle texture**: Durante l'apertura di un file `.qfurn` su un altro computer o da una cartella spostata, QuadFurn cerca le texture:
  1. Incorporate nel file del progetto (Base64)
  2. Nella stessa cartella del file `.qfurn`
  3. In una sottocartella `./textures/` adiacente al file
  4. Tramite percorso relativo o percorso assoluto originale
- **Incorporazione texture nel file (`📦 Incorpora texture nel file di progetto`)**: Attiva l'opzione apposita nella barra superiore o nella scheda Materiali per salvare le immagini delle texture come stringhe Base64 all'interno del file `.qfurn`. Il file diventa completamente autonomo e trasferibile senza perdita di texture.
- **Protezione della memoria grafica**: File di dimensioni superiori a 60 MB vengono bloccati con avviso esplicito, mentre texture con risoluzione oltre 4096 px vengono ridimensionate automaticamente per preservare le prestazioni della GPU.
- Gestione dei **Tipi di bordatura** con spessore in millimetri e colore di evidenziazione. L'eliminazione di un materiale o bordo in uso apre una finestra di riassegnazione sicura.

---

### 8. Temi visivi e localizzazione in 22 lingue

- **Temi grafici**: Passa facilmente tra i temi **🌙 Scuro**, **☀ Chiaro** e **🖥 Sistema** dalla barra degli strumenti.
- **22 lingue integrate**:
  `Italiano`, `English`, `Русский`, `Українська`, `Deutsch`, `Polski`, `Español`, `Français`, `Português`, `Čeština`, `Slovenčina`, `Slovenščina`, `Nederlands`, `Dansk`, `Norsk`, `Svenska`, `Magyar`, `Română`, `Türkçe`, `简体中文`, `日本語` e `한국어`.
- **Rilevamento automatico della lingua di sistema**: Al primo avvio QuadFurn individua la lingua del sistema operativo e la imposta automaticamente. Le impostazioni vengono conservate in `%APPDATA%\QuadFurn\settings.json`.

---

## Controlli e scorciatoie da tastiera

| Scorciatoia | Azione |
| :--- | :--- |
| **`V`** | Attiva lo strumento **Seleziona e Sposta** (con 8 maniglie e riquadro di selezione) |
| **`B`** | Attiva lo strumento **Crea elemento con riquadro** nelle viste 2D |
| **`M`** | Attiva lo strumento **Metro a nastro** nelle viste 2D |
| **Trascina `Tasto sinistro` (2D)** | Sposta le parti selezionate, ridimensiona con maniglia o crea riquadro di selezione su spazio vuoto |
| **`Maiusc + Tasto sinistro` (2D)** | Aggiunge o rimuove una parte dalla selezione multipla |
| **Trascina `Tasto destro` / `Tasto centrale` (2D)** | Panoramica della vista 2D |
| **`Rotellina mouse` (2D / 3D)** | Zoom avanti e indietro rispetto al cursore |
| **Trascina `Tasto destro` / `Tasto centrale` (3D)** | Rotazione orbitale della telecamera 3D |
| **`Z` (sopra la vista 3D)** | Attiva / disattiva la **Modalità Volo Libero in Prima Persona (Noclip)** |
| **`W` `A` `S` `D` / `Spazio` / `Ctrl`** | Movimento avanti/sinistra/indietro/destra/alto/basso in volo 3D (`Maiusc` per accelerare) |
| **`Ctrl + S`** | Salva il mobile corrente (`.qfurn` / `.json`) |
| **`Ctrl + Z`** | Annulla l'ultima azione (cronologia fino a 40 passaggi) |
| **`Ctrl + C` / `Ctrl + V`** | Copia / Incolla le parti selezionate (funziona tra schede diverse) |
| **`Ctrl + D`** | Duplica le parti selezionate |
| **`Canc` / `Backspace`** | Elimina le parti selezionate |
| **`←` `→` `↑` `↓` (viste 2D)** | Sposta le parti selezionate del passo della griglia (`Shift` per raddoppiare) |

---

## Guida introduttiva

### File binari precompilati (GitHub Releases)
Per ogni release vengono generati pacchetti pronti all'uso tramite [GitHub Actions](https://github.com/rbm13s/QuadFurn/actions):
- **Windows (`x86_64`)**: `QuadFurn-windows-x86_64.zip` — eseguibile portatile `QuadFurn.exe` con icona incorporata.
- **Linux (`Debian 12+ / Ubuntu 22.04+ x86_64`)**:
  - `quadfurn_<versione>_amd64.deb` — pacchetto d'installazione Debian/Ubuntu (installa binario, voce di menu e icone):
    ```bash
    sudo apt install ./quadfurn_0.2.2_amd64.deb
    ```
  - `QuadFurn-linux-x86_64.tar.gz` — archivio portatile autonomo.
- **macOS (`Universale: Apple Silicon M1–M4 + Intel x86_64`)**: `QuadFurn-macos-universal.zip` — applicazione nativa `QuadFurn.app` con icona Retina.

---

### Compilazione dal codice sorgente

#### Requisiti preliminari
- [Rust Toolchain](https://rustup.rs/) (`stable`, edizione Rust 2021)
- Pacchetti di sistema su **Linux (Debian / Ubuntu)**:
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-dev libssl-dev \
    libgtk-3-dev libglib2.0-dev libatk1.0-dev libgdk-pixbuf-2.0-dev \
    libcairo2-dev libpango1.0-dev libgl1-mesa-dev fonts-dejavu-core fonts-noto-cjk
  ```
- **macOS**: Strumenti da riga di comando Xcode (`xcode-select --install`)
- **Windows**: Windows 10 / 11 con Visual C++ Build Tools

#### Comandi di compilazione ed esecuzione
```bash
# Esegui in modalità di sviluppo
cargo run

# Compila l'eseguibile ottimizzato per la distribuzione
cargo build --release

# Esegui tutti i test unitari
cargo test
```

---

## Impostazioni e formati di file

- **Configurazione utente (`settings.json`)**:
  Salvata automaticamente nella cartella standard di ciascun sistema operativo:
  - **Windows**: `%APPDATA%\QuadFurn\settings.json`
  - **Linux**: `~/.config/QuadFurn/settings.json`
  - **macOS**: `~/Library/Application Support/QuadFurn/settings.json`
  
  Memorizza:
  - `theme_choice` (`Dark`, `Light`, `System`)
  - `language` (`It`, `En`, `Ru`, ecc.)
  - `recent_projects` (fino a 10 progetti aperti di recente)
- **File di progetto (`.qfurn` / `.json`)**:
  File di testo JSON formattati in UTF-8 contenenti parti, gruppi, materiali, texture, bordature e parametri di taglio. Se l'incorporazione delle texture è abilitata, le immagini vengono memorizzate come stringhe Base64 all'interno del file stesso.

---

## Architettura del progetto

```text
QuadFurn/
├── .github/
│   └── workflows/
│       └── build.yml          # Pipeline CI/CD GitHub Actions (Linux .deb/.tar.gz, macOS .app, Windows .zip)
├── assets/
│   ├── icon.jpeg              # Logo dell'applicazione e icona sorgente ad alta risoluzione
│   └── screenshots/           # Schermate per la documentazione
├── docs/                      # Documentazione localizzata in 22 lingue
├── src/
│   ├── main.rs                # Punto d'ingresso, gestione schede multi-progetto, pagina iniziale e scorciatoie
│   ├── model.rs               # Modello dati (Project, PartBlock, SheetMaterial, EdgeBanding, OBB)
│   ├── viewport.rs            # Motore a 4 viste Hammer/CorelDRAW, rendering 2D e 3D wgpu/painter
│   ├── snapping.rs            # Snap magnetico alla griglia e ai bordi, assi di proiezione
│   ├── cutter.rs              # Motore di taglio lastre a ghigliottina e detrazione bordi
│   ├── export.rs              # Generatore di PDF vettoriali A4 Orizzontale localizzati ed esportatore CSV
│   ├── ui_panels.rs           # Barra CorelDRAW, Ispettore elementi, Distinta base, Taglio e Materiali
│   └── i18n.rs                # Tabelle di localizzazione per 22 lingue e autorilevamento lingua OS
├── build.rs                   # Generazione automatica dell'icona ICO per Windows e risorsa PE
├── Cargo.toml                 # Manifest e dipendenze del pacchetto
└── LICENSE                    # Licenza open-source MIT
```

---

## Licenza

Questo progetto è rilasciato sotto licenza [MIT](../LICENSE) — libero per uso personale o commerciale, modifica e redistribuzione.


