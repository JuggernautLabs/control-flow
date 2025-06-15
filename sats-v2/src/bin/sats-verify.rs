//! SATS v2 Verification Binary
//! 
//! Given a claim and a code directory, this binary:
//! 1. Scans the directory for relevant code files
//! 2. Extracts verification-focused claims from the codebase
//! 3. Generates work items to verify the initial claim
//! 4. Uses actual Claude LLM calls for analysis

use clap::Parser;
use sats_v2::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, warn, error};

#[derive(Parser)]
#[command(name = "sats-verify")]
#[command(about = "Verify claims against a codebase using SATS v2")]
#[command(version)]
struct Args {
    /// The claim to verify against the codebase
    #[arg(short, long)]
    claim: String,

    /// Directory containing the code to analyze
    #[arg(short, long)]
    directory: PathBuf,

    /// File extensions to include (comma-separated, e.g., "rs,py,js")
    #[arg(short, long, default_value = "rs")]
    extensions: String,

    /// Maximum number of files to analyze
    #[arg(short, long, default_value = "50")]
    max_files: usize,

    /// Output format: json, markdown, or text
    #[arg(short, long, default_value = "text")]
    output: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip actual LLM calls (for testing)
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    let subscriber = if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .finish()
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .finish()
    };
    tracing::subscriber::set_global_default(subscriber)?;

    println!("🚀 SATS v2 Claim Verification");
    println!("=============================");
    println!("Claim: {}", args.claim);
    println!("Directory: {}", args.directory.display());
    println!();

    // Validate directory
    if !args.directory.exists() || !args.directory.is_dir() {
        error!("Directory does not exist or is not a directory: {}", args.directory.display());
        std::process::exit(1);
    }

    // Check for API key unless dry run
    if !args.dry_run {
        std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable must be set (check .env file)")?;
    }

    // Step 1: Scan directory for code files
    println!("📂 Scanning directory for code files...");
    let extensions: Vec<&str> = args.extensions.split(',').collect();
    let code_files = scan_directory(&args.directory, &extensions, args.max_files)?;
    println!("Found {} code files", code_files.len());

    if code_files.is_empty() {
        warn!("No code files found with extensions: {}", args.extensions);
        return Ok(());
    }

    // Step 2: Convert files to artifacts
    println!("📄 Converting files to artifacts...");
    let artifacts = create_artifacts_from_files(&code_files)?;
    println!("Created {} artifacts for analysis", artifacts.len());

    // Step 3: Extract claims from codebase
    println!("🔍 Extracting claims from codebase...");
    let mut all_claims = Vec::new();
    
    if args.dry_run {
        println!("  (Dry run mode - skipping actual LLM calls)");
        // Create mock claims for testing
        all_claims = create_mock_claims(&artifacts, &args.claim);
    } else {
        // Use actual Claude LLM
        let api_key = std::env::var("ANTHROPIC_API_KEY")?;
        let claim_extractor = ClaudeVerificationExtractor::new(api_key);
        
        for artifact in &artifacts {
            println!("  Analyzing {}", artifact.location.display());
            
            match claim_extractor.extract_verification_claims(artifact).await {
                Ok(result) => {
                    println!("    ✅ Extracted {} claims in {}ms", 
                             result.claims.len(), 
                             result.processing_time_ms);
                    
                    if args.verbose {
                        for (i, claim) in result.claims.iter().enumerate() {
                            println!("      {}. {}", i + 1, claim.statement);
                        }
                    }
                    
                    all_claims.extend(result.claims);
                }
                Err(e) => {
                    warn!("Failed to extract claims from {}: {}", artifact.location.display(), e);
                }
            }
        }
    }

    println!("📊 Total claims extracted: {}", all_claims.len());

    // Step 4: Analyze claims against the input claim
    println!("🎯 Analyzing claims against target claim: '{}'", args.claim);
    let analysis_result = analyze_claims_against_target(&all_claims, &args.claim).await?;

    // Step 5: Generate work items for verification
    println!("🔧 Generating verification work items...");
    let work_items = generate_verification_work_items(&analysis_result, &args.claim).await?;

    // Step 6: Output results
    match args.output.as_str() {
        "json" => output_json(&analysis_result, &work_items)?,
        "markdown" => output_markdown(&analysis_result, &work_items, &args.claim)?,
        _ => output_text(&analysis_result, &work_items, &args.claim)?,
    }

    println!("✅ Verification analysis complete!");
    Ok(())
}

