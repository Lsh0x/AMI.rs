# Permission Checking Guide

Use WAMI's policy evaluation engine **without** a full store for lightweight permission checking.

## Use Cases

✅ **Perfect for:**
- Policy validation in CI/CD
- Unit testing IAM policies
- Static policy analysis
- Permission simulators
- Policy linters and validators

❌ **Not suitable for:**
- Full IAM management (use [IAM Guide](IAM_GUIDE.md))
- Persistent storage (use [Store Implementation](STORE_IMPLEMENTATION.md))
- Multi-tenant systems (use [Multi-Tenant Guide](MULTI_TENANT_GUIDE.md))

## Quick Start

### Basic Permission Check

```rust
use wami::iam::{PolicyEvaluator, PolicyDocument};
use wami::iam::policy_evaluation::Action;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a policy
    let policy_json = r#"
    {
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": [
                "s3:GetObject",
                "s3:PutObject"
            ],
            "Resource": "arn:aws:s3:::my-bucket/*"
        }]
    }
    "#;
    
    let policy: PolicyDocument = serde_json::from_str(policy_json)?;
    let evaluator = PolicyEvaluator::new();
    
    // Check if action is allowed
    let result = evaluator.evaluate(
        &[policy],
        &Action::from("s3:GetObject"),
        "arn:aws:s3:::my-bucket/file.txt",
        &Default::default(),
    );
    
    match result {
        Ok(allowed) if allowed => println!("✓ Access allowed"),
        Ok(_) => println!("✗ Access denied"),
        Err(e) => println!("Error: {:?}", e),
    }
    
    Ok(())
}
```

## Core Concepts

### Policy Evaluation Flow

```
1. Parse Policy JSON → PolicyDocument
2. Create PolicyEvaluator
3. Call evaluate(policies, action, resource, context)
4. Returns: Ok(true) = Allow, Ok(false) = Deny, Err = Error
```

### Components

```rust
use wami::iam::policy_evaluation::{
    PolicyEvaluator,      // Evaluates policies
    PolicyDocument,       // AWS policy structure
    Action,               // e.g., "s3:GetObject"
    EvaluationContext,    // Additional context (IP, time, etc.)
};
```

## Examples

### Example 1: Multiple Policies

```rust
use wami::iam::{PolicyEvaluator, PolicyDocument};
use wami::iam::policy_evaluation::Action;

let admin_policy: PolicyDocument = serde_json::from_str(r#"{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Allow",
        "Action": "*",
        "Resource": "*"
    }]
}"#)?;

let deny_delete_policy: PolicyDocument = serde_json::from_str(r#"{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Deny",
        "Action": "s3:DeleteBucket",
        "Resource": "*"
    }]
}"#)?;

let evaluator = PolicyEvaluator::new();

// Admin can do most things
let can_read = evaluator.evaluate(
    &[admin_policy.clone(), deny_delete_policy.clone()],
    &Action::from("s3:GetObject"),
    "arn:aws:s3:::bucket/key",
    &Default::default(),
)?;
println!("Can read: {}", can_read); // true

// But explicit deny blocks deletion
let can_delete = evaluator.evaluate(
    &[admin_policy, deny_delete_policy],
    &Action::from("s3:DeleteBucket"),
    "arn:aws:s3:::bucket",
    &Default::default(),
)?;
println!("Can delete bucket: {}", can_delete); // false - explicit deny wins
```

### Example 2: Condition Evaluation

```rust
use wami::iam::policy_evaluation::EvaluationContext;
use std::collections::HashMap;

let policy: PolicyDocument = serde_json::from_str(r#"{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Allow",
        "Action": "s3:*",
        "Resource": "arn:aws:s3:::my-bucket/*",
        "Condition": {
            "IpAddress": {
                "aws:SourceIp": ["192.168.1.0/24"]
            }
        }
    }]
}"#)?;

let evaluator = PolicyEvaluator::new();

// Without context - denied
let result = evaluator.evaluate(
    &[policy.clone()],
    &Action::from("s3:GetObject"),
    "arn:aws:s3:::my-bucket/file.txt",
    &Default::default(),
)?;
println!("Without IP context: {}", result); // false

// With matching IP - allowed
let mut context = EvaluationContext::default();
context.insert("aws:SourceIp".to_string(), "192.168.1.100".to_string());

let result = evaluator.evaluate(
    &[policy],
    &Action::from("s3:GetObject"),
    "arn:aws:s3:::my-bucket/file.txt",
    &context,
)?;
println!("With matching IP: {}", result); // true
```

