# Prototype of team 51

> Faculty of Engineering at University of Porto  
> Master's Degree in Informatics and Computing Engineering  
> Large Scale Software Development (M.EIC002) 2025/2026  
>
> - Ademar Manuel Teixeira de Aguiar (Regent of the curricular unit)
> - Filipe Figueiredo Correia (Theoretical-Practical classes)

> **Class 05; Group 01**
>
> - Guilherme Duarte Silva Matos <up202208755@up.pt>
> - João Vítor da Costa Ferreira <up202208393@up.pt> **(Scrum Master)**
> - Lucas Gonçalves Bessa <up202208396@up.pt>
> - Luís Miguel Melo Arruda <up202206970@up.pt> **(Product Owner)**
> - Pedro Paulo Gorobey <up202210292@up.pt>
> - Tomás de Campos Sucena de Sequeiros Lopes <up202108701@up.pt>

## Theme

An encrypted and private messaging system between users to communicate about products and deals.

See the Epic [#131](https://github.com/FEUP-MEIC-DS-2025-26/madeinportugal.store/issues/131) and the User Stories 
[#204](https://github.com/FEUP-MEIC-DS-2025-26/madeinportugal.store/issues/204),
[#205](https://github.com/FEUP-MEIC-DS-2025-26/madeinportugal.store/issues/205),
[#206](https://github.com/FEUP-MEIC-DS-2025-26/madeinportugal.store/issues/206), and
[#207](https://github.com/FEUP-MEIC-DS-2025-26/madeinportugal.store/issues/207) for more details.

## Tech Stack

- [Actix](https://actix.rs/) as a backend and API server;
- [NextJS](https://nextjs.org/docs) with `export` mode as a frontend;

## Deployment

- Build the production image
```bash
# --no-cache to do everything from scratch (because file changes are not detected by Docker cache)
docker compose build --no-cache
```

- Run the docker image
```bash
docker compose up
```

## Development

### Prerequisites
| Software | Recommended version | Download link |
| -------- | ------------------- | ------------ |
| Rust & Cargo | 1.90 | https://rustup.rs/ |
| NPM (NodeJS) | 10.9.3 | https://nodejs.org/en/download |

### Setup

```bash
# Optional; just to speed up next builds
cargo build -r
```

```bash
cd frontend
npm install
```

### How to run

- Compile the frontend (needed to run everytime a file changes):
```bash
cd frontend
npm run build
```

- Run the backend server
```bash
RUST_LOG=info cargo run -r -- kiosk
```
