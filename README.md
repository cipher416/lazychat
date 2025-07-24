# lazychat

[![CI](https://github.com/cipher416/lazychat/workflows/CI/badge.svg)](https://github.com/cipher416/lazychat/actions)

A terminal-based chat client for LLM interactions built with Rust and ratatui.

## Features

- **Terminal User Interface**: Clean, responsive TUI built with ratatui
- **LLM Integration**: Connect to OpenRouter API for AI chat completions
- **Real-time Chat**: Interactive chat interface with loading states
- **Message History**: Persistent chat history during session
- **Keyboard Navigation**: Full keyboard-driven interface
- **Configurable**: Customizable tick rate and frame rate
- **Error Handling**: Robust error handling with user-friendly messages

## Prerequisites

- Rust (latest stable version)
- OpenRouter API key

## Installation

1. Clone the repository:

```bash
git clone https://github.com/cipher416/lazychat.git
cd lazychat
```

2. Build the project:

```bash
cargo build --release
```

3. Set up your environment:

```bash
# Create a .env file or export the variable
export OPENROUTER_API_KEY="your_api_key_here"
```

## Usage

Run the application:

```bash
cargo run
```

Or run the compiled binary:

```bash
./target/release/lazychat
```

### Command Line Options

- `-t, --tick-rate <FLOAT>`: Set tick rate (ticks per second, default: 4.0)
- `-f, --frame-rate <FLOAT>`: Set frame rate (frames per second, default: 60.0)
- `-h, --help`: Show help information
- `-V, --version`: Show version information

Example:

```bash
cargo run -- --tick-rate 2.0 --frame-rate 30.0
```

## Interface

The application features a split-screen layout:

- **Chat Area** (top 3/4): Displays conversation history with user and assistant messages
- **Input Area** (bottom 1/4): Text input field for typing messages

### Controls

- **Enter**: Send message
- **Ctrl+C**: Quit application
- **Mouse**: Enabled for interaction (optional)

## Configuration

The application uses configuration files located in:

- Config directory: Platform-specific config directory
- Data directory: Platform-specific data directory

Use `lazychat --version` to see the exact paths on your system.

## Architecture

The project follows a component-based architecture:

- `app.rs`: Main application logic and state management
- `tui.rs`: Terminal UI setup and event handling
- `components/`: UI components (ChatWindow, Input, Home)
- `config.rs`: Configuration management
- `cli.rs`: Command-line interface
- `action.rs`: Application actions and events

## API Integration

Currently integrates with:

- **OpenRouter**: Uses the Mistral Nemo model via OpenRouter API
- **Model**: `mistralai/mistral-nemo`

## Development

### Building

```bash
cargo build
```

### Running in Development

```bash
cargo run
```

### Testing

```bash
cargo test
```

## Dependencies

Key dependencies include:

- `ratatui`: Terminal UI framework
- `tokio`: Async runtime
- `reqwest`: HTTP client for API calls
- `clap`: Command-line argument parsing
- `serde_json`: JSON serialization
- `crossterm`: Cross-platform terminal manipulation

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Cristoper Anderson <cristoper.anderson@gmail.com>

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui) for the terminal interface
- Uses [OpenRouter](https://openrouter.ai/) for LLM API access
