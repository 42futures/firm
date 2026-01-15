# Installation

## Homebrew (recommended)

```bash
brew tap 42futures/firm
brew install firm
```

## Manual installation

1. **Download the release**
   - Go to [GitHub Releases](https://github.com/42futures/firm/releases/)
   - Download the appropriate archive for your operating system and architecture
   - Run `uname -m` in your terminal if you're not sure which architecture to choose

2. **Extract the archive**
   ```bash
   tar -xzf firm-[OS]-[ARCH].tar.gz
   cd firm-[OS]-[ARCH]
   ```

3. **Install globally**
   ```bash
   chmod +x firm
   sudo mv firm /usr/local/bin/
   ```

## Verify installation

After installation, verify that Firm is working:

```bash
firm --version
```

You should see the version number printed to your terminal.


