# Chisel

The project brain for founders building with AI agents.

Chisel is a local-first knowledge base for your project. Specs for active work. Docs for permanent knowledge. Both structured so your LLM tools can read them as context when they need it.

## Features

- **Chisel Specs**: Lifecycle-aware specs for active work. Draft → ready → in-progress → shipped → archived.
- **Chisel Docs**: A knowledge base that understands structure. Markdown-first. Git-backed.
- **Human Mode & Machine Mode**: Every tool supports a mode optimized for LLM context windows and parsing.
- **Local-first**: Data lives with you by default. Version-controlled alongside your code.
- **Rust-powered**: Near-zero overhead and sub-10ms latency.

## Installation

```bash
curl -sL https://install.chisel.build | sh
```

## Getting Started

Initialize a new Chisel workspace in your project:

```bash
chisel init
```

Create your first spec:

```bash
chisel spec new "user authentication"
```

Explore your docs:

```bash
chisel docs
```

Generate LLM context:

```bash
chisel context create "auth"
```

## Contributing

Contributions are welcome! See our [Contributing Guide](./CONTRIBUTING.md) to get started.

## License

Chisel is distributed under the [Functional Source License, Version 1.1](./LICENSE.md) (FSL-1.1-Apache-2.0).

The Functional Source License allows you to use, study, and modify the software for any purpose other than providing a competing service. It automatically converts to the Apache License, Version 2.0 after two years.
