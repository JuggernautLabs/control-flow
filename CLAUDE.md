# Semantic Alignment Tracking System (SATS)
## Technical Specification v0.1

### Executive Summary

SATS tracks semantic relationships between all project artifacts (tickets, code, tests, docs) and measures how well they align. Unlike traditional static analysis, SATS uses LLMs to understand implicit claims across different representations and identify gaps in implementation, testing, and documentation.

### Problem Statement

Software projects contain multiple representations of the same concepts:
- Product requirements in tickets
- Implementation in code  
- Verification in tests
- Explanation in documentation

These artifacts drift apart over time. Current tools can't detect when:
- Code no longer matches its requirements
- Tests don't actually test what they claim
- Documentation describes behavior that doesn't exist
- Requirements have no implementation

### Core Innovation

SATS uses LLMs to:
1. Extract implicit claims from each artifact type
2. Find semantic relationships across artifact types
3. Measure alignment between related artifacts
4. Identify gaps and contradictions

This is NOT about verifying correctness - it's about verifying consistency.

### System Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Ingestion     │     │    Analysis     │     │    Queries      │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ • Git Commits   │────▶│ • Claim Extract │────▶│ • Validate      │
│ • Code Files    │     │ • Link Discovery│     │ • Find Gaps     │
│ • Test Files    │     │ • Alignment Calc│     │ • Trace Reqs    │
│ • Tickets       │     │                 │     │ • Health Report │
│ • Documentation │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                       │                        │
         └───────────┬───────────┘                        │
                     ▼                                     ▼
         ┌─────────────────────┐              ┌──────────────────┐
         │   Artifact Store    │              │   Query Results  │
         ├─────────────────────┤              └──────────────────┘
         │ • Artifacts         │
         │ • Claims            │
         │ • Alignments        │
         │ • Semantic Links    │
         └─────────────────────┘
```

### Data Model

#### Core Entities

```typescript
interface Artifact {
  id: UUID;
  type: 'code' | 'test' | 'doc' | 'commit' | 'ticket' | 'comment' | 'spec';
  content: string;
  location: string;  // URI: file:line, url, commit:hash, etc
  timestamp: Date;
  author: string;
  metadata: Record<string, any>;
}

interface Claim {
  id: UUID;
  artifactId: UUID;
  statement: string;  // "This function handles user authentication"
  confidence: number; // 0-1: How confident we are this claim is being made
  type: 'functional' | 'performance' | 'security' | 'behavior' | 'structure';
  extractedFrom: string; // The specific part of artifact this came from
}

interface Alignment {
  id: UUID;
  claimId: UUID;
  evidenceArtifactId: UUID;
  alignmentScore: number; // 0-1: How well evidence supports claim
  explanation: string;    // LLM's reasoning
  checkedAt: Date;
}

interface SemanticLink {
  sourceArtifactId: UUID;
  targetArtifactId: UUID;
  relationshipType: 'implements' | 'tests' | 'documents' | 'references' | 'contradicts';
  confidence: number;
  metadata: Record<string, any>;
}
```

### Core Operations

#### 1. Artifact Ingestion Pipeline

```python
class ArtifactIngester:
    def ingest(self, content: str, artifact_type: ArtifactType, location: str) -> Artifact:
        # Create artifact record
        artifact = Artifact(
            id=generate_uuid(),
            type=artifact_type,
            content=content,
            location=location,
            timestamp=now(),
            author=get_current_user()
        )
        
        # Extract claims based on type
        claims = self.claim_extractor.extract(artifact)
        
        # Find semantic links to existing artifacts
        links = self.link_discoverer.discover(artifact)
        
        # Store everything
        self.store.save_artifact(artifact)
        self.store.save_claims(claims)
        self.store.save_links(links)
        
        return artifact
```

#### 2. Claim Extraction

Different artifact types require different extraction strategies:

```python
class ClaimExtractor:
    def extract(self, artifact: Artifact) -> List[Claim]:
        strategy = self.strategies[artifact.type]
        return strategy.extract_claims(artifact)

class CodeClaimStrategy:
    def extract_claims(self, artifact: Artifact) -> List[Claim]:
        prompt = """
        Analyze this code and extract all implicit and explicit claims:
        - What does the function/class name promise?
        - What do comments claim?
        - What behaviors are implied by the implementation?
        - What constraints or requirements are assumed?
        
        Code:
        {artifact.content}
        
        Return claims as JSON with confidence scores.
        """
        return self.llm.extract_structured(prompt)

