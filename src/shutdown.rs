use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::signal::ctrl_c;
use tokio::select;

use crate::cleanup::cleanup_all_temp_files;
use crate::queue::QueueMap;

/// Shutdown handler for graceful termination
pub struct ShutdownHandler {
    queues: QueueMap,
    is_shutting_down: Arc<Mutex<bool>>,
}

impl ShutdownHandler {
    pub fn new(queues: QueueMap) -> Self {
        Self {
            queues,
            is_shutting_down: Arc::new(Mutex::new(false)),
        }
    }

    /// Check if shutdown is in progress
    pub async fn is_shutting_down(&self) -> bool {
        *self.is_shutting_down.lock().await
    }

    /// Wait for shutdown signal (SIGINT or SIGTERM)
    pub async fn wait_for_shutdown(&self) {
        // Wait for Ctrl+C
        match ctrl_c().await {
            Ok(()) => {
                println!("\n🛑 Received shutdown signal (Ctrl+C)...");
            }
            Err(err) => {
                eprintln!("❌ Unable to listen for shutdown signal: {}", err);
            }
        }
    }

    /// Perform graceful shutdown
    pub async fn shutdown(&self) {
        // Set shutdown flag
        *self.is_shutting_down.lock().await = true;

        println!("🧹 Cleaning up...");

        // Clear all queues
        let mut queues = self.queues.lock().await;
        let queue_count = queues.len();
        queues.clear();
        drop(queues);

        println!("📝 Cleared {} queue(s)", queue_count);

        // Clean up temp files
        cleanup_all_temp_files().await;

        println!("✅ Shutdown complete. Goodbye!");
    }

    /// Run the shutdown handler - waits for signal then shuts down
    pub async fn run(&self) {
        self.wait_for_shutdown().await;
        self.shutdown().await;
    }
}