### Example 3: Policy Validation Tool

```rust
use wami::iam::PolicyDocument;

fn validate_policy(policy_json: &str) -> Result<(), String> {
    // Parse policy
    let policy: PolicyDocument = serde_json::from_str(policy_json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    // Validate version
    if policy.version != "2012-10-17" {
        return Err("Policy must use Version 2012-10-17".to_string());
    }
    
    // Validate statements
    for (i, statement) in policy.statement.iter().enumerate() {
        // Check effect
        if statement.effect != "Allow" && statement.effect != "Deny" {
            return Err(format!("Statement {}: Effect must be Allow or Deny", i));
        }
        
        // Check actions
        if statement.action.is_empty() {
            return Err(format!("Statement {}: Action cannot be empty", i));
        }
        
        // Check resources
        if statement.resource.is_empty() {
            return Err(format!("Statement {}: Resource cannot be empty", i));
        }
    }
    
    Ok(())
}

// Usage
match validate_policy(policy_json) {
    Ok(_) => println!("✓ Policy is valid"),
    Err(e) => println!("✗ Invalid policy: {}", e),
}
```

### Example 4: Policy Simulator

```rust
use wami::iam::{PolicyEvaluator, PolicyDocument};
use wami::iam::policy_evaluation::Action;

struct PolicySimulator {
    evaluator: PolicyEvaluator,
    policies: Vec<PolicyDocument>,
}

impl PolicySimulator {
    fn new(policies: Vec<PolicyDocument>) -> Self {
        Self {
            evaluator: PolicyEvaluator::new(),
            policies,
        }
    }
    
    fn can(&self, action: &str, resource: &str) -> bool {
        self.evaluator
            .evaluate(
                &self.policies,
                &Action::from(action),
                resource,
                &Default::default(),
            )
            .unwrap_or(false)
    }
    
    fn simulate_scenario(&self, name: &str, actions: Vec<(&str, &str)>) {
        println!("\n📋 Scenario: {}", name);
        for (action, resource) in actions {
            let result = self.can(action, resource);
            let icon = if result { "✓" } else { "✗" };
            println!("  {} {} on {}", icon, action, resource);
        }
    }
}

// Usage
let policies = vec![/* your policies */];
let simulator = PolicySimulator::new(policies);

simulator.simulate_scenario("Developer Access", vec![
    ("s3:GetObject", "arn:aws:s3:::dev-bucket/*"),
    ("s3:PutObject", "arn:aws:s3:::dev-bucket/*"),
    ("s3:DeleteBucket", "arn:aws:s3:::dev-bucket"),
    ("dynamodb:Query", "arn:aws:dynamodb:us-east-1:123456789012:table/DevTable"),
]);
```

### Example 5: CI/CD Policy Linter

```rust
use std::fs;
use wami::iam::PolicyDocument;

fn lint_policies_in_directory(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let content = fs::read_to_string(&path)?;
            
            match serde_json::from_str::<PolicyDocument>(&content) {
                Ok(policy) => {
                    println!("✓ {} - Valid", path.display());
                    
                    // Additional checks
                    if policy.statement.is_empty() {
                        println!("  ⚠ Warning: No statements");
                    }
                    
                    for stmt in &policy.statement {
                        if stmt.action.contains(&"*".to_string()) {
                            println!("  ⚠ Warning: Overly permissive action: *");
                        }
                        if stmt.resource.contains(&"*".to_string()) {
                            println!("  ⚠ Warning: Overly permissive resource: *");
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("{}: {}", path.display(), e));
                    println!("✗ {} - Invalid: {}", path.display(), e);
                }
            }
        }
    }
    
    if !errors.is_empty() {
        return Err(format!("Found {} invalid policies", errors.len()).into());
    }
    
    Ok(())
}

// Run in CI/CD
fn main() {
    if let Err(e) = lint_policies_in_directory("./policies") {
        eprintln!("Policy validation failed: {}", e);
        std::process::exit(1);
    }
    println!("\n✓ All policies valid!");
}
```