class TestClaimStrategy:
    def extract_claims(self, artifact: Artifact) -> List[Claim]:
        prompt = """
        What behaviors do these tests claim to verify?
        - Test names and their implied coverage
        - Assertions and what they validate
        - Setup/teardown implying requirements
        - Comments about what's being tested
        
        Tests:
        {artifact.content}
        """
        return self.llm.extract_structured(prompt)
```

#### 3. Alignment Checking

```python
class AlignmentChecker:
    def check_alignment(self, claim: Claim, artifact: Artifact) -> Alignment:
        prompt = f"""
        Determine if this artifact provides evidence for the claim.
        
        Claim: {claim.statement}
        Claim source: {claim.extracted_from}
        
        Artifact type: {artifact.type}
        Artifact content: {artifact.content}
        
        Score 0-1 based on:
        - Direct evidence (explicitly implements/tests/documents the claim)
        - Indirect evidence (related functionality that supports the claim)
        - Contradictions (evidence that the claim is NOT true)
        
        Explain your reasoning.
        """
        
        result = self.llm.analyze(prompt)
        
        return Alignment(
            claim_id=claim.id,
            evidence_artifact_id=artifact.id,
            alignment_score=result.score,
            explanation=result.reasoning
        )
```

#### 4. Gap Analysis

```python
class GapAnalyzer:
    def find_gaps(self, min_alignment_threshold: float = 0.5) -> List[Gap]:
        gaps = []
        
        for claim in self.store.get_all_claims():
            alignments = self.store.get_alignments_for_claim(claim.id)
            
            # No evidence at all
            if not alignments:
                gaps.append(Gap(
                    type='no_evidence',
                    claim=claim,
                    severity='high'
                ))
            
            # Weak evidence
            elif max(a.alignment_score for a in alignments) < min_alignment_threshold:
                gaps.append(Gap(
                    type='weak_evidence',
                    claim=claim,
                    best_alignment=max(alignments, key=lambda a: a.alignment_score),
                    severity='medium'
                ))
        
        return gaps
```

### Integration Points

#### Git Integration

```bash
#!/bin/bash
# .git/hooks/post-commit

commit_msg=$(git log -1 --pretty=%B)
changed_files=$(git diff-tree --no-commit-id --name-only -r HEAD)

# Ingest commit as artifact
sats ingest commit "$commit_msg" --hash "$(git rev-parse HEAD)"

# Ingest changed files
for file in $changed_files; do
    sats ingest file "$file" --type "$(sats detect-type $file)"
done

# Check alignment
sats check-alignment --recent
```

#### CI/CD Integration

```yaml
# .github/workflows/sats-check.yml
name: Semantic Alignment Check

on: [push, pull_request]

jobs:
  alignment-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Run SATS validation
        run: |
          sats validate "All endpoints have authentication"
          sats validate "All external calls have error handling"
          
      - name: Check for gaps
        run: |
          sats gaps --min-alignment 0.7 --fail-on high
          
      - name: Generate report
        run: |
          sats report --output alignment-report.html
          
      - name: Upload report
        uses: actions/upload-artifact@v2
        with:
          name: alignment-report
          path: alignment-report.html
```

#### IDE Integration

```typescript
// VSCode Extension
class SATSExtension {
    async onDocumentChange(document: TextDocument) {
        const artifact = await this.createArtifact(document);
        const relatedClaims = await this.sats.findRelatedClaims(artifact);
        
        const diagnostics: Diagnostic[] = [];
        
        for (const claim of relatedClaims) {
            const alignment = await this.sats.checkAlignment(claim, artifact);
            
            if (alignment.score < 0.5) {
                diagnostics.push({
                    severity: DiagnosticSeverity.Warning,
                    range: this.getClaimRange(claim),
                    message: `Low alignment with claim: "${claim.statement}" (score: ${alignment.score})`,
                    source: 'SATS'
                });
            }
        }
        
        this.diagnosticCollection.set(document.uri, diagnostics);
    }
}
```

### Query Interface

#### 1. Statement Validation

```bash
$ sats validate "The authentication system supports OAuth2"

Analyzing statement...

Extracted claims:
✓ Authentication system exists
  Evidence: auth_controller.py (0.95), auth_service.js (0.92)
  
✓ System supports OAuth2
  Evidence: oauth_handler.py (0.88), test_oauth_flow.py (0.91)
  
⚠ OAuth2 is fully integrated
  Weak evidence: Missing OAuth2 tests for refresh tokens
  Best match: oauth_handler.py (0.42)

Overall: Statement is 73% supported by codebase
Missing: OAuth2 refresh token implementation and tests
```

#### 2. Requirement Tracing

```bash
$ sats trace JIRA-123

