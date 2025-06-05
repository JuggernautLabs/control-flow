# SATS Setup Guide

## Quick Start

### 1. Mock Analysis (No API Key Required)
```bash
cd sats-example
cargo run
```

### 2. Real Claude-Powered Analysis

#### Setup API Key
1. **Get Claude API Key**: Visit [Anthropic Console](https://console.anthropic.com/) to get your API key

2. **Configure Environment**:
   ```bash
   cd sats-example
   cp .env.example .env
   ```

3. **Edit .env file**:
   ```bash
   # Claude API Configuration
   ANTHROPIC_API_KEY=your-actual-api-key-here
   
   # Optional: Configure logging level
   RUST_LOG=info
   ```

#### Run Analysis
```bash
cargo run --bin real_claude_analysis
```

## Available Programs

| Program | Description | API Key Required |
|---------|-------------|------------------|
| `cargo run` | Mock analysis demo | ❌ No |
| `cargo run --bin real_claude_analysis` | Live Claude API analysis | ✅ Yes |
| `cargo run --bin demo_without_api` | Implementation structure demo | ❌ No |

## Expected Costs

The real Claude analysis makes approximately:
- **47 claim extractions** (~$1.50)
- **188 alignment checks** (~$3.00)
- **Total: ~$4.50** for complete analysis

Processing time: 5-8 minutes depending on API response times.

## What Each Program Shows

### Mock Analysis (`cargo run`)
- SATS architecture without API costs
- Gap detection and project health metrics
- OAuth2 authentication system analysis
- Perfect for understanding the system

### Real Claude Analysis (`cargo run --bin real_claude_analysis`) 
- **Live AI-powered claim extraction**
- **Semantic alignment checking with reasoning**
- Multi-factor authentication project analysis
- Claude's natural language understanding
- Production-ready analysis capabilities

### Demo (`cargo run --bin demo_without_api`)
- Implementation structure walkthrough
- Prompt engineering examples
- Expected Claude response formats
- Setup instructions and benefits

## Troubleshooting

### "ANTHROPIC_API_KEY not set" Error
```bash
# Make sure .env file exists and contains your key
cat .env

# Should show:
ANTHROPIC_API_KEY=your-key-here
```

### Rate Limiting
The program includes 100ms delays between API calls to be respectful. If you hit rate limits:
- The built-in retry logic will handle temporary limits
- For persistent issues, check your API key's rate limits

### API Costs
- Monitor your usage at [Anthropic Console](https://console.anthropic.com/)
- Each run costs ~$4.50 for the full analysis
- Start with the mock version to understand the system first

## Next Steps

1. **Start with mock analysis** to understand SATS capabilities
2. **Run demo** to see implementation details  
3. **Set up .env file** when ready for live API analysis
4. **Run real analysis** to see Claude's AI-powered insights

The SATS system provides unprecedented insights into semantic consistency across your software projects!