/// Scan directory for code files with specified extensions
fn scan_directory(dir: &Path, extensions: &[&str], max_files: usize) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    fn scan_recursive(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>, max_files: usize) -> Result<(), Box<dyn std::error::Error>> {
        if files.len() >= max_files {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip common directories that don't contain source code
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if matches!(dir_name.as_ref(), "target" | "node_modules" | ".git" | "build" | "dist") {
                    continue;
                }
                scan_recursive(&path, extensions, files, max_files)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if extensions.contains(&ext_str.as_ref()) {
                        files.push(path);
                        if files.len() >= max_files {
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    scan_recursive(dir, extensions, &mut files, max_files)?;
    Ok(files)
}

/// Convert file paths to SATS artifacts
fn create_artifacts_from_files(files: &[PathBuf]) -> Result<Vec<Artifact>, Box<dyn std::error::Error>> {
    let mut artifacts = Vec::new();

    for file_path in files {
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read file {}: {}", file_path.display(), e);
                continue;
            }
        };

        // Skip very large files
        if content.len() > 100_000 {
            warn!("Skipping large file {} ({} bytes)", file_path.display(), content.len());
            continue;
        }

        // Skip empty files
        if content.trim().is_empty() {
            continue;
        }

        // Determine artifact type based on file extension and content
        let artifact_type = determine_artifact_type(file_path, &content);

        let artifact = Artifact {
            id: uuid::Uuid::new_v4(),
            artifact_type,
            content,
            location: Location::File {
                path: file_path.to_string_lossy().to_string(),
                line_range: None,
            },
            created_at: chrono::Utc::now(),
            author: None,
            metadata: HashMap::from([
                ("file_size".to_string(), fs::metadata(file_path)?.len().to_string()),
                ("extension".to_string(), 
                 file_path.extension().unwrap_or_default().to_string_lossy().to_string()),
            ]),
        };

        artifacts.push(artifact);
    }

    Ok(artifacts)
}

/// Determine artifact type based on file path and content
fn determine_artifact_type(file_path: &Path, content: &str) -> ArtifactType {
    // Check file name patterns
    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    
    if file_name.contains("test") || file_name.contains("spec") {
        return ArtifactType::Test;
    }
    
    if file_name.contains("readme") || file_name.contains("doc") {
        return ArtifactType::Documentation;
    }

    // Check content patterns
    let content_lower = content.to_lowercase();
    
    if content_lower.contains("#[test]") || 
       content_lower.contains("fn test_") ||
       content_lower.contains("describe(") ||
       content_lower.contains("it(") ||
       content_lower.contains("def test_") {
        return ArtifactType::Test;
    }

    if content_lower.starts_with('#') || // Markdown
       content_lower.contains("## ") ||
       content_lower.contains("documentation") {
        return ArtifactType::Documentation;
    }

    // Default to code
    ArtifactType::Code
}

/// Create mock claims for dry run mode
fn create_mock_claims(artifacts: &[Artifact], target_claim: &str) -> Vec<Claim> {
    let mut claims = Vec::new();
    
    for (i, artifact) in artifacts.iter().take(3).enumerate() {
        let statement = match i {
            0 => format!("Code implements functionality related to: {}", target_claim),
            1 => format!("Tests verify behavior described in: {}", target_claim),
            2 => format!("Documentation describes: {}", target_claim),
            _ => format!("System supports: {}", target_claim),
        };

        let claim = Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: artifact.id,
            statement,
            claim_type: match i {
                0 => ClaimType::Functional,
                1 => ClaimType::Testing,
                2 => ClaimType::Structure,
                _ => ClaimType::Behavior,
            },
            extraction_confidence: Confidence::new(0.8).unwrap(),
            source_excerpt: artifact.content.chars().take(100).collect::<String>(),
            extracted_at: chrono::Utc::now(),
            verification_chain: Some(VerificationChain {
                claim_id: uuid::Uuid::new_v4(),
                status: ChainStatus::NotStarted,
                links: vec![],
                created_at: chrono::Utc::now(),
                last_verified_at: None,
                missing_links: vec![WorkItemType::CreateTests, WorkItemType::ImplementRequirements],
            }),
        };

        claims.push(claim);
    }

    claims
}

