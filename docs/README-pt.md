<div align="center">

<img src="../assets/icon.jpeg" alt="QuadFurn CAD Logo" width="128" height="128" style="border-radius: 18px;" />

# QuadFurn CAD

**CAD para móveis em painéis e aglomerado — Modelador espacial de 4 vistas + Otimizador de corte de chapas por guilhotina**

[![Build & Release](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml/badge.svg)](https://github.com/rbm13s/QuadFurn/actions/workflows/build.yml)
[![Downloads](https://img.shields.io/github/downloads/rbm13s/QuadFurn/total?logo=github&color=blue)](https://github.com/rbm13s/QuadFurn/releases)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui_%2F_eframe_0.29-blue)](https://github.com/emilk/egui)
[![Graphics](https://img.shields.io/badge/Renderer-wgpu-green)](https://wgpu.rs/)
[![Languages](https://img.shields.io/badge/Languages-37_Locales-purple)](#8-temas-visuais-e-localização-em-22-idiomas)
[![Platform](https://img.shields.io/badge/Platform-Windows_%7C_Linux_%7C_macOS-0078D6)](#primeiros-passos)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)

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
**Português** •
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

## Índice

- [Introdução](#introdução)
- [Capturas de ecrã e apresentação visual](#capturas-de-ecrã-e-apresentação-visual)
- [Principais funcionalidades](#principais-funcionalidades)
- [Documentação funcional](#documentação-funcional)
  - [1. Modelador CAD de 4 vistas e controlos de precisão](#1-modelador-cad-de-4-vistas-e-controlos-de-precisão)
  - [2. Expressões aritméticas em campos numéricos](#2-expressões-aritméticas-em-campos-numéricos)
  - [3. Fita de bordo e dedução automática da medida de corte](#3-fita-de-bordo-e-dedução-automática-da-medida-de-corte)
  - [4. Otimização de corte de chapas 2D por guilhotina](#4-otimização-de-corte-de-chapas-2d-por-guilhotina)
  - [5. Separadores de projetos e área de transferência entre projetos](#5-separadores-de-projetos-e-área-de-transferência-entre-projetos)
  - [6. Exportação do passaporte técnico (PDF vetorial, CSV, área de transferência)](#6-exportação-do-passaporte-técnico-pdf-vetorial-csv-área-de-transferência)
  - [7. Catálogo de chapas, texturas e fitas de bordo](#7-catálogo-de-chapas-texturas-e-fitas-de-bordo)
  - [8. Temas visuais e localização em 37 idiomas](#8-temas-visuais-e-localização-em-22-idiomas)
- [Controlos e atalhos de teclado](#controlos-e-atalhos-de-teclado)
- [Primeiros passos](#primeiros-passos)
- [Definições e formatos de ficheiros](#definições-e-formatos-de-ficheiros)
- [Arquitetura do projeto](#arquitetura-do-projeto)
- [Licença](#licença)

---

## Introdução

O **QuadFurn CAD** é uma aplicação CAD multiplataforma de alto desempenho escrita em Rust, disponível tanto como aplicação desktop nativa rápida (Windows, Linux, macOS) como versão online WebAssembly (WASM) sem instalação executada diretamente nos navegadores modernos. Foi desenvolvida para o desenho de mobiliário em painéis (MDP/MDF melamínico, contraplacado, HDF), geração automática de lista de corte e especificações, e otimização de corte 2D por guilhotina.

O QuadFurn reúne a ergonomia de três ferramentas consagradas num único fluxo de trabalho:
1. **Modelação espacial em 4 vistas**: Edição simultânea em três projeções ortogonais 2D (**Planta XY**, **Elevação Frontal XZ**, **Corte Lateral YZ**) com barras de rolagem, atração magnética às arestas, botão de equalização de vistas (`⊞ Equalizar vistas`), proteção inteligente da espessura para peças de topo e uma vista **3D Órbita / Voo livre em primeira persona (`Z` Noclip)** em tempo real.
2. **Barra de transformação de precisão**: Entrada numérica rápida de coordenadas (`X`, `Y`, `Z`), dimensões totais (`L × W × T`) e rotação horizontal (`∠Z`), com cálculo matemático direto nos próprios campos de texto (por exemplo, digitando `600-4` ou `720+18`).
3. **Corte de chapas e dedução automática de orlas**: Subtração automática da espessura da fita de bordo em PVC/ABS dos 4 lados da peça para calcular as **dimensões exatas de corte bruto na serra**, em conjunto com um otimizador 2D por passagens guilhotina que respeita o sentido do veio da madeira, a espessura da serra e os refilos de fábrica.

---

## Capturas de ecrã e apresentação visual

### 1. Modelador CAD de 4 vistas (Tema Escuro)
Projeções ortogonais sincronizadas (**Planta XY**, **Elevação XZ**, **Corte YZ**) e **Vista 3D** com texturas com correção de perspetiva, manipuladores de 8 pontos de redimensionamento e **Inspetor de peças** à direita.

![Modelador CAD de 4 vistas](screenshots/pt/1designer.png)

### 2. Lista de peças (Especificações) e corte
Tabela completa exibindo tanto as **Dimensões finais da peça (`L × W`)** quanto as **Medidas de corte na serra (`CUT L × W`)** após dedução das fitas de bordo em cada face (`L1`, `L2`, `W1`, `W2`).

![Tabela de especificações](screenshots/pt/2specs.png)

### 3. Plano de corte de chapas por guilhotina
Otimização em chapas padrão (`2800 × 2070 mm`, etc.) com cortes retos de guilhotina de ponta a ponta, compensação da serra, margens de refilo, marcas visuais de fitas de bordo e sobras aproveitáveis.

![Plano de corte de chapas](screenshots/pt/3cutting.png)

### 4. Catálogo de materiais e fitas de bordo
Gestão de espessuras de placas, cores 3D, texturas personalizadas (`PNG`, `JPG`, `WEBP`), restrições de veios e tipos de fitas de bordo em PVC/ABS.

![Catálogo de materiais e fitas de bordo](screenshots/pt/4materials.png)

### 5. Suporte para Tema Claro
Tema claro nítido e de alto contraste a par do Tema Escuro e com deteção automática do tema do sistema operativo.

![Interface em Tema Claro](screenshots/pt/5lighttheme.png)

### 6. 37 idiomas integrados (Interface e exportação PDF)
Troca instantânea de idioma sem reiniciar através do seletor **`🌐`** na barra superior, deteção automática do idioma do sistema operativo no primeiro arranque e relatórios PDF totalmente traduzidos.

![37 idiomas disponíveis](screenshots/pt/6languages.png)

---

## Principais funcionalidades

- **Espaço de trabalho de 4 vistas sincronizado**: Alterne ou edite livremente em Planta (`XY`), Frontal (`XZ`), Lateral (`YZ`), ou expanda qualquer janela com um clique (`[Maximizar]` / `[Restaurar 4 vistas]`).
- **Pré-visualização 3D acelerada por hardware**: Câmara orbital (`Botão direito`/`Botão central`) + voo livre em primeira pessoa em primeira pessoa (`tecla Z` + `WASD`/`Espaço`/`Ctrl`) com mapeamento de texturas e seleção precisa por raycasting OBB.
- **Encaixe magnético e cotas dinâmicas de folga**: Atração automática à grelha (`1`, `9`, `18`, `50 mm` ou personalizada), encaixe magnético a faces e cantos adjacentes (`0 mm`) e medição dinâmica da distância entre peças vizinhas.
- **Seleção por caixa e transformação em grupo**: Arraste um retângulo de seleção em qualquer vista 2D para mover várias peças juntas, duplicá-las (`Ctrl+D`) ou copiá-las e colá-las entre móveis diferentes (`Ctrl+C` / `Ctrl+V`).
- **Calculadora aritmética integrada**: Todos os campos numéricos aceitam fórmulas como `600+100`, `800-36`, `720/2` ou `45+15`.
- **Dedução automática da fita de bordo no corte**: Defina fitas de bordo em PVC/ABS (`0.4 mm`, `0.5 mm`, `1.0 mm`, `2.0 mm`, etc.) nos 4 lados (`L1`, `L2`, `W1`, `W2`) e visualize a dimensão exata de corte:
  $$\text{Comprimento de corte} = L_{\text{final}} - E_{L1} - E_{L2}, \quad \text{Largura de corte} = W_{\text{final}} - E_{W1} - E_{W2}$$
- **Motor de encaixe 2D por guilhotina**: Algoritmo de corte para múltiplas chapas que respeita o sentido do veio, espessura da lâmina de corte e refilos perimétricos nos 4 lados.
- **Passaporte técnico completo em PDF vetorial A4 Horizontal**: Exportação com um clique no idioma ativo contendo:
  1. Tabela completa de especificação de peças
  2. Pranchas de desenho técnico de 4 vistas (`Planta XY`, `Isometria 3D`, `Frontal XZ`, `Lateral YZ`)
  3. Planos de corte por guilhotina de cada chapa utilizada
  4. Resumo de chapas e cálculo de fita de bordo linear com folga de `+15%`

---

## Documentação funcional

### 1. Modelador CAD de 4 vistas e controlos de precisão

O separador **Modelador** (`📐 1. Modelador`) divide-se em três áreas fundamentais:

1. **Barras de ferramentas superiores**:
   - **Ferramentas de desenho**:
     - `↖ Selecionar / Mover (V)`: Clique numa peça para selecioná-la, arraste para movê-la, ou use os manipuladores (8 pegas em vista frontal e 2 em vista de topo). Arraste no espaço vazio para abrir uma caixa de seleção múltipla. Ao arrastar em 2D: segure **`Shift`** para restringir o movimento ao eixo horizontal ou vertical, **`Ctrl` (`Cmd`)** para desativar o encaixe magnético e **`Alt`** para duplicar a peça na posição de destino.
     - `➕ Criar peça por retângulo (B)`: Desenhe um retângulo numa vista 2D (`Planta XY`, `Frontal XZ` ou `Lateral YZ`) para inserir um painel correspondente àquele plano.
     - `📏 Fita métrica (M)`: Meça distâncias precisas ponto a ponto (`ΔX`, `ΔY`, `L`) em milímetros.
   - **Inserção rápida de peças**: Botões para adicionar com um clique prateleiras (`+ Prateleira (XY)`), laterais (`+ Lateral (YZ)`) ou portas/fundos (`+ Porta/Fundo (XZ)`).
   - **Grelha e alinhamento**: Escolha passos de grelha predefinidos (`1`, `9`, `18`, `50 mm`), encaixe magnético a faces vizinhas e cotas visíveis de espaçamento.
   - **Enquadramento da câmara**:
     - `🔍 Centrar vista`: Centraliza a câmara na origem/centro do móvel.
     - `📐 Ajustar à janela`: Calcula as dimensões exteriores do móvel e enquadra perfeitamente as 4 vistas.
   - **Barra de transformação de precisão**: Entrada e alteração rápida de coordenadas `X`, `Y`, `Z`, dimensões `L`, `W`, `T`, etiqueta verde de corte `[Corte: L × W mm]` e ângulo `∠Z`.

2. **Janelas de 4 vistas**:
   - **Vistas ortogonais 2D (`Planta XY`, `Frontal XZ`, `Lateral YZ`)** com barras de rolagem dinâmicas, réguas milimétricas e linhas coloridas indicando as fitas de bordo aplicadas.
   - **Vista 3D**: Rotação orbital (`Botão direito` ou `Botão central`), zoom com roda do rato, seleção 3D e **Modo Voo Livre em Primeira Pessoa** (prima **`Z`** com o rato sobre a janela 3D para voar com `WASD`, `Espaço`, `Ctrl`/`Shift`).

3. **Inspetor de peças lateral direito**:
   - Organização de peças em **Grupos** (*Estrutura*, *Portas*, *Gavetas*).
   - Comandos de visibilidade (`👁`) e bloqueio contra edição (`🔒`).
   - Material, orientação, dimensões (`L × W` com botão de inversão `⇄`), sentido do veio (`🌲 Fixar veio`), veio transversal (`🔄 Veio na largura`) e esquema visual dos 4 lados de fita de bordo.

---

### 2. Expressões aritméticas em campos numéricos

Todos os campos numéricos da aplicação aceitam fórmulas aritméticas:
- **Somas e subtrações**: Num campo com `600`, digite `600+100` ou `600-4` e prima `Enter` — o valor converter-se-á imediatamente em `700` ou `596`.
- **Multiplicações, divisões e parêntesis**: Fórmulas como `(800-36)/2` ou `18*2` são calculadas automaticamente.
- Tanto o ponto `.` quanto a vírgula `,` funcionam como separadores decimais, e sufixos de unidades (`mm`, `°`) são desconsiderados sem causar erros.

---

### 3. Fita de bordo e dedução automática da medida de corte

- Cada peça retangular tem 4 lados: **`L1`** (Comprimento 1 / Frente), **`L2`** (Comprimento 2 / Traseira), **`W1`** (Largura 1 / Topo 1) e **`W2`** (Largura 2 / Topo 2).
- Ao aplicar fita PVC de `2.0 mm` em `L1` e `0.5 mm` em `L2`, `W1`, `W2` numa peça de `680 × 200 mm`, o sistema calcula automaticamente a **Medida de corte bruta**:
  - $\text{Corte } L = 680 - 2.0 - 0.5 = 677.5\text{ mm}$
  - $\text{Corte } W = 200 - 0.5 - 0.5 = 199.0\text{ mm}$
- As fitas de bordo são assinaladas com barras coloridas nas vistas de desenho e nos esquemas de corte de chapa.

---

### 4. Otimização de corte de chapas 2D por guilhotina

O separador **Plano de corte** (`🪚 3. Plano de corte`) calcula a melhor distribuição de peças nas chapas:
- **Parâmetros de corte ajustáveis**:
  - **Espessura do corte da serra** (padrão `4.0 mm`).
  - **Margens de refilo da chapa** (`Esquerda`, `Direita`, `Superior`, `Inferior`, padrão `10 mm`) para aparar imperfeições de fábrica.
- **Respeito pelo veio da madeira**:
  - Peças com **Veio bloqueado (`🌲`)**: mantidas sempre paralelas ao comprimento da chapa (`Comprimento L`).
  - Peças com **Veio livre**: podem ser giradas em $90^\circ$ pelo otimizador se isso aumentar o aproveitamento da chapa.
- **Mapa visual de corte e sobras úteis**:
  - Exibição de cada chapa com linhas de corte vermelhas, numeração de peças, medidas de corte, fitas de bordo e sobras aproveitáveis cotadas.
  - Relatório detalhado: número de chapas necessárias, rendimento em percentagem (%), metros lineares de corte e consumo de fitas de bordo.

---

### 5. Separadores de projetos e área de transferência entre projetos

- **Ecrã inicial e projetos recentes**: Crie um móvel em branco, inicie a partir do modelo de armário de 2 portas, abra ficheiros `.qfurn`/`.json` ou aceda a qualquer um dos **10 projetos mais recentes**.
- **Múltiplos separadores de móveis**: Desenhe vários móveis em simultâneo (clique duas vezes no nome do separador para renomeá-lo).
- **Copiar e colar entre projetos**: Selecione peças num separador, prima `Ctrl+C`, mude para outro separador (ou outra janela aberta do QuadFurn) e prima `Ctrl+V`. Quaisquer materiais ou fitas de bordo em falta são criados automaticamente no projeto de destino.
- **Proteção contra perdas acidentais**: Ao fechar um separador alterado ou fechar a aplicação, uma janela de aviso pergunta (`Guardar e sair`, `Sair sem guardar`, `Cancelar`).

---

### 6. Exportação do passaporte técnico (PDF vetorial, CSV, área de transferência)

No separador **Especificação** (`📋 2. Especificação`):
1. **`📄 Exportar para PDF` (Passaporte técnico vetorial A4 Horizontal)**:
   - **Secção 1**: Lista de peças com dimensões finais e de corte, fitas de bordo e indicação de veio.
   - **Secção 2**: Desenhos técnicos de 4 vistas (`Planta XY`, `Isometria 3D`, `Frontal XZ`, `Lateral YZ`) com cotas totais.
   - **Secção 3**: Esquemas de corte detalhados chapa a chapa.
   - **Secção 4**: Relação de chapas e metragem de fitas de bordo (metros úteis e $+15\%$ de desperdício).
   - Documento produzido no **idioma atualmente ativo** com fontes vetoriais TrueType/CID incorporadas.
2. **`💾 Exportar para CSV (Excel)`**: Ficheiro de folha de cálculo UTF-8 com BOM e delimitador `;`, pronto para o Excel ou LibreOffice.
3. **`📋 Copiar lista de corte`**: Copia a tabela com delimitadores de tabulação para a área de transferência, perfeita para importar em programas de corte externos ou folhas de cálculo.

---

### 7. Catálogo de chapas, texturas e fitas de bordo

No separador **Materiais** (`🎨 4. Materiais`):
- Registe ou edite **Materiais de chapas** (`Aglomerado 18 mm`, `MDF 16 mm`, `HDF 3 mm`, etc.) com cores RGB, dimensões da chapa bruta (`L × W`) e sentido padrão de veio.
- Carregue texturas de superfícies em formato **`PNG`**, **`JPG`/`JPEG`** ou **`WEBP`** para visualização 3D fidedigna.
- **Portabilidade e procura automática de texturas**: Ao reabrir um ficheiro `.qfurn` noutro computador ou pasta, o QuadFurn localiza as imagens:
  1. Guardadas dentro do ficheiro de projeto (Base64)
  2. Na mesma pasta do ficheiro de projeto `.qfurn`
  3. Numa subpasta `./textures/` junto ao projeto
  4. Por caminho relativo ou caminho absoluto original
- **Empacotamento de texturas (`📦 Embutir texturas no ficheiro de projeto`)**: Ative esta opção na barra superior ou no painel de Materiais para guardar as imagens diretamente em Base64 no ficheiro `.qfurn`. O projeto torna-se autónomo e pode ser enviado para qualquer lugar sem risco de perder texturas.
- **Proteção da placa gráfica**: Imagens com mais de 60 MB são recusadas com mensagem clara, e resoluções muito altas (> 4096 px) são redimensionadas para manter o desempenho fluido.
- Gestão de **Fitas de bordo** com espessuras em milímetros e cores de identificação. A eliminação de materiais ou fitas em uso solicita confirmação prévia para proteger o modelo.

---

### 8. Temas visuais e localização em 37 idiomas

- **Temas visuais**: Alterne entre **🌙 Escuro**, **☀ Claro** e **🖥 Sistema** na barra de ferramentas.
- **37 idiomas disponíveis**:
  `Português`, `English`, `Русский`, `Українська`, `Deutsch`, `Polski`, `Español`, `Français`, `Italiano`, `Čeština`, `Slovenčina`, `Slovenščina`, `Nederlands`, `Dansk`, `Norsk`, `Svenska`, `Magyar`, `Română`, `Türkçe`, `简体中文`, `日本語` e `한국어`.
- **Deteção automática no primeiro arranque**: Identifica o idioma do seu sistema operativo e configura-o por defeito. As preferências ficam gravadas em `%APPDATA%\QuadFurn\settings.json`.

---

## Controlos e atalhos de teclado

| Atalho | Ação |
| :--- | :--- |
| **`V`** | Ativar ferramenta **Selecionar e Mover** (8 pegas em vista frontal e 2 em vista de topo, mais seleção por caixa) |
| **`B`** | Ativar ferramenta **Criar peça por retângulo** nas vistas 2D |
| **`M`** | Ativar ferramenta **Fita métrica** nas vistas 2D |
| **Arrastar `Botão esquerdo` (2D)** | Mover peças selecionadas, redimensionar ou abrir caixa de seleção múltipla em espaço vazio |
| **`Shift + Botão esq.` Arrastar (2D)** | Restringir o movimento estritamente ao eixo horizontal ou vertical |
| **`Ctrl / Cmd + Botão esq.` Arrastar (2D)** | Desativar o encaixe magnético às bordas durante o arrasto |
| **`Alt + Botão esq.` Arrastar (2D)** | Duplicar a(s) peça(s) selecionada(s) na posição de destino ao soltar |
| **`Shift / Ctrl (Cmd) + Botão esquerdo` (2D)** | Adicionar ou remover peças da seleção |
| **Arrastar `Botão direito` / `Botão central` (2D)** | Mover a vista (Panorâmica 2D) |
| **`Roda do rato` (2D / 3D)** | Zoom de aproximação/afastamento centrado no cursor |
| **Arrastar `Botão direito` / `Botão central` (3D)** | Rotação orbital da câmara 3D |
| **`Z` (com o cursor na vista 3D)** | Ativar / desativar **Modo Voo Livre em Primeira Pessoa (Noclip)** |
| **`W` `A` `S` `D` / `Espaço` / `Ctrl (Cmd)`** | Movimento para a frente/esquerda/trás/direita/cima/baixo em voo 3D (`Shift` acelera) |
| **`Ctrl + S` / `Cmd (⌘) + S`** | Guardar o móvel atual (`.qfurn` / `.json`) |
| **`Ctrl + Z` / `Cmd (⌘) + Z`** | Desfazer a última ação (histórico até 40 passos) |
| **`Ctrl + C` / `Cmd (⌘) + C`** | Copiar peças selecionadas para a área de transferência |
| **`Ctrl + V` / `Cmd (⌘) + V`** | Colar peças (funciona entre diferentes separadores) |
| **`Ctrl + D` / `Cmd (⌘) + D`** | Duplicar peças selecionadas |
| **`Delete` / `Backspace`** | Eliminar peças selecionadas |
| **`←` `→` `↑` `↓` (vistas 2D)** | Mover as peças selecionadas pelo passo da grade (`Shift` para duplicar) |

---

## Primeiros passos

### Ficheiros binários pré-compilados (GitHub Releases)
Cada versão gera executáveis prontos para uso através do [GitHub Actions](https://github.com/rbm13s/QuadFurn/actions):
- **Windows (`x86_64`)**: `QuadFurn-windows-x86_64.zip` — executável portátil `QuadFurn.exe` com ícone embutido.
- **Linux (`Debian 12+ / Ubuntu 22.04+ x86_64`)**:
  - `quadfurn_<versão>_amd64.deb` — pacote oficial para Debian/Ubuntu (instala o executável, entrada de menu e ícones do sistema):
    ```bash
    sudo apt install ./quadfurn_0.2.6_amd64.deb
    ```
  - `QuadFurn-linux-x86_64.tar.gz` — arquivo Linux portátil.
- **macOS (`Universal: Apple Silicon M1–M4 + Intel x86_64`)**: `QuadFurn-macos-universal.zip` — aplicação nativa `QuadFurn.app` com ícone Retina.

---

### Compilação a partir do código-fonte

#### Pré-requisitos
- [Rust Toolchain](https://rustup.rs/) (`stable`, edição Rust 2021)
- Bibliotecas do sistema em **Linux (Debian / Ubuntu)**:
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-dev libssl-dev \
    libgtk-3-dev libglib2.0-dev libatk1.0-dev libgdk-pixbuf-2.0-dev \
    libcairo2-dev libpango1.0-dev libgl1-mesa-dev fonts-dejavu-core fonts-noto-cjk
  ```
- **macOS**: Ferramentas de linha de comando do Xcode (`xcode-select --install`)
- **Windows**: Windows 10 / 11 com ferramentas de compilação MSVC C++

#### Comandos de compilação
```bash
# Executar em modo de desenvolvimento
cargo run

# Compilar versão final otimizada
cargo build --release

# Executar testes unitários
cargo test
```

---

## Definições e formatos de ficheiros

- **Configuração do utilizador (`settings.json`)**:
  Guardada automaticamente na diretoria padrão do sistema:
  - **Windows**: `%APPDATA%\QuadFurn\settings.json`
  - **Linux**: `~/.config/QuadFurn/settings.json`
  - **macOS**: `~/Library/Application Support/QuadFurn/settings.json`
  
  Guarda:
  - `theme_choice` (`Dark`, `Light`, `System`)
  - `language` (`Pt`, `En`, `Ru`, etc.)
  - `recent_projects` (até 10 projetos recentes)
- **Ficheiros de projeto (`.qfurn` / `.json`)**:
  Ficheiros de texto JSON em UTF-8 com peças, grupos, materiais, texturas, fitas de bordo e parâmetros de corte. Ao ativar a incorporação de texturas, as imagens são armazenadas como cadeias Base64 no próprio ficheiro.

---

## Arquitetura do projeto

```text
QuadFurn/
├── .github/
│   └── workflows/
│       └── build.yml          # Integração contínua GitHub Actions (Linux .deb/.tar.gz, macOS .app, Windows .zip)
├── assets/
│   ├── icon.jpeg              # Logótipo da aplicação e ícone fonte
│   └── screenshots/           # Capturas de ecrã para a documentação
├── docs/                      # Documentação traduzida em 37 idiomas
├── src/
│   ├── main.rs                # Ponto de entrada, gestão de separadores, ecrã inicial e atalhos
│   ├── model.rs               # Estrutura de dados (Project, PartBlock, SheetMaterial, EdgeBanding, OBB)
│   ├── viewport.rs            # Controlador de 4 vistas, renderização 2D e 3D wgpu/painter
│   ├── snapping.rs            # Encaixe magnético na grelha e arestas, eixos de projeção
│   ├── cutter.rs              # Otimizador de corte de chapas por guilhotina e cálculo de fitas de bordo
│   ├── export.rs              # Exportador de PDF vetorial A4 Horizontal e ficheiros CSV
│   ├── ui_panels.rs           # Barra de transformação, Inspetor de peças, Especificação, Corte e Materiais
│   └── i18n.rs                # Tabelas de tradução para 37 idiomas e deteção do sistema
├── build.rs                   # Conversão automática do ícone ICO para Windows e recurso PE
├── Cargo.toml                 # Configuração do pacote e dependências
└── LICENSE                    # Licença MIT
```

---

## Licença

Este projeto está licenciado sob a [Licença MIT](../LICENSE) — gratuito para utilização pessoal ou comercial, modificação e distribuição.


