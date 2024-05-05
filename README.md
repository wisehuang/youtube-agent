# YouTube Agent

YouTube Agent is a project inspired by the article [Building AI Agents with Rust](https://www.shuttle.rs/blog/2024/04/30/building-ai-agents-rust). It is a Rust-based application that uses AI to summarize YouTube video transcripts.

## Features

- Summarizes YouTube video transcripts into main talking points and a complete summary.
- Uses the GPT-4 model from OpenAI for generating summaries.
- Built with the Rocket web framework in Rust.
- Built and deployed using Cargo Shuttle.

## Getting Started

### Prerequisites

- Rust programming language
- Cargo package manager
- Cargo Shuttle

### Installation

1. Clone the repository
```bash
git clone https://github.com/wisehuang/youtube-agent.git
```
2. Navigate to the project directory
```bash
cd youtube-agent
```
3. Build the project with Cargo Shuttle
```bash
cargo shuttle run
```
4. Deploy the project on Shuttle.rs
```bash
cargo shuttle deploy --allow-dirty
```

For more information, you may check this documentation: [Cargo Shuttle Quick Start](https://docs.shuttle.rs/getting-started/quick-start)

## Usage

To get a summary of a YouTube video, make a GET request to the `/youtube/<video_id>` endpoint. The response will be the main talking points and a complete summary of the video transcript.

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.

## Acknowledgements

- [Building AI Agents with Rust](https://www.shuttle.rs/blog/2024/04/30/building-ai-agents-rust)
- [Rocket](https://rocket.rs/)
- [OpenAI](https://openai.com/)
- [Cargo Shuttle](https://shuttle.rs/)