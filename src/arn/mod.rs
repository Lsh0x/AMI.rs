//! ARN (Amazon Resource Name) module for WAMI's multi-tenant, multi-cloud architecture.
//!
//! This module provides a comprehensive ARN system that supports:
//! - Multi-tenant hierarchies (e.g., `t1/t2/t3`)
//! - Multi-cloud provider mapping (AWS, GCP, Azure, Scaleway)
//! - Resource identification by stable IDs (not names)
//! - Bidirectional transformation between WAMI and provider-specific formats
//!
//! # ARN Format
//!
//! ## WAMI Native (no cloud sync):
//! ```text
//! arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{resource_type}/{resource_id}
//! Example: arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755
//! ```
//!
//! ## Cloud-Synced Resources:
//! ```text
//! arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{provider}:{provider_account_id}:{resource_type}/{resource_id}
//! Examples:
//! - arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:user/77557755
//! - arn:wami:iam:t1/t2/t3:wami:999888777:gcp:554433221:user/77557755
//! ```
//!
//! # Usage
//!
//! ## Building ARNs
//!
//! Use the fluent builder API:
//!
//! ```
//! use wami::arn::{WamiArn, Service};
//!
//! // WAMI native ARN
//! let arn = WamiArn::builder()
//!     .service(Service::Iam)
//!     .tenant_hierarchy(vec!["t1", "t2", "t3"])
//!     .wami_instance("999888777")
//!     .resource("user", "77557755")
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(
//!     arn.to_string(),
//!     "arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755"
//! );
//!
//! // Cloud-synced ARN
//! let arn = WamiArn::builder()
//!     .service(Service::Iam)
//!     .tenant("t1")
//!     .wami_instance("999888777")
//!     .cloud_provider("aws", "223344556677")
//!     .resource("user", "77557755")
//!     .build()
//!     .unwrap();
//!
//! assert!(arn.is_cloud_synced());
//! ```
//!
//! ## Parsing ARNs
//!
//! ```
//! use wami::arn::WamiArn;
//! use std::str::FromStr;
//!
//! let arn = WamiArn::from_str("arn:wami:iam:t1:wami:999888777:user/77557755").unwrap();
//! assert_eq!(arn.resource_type(), "user");
//! assert_eq!(arn.primary_tenant(), Some("t1"));
//! ```
//!
//! ## Transforming to Provider Formats
//!
//! ```
//! use wami::arn::{WamiArn, Service, AwsArnTransformer, ArnTransformer};
//!
//! let arn = WamiArn::builder()
//!     .service(Service::Iam)
//!     .tenant("t1")
//!     .wami_instance("999888777")
//!     .cloud_provider("aws", "223344556677")
//!     .resource("user", "77557755")
//!     .build()
//!     .unwrap();
//!
//! let transformer = AwsArnTransformer;
//! let aws_arn = transformer.to_provider_arn(&arn).unwrap();
//! assert_eq!(aws_arn, "arn:aws:iam::223344556677:user/77557755");
//! ```

pub mod builder;
pub mod parser;
pub mod transformer;
pub mod types;

// Re-export key types and functions
pub use builder::ArnBuilder;
pub use parser::{parse_arn, ArnParseError};
pub use transformer::{
    get_transformer, ArnTransformer, AwsArnTransformer, AzureArnTransformer, GcpArnTransformer,
    ProviderArnInfo, ScalewayArnTransformer,
};
pub use types::{CloudMapping, Resource, Service, TenantPath, WamiArn};
