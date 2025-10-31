//! Fluent builder for constructing WAMI ARNs.

use super::types::{CloudMapping, Resource, Service, TenantPath, WamiArn};
use crate::error::{AmiError, Result};

/// A fluent builder for constructing WAMI ARNs.
///
/// # Examples
///
/// ## Building a WAMI-native ARN
///
/// ```
/// use wami::arn::{WamiArn, Service};
///
/// let arn = WamiArn::builder()
///     .service(Service::Iam)
///     .tenant_hierarchy(vec!["t1", "t2", "t3"])
///     .wami_instance("999888777")
///     .resource("user", "77557755")
///     .build()
///     .unwrap();
///
/// assert_eq!(
///     arn.to_string(),
///     "arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755"
/// );
/// ```
///
/// ## Building a cloud-synced ARN
///
/// ```
/// use wami::arn::{WamiArn, Service};
///
/// let arn = WamiArn::builder()
///     .service(Service::Iam)
///     .tenant_hierarchy(vec!["t1", "t2", "t3"])
///     .wami_instance("999888777")
///     .cloud_provider("aws", "223344556677")
///     .resource("user", "77557755")
///     .build()
///     .unwrap();
///
/// assert_eq!(
///     arn.to_string(),
///     "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:global:user/77557755"
/// );
/// ```
#[derive(Debug, Default)]
pub struct ArnBuilder {
    service: Option<Service>,
    tenant_path: Option<TenantPath>,
    wami_instance_id: Option<String>,
    cloud_mapping: Option<CloudMapping>,
    resource: Option<Resource>,
}

impl ArnBuilder {
    /// Creates a new ARN builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the service.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::{WamiArn, Service};
    ///
    /// let builder = WamiArn::builder().service(Service::Iam);
    /// ```
    pub fn service(mut self, service: Service) -> Self {
        self.service = Some(service);
        self
    }

    /// Sets the service from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder().service_str("iam");
    /// ```
    pub fn service_str(mut self, service: impl Into<String>) -> Self {
        self.service = Some(Service::from(service.into().as_str()));
        self
    }

    /// Sets the tenant path.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::{WamiArn, TenantPath};
    ///
    /// let path = TenantPath::new(vec!["t1".to_string(), "t2".to_string()]);
    /// let builder = WamiArn::builder().tenant_path(path);
    /// ```
    pub fn tenant_path(mut self, path: TenantPath) -> Self {
        self.tenant_path = Some(path);
        self
    }

    /// Sets the tenant hierarchy from a vector of tenant IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder()
    ///     .tenant_hierarchy(vec!["t1", "t2", "t3"]);
    /// ```
    pub fn tenant_hierarchy<S: Into<String>>(mut self, segments: Vec<S>) -> Self {
        self.tenant_path = Some(TenantPath::new(
            segments.into_iter().map(|s| s.into()).collect(),
        ));
        self
    }

