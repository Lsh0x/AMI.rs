//! Service-Specific Credential Builder

// TODO: Fix after adding ServiceSpecificCredential to ResourceType enum
// use crate::provider::{CloudProvider, ResourceType};
// pub fn build_service_credential(
//     user_name: String,
//     service_name: String,
//     provider: &dyn CloudProvider,
//     account_id: &str,
// ) -> ServiceSpecificCredential {
//     let credential_id = provider.generate_resource_id(ResourceType::ServiceSpecificCredential);
//     let wami_arn = provider.generate_wami_arn(
//         ResourceType::ServiceSpecificCredential,
//         account_id,
//         "/",
//         &credential_id,
//     );
//
//     // Generate service-specific password (for CodeCommit, IoT, etc.)
//     let password = uuid::Uuid::new_v4().to_string().replace('-', "");
//
//     ServiceSpecificCredential {
//         user_name,
//         service_name,
//         service_user_name: format!("{}-{}", user_name, &credential_id[..8]),
//         service_password: password,
//         service_specific_credential_id: credential_id,
//         status: "Active".to_string(),
//         create_date: chrono::Utc::now(),
//         wami_arn,
//         providers: vec![],
//     }
// }
