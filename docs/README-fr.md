<div align="center">

<img src="../assets/icon.jpeg" alt="QuadFurn CAD Logo" width="128" height="128" style="border-radius: 18px;" />

# QuadFurn CAD

**CAO pour meubles en panneaux et aggloméré — Concepteur spatial 4 vues + Optimiseur de découpe de panneaux par guillotine**

[![Build & Release](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml/badge.svg)](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml)
[![Downloads](https://img.shields.io/github/downloads/rbm13s/QuadFurn/total?logo=github&color=blue)](https://github.com/rbm13s/QuadFurn/releases)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe_0.29-blue)](https://github.com/emilk/egui)
[![Graphics](https://img.shields.io/badge/Renderer-wgpu-green)](https://wgpu.rs/)
[![Languages](https://img.shields.io/badge/Languages-22_Locales-purple)](#8-thèmes-visuels-et-localisation-en-22-langues)
[![Platform](https://img.shields.io/badge/Platform-Windows_%7C_Linux_%7C_macOS-0078D6)](#premiers-pas)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)

<br/>

**Languages / Языки / Мови / Sprachen:**<br/>
[English](../README.md) •
[Русский](README-ru.md) •
[Українська](README-uk.md) •
[Deutsch](README-de.md) •
[Polski](README-pl.md) •
[Español](README-es.md) •
**Français** •
[Italiano](README-it.md) •
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

## Sommaire

- [Introduction](#introduction)
- [Captures d'écran et visite visuelle](#captures-décran-et-visite-visuelle)
- [Fonctionnalités clés](#fonctionnalités-clés)
- [Documentation fonctionnelle](#documentation-fonctionnelle)
  - [1. Concepteur CAO 4 vues et contrôles de précision](#1-concepteur-cao-4-vues-et-contrôles-de-précision)
  - [2. Expressions arithmétiques dans les champs numériques](#2-expressions-arithmétiques-dans-les-champs-numériques)
  - [3. Placage de chants et déduction automatique de la cote brute de sciage](#3-placage-de-chants-et-déduction-automatique-de-la-cote-brute-de-sciage)
  - [4. Optimisation de découpe 2D par guillotine](#4-optimisation-de-découpe-2d-par-guillotine)
  - [5. Onglets multi-projets et presse-papiers inter-projets](#5-onglets-multi-projets-et-presse-papiers-inter-projets)
  - [6. Exportation du passeport technique (PDF vectoriel, CSV, presse-papiers)](#6-exportation-du-passeport-technique-pdf-vectoriel-csv-presse-papiers)
  - [7. Catalogue de panneaux, textures et chants](#7-catalogue-de-panneaux-textures-et-chants)
  - [8. Thèmes visuels et localisation en 22 langues](#8-thèmes-visuels-et-localisation-en-22-langues)
- [Contrôles et raccourcis clavier](#contrôles-et-raccourcis-clavier)
- [Premiers pas](#premiers-pas)
- [Paramètres et formats de fichiers](#paramètres-et-formats-de-fichiers)
- [Architecture du projet](#architecture-du-projet)
- [Licence](#licence)

---

## Introduction

**QuadFurn CAD** est une application CAO de bureau ultra-rapide et native écrite en Rust, conçue pour la conception de meubles en panneaux (mélaminé/panneaux de particules, MDF, HDF, contreplaqué), avec génération automatique de nomenclatures et optimisation de découpe de panneaux 2D par passes guillotine.

QuadFurn associe l'ergonomie de trois outils de référence en un flux de travail continu :
1. **Modélisation spatiale 4 vues** : Édition synchronisée sur trois projections orthogonales 2D (**Haut XY — Plan**, **Face XZ — Élévation**, **Côté YZ — Coupe**) avec ascenseurs, magnétisme d'arêtes et vue **3D Orbit / Vol libre première personne (`Z` Noclip)** en temps réel.
2. **Barre de transformation de précision** : Saisie numérique immédiate des coordonnées (`X`, `Y`, `Z`), dimensions hors-tout (`L × W × T`) et angle de rotation (`∠Z`), avec évaluation mathématique en ligne (par exemple `600-4` ou `720+18`).
3. **Découpe de panneaux et déduction des chants** : Déduction automatique de l'épaisseur du chant PVC/ABS sur les 4 côtés pour calculer les **cotes brutes de sciage**, couplée à un optimiseur de découpe 2D par guillotine respectant le sens du fil du bois, la largeur du trait de scie et les chutes réutilisables.

---

## Captures d'écran et visite visuelle

### 1. Concepteur CAO 4 vues (Thème Sombre)
Projections orthogonales synchronisées (**Haut XY**, **Face XZ**, **Côté YZ**) et **Vue 3D** texturée avec perspective correcte, gizmos de redimensionnement à 8 poignées et **Inspecteur de pièces** à droite.

![Concepteur CAO 4 vues](../assets/screenshots/1designer.png)

### 2. Nomenclatures et spécifications
Tableau complet listant les **dimensions finies (`L × W`)** et les **cotes de sciage nettes (`CUT L × W`)** après déduction des chants sur chaque rive (`L1`, `L2`, `W1`, `W2`).

![Tableau de spécifications](../assets/screenshots/2specs.png)

### 3. Plans de calepinage et découpe guillotine
Imbrication 2D sur panneaux standards (`2800 × 2070 mm`, etc.) avec découpes débouchantes guillotine, trait de coupe, marges de détourage, repères de chants et chutes réutilisables.

![Plan de découpe de panneaux](../assets/screenshots/3cutting.png)

### 4. Catalogue de matériaux et chants
Gestion des épaisseurs de panneaux, couleurs 3D, textures personnalisées (`PNG`, `JPG`, `WEBP`), sens du veinage et types de chants PVC/ABS.

![Catalogue de matériaux et chants](../assets/screenshots/4materials.png)

### 5. Support du Thème Clair
Thème clair moderne et contrasté aux côtés du Thème Sombre et de la synchronisation automatique avec le système Windows/macOS/Linux.

![Interface en Thème Clair](../assets/screenshots/5lighttheme.png)

### 6. 22 langues intégrées (Interface et export PDF)
Changement de langue instantané sans redémarrage via le sélecteur **`🌐`** dans la barre d'outils, détection automatique de la langue de l'OS au premier lancement et exportations PDF localisées.

![22 langues d'interface](../assets/screenshots/6languages.png)

---

## Fonctionnalités clés

- **Espace de travail 4 vues synchronisé** : Travaillez dans Haut (`XY`), Face (`XZ`), Côté (`YZ`), ou maximisez n'importe quelle vue en un clic (`[Agrandir]` / `[Restaurer 4 vues]`).
- **Prévisualisation 3D accélérée par GPU** : Caméra orbitale (`Bouton droit`/`Bouton central`) + mode vol libre à la première personne en vue première personne (`touche Z` + `ZQSD`/`Espace`/`Ctrl`) avec texturation UV et sélection précise par lancer de rayons OBB.
- **Magnétisme et cotation dynamique des jeux** : Accrochage automatique à la grille (`1`, `9`, `18`, `50 mm` ou personnalisée), collage magnétique aux faces et coins adjacents (`0 mm`), et affichage dynamique des distances aux pièces voisines.
- **Sélection par rectangle et transformation de groupe** : Tracez un cadre de sélection dans n'importe quelle vue 2D pour manipuler plusieurs pièces, les dupliquer (`Ctrl+D`) ou les copier/coller d'un meuble à un autre (`Ctrl+C` / `Ctrl+V`).
- **Calculatrice arithmétique intégrée** : Tous les champs numériques acceptent des formules telles que `600+100`, `800-36`, `720/2` ou `45+15`.
- **Déduction automatique des chants au sciage** : Attribuez des chants PVC/ABS (`0.4 mm`, `0.5 mm`, `1.0 mm`, `2.0 mm`, etc.) sur les 4 rives (`L1`, `L2`, `W1`, `W2`) et visualisez immédiatement la cote brute :
  $$\text{Longueur de sciage} = L_{\text{finie}} - E_{L1} - E_{L2}, \quad \text{Largeur de sciage} = W_{\text{finie}} - E_{W1} - E_{W2}$$
- **Moteur d'imbrication 2D par guillotine** : Calepinage multi-panneaux respectant l'orientation du fil du bois, l'épaisseur de lame de scie et le délignage d'équarrissage des 4 bords.
- **Passeport technique complet en PDF vectoriel A4 Paysage** : Exportation en un clic dans la langue active de l'interface :
  1. Tableau complet de la nomenclature des pièces
  2. Planches techniques 4 vues (`Haut XY`, `Vue 3D`, `Face XZ`, `Côté YZ`)
  3. Plans de découpe guillotine pour chaque panneau
  4. Récapitulatif des panneaux et métré linéaire des chants avec surplus de `+15%`

---

## Documentation fonctionnelle

### 1. Concepteur CAO 4 vues et contrôles de précision

L'onglet **Concepteur** (`📐 1. Concepteur`) comprend trois zones :

1. **Barres d'outils supérieures** :
   - **Outils d'édition** :
     - `↖ Sélectionner / Déplacer (V)` : Clic pour sélectionner, glisser pour déplacer, ou manipuler les 8 poignées de redimensionnement en 2D. Glisser sur le vide trace un cadre de sélection multiple.
     - `➕ Créer une pièce par rectangle (B)` : Tracer un rectangle dans une vue 2D (`Haut XY`, `Face XZ` ou `Côté YZ`) pour insérer un panneau orienté selon ce plan.
     - `📏 Mètre ruban (M)` : Mesurer les cotes exactes point à point (`ΔX`, `ΔY`, `L`) en millimètres.
   - **Insertion rapide** : Boutons d'ajout rapide (`+ Étagère (XY)`, `+ Montant (YZ)`, `+ Façade/Fond (XZ)`).
   - **Grille et magnétisme** : Pas de grille préréglés (`1`, `9`, `18`, `50 mm`), magnétisme aux arêtes des pièces voisines et affichage des jeux en millimètres.
   - **Cadrage de caméra** :
     - `🔍 Centrer la vue` : Recentrage sur l'origine/le centre du meuble.
     - `📐 Ajuster à la fenêtre` : Zoom automatique calculant l'encombrement global du meuble pour l'adapter à chaque fenêtre.
   - **Barre de transformation de précision** : Affichage et édition directe des coordonnées `X`, `Y`, `Z`, dimensions hors-tout `L`, `W`, `T`, badge vert de cote de sciage `[Découpe: L × W mm]` et rotation `∠Z`.

2. **Fenêtres 4 vues** :
   - **Vues 2D (`Haut XY`, `Face XZ`, `Côté YZ`)** avec ascenseurs interactifs, règles graduées en mm et surlignage couleur des chants plaqués.
   - **Vue 3D** : Rotation orbitale (`Clic droit` ou `Bouton central`), zoom molette, sélection 3D et **Mode Vol Libre Première Personne** (pressez **`Z`** au-dessus de la vue 3D pour voler avec `ZQSD`, `Espace`, `Ctrl`/`Maj`).

3. **Inspecteur de pièces latéral droit** :
   - Regroupement des pièces par **Groupes** personnalisés (*Caisson*, *Portes*, *Tiroirs*).
   - Contrôles de visibilité (`👁`) et de verrouillage (`🔒`).
   - Orientation, matériau, dimensions (`L × W` avec permutation `⇄`), verrouillage du fil (`🌲 Verrouiller le fil`), fil traversant (`🔄 Fil en travers`) et schéma interactif des 4 chants.

---

### 2. Expressions arithmétiques dans les champs numériques

Tous les champs numériques supportent les formules mathématiques :
- **Additions et soustractions** : Dans un champ valant `600`, écrivez `600+100` ou `600-4` et validez par `Entrée` — la valeur devient instantanément `700` ou `596`.
- **Multiplications, divisions et parenthèses** : Des expressions complexes comme `(800-36)/2` ou `18*2` sont évaluées automatiquement.
- Les séparateurs décimaux `.` et `,` sont acceptés, et les suffixes d'unités (`mm`, `°`) sont ignorés sans erreur.

---

### 3. Placage de chants et déduction automatique de la cote brute de sciage

- Chaque pièce possède 4 rives : **`L1`** (Longueur 1 / Avant), **`L2`** (Longueur 2 / Arrière), **`W1`** (Largeur 1 / Côté 1), et **`W2`** (Largeur 2 / Côté 2).
- Pour un panneau de `680 × 200 mm` recevant un chant PVC `2.0 mm` sur `L1` et `0.5 mm` sur `L2`, `W1`, `W2`, le logiciel calcule automatiquement la **Cote brute de débit** :
  - $\text{Découpe } L = 680 - 2.0 - 0.5 = 677.5\text{ mm}$
  - $\text{Découpe } W = 200 - 0.5 - 0.5 = 199.0\text{ mm}$
- Les chants sont matérialisés par des liserés de couleur distincts sur les vues 2D/3D et les plans de calepinage.

---

### 4. Optimisation de découpe 2D par guillotine

L'onglet **Plan de découpe** (`🪚 3. Plan de découpe`) optimise le calepinage de l'ensemble des panneaux :
- **Paramètres de sciage réglables** :
  - **Épaisseur du trait de scie** (par défaut `4.0 mm`).
  - **Délignage d'équarrissage des bords** (`Gauche`, `Droite`, `Haut`, `Bas`, par défaut `10 mm`).
- **Respect du veinage du bois** :
  - Pièces à **Fil verrouillé (`🌲`)** : maintenues impérativement parallèles à la longueur du panneau d'origine.
  - Pièces à **Fil libre** : pivotables à $90^\circ$ si cela optimise l'imbrication et réduit les chutes.
- **Cartographie des coupes et chutes réutilisables** :
  - Représentation vectorielle de chaque panneau avec traits de coupe guillotine en rouge, numérotation des pièces, cotes de sciage, bandes de chants et chutes réutilisables cotées.
  - Statistiques complètes : nombre de panneaux requis, rendement surfacique (%), métré linéaire de sciage et de chants.

---

### 5. Onglets multi-projets et presse-papiers inter-projets

- **Page d'accueil et projets récents** : Création d'un meuble vierge, départ depuis un modèle de meuble bas à 2 portes, ouverture de fichiers `.qfurn`/`.json` ou réouverture rapide parmi les **10 projets récents**.
- **Onglets multi-meubles** : Travaillez sur plusieurs modules ou caissons simultanément (double-cliquez sur un onglet pour le renommer).
- **Copier-coller inter-projets** : Sélectionnez des pièces dans un meuble, faites `Ctrl+C`, basculez sur un autre onglet (ou une autre instance de l'application), et faites `Ctrl+V`. Les matériaux et chants manquants sont automatiquement créés dans le projet cible.
- **Protection contre la perte de données** : Confirmation automatique lors de la fermeture d'un onglet non enregistré ou lors de la sortie de l'application (`Enregistrer et quitter`, `Quitter sans enregistrer`, `Annuler`).

---

### 6. Exportation du passeport technique (PDF vectoriel, CSV, presse-papiers)

Depuis l'onglet **Spécification** (`📋 2. Spécification`) :
1. **`📄 Exporter en PDF` (Passeport de fabrication A4 Paysage)** :
   - **Section 1** : Tableau de nomenclature complet (cotes finies et de sciage, chants, fil du bois).
   - **Section 2** : Planches techniques 4 vues (`Haut XY`, `3D Isométrique`, `Face XZ`, `Côté YZ`) avec encombrement global.
   - **Section 3** : Plans de calepinage guillotine détaillés pour chaque panneau utilisé.
   - **Section 4** : Récapitulatif des matériaux et métrés de chants (longueur nette et $+15\%$ de sécurité).
   - Document généré dans la **langue actuellement active** avec polices vectorielles intégrées (`Identity-H`).
2. **`💾 Exporter en CSV (Excel)`** : Fichier tableur encodé en UTF-8 avec BOM et séparateur `;`, directement exploitable sous Excel ou LibreOffice Calc.
3. **`📋 Copier la liste de débit`** : Copie dans le presse-papiers des cotes de sciage prêtes à coller dans des logiciels de découpe externes ou tableurs.

---

### 7. Catalogue de panneaux, textures et chants

Dans l'onglet **Matériaux** (`🎨 4. Matériaux`) :
- Définissez vos **Matériaux de panneaux** (`Mélaminé 18 mm`, `MDF 16 mm`, `HDF 3 mm`, etc.) avec couleurs RGB, dimensions de panneau brutes (`L × W`) et verrouillage de fil par défaut.
- Chargez des textures au format **`PNG`**, **`JPG`/`JPEG`** ou **`WEBP`** pour un rendu 3D ultra-réaliste.
- **Portabilité et recherche de textures** : QuadFurn recherche automatiquement les images :
  1. Embarquées dans le fichier projet (Base64)
  2. Dans le dossier du fichier projet `.qfurn`
  3. Dans un sous-dossier `./textures/` adjacent
  4. Via chemin relatif ou chemin absolu d'origine
- **Intégration autonome (`📦 Incorporer les textures dans le projet`)** : Activez l'option d'empaquetage dans la barre supérieure ou dans le gestionnaire de matériaux pour enregistrer les textures sous forme Base64 directement au sein du fichier `.qfurn`. Le fichier devient totalement autonome et déplaçable sans risque de textures manquantes.
- **Sécurité GPU** : Les images supérieures à 60 Mo sont bloquées avec avertissement, et les résolutions très élevées (> 4096 px) sont redimensionnées automatiquement.
- Gestion des **Types de chants** avec épaisseurs en mm et couleurs repères. La suppression d'un matériau ou d'un chant en cours d'utilisation déclenche une confirmation sécurisée pour réassigner proprement les pièces.

---

### 8. Thèmes visuels et localisation en 22 langues

- **Thèmes de couleurs** : Basculez entre **🌙 Sombre**, **☀ Clair** et **🖥 Système** depuis la barre supérieure.
- **22 langues intégrées** :
  `Français`, `English`, `Русский`, `Українська`, `Deutsch`, `Polski`, `Español`, `Italiano`, `Português`, `Čeština`, `Slovenčina`, `Slovenščina`, `Nederlands`, `Dansk`, `Norsk`, `Svenska`, `Magyar`, `Română`, `Türkçe`, `简体中文`, `日本語` et `한국어`.
- **Détection automatique au premier lancement** : Détecte la langue de votre système d'exploitation et l'applique immédiatement. Vos préférences sont conservées dans `%APPDATA%\QuadFurn\settings.json`.

---

## Contrôles et raccourcis clavier

| Raccourci | Action |
| :--- | :--- |
| **`V`** | Activer l'outil **Sélectionner et Déplacer** (avec 8 poignées et cadre de sélection) |
| **`B`** | Activer l'outil **Créer une pièce par rectangle** dans les vues 2D |
| **`M`** | Activer le **Mètre ruban** dans les vues 2D |
| **Glisser `Clic gauche` (2D)** | Déplacer la pièce, ajuster une poignée ou tracer un cadre de sélection dans le vide |
| **`Maj + Clic gauche` (2D)** | Ajouter / retirer une pièce de la sélection multiple |
| **Glisser `Clic droit` / `Clic central` (2D)** | Déplacer la vue (Panoramique 2D) |
| **`Molette souris` (2D / 3D)** | Zoom avant / arrière centré sur le curseur |
| **Glisser `Clic droit` / `Clic central` (3D)** | Rotation orbitale de la caméra 3D |
| **`Z` (au-dessus de la vue 3D)** | Activer / désactiver le **Mode Vol Libre Première Personne (Noclip)** |
| **`Z` `Q` `S` `D` / `Espace` / `Ctrl`** | Voler en avant/gauche/arrière/droite/haut/bas en vue 3D (`Maj` pour accélérer) |
| **`Ctrl + S`** | Enregistrer le meuble actif (`.qfurn` / `.json`) |
| **`Ctrl + Z`** | Annuler la dernière action (historique jusqu'à 40 étapes) |
| **`Ctrl + C` / `Ctrl + V`** | Copier / Coller les pièces sélectionnées (fonctionne entre onglets) |
| **`Ctrl + D`** | Dupliquer les pièces sélectionnées |
| **`Suppr` / `Retour arrière`** | Supprimer les pièces sélectionnées |
| **`←` `→` `↑` `↓` (vues 2D)** | Déplacer les pièces sélectionnées du pas de grille choisi (`Shift` pour doubler le pas) |

---

## Premiers pas

### Binaires précompilés (GitHub Releases)
Chaque version produit automatiquement des paquets exécutables via [GitHub Actions](https://github.com/rbm13s/QuadFurn/actions) :
- **Windows (`x86_64`)** : `QuadFurn-windows-x86_64.zip` — application portable `QuadFurn.exe` avec icône intégrée.
- **Linux (`Debian 12+ / Ubuntu 22.04+ x86_64`)** :
  - `quadfurn_<version>_amd64.deb` — paquet d'installation officiel Debian/Ubuntu (installe l'exécutable, l'entrée de menu et les icônes de bureau) :
    ```bash
    sudo apt install ./quadfurn_0.2.2_amd64.deb
    ```
  - `QuadFurn-linux-x86_64.tar.gz` — archive Linux autonome portable.
- **macOS (`Universel : Apple Silicon M1–M4 + Intel x86_64`)** : `QuadFurn-macos-universal.zip` — bundle `QuadFurn.app` natif avec icône Retina.

---

### Compilation à partir des sources

#### Prérequis
- [Rust Toolchain](https://rustup.rs/) (`stable`, édition Rust 2021)
- Bibliothèques système sous **Linux (Debian / Ubuntu)** :
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-dev libssl-dev \
    libgtk-3-dev libglib2.0-dev libatk1.0-dev libgdk-pixbuf-2.0-dev \
    libcairo2-dev libpango1.0-dev libgl1-mesa-dev fonts-dejavu-core fonts-noto-cjk
  ```
- **macOS** : Outils de ligne de commande Xcode (`xcode-select --install`)
- **Windows** : Windows 10 / 11 avec les outils de génération MSVC C++

#### Commandes de compilation
```bash
# Lancer en mode développement
cargo run

# Compiler l'exécutable optimisé pour la production
cargo build --release

# Lancer la suite de tests unitaires
cargo test
```

---

## Paramètres et formats de fichiers

- **Configuration utilisateur (`settings.json`)** :
  Enregistré automatiquement dans le dossier standard du système d'exploitation :
  - **Windows** : `%APPDATA%\QuadFurn\settings.json`
  - **Linux** : `~/.config/QuadFurn/settings.json`
  - **macOS** : `~/Library/Application Support/QuadFurn/settings.json`
  
  Mémorise :
  - `theme_choice` (`Dark`, `Light`, `System`)
  - `language` (`Fr`, `En`, `Ru`, etc.)
  - `recent_projects` (jusqu'à 10 derniers fichiers ouverts)
- **Fichiers de projet (`.qfurn` / `.json`)** :
  Fichiers texte JSON lisibles en UTF-8 contenant la liste des pièces, groupes, matériaux, textures, chants et paramètres de calepinage. Si l'option d'incorporation des textures est cochée, les images sont stockées sous forme Base64 au sein du fichier.

---

## Architecture du projet

```text
QuadFurn/
├── .github/
│   └── workflows/
│       └── build.yml          # CI/CD GitHub Actions (Linux .deb/.tar.gz, macOS .app, Windows .zip)
├── assets/
│   ├── icon.jpeg              # Logo et icône source multi-résolutions
│   └── screenshots/           # Captures d'écran pour la documentation
├── docs/                      # Documentation localisée en 22 langues
├── src/
│   ├── main.rs                # Point d'entrée, gestionnaire multi-onglets, page d'accueil et raccourcis
│   ├── model.rs               # Modèle de données (Project, PartBlock, SheetMaterial, EdgeBanding, OBB)
│   ├── viewport.rs            # Contrôleur 4 vues, rendu 2D et 3D wgpu/painter
│   ├── snapping.rs            # Magnétisme aux arêtes et à la grille, axes de projection
│   ├── cutter.rs              # Moteur d'imbrication 2D et déduction des chants au sciage
│   ├── export.rs              # Générateur de PDF vectoriel A4 Paysage localisé et export CSV
│   ├── ui_panels.rs           # Barre de transformation, Inspecteur de pièces, Spécification, Calepinage et Matériaux
│   └── i18n.rs                # Dictionnaires de traduction pour 22 langues et détection de l'OS
├── build.rs                   # Conversion automatique d'icône Windows ICO et ressource PE
├── Cargo.toml                 # Dépendances et configuration du paquet
└── LICENSE                    # Licence MIT
```

---

## Licence

Ce projet est distribué sous la licence [MIT](../LICENSE) — libre d'utilisation, de modification et de distribution pour des projets personnels ou commerciaux.


