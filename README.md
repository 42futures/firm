# Firm: Business-as-code

A text-based work management system for technologists.

![Firm CLI demo](docs/src/media/demo.gif)

## Why Firm?

Modern businesses are natively digital, but lack a unified view. Your data is scattered across SaaS tools you don't control, so you piece together answers by jumping between platforms.

Think of your business as a graph: organizations link to people, people link to projects, projects link to tasks, and so on. Firm lets you define these relationships in plain text files.

Version controlled, locally stored and structured as code with the Firm DSL. This structured representation of your work, *business-as-code*, makes your business accessible to yourself and to the robots that help you run it.

### Features

- **Everything in one place:** Organizations, contacts, projects, and their relationships.
- **Own your data:** Plain text files and tooling that works on your machine.
- **Open data model:** Tailor to your business with custom schemas.
- **Automate anything:** Search, report, integrate, whatever. It's just code.
- **AI-ready:** Bots can easily read, write, and query your business structure.

## Quick start

### Install Firm

**With Homebrew:**
```bash
brew tap 42futures/firm
brew install firm
```

**Or download from Github releases:**
https://github.com/42futures/firm/releases

### Initialize your workspace

```bash
cd my_workspace
firm init
```

### Add an entity

```bash
firm add --type organization --id megacorp --field name "Megacorp Ltd."
```

### Query your data

```bash
firm list organization
firm query 'from organization | where name contains "Megacorp"'
```

**[Read the full documentation to learn more.](https://firm.42futures.com)**

## Contributing

Contributions are welcome. Please feel free to submit an issue or pull request.

## License

Firm is licensed under AGPL-v3. That means you can use and extend Firm freely. If you build commercial extensions or services on top of Firm, those must also be open source under AGPL-v3.

See [LICENSE](LICENSE) for full details.