## Testing Policies

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wami::iam::{PolicyEvaluator, PolicyDocument};
    use wami::iam::policy_evaluation::Action;
    
    #[test]
    fn test_s3_read_only_policy() {
        let policy: PolicyDocument = serde_json::from_str(r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:GetObject", "s3:ListBucket"],
                "Resource": ["arn:aws:s3:::my-bucket", "arn:aws:s3:::my-bucket/*"]
            }]
        }"#).unwrap();
        
        let evaluator = PolicyEvaluator::new();
        
        // Should allow reads
        assert!(evaluator.evaluate(
            &[policy.clone()],
            &Action::from("s3:GetObject"),
            "arn:aws:s3:::my-bucket/file.txt",
            &Default::default(),
        ).unwrap());
        
        // Should deny writes
        assert!(!evaluator.evaluate(
            &[policy],
            &Action::from("s3:PutObject"),
            "arn:aws:s3:::my-bucket/file.txt",
            &Default::default(),
        ).unwrap());
    }
}
```

## Policy Evaluation Rules

### 1. Default Deny
If no policy explicitly allows, access is denied.

### 2. Explicit Deny Wins
A "Deny" statement always overrides "Allow".

### 3. Action Matching
- Exact match: `"s3:GetObject"` matches `"s3:GetObject"`
- Wildcard: `"s3:*"` matches all S3 actions
- Multiple: `["s3:GetObject", "s3:PutObject"]`

### 4. Resource Matching
- Exact: `"arn:aws:s3:::bucket/key"`
- Wildcard: `"arn:aws:s3:::bucket/*"`
- Multiple: `["arn:...", "arn:..."]`

### 5. Condition Evaluation
Conditions must ALL be true for statement to apply.

```json
{
    "Condition": {
        "StringEquals": {"aws:username": "alice"},
        "IpAddress": {"aws:SourceIp": "192.168.1.0/24"}
    }
}
```
Both conditions must match.

## Best Practices

### ✅ Do

- **Validate in CI/CD**: Catch policy errors before deployment
- **Test combinations**: Test Allow + Deny interactions
- **Use specific actions**: Avoid wildcard `*` where possible
- **Document policies**: Add comments explaining intent
- **Version control**: Keep policies in git

### ❌ Don't

- **Don't use for production auth**: This is for validation, not runtime auth
- **Don't skip error handling**: Policy errors can fail silently
- **Don't assume default allow**: Default is always deny
- **Don't forget conditions**: They can drastically change behavior

## Integration Examples

### With CI/CD (GitHub Actions)

```yaml
name: Validate IAM Policies
on: [pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --test policy_validation
```

### As a CLI Tool

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate { file: String },
    Simulate { policy: String, action: String, resource: String },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Validate { file } => {
            // Validate policy file
        }
        Commands::Simulate { policy, action, resource } => {
            // Simulate access
        }
    }
}
```

## Next Steps

- **[IAM Guide](IAM_GUIDE.md)** - Full IAM management
- **[Store Implementation](STORE_IMPLEMENTATION.md)** - Persistent storage
- **[Multi-Tenant](MULTI_TENANT_GUIDE.md)** - Tenant isolation
- **[Examples](EXAMPLES.md)** - More code samples

## Resources

- **Policy Evaluator**: `src/iam/policy_evaluation/`
- **Examples**: `examples/policy_validation.rs`
- **API Docs**: `cargo doc --open`

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