/// Analysis result structure
#[derive(Debug)]
struct ClaimAnalysisResult {
    target_claim: String,
    supporting_claims: Vec<Claim>,
    contradicting_claims: Vec<Claim>,
    related_claims: Vec<Claim>,
    gap_analysis: Vec<String>,
}

/// Analyze extracted claims against the target claim
async fn analyze_claims_against_target(claims: &[Claim], target_claim: &str) -> Result<ClaimAnalysisResult, Box<dyn std::error::Error>> {
    info!("Analyzing {} claims against target: {}", claims.len(), target_claim);

    let mut supporting_claims = Vec::new();
    let mut contradicting_claims = Vec::new();
    let mut related_claims = Vec::new();
    let mut gap_analysis = Vec::new();

    // Simple text-based analysis for now
    // In a real implementation, this would use LLM analysis
    let target_lower = target_claim.to_lowercase();
    let target_keywords: Vec<&str> = target_lower.split_whitespace().collect();

    for claim in claims {
        let claim_lower = claim.statement.to_lowercase();
        let matching_keywords = target_keywords.iter()
            .filter(|&&keyword| claim_lower.contains(keyword))
            .count();

        let relevance_score = matching_keywords as f64 / target_keywords.len() as f64;

        if relevance_score > 0.5 {
            if claim_lower.contains("not") || claim_lower.contains("doesn't") || claim_lower.contains("missing") {
                contradicting_claims.push(claim.clone());
            } else {
                supporting_claims.push(claim.clone());
            }
        } else if relevance_score > 0.2 {
            related_claims.push(claim.clone());
        }
    }

    // Identify gaps
    if supporting_claims.is_empty() {
        gap_analysis.push("No claims found that directly support the target claim".to_string());
    }

    let has_tests = claims.iter().any(|c| c.claim_type == ClaimType::Testing);
    if !has_tests {
        gap_analysis.push("No test-related claims found to verify the target claim".to_string());
    }

    // Would need to check related artifacts to determine if docs exist
    let has_docs = false; // Simplified for this implementation
    if !has_docs {
        gap_analysis.push("No documentation found to support the target claim".to_string());
    }

    Ok(ClaimAnalysisResult {
        target_claim: target_claim.to_string(),
        supporting_claims,
        contradicting_claims,
        related_claims,
        gap_analysis,
    })
}

/// Generate work items to verify the target claim
async fn generate_verification_work_items(analysis: &ClaimAnalysisResult, _target_claim: &str) -> Result<Vec<WorkItem>, Box<dyn std::error::Error>> {
    let mut work_items = Vec::new();

    // Generate work items based on gaps
    for (i, gap) in analysis.gap_analysis.iter().enumerate() {
        let work_item_type = if gap.contains("test") {
            WorkItemType::CreateTests
        } else if gap.contains("documentation") {
            WorkItemType::Documentation
        } else {
            WorkItemType::ImplementRequirements
        };

        let work_item = WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type,
            claim_id: uuid::Uuid::new_v4(), // Would link to specific claim
            title: format!("Address gap: {}", gap),
            description: format!("Work item to address identified gap: {}", gap),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: 5, // Medium effort
            required_skills: vec!["development".to_string()],
            specification: serde_json::json!({
                "gap_description": gap,
                "priority": if i == 0 { "high" } else { "medium" }
            }),
        };

        work_items.push(work_item);
    }

    // Generate work items for claims that need verification
    for claim in &analysis.supporting_claims {
        if let Some(chain) = &claim.verification_chain {
            for missing_link in &chain.missing_links {
                let work_item = WorkItem {
                    id: uuid::Uuid::new_v4(),
                    work_item_type: missing_link.clone(),
                    claim_id: claim.id,
                    title: format!("Verify claim: {}", claim.statement.chars().take(50).collect::<String>()),
                    description: format!("Verification work for: {}", claim.statement),
                    status: WorkItemStatus::Pending,
                    created_at: chrono::Utc::now(),
                    assignee: None,
                    estimated_effort: 3,
                    required_skills: vec!["testing".to_string(), "verification".to_string()],
                    specification: serde_json::json!({
                        "claim_statement": claim.statement,
                        "verification_type": format!("{:?}", missing_link)
                    }),
                };

                work_items.push(work_item);
            }
        }
    }

    Ok(work_items)
}

