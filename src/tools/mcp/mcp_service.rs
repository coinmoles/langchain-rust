use rmcp::{model::InitializeRequestParam, service::RunningService, RoleClient};

pub type McpService = RunningService<RoleClient, InitializeRequestParam>;
