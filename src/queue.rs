use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Queue type alias for easier imports
pub type Queue = Arc<Mutex<Vec<String>>>;
pub type QueueMap = Arc<Mutex<HashMap<u64, Queue>>>;