/// Output results in JSON format
fn output_json(analysis: &ClaimAnalysisResult, work_items: &[WorkItem]) -> Result<(), Box<dyn std::error::Error>> {
    let output = serde_json::json!({
        "target_claim": analysis.target_claim,
        "analysis": {
            "supporting_claims": analysis.supporting_claims.len(),
            "contradicting_claims": analysis.contradicting_claims.len(),
            "related_claims": analysis.related_claims.len(),
            "gaps_identified": analysis.gap_analysis.len()
        },
        "work_items": work_items.len(),
        "gaps": analysis.gap_analysis
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Output results in Markdown format
fn output_markdown(analysis: &ClaimAnalysisResult, work_items: &[WorkItem], target_claim: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("# SATS v2 Verification Report");
    println!();
    println!("**Target Claim:** {}", target_claim);
    println!("**Analysis Date:** {}", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"));
    println!();

    println!("## Analysis Summary");
    println!("- Supporting Claims: {}", analysis.supporting_claims.len());
    println!("- Contradicting Claims: {}", analysis.contradicting_claims.len());
    println!("- Related Claims: {}", analysis.related_claims.len());
    println!("- Gaps Identified: {}", analysis.gap_analysis.len());
    println!();

    if !analysis.gap_analysis.is_empty() {
        println!("## Identified Gaps");
        for gap in &analysis.gap_analysis {
            println!("- {}", gap);
        }
        println!();
    }

    if !work_items.is_empty() {
        println!("## Generated Work Items");
        for (i, item) in work_items.iter().enumerate() {
            println!("{}. **{}** ({})", i + 1, item.title, format!("{:?}", item.work_item_type));
            println!("   - {}", item.description);
            println!("   - Effort: {}/10", item.estimated_effort);
            println!();
        }
    }

    Ok(())
}

/// Output results in text format
fn output_text(analysis: &ClaimAnalysisResult, work_items: &[WorkItem], target_claim: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("🎯 VERIFICATION ANALYSIS RESULTS");
    println!("================================");
    println!("Target Claim: {}", target_claim);
    println!();

    println!("📊 ANALYSIS SUMMARY:");
    println!("  Supporting Claims: {}", analysis.supporting_claims.len());
    println!("  Contradicting Claims: {}", analysis.contradicting_claims.len());
    println!("  Related Claims: {}", analysis.related_claims.len());
    println!("  Gaps Identified: {}", analysis.gap_analysis.len());
    println!();

    if !analysis.supporting_claims.is_empty() {
        println!("✅ SUPPORTING CLAIMS:");
        for (i, claim) in analysis.supporting_claims.iter().enumerate() {
            println!("  {}. {}", i + 1, claim.statement);
            println!("     Type: {:?} | Confidence: {:.2}", claim.claim_type, claim.extraction_confidence.value());
        }
        println!();
    }

    if !analysis.contradicting_claims.is_empty() {
        println!("❌ CONTRADICTING CLAIMS:");
        for (i, claim) in analysis.contradicting_claims.iter().enumerate() {
            println!("  {}. {}", i + 1, claim.statement);
        }
        println!();
    }

    if !analysis.gap_analysis.is_empty() {
        println!("⚠️  IDENTIFIED GAPS:");
        for (i, gap) in analysis.gap_analysis.iter().enumerate() {
            println!("  {}. {}", i + 1, gap);
        }
        println!();
    }

    if !work_items.is_empty() {
        println!("🔧 GENERATED WORK ITEMS:");
        for (i, item) in work_items.iter().enumerate() {
            println!("  {}. {} ({})", i + 1, item.title, format!("{:?}", item.work_item_type));
            println!("     Description: {}", item.description);
            println!("     Effort: {}/10 | Skills: {}", item.estimated_effort, item.required_skills.join(", "));
        }
        println!();
    }

    if analysis.supporting_claims.is_empty() && analysis.gap_analysis.is_empty() {
        println!("🎉 CONCLUSION: The target claim appears to be well-supported by the codebase!");
    } else {
        println!("💡 CONCLUSION: {} work items generated to improve claim verification.", work_items.len());
    }

    Ok(())
}