    /// Sets a single tenant (non-hierarchical).
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder().tenant("t1");
    /// ```
    pub fn tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_path = Some(TenantPath::single(tenant_id));
        self
    }

    /// Sets the WAMI instance ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder().wami_instance("999888777");
    /// ```
    pub fn wami_instance(mut self, instance_id: impl Into<String>) -> Self {
        self.wami_instance_id = Some(instance_id.into());
        self
    }

    /// Sets the cloud provider mapping without a region (global resource).
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder()
    ///     .cloud_provider("aws", "223344556677");
    /// ```
    pub fn cloud_provider(
        mut self,
        provider: impl Into<String>,
        account_id: impl Into<String>,
    ) -> Self {
        self.cloud_mapping = Some(CloudMapping::new(provider, account_id));
        self
    }

    /// Sets the cloud provider mapping with a specific region.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder()
    ///     .cloud_provider_with_region("aws", "223344556677", "us-east-1");
    /// ```
    pub fn cloud_provider_with_region(
        mut self,
        provider: impl Into<String>,
        account_id: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        self.cloud_mapping = Some(CloudMapping::with_region(provider, account_id, region));
        self
    }

    /// Sets the region for the current cloud mapping.
    /// If no cloud mapping exists, this does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder()
    ///     .cloud_provider("aws", "223344556677")
    ///     .region("us-east-1");
    /// ```
    pub fn region(mut self, region: impl Into<String>) -> Self {
        if let Some(ref mut mapping) = self.cloud_mapping {
            mapping.region = Some(region.into());
        }
        self
    }

    /// Sets the cloud mapping directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::{WamiArn, CloudMapping};
    ///
    /// let mapping = CloudMapping::new("gcp", "554433221");
    /// let builder = WamiArn::builder().cloud_mapping(mapping);
    /// ```
    pub fn cloud_mapping(mut self, mapping: CloudMapping) -> Self {
        self.cloud_mapping = Some(mapping);
        self
    }

    /// Removes any cloud mapping (creates a WAMI-native ARN).
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder()
    ///     .cloud_provider("aws", "123456")
    ///     .no_cloud_mapping();
    /// ```
    pub fn no_cloud_mapping(mut self) -> Self {
        self.cloud_mapping = None;
        self
    }

    /// Sets the resource.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::{WamiArn, Resource};
    ///
    /// let resource = Resource::new("user", "77557755");
    /// let builder = WamiArn::builder().resource_obj(resource);
    /// ```
    pub fn resource_obj(mut self, resource: Resource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Sets the resource from type and ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    ///
    /// let builder = WamiArn::builder().resource("user", "77557755");
    /// ```
    pub fn resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.resource = Some(Resource::new(resource_type, resource_id));
        self
    }

    /// Builds the ARN, returning an error if any required fields are missing.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the following fields are not set:
    /// - service
    /// - tenant_path
    /// - wami_instance_id
    /// - resource
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::{WamiArn, Service};
    ///
    /// let result = WamiArn::builder()
    ///     .service(Service::Iam)
    ///     .tenant("t1")
    ///     .wami_instance("999888777")
    ///     .resource("user", "77557755")
    ///     .build();
    ///
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::result_large_err)]
    pub fn build(self) -> Result<WamiArn> {
        let service = self.service.ok_or_else(|| AmiError::InvalidParameter {
            message: "ARN builder: service is required".to_string(),
        })?;

        let tenant_path = self.tenant_path.ok_or_else(|| AmiError::InvalidParameter {
            message: "ARN builder: tenant_path is required".to_string(),
        })?;

        let wami_instance_id = self
            .wami_instance_id
            .ok_or_else(|| AmiError::InvalidParameter {
                message: "ARN builder: wami_instance_id is required".to_string(),
            })?;

        let resource = self.resource.ok_or_else(|| AmiError::InvalidParameter {
            message: "ARN builder: resource is required".to_string(),
        })?;

        // Validate tenant path is not empty
        if tenant_path.segments.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "ARN builder: tenant_path cannot be empty".to_string(),
            });
        }

        // Validate wami_instance_id is not empty
        if wami_instance_id.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "ARN builder: wami_instance_id cannot be empty".to_string(),
            });
        }

        // Validate resource type and ID are not empty
        if resource.resource_type.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "ARN builder: resource_type cannot be empty".to_string(),
            });
        }

        if resource.resource_id.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "ARN builder: resource_id cannot be empty".to_string(),
            });
        }

        Ok(WamiArn {
            service,
            tenant_path,
            wami_instance_id,
            cloud_mapping: self.cloud_mapping,
            resource,
        })
    }
}

impl WamiArn {
    /// Creates a new ARN builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::{WamiArn, Service};
    ///
    /// let arn = WamiArn::builder()
    ///     .service(Service::Iam)
    ///     .tenant("t1")
    ///     .wami_instance("999888777")
    ///     .resource("user", "77557755")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> ArnBuilder {
        ArnBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_wami_native() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_hierarchy(vec!["t1", "t2", "t3"])
            .wami_instance("999888777")
            .resource("user", "77557755")
            .build()
            .unwrap();

        assert_eq!(arn.service, Service::Iam);
        assert_eq!(arn.tenant_path.segments, vec!["t1", "t2", "t3"]);
        assert_eq!(arn.wami_instance_id, "999888777");
        assert_eq!(arn.cloud_mapping, None);
        assert_eq!(arn.resource.resource_type, "user");
        assert_eq!(arn.resource.resource_id, "77557755");
        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755"
        );
    }

    #[test]
    fn test_builder_cloud_synced() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_hierarchy(vec!["t1", "t2", "t3"])
            .wami_instance("999888777")
            .cloud_provider("aws", "223344556677")
            .resource("user", "77557755")
            .build()
            .unwrap();

        assert_eq!(arn.service, Service::Iam);
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().provider, "aws");
        assert_eq!(
            arn.cloud_mapping.as_ref().unwrap().account_id,
            "223344556677"
        );
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().region, None);
        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:global:user/77557755"
        );
    }

    #[test]
    fn test_builder_cloud_synced_with_region() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_hierarchy(vec!["t1", "t2", "t3"])
            .wami_instance("999888777")
            .cloud_provider_with_region("aws", "223344556677", "us-east-1")
            .resource("user", "77557755")
            .build()
            .unwrap();

        assert_eq!(
            arn.cloud_mapping.as_ref().unwrap().region,
            Some("us-east-1".to_string())
        );
        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:us-east-1:user/77557755"
        );
    }

    #[test]
    fn test_builder_region_method() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant("t1")
            .wami_instance("999888777")
            .cloud_provider("aws", "223344556677")
            .region("eu-west-1")
            .resource("user", "77557755")
            .build()
            .unwrap();

        assert_eq!(
            arn.cloud_mapping.as_ref().unwrap().region,
            Some("eu-west-1".to_string())
        );
    }

    #[test]
    fn test_builder_single_tenant() {
        let arn = WamiArn::builder()
            .service(Service::Sts)
            .tenant("t1")
            .wami_instance("111222333")
            .resource("session", "sess123")
            .build()
            .unwrap();

        assert_eq!(arn.tenant_path.segments, vec!["t1"]);
        assert_eq!(
            arn.to_string(),
            "arn:wami:sts:t1:wami:111222333:session/sess123"
        );
    }

    #[test]
    fn test_builder_service_str() {
        let arn = WamiArn::builder()
            .service_str("iam")
            .tenant("t1")
            .wami_instance("999888777")
            .resource("policy", "pol123")
            .build()
            .unwrap();

        assert_eq!(arn.service, Service::Iam);
    }

    #[test]
    fn test_builder_custom_service() {
        let arn = WamiArn::builder()
            .service_str("custom-service")
            .tenant("t1")
            .wami_instance("999888777")
            .resource("resource", "res123")
            .build()
            .unwrap();

        assert_eq!(arn.service, Service::Custom("custom-service".to_string()));
        assert_eq!(
            arn.to_string(),
            "arn:wami:custom-service:t1:wami:999888777:resource/res123"
        );
    }

    #[test]
    fn test_builder_no_cloud_mapping() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant("t1")
            .wami_instance("999888777")
            .cloud_provider("aws", "123456")
            .no_cloud_mapping()
            .resource("user", "77557755")
            .build()
            .unwrap();

        assert_eq!(arn.cloud_mapping, None);
    }

    #[test]
    fn test_builder_missing_service() {
        let result = WamiArn::builder()
            .tenant("t1")
            .wami_instance("999888777")
            .resource("user", "77557755")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("service is required"));
    }

    #[test]
    fn test_builder_missing_tenant() {
        let result = WamiArn::builder()
            .service(Service::Iam)
            .wami_instance("999888777")
            .resource("user", "77557755")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("tenant_path is required"));
    }

    #[test]
    fn test_builder_missing_instance() {
        let result = WamiArn::builder()
            .service(Service::Iam)
            .tenant("t1")
            .resource("user", "77557755")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("wami_instance_id is required"));
    }

    #[test]
    fn test_builder_missing_resource() {
        let result = WamiArn::builder()
            .service(Service::Iam)
            .tenant("t1")
            .wami_instance("999888777")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("resource is required"));
    }

    #[test]
    fn test_builder_empty_tenant_path() {
        let result = WamiArn::builder()
            .service(Service::Iam)
            .tenant_path(TenantPath::new(vec![]))
            .wami_instance("999888777")
            .resource("user", "77557755")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("tenant_path cannot be empty"));
    }

    #[test]
    fn test_builder_resource_obj() {
        let resource = Resource::new("role", "role123");
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant("t1")
            .wami_instance("999888777")
            .resource_obj(resource)
            .build()
            .unwrap();

        assert_eq!(arn.resource.resource_type, "role");
        assert_eq!(arn.resource.resource_id, "role123");
    }

    #[test]
    fn test_builder_cloud_mapping_obj() {
        let mapping = CloudMapping::new("gcp", "554433221");
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant("t1")
            .wami_instance("999888777")
            .cloud_mapping(mapping)
            .resource("user", "77557755")
            .build()
            .unwrap();

        assert_eq!(arn.cloud_mapping.as_ref().unwrap().provider, "gcp");
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().account_id, "554433221");
    }
}
