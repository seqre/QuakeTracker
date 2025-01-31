# QuakeTracker

## Setup

First, download PMTiles using instructions
from [here](https://docs.protomaps.com/guide/getting-started#_3-extract-any-area) and extract the contents to the root
of the project.

Then, run the following commands:

```shell
yarn install
cargo tauri dev
```

## Technologies Used

- [Rust](https://www.rust-lang.org/)
- [Tauri](https://tauri.app/)
- [TypeScript](https://www.typescriptlang.org/)
- [Apache ECharts](https://echarts.apache.org)
- [PMTiles](https://protomaps.com/)

## Future Work

- [ ] Streamline data processing pipeline
- [ ] Add more data visualization options
- [ ] Improve UI/UX
- [ ] Add more filtering options

## Data Sources & Official Documentation

- https://www.seismicportal.eu/realtime.html
- https://www.seismicportal.eu/fdsn-wsevent.html
- https://www.seismicportal.eu/fdsnws/event/1/application.wadl
- https://www.fdsn.org/webservices/