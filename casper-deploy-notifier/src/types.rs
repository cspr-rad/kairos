#[derive(Debug)]
pub struct Notification {
    pub deploy_hash: String,
    pub public_key: String,
    pub success: bool,
}
