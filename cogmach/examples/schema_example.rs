use cogmach::FundamentalCognitionMachine;
use client_implementations::client::MockVoid;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    println!("🧠 Cogmach Schema-Aware Demo");
    println!("This demonstrates how the updated cogmach automatically generates");
    println!("JSON schemas for all AI interactions, ensuring consistent responses.\n");

    // Create the cognition machine with a mock client
    let client = MockVoid;
    let cogmach = FundamentalCognitionMachine::new(client);

    // Example 1: Structure Analysis
    println!("=== Example 1: Python Code Structure Analysis ===");
    let python_code = r#"
def fibonacci(n):
    """Calculate the nth Fibonacci number."""
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

class Calculator:
    """A simple calculator class."""
    
    def add(self, a, b):
        return a + b
    
    def multiply(self, a, b):
        return a * b

# Tests
def test_fibonacci():
    assert fibonacci(0) == 0
    assert fibonacci(1) == 1
    assert fibonacci(5) == 5
"#;

    // Note: This will fail with MockVoid since it only returns "{}", but it demonstrates
    // the schema generation. With a real AI client, it would work properly.
    let lens = cogmach::Lens::Structure(cogmach::Parser::PythonCode);
    let result = cogmach.observe(&lens, Some(python_code)).await;
    
    match result {
        Ok(reality) => println!("Analysis result: {:?}", reality),
        Err(e) => println!("Expected error with MockVoid (schema was still generated): {}", e),
    }

    println!("\n=== Example 2: Test Suite Analysis ===");
    let test_code = r#"
import pytest

def test_calculator_add():
    calc = Calculator()
    assert calc.add(2, 3) == 5

def test_calculator_multiply():
    calc = Calculator()
    assert calc.multiply(4, 5) == 20

def test_fibonacci_edge_cases():
    assert fibonacci(0) == 0
    assert fibonacci(1) == 1

@pytest.mark.parametrize("n,expected", [(2, 1), (3, 2), (4, 3), (5, 5)])
def test_fibonacci_values(n, expected):
    assert fibonacci(n) == expected
"#;

    let lens = cogmach::Lens::Structure(cogmach::Parser::TestSuite);
    let result = cogmach.observe(&lens, Some(test_code)).await;
    
    match result {
        Ok(reality) => println!("Test analysis result: {:?}", reality),
        Err(e) => println!("Expected error with MockVoid (schema was still generated): {}", e),
    }

    println!("\n=== Schema Benefits ===");
    println!("✅ Automatic schema generation from struct definitions");
    println!("✅ No manual JSON format maintenance in prompts");
    println!("✅ Type-safe responses with compile-time validation");
    println!("✅ Consistent AI responses with clear field descriptions");
    println!("✅ Automatic adaptation when struct definitions change");
    println!("✅ Rich semantic documentation embedded in schemas");
    println!("✅ Structure-agnostic prompts with maximum clarity");

    println!("\n=== Rich Schema Documentation Features ===");
    println!("• Field descriptions with semantic meaning and examples");
    println!("• Value ranges and constraints (e.g., 0.0-1.0 for confidence)");
    println!("• Enum variants with clear semantic descriptions");
    println!("• Title and description metadata for entire schemas");
    
    println!("\n=== Prompts Are Now Structure-Agnostic ===");
    println!("Before: Complex prompts with manual JSON schemas");
    println!("After: Simple, semantic prompts like:");
    println!("  • 'Analyze this code structure'");
    println!("  • 'Examine these tests'");
    println!("  • 'Check the relationship between X and Y'");
    println!("  • 'Generate code for this requirement'");
    println!("  • 'Fix this issue'");
    println!("\nThe schema provides ALL structural details automatically!");

    Ok(())
}