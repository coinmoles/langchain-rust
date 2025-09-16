use rmcp::RoleClient;
use rmcp::model::InitializeRequestParam;
use rmcp::service::RunningService;

pub type McpService = RunningService<RoleClient, InitializeRequestParam>;