Requirement: "Add password reset functionality"
Source: JIRA-123 (created: 2024-01-15)

Implementation trace:
├─ Claim: "Users can reset passwords"
│  ├─ Code: password_reset.py::send_reset_email() [0.89]
│  ├─ Code: password_reset.py::verify_reset_token() [0.91]
│  ├─ Test: test_password_reset.py::test_reset_flow() [0.94]
│  └─ Docs: api_docs.md#password-reset [0.87]
│
├─ Claim: "Reset tokens expire after 1 hour"
│  ├─ Code: password_reset.py::TOKEN_EXPIRY [0.96]
│  ├─ Test: test_password_reset.py::test_token_expiry() [0.93]
│  └─ Docs: ❌ Not documented
│
└─ Claim: "Rate limit reset requests"
   └─ ❌ No implementation found

Completeness: 67% - Missing rate limiting and some documentation
```

#### 3. Project Health Report

```bash
$ sats health

Project: my-app
Analysis date: 2024-03-15

Semantic Alignment Metrics:
├─ Total claims extracted: 1,247
├─ Fully supported claims: 892 (71.5%)
├─ Partially supported: 234 (18.8%)
├─ Unsupported claims: 121 (9.7%)
└─ Average alignment score: 0.74

Artifact Coverage:
├─ Code files with tests: 78%
├─ Tests with documentation: 45%
├─ Tickets with implementation: 83%
└─ Commits matching changes: 91%

Top Gaps:
1. "API rate limiting" (JIRA-456) - No implementation
2. "Session timeout handling" (auth_spec.md) - No tests
3. "Error logging format" (logging.py) - Doesn't match docs

Drift Indicators:
- 23 tests that don't match their descriptions
- 45 documented APIs not found in code
- 12 security claims without implementation
```

### LLM Prompting Strategy

The system uses structured prompts optimized for different tasks:

```python
CLAIM_EXTRACTION_PROMPT = """
You are analyzing {artifact_type} artifacts to extract claims.

A claim is any statement (implicit or explicit) about what the system does, should do, or guarantees.

For {artifact_type}, focus on:
{type_specific_instructions}

Artifact:
{content}

Extract claims as JSON:
{{
  "claims": [
    {{
      "statement": "clear description of the claim",
      "confidence": 0.0-1.0,
      "type": "functional|performance|security|behavior|structure",
      "extracted_from": "specific line or section"
    }}
  ]
}}
"""

ALIGNMENT_PROMPT = """
Score how well the evidence supports the claim.

Claim: {claim}
Evidence type: {evidence_type}
Evidence: {evidence}

Scoring guidelines:
- 0.9-1.0: Direct, complete implementation/test/documentation
- 0.7-0.9: Strong evidence with minor gaps
- 0.5-0.7: Partial evidence, key aspects missing
- 0.3-0.5: Weak relationship, major gaps
- 0.0-0.3: Little to no supporting evidence

Provide score and explanation as JSON.
"""
```

### Performance Considerations

1. **Caching**: Cache LLM responses for identical content
2. **Incremental processing**: Only analyze changed artifacts
3. **Embedding search**: Use embeddings to find candidate alignments before LLM analysis
4. **Batch processing**: Group similar analysis tasks
5. **Async processing**: Non-blocking analysis in development workflows

### Privacy and Security

1. **Local LLM option**: Support local models for sensitive codebases
2. **Artifact filtering**: Exclude sensitive files/content
3. **Audit logging**: Track all analysis operations
4. **Access control**: Integrate with existing repository permissions

### Metrics for Success

1. **Adoption**: % of commits with SATS analysis
2. **Gap reduction**: Decrease in unsupported claims over time
3. **Drift detection**: Time to identify semantic misalignments
4. **Developer satisfaction**: Survey on usefulness of insights
5. **False positive rate**: % of reported gaps that are actually valid

### Roadmap

#### Phase 1: MVP
- Basic claim extraction for code and tests
- Simple alignment checking
- CLI interface

#### Phase 2: Integration
- Git hooks
- CI/CD integration
- Basic IDE plugin

#### Phase 3: Intelligence
- Improved LLM prompts
- Custom fine-tuned models
- Trend analysis

#### Phase 4: Scale
- Distributed processing
- Enterprise features
- API for custom integrations

### Conclusion

SATS provides a novel approach to maintaining semantic consistency across software projects. By leveraging LLMs to understand implicit relationships between artifacts, it can identify gaps and misalignments that traditional tools miss. The system provides concrete, actionable insights rather than abstract metrics, making it a practical tool for real development teams.