# QuakeTracker

A real-time earthquake monitoring and analytics application built with Rust and SvelteKit. Features interactive maps, comprehensive analytics, and live seismic data from the European-Mediterranean Seismological Centre (EMSC).

## Features

- **Real-time earthquake monitoring** via WebSocket connection to EMSC
- **Interactive maps** with MapLibre GL and PMTiles vector tiles
- **Comprehensive analytics** including magnitude distributions, temporal patterns, and risk assessments
- **Advanced visualizations** with Apache ECharts for statistical analysis
- **Geographic clustering** and regional hotspot identification
- **Gutenberg-Richter analysis** for seismic hazard assessment

## Setup

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+ recommended)
- [Yarn](https://yarnpkg.com/)

### Installation
```shell
yarn install
cargo tauri dev
```

### Development Commands
- `cargo tauri dev` - Start development server (frontend + backend)
- `yarn dev` - Frontend only (web testing)
- `yarn build` - Build for production
- `cargo fmt` - Format Rust code
- `cargo clippy` - Lint Rust code
- `yarn check` - TypeScript checking

## Architecture

### Frontend (SvelteKit)
- **Framework**: SvelteKit 5 with TypeScript
- **Styling**: TailwindCSS
- **Maps**: MapLibre GL with PMTiles
- **Charts**: Apache ECharts

### Backend (Rust + Tauri)
- **Framework**: Tauri 2.0
- **Data Processing**: Polars DataFrames
- **Analytics**: 6 specialized processors for seismic analysis
- **Real-time**: WebSocket integration with EMSC
- **Concurrency**: Thread-safe state management

## Technologies Used

- [Rust](https://www.rust-lang.org/) - Backend performance and safety
- [Tauri 2.0](https://tauri.app/) - Desktop app framework
- [SvelteKit](https://kit.svelte.dev/) - Frontend framework
- [TypeScript](https://www.typescriptlang.org/) - Type safety
- [Apache ECharts](https://echarts.apache.org) - Data visualization
- [MapLibre GL](https://maplibre.org/) - Interactive maps
- [PMTiles](https://protomaps.com/) - Vector tile format
- [Polars](https://pola.rs/) - High-performance DataFrames

## Data Sources & Official Documentation

- https://www.seismicportal.eu/realtime.html
- https://www.seismicportal.eu/fdsn-wsevent.html
- https://www.seismicportal.eu/fdsnws/event/1/application.wadl
- https://www.fdsn.org/webservices/