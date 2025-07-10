# PLX desktop

## Introduction
This is the desktop app, replacing the previous TUI, currently in construction.

## Build

Make sure your have, Rust, NodeJS, PNPM and Git

Install `typeshare`
```sh
cargo install typeshare-cli
```

```sh
pnpm tauri dev
```

To generate installers for different plateforms
```sh
pnpm tauri bundle
```

## Frontend stack
The frontend is built with VueJS 3, using Pinia to manage global stores and TailwindCSS as the CSS framework.

The project is divided a few logical folders
1. `blocks` all the reusable components as building blocks for other components or pages
1. `pages`: the components displayed as whole pages by `App.vue`
1. `stores`: the various Pinia stores for different features
1. `ts`: hand written or auto generated types

```sh
> cd desktop
> tree
.
├── App.vue
├── blocks
│   ├── AnswerShow.vue
│   ├── CheckResultBox.vue
│   ├── CodeExo.vue
│   ├── Code.vue
│   └── Markdown.vue
├── client.ts
├── main.css
├── main.ts
├── pages
│   ├── Course.vue
│   ├── Dashboard.vue
│   ├── Home.vue
│   └── Train.vue
├── stores
│   ├── GlobalStore.ts
│   ├── LiveStore.ts
│   └── TrainStore.ts
├── ts
│   ├── commands.ts
│   ├── complement.ts
│   ├── README.md
│   └── shared.ts
└── vite-env.d.ts
```
