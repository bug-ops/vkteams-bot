# Auto Listener Daemon - Архитектура автоматического режима прослушивания чата

## Цель и требования

Создать специальный daemon режим для vkteams-bot CLI и MCP, который:
- Автоматически запускается и начинает слушать чат 
- Сохраняет все события/сообщения в базу данных в реальном времени
- Работает в фоне как service/daemon
- Предоставляет данные для MCP клиента через готовую storage инфраструктуру
- Интегрируется с существующей архитектурой CLI-as-backend

## Текущее состояние

### ✅ Уже готово
- Longpoll event listener с backoff стратегиями (`crates/vkteams-bot/src/bot/longpoll.rs`)
- Storage manager с PostgreSQL + vector search (`crates/vkteams-bot-cli/src/storage/`)
- CLI команды для storage операций (search, stats, etc.)
- MCP сервер с CLI bridge для доступа к данным
- Unified configuration system

### 🔧 Нужно добавить
- Daemon режим запуска CLI
- Автоматическое сохранение событий в storage
- Process management и graceful shutdown
- Health monitoring и recovery

## Архитектура решения

```
┌──────────────────────┐       events       ┌─────────────────────┐
│   VK Teams API       │ ──────longpoll────► │  Auto Listener      │
│                      │                     │  Daemon Mode        │
└──────────────────────┘                     └─────────────────────┘
                                                        │
                                             auto-save events
                                                        ▼
┌──────────────────────┐                     ┌─────────────────────┐
│   MCP Client         │                     │  Storage Manager    │
│ (Claude, etc.)       │                     │                     │
└──────────────────────┘                     │ • PostgreSQL        │
           │                                 │ • Vector Search     │
           │ query historical data           │ • Embeddings        │
           ▼                                 └─────────────────────┘
┌──────────────────────┐    subprocess      ┌─────────────────────┐
│   MCP Server         │ ─────────────────► │  CLI Binary         │
│                      │                    │                     │
│ • search_semantic    │                    │ • search semantic   │
│ • search_text        │                    │ • search text       │
│ • get_database_stats │                    │ • database stats    │
│ • get_context        │                    │ • get context       │
└──────────────────────┘                    └─────────────────────┘
```

## Компоненты реализации

### 1. Daemon Command в CLI

**Файл**: `crates/vkteams-bot-cli/src/commands/daemon.rs`

```rust
use clap::Subcommand;
use crate::errors::prelude::Result as CliResult;
use crate::commands::{Command, OutputFormat, CommandResult};
use vkteams_bot::prelude::Bot;
use vkteams_bot::api::types::ResponseEventsGet;
use crate::storage::StorageManager;
use tokio::signal;
use tracing::{info, error, warn};

#[derive(Subcommand, Debug, Clone)]
pub enum DaemonCommands {
    /// Start automatic chat listener daemon
    #[command(name = "start")]
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
        
        /// PID file path
        #[arg(long, value_name = "PATH")]
        pid_file: Option<String>,
        
        /// Enable auto-storage of events
        #[arg(long)]
        auto_save: bool,
        
        /// Chat ID to listen (optional, uses config default)
        #[arg(long)]
        chat_id: Option<String>,
    },
    
    /// Stop daemon
    #[command(name = "stop")]
    Stop {
        /// PID file path
        #[arg(long, value_name = "PATH")]
        pid_file: Option<String>,
    },
    
    /// Check daemon status
    #[command(name = "status")]
    Status {
        /// PID file path
        #[arg(long, value_name = "PATH")]
        pid_file: Option<String>,
    },
}

#[async_trait::async_trait]
impl Command for DaemonCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            DaemonCommands::Start { foreground, auto_save, .. } => {
                if *foreground {
                    start_foreground_daemon(bot, *auto_save).await
                } else {
                    start_background_daemon(bot, *auto_save).await
                }
            }
            DaemonCommands::Stop { .. } => stop_daemon().await,
            DaemonCommands::Status { .. } => check_daemon_status().await,
        }
    }

    async fn execute_with_output(&self, bot: &Bot, format: &OutputFormat) -> CliResult<()> {
        let result = match self {
            DaemonCommands::Start { .. } => {
                self.execute(bot).await?;
                CommandResult::success_with_message("Daemon started successfully")
            }
            DaemonCommands::Stop { .. } => {
                self.execute(bot).await?;
                CommandResult::success_with_message("Daemon stopped successfully")
            }
            DaemonCommands::Status { .. } => {
                // TODO: Get actual daemon status
                CommandResult::success_with_data(serde_json::json!({
                    "status": "running",
                    "pid": 12345,
                    "uptime": "2h 30m",
                    "events_processed": 1250
                }))
            }
        };
        
        result.display(format)
    }

    fn name(&self) -> &'static str {
        "daemon"
    }
}

async fn start_foreground_daemon(bot: &Bot, auto_save: bool) -> CliResult<()> {
    info!("Starting VKTeams Bot daemon in foreground mode");
    
    let storage = if auto_save {
        Some(create_storage_manager().await?)
    } else {
        None
    };
    
    // Setup graceful shutdown
    let shutdown_signal = setup_shutdown_signal();
    
    tokio::select! {
        result = run_event_listener(bot, storage) => {
            match result {
                Ok(_) => info!("Event listener finished successfully"),
                Err(e) => error!("Event listener error: {}", e),
            }
        }
        _ = shutdown_signal => {
            info!("Received shutdown signal, stopping daemon...");
        }
    }
    
    Ok(())
}
```

### 2. Auto-Save Event Processor

**Файл**: `crates/vkteams-bot-cli/src/daemon/processor.rs`

```rust
use crate::storage::StorageManager;
use vkteams_bot::prelude::{Bot, ResponseEventsGet};
use vkteams_bot::api::types::EventMessage;
use crate::errors::prelude::Result as CliResult;
use tracing::{info, debug, error, warn};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct AutoSaveEventProcessor {
    storage: StorageManager,
    stats: Arc<ProcessorStats>,
}

#[derive(Default)]
pub struct ProcessorStats {
    events_processed: std::sync::atomic::AtomicU64,
    events_saved: std::sync::atomic::AtomicU64,
    events_failed: std::sync::atomic::AtomicU64,
    last_processed_time: std::sync::Mutex<Option<chrono::DateTime<chrono::Utc>>>,
}

impl AutoSaveEventProcessor {
    pub fn new(storage: StorageManager) -> Self {
        Self {
            storage,
            stats: Arc::new(ProcessorStats::default()),
        }
    }

    /// Process events batch and auto-save to storage
    pub async fn process_events(&self, bot: Bot, events: ResponseEventsGet) -> CliResult<()> {
        let event_count = events.events.len();
        debug!("Auto-saving {} events to storage", event_count);
        
        let start_time = std::time::Instant::now();
        let mut saved_count = 0;
        let mut failed_count = 0;

        // Process events in parallel batches for better performance
        let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent saves
        let futures: Vec<_> = events.events.into_iter().map(|event| {
            let storage = &self.storage;
            let semaphore = semaphore.clone();
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                match storage.process_event_with_embeddings(&event).await {
                    Ok(event_id) => {
                        debug!("Saved event {} with ID {}", event.event_id, event_id);
                        Ok(())
                    }
                    Err(e) => {
                        warn!("Failed to save event {}: {}", event.event_id, e);
                        Err(e)
                    }
                }
            }
        }).collect();

        let results = futures::future::join_all(futures).await;
        
        for result in results {
            match result {
                Ok(_) => saved_count += 1,
                Err(_) => failed_count += 1,
            }
        }

        let duration = start_time.elapsed();
        
        // Update statistics
        self.stats.events_processed.fetch_add(event_count as u64, std::sync::atomic::Ordering::Relaxed);
        self.stats.events_saved.fetch_add(saved_count, std::sync::atomic::Ordering::Relaxed);
        self.stats.events_failed.fetch_add(failed_count, std::sync::atomic::Ordering::Relaxed);
        
        if let Ok(mut last_time) = self.stats.last_processed_time.lock() {
            *last_time = Some(chrono::Utc::now());
        }

        info!(
            "Processed {} events in {:?}: {} saved, {} failed", 
            event_count, duration, saved_count, failed_count
        );

        if failed_count > 0 {
            warn!("{} events failed to save - check storage connection", failed_count);
        }

        Ok(())
    }

    /// Get processor statistics
    pub fn get_stats(&self) -> ProcessorStatsSnapshot {
        ProcessorStatsSnapshot {
            events_processed: self.stats.events_processed.load(std::sync::atomic::Ordering::Relaxed),
            events_saved: self.stats.events_saved.load(std::sync::atomic::Ordering::Relaxed),
            events_failed: self.stats.events_failed.load(std::sync::atomic::Ordering::Relaxed),
            last_processed_time: self.stats.last_processed_time.lock().unwrap().clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorStatsSnapshot {
    pub events_processed: u64,
    pub events_saved: u64,
    pub events_failed: u64,
    pub last_processed_time: Option<chrono::DateTime<chrono::Utc>>,
}
```

### 3. Daemon Management

**Файл**: `crates/vkteams-bot-cli/src/daemon/manager.rs`

```rust
use std::process::{Command, Stdio};
use std::fs;
use std::io::{self, Write};
use crate::errors::prelude::Result as CliResult;
use tracing::{info, error, debug};

pub struct DaemonManager {
    pid_file: String,
}

impl DaemonManager {
    pub fn new(pid_file: Option<String>) -> Self {
        let pid_file = pid_file.unwrap_or_else(|| {
            format!("/tmp/vkteams-bot-daemon-{}.pid", std::process::id())
        });
        
        Self { pid_file }
    }

    /// Start daemon process in background
    pub async fn start_background(&self, args: Vec<String>) -> CliResult<()> {
        // Check if already running
        if self.is_running().await? {
            return Err(crate::errors::prelude::CliError::DaemonAlreadyRunning);
        }

        // Get current executable path
        let exe_path = std::env::current_exe()
            .map_err(|e| crate::errors::prelude::CliError::System(format!("Cannot get executable path: {}", e)))?;

        // Spawn daemon process
        let mut cmd = Command::new(exe_path);
        cmd.args(&args)
           .arg("--foreground") // Use foreground mode in spawned process
           .stdin(Stdio::null())
           .stdout(Stdio::null())
           .stderr(Stdio::null());

        let child = cmd.spawn()
            .map_err(|e| crate::errors::prelude::CliError::System(format!("Failed to spawn daemon: {}", e)))?;

        // Save PID
        self.save_pid(child.id()).await?;
        
        info!("Daemon started with PID {}", child.id());
        Ok(())
    }

    /// Stop daemon process
    pub async fn stop(&self) -> CliResult<()> {
        let pid = self.read_pid().await?;
        
        if pid == 0 {
            return Err(crate::errors::prelude::CliError::DaemonNotRunning);
        }

        // Send SIGTERM to daemon
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            
            let pid = Pid::from_raw(pid as i32);
            match signal::kill(pid, Signal::SIGTERM) {
                Ok(_) => {
                    info!("Sent SIGTERM to daemon PID {}", pid);
                    
                    // Wait a bit and check if process is still running
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    
                    if !self.is_running().await? {
                        self.cleanup_pid_file().await?;
                        info!("Daemon stopped successfully");
                    } else {
                        warn!("Daemon may still be running after SIGTERM");
                    }
                }
                Err(e) => {
                    error!("Failed to send SIGTERM: {}", e);
                    return Err(crate::errors::prelude::CliError::System(format!("Failed to stop daemon: {}", e)));
                }
            }
        }
        
        #[cfg(windows)]
        {
            // Windows implementation would go here
            todo!("Windows daemon management not implemented");
        }

        Ok(())
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> CliResult<bool> {
        let pid = self.read_pid().await?;
        
        if pid == 0 {
            return Ok(false);
        }

        // Check if process with this PID exists
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            
            let pid = Pid::from_raw(pid as i32);
            match signal::kill(pid, Signal::SIGUSR1) {
                Ok(_) => Ok(true),
                Err(nix::errno::Errno::ESRCH) => {
                    // Process doesn't exist, cleanup stale PID file
                    debug!("Stale PID file detected, cleaning up");
                    self.cleanup_pid_file().await?;
                    Ok(false)
                }
                Err(e) => {
                    error!("Error checking process: {}", e);
                    Ok(false)
                }
            }
        }
        
        #[cfg(windows)]
        {
            // Windows implementation
            Ok(false) // Placeholder
        }
    }

    async fn save_pid(&self, pid: u32) -> CliResult<()> {
        fs::write(&self.pid_file, pid.to_string())
            .map_err(|e| crate::errors::prelude::CliError::System(format!("Cannot write PID file: {}", e)))?;
        Ok(())
    }

    async fn read_pid(&self) -> CliResult<u32> {
        match fs::read_to_string(&self.pid_file) {
            Ok(content) => {
                content.trim().parse()
                    .map_err(|e| crate::errors::prelude::CliError::System(format!("Invalid PID file content: {}", e)))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(0),
            Err(e) => Err(crate::errors::prelude::CliError::System(format!("Cannot read PID file: {}", e))),
        }
    }

    async fn cleanup_pid_file(&self) -> CliResult<()> {
        if fs::metadata(&self.pid_file).is_ok() {
            fs::remove_file(&self.pid_file)
                .map_err(|e| crate::errors::prelude::CliError::System(format!("Cannot remove PID file: {}", e)))?;
        }
        Ok(())
    }
}
```

### 4. Интеграция в основной CLI

**Добавить в `crates/vkteams-bot-cli/src/commands/mod.rs`**:

```rust
pub mod daemon;

// В enum Commands добавить:
#[command(flatten)]
Daemon(daemon::DaemonCommands),

// В impl Command for Commands добавить:
Commands::Daemon(cmd) => cmd.execute(bot).await,
```

**Добавить в `crates/vkteams-bot-cli/src/cli.rs`**:

```rust
/// Additional daemon-specific options
#[derive(Parser, Debug)]
pub struct Cli {
    // ... existing fields ...
    
    /// Enable daemon mode
    #[arg(long, global = true)]
    pub daemon: bool,
    
    /// PID file for daemon mode
    #[arg(long, global = true, value_name = "PATH")]
    pub pid_file: Option<String>,
}
```

## Использование и примеры

### Запуск daemon в foreground режиме

```bash
# Запуск с автоматическим сохранением в БД
vkteams-bot-cli daemon start --foreground --auto-save

# Запуск только для прослушивания (без сохранения)
vkteams-bot-cli daemon start --foreground

# Запуск для конкретного чата
vkteams-bot-cli daemon start --foreground --auto-save --chat-id "chat_123"
```

### Запуск daemon в background режиме

```bash
# Запуск в фоне
vkteams-bot-cli daemon start --auto-save

# Остановка daemon
vkteams-bot-cli daemon stop

# Проверка статуса
vkteams-bot-cli daemon status --output json
```

### Интеграция с MCP

После запуска daemon, MCP клиенты смогут получать доступ к историческим данным:

```python
# MCP client может использовать:
mcp_client.call_tool("search_semantic", {
    "query": "важные планы на завтра",
    "limit": 10
})

mcp_client.call_tool("get_database_stats", {
    "chat_id": "chat_123"
})

mcp_client.call_tool("get_context", {
    "context_type": "recent",
    "timeframe": "2h"
})
```

## Мониторинг и логирование

### Health Check Endpoint

```bash
# Проверка здоровья всех компонентов
vkteams-bot-cli diagnostic health --output json

# Результат:
{
  "success": true,
  "data": {
    "api_connection": "ok",
    "database": "ok", 
    "vector_store": "ok",
    "daemon": {
      "status": "running",
      "uptime": "2h 30m",
      "events_processed": 1250,
      "last_activity": "2024-01-01T10:30:00Z"
    }
  }
}
```

### Логирование

Daemon будет логировать:
- Startup/shutdown события
- Статистику обработки событий
- Ошибки сохранения в БД
- Performance метрики

## Развертывание

### Systemd Service (Linux)

```ini
[Unit]
Description=VKTeams Bot Auto Listener Daemon
After=network.target postgresql.service

[Service]
Type=forking
User=vkteams
Group=vkteams
ExecStart=/usr/local/bin/vkteams-bot-cli daemon start --auto-save
ExecStop=/usr/local/bin/vkteams-bot-cli daemon stop
PIDFile=/var/run/vkteams-bot-daemon.pid
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Docker Compose

```yaml
version: '3.8'
services:
  vkteams-daemon:
    image: vkteams-bot:latest
    command: ["daemon", "start", "--foreground", "--auto-save"]
    environment:
      - VKTEAMS_BOT_CONFIG=/config/bot.toml
      - VKTEAMS_BOT_CHAT_ID=chat_123
    volumes:
      - ./config:/config
    depends_on:
      - postgres
    restart: unless-stopped
    
  vkteams-mcp:
    image: vkteams-bot-mcp:latest
    ports:
      - "8080:8080"
    environment:
      - VKTEAMS_BOT_CONFIG=/config/bot.toml
    volumes:
      - ./config:/config
    depends_on:
      - postgres
      - vkteams-daemon
    restart: unless-stopped
```

## Преимущества архитектуры

1. **Разделение ответственности**: Daemon слушает и сохраняет, MCP предоставляет доступ
2. **Масштабируемость**: Множественные MCP инстансы могут читать из общей БД
3. **Надежность**: Автоматический restart, graceful shutdown, error recovery
4. **Мониторинг**: Встроенная статистика и health checks
5. **Простота развертывания**: Один конфиг для всех компонентов

## Статус реализации

### ✅ Выполненные шаги

1. ✅ **Архитектура спроектирована** - создан детальный план реализации
2. ✅ **Daemon commands реализованы** - добавлены start/stop/status команды
3. ✅ **Auto-save processor создан** - AutoSaveEventProcessor с статистикой
4. ✅ **CLI интеграция завершена** - команды доступны через vkteams-bot-cli
5. ✅ **Error handling расширен** - новые типы ошибок для daemon
6. ✅ **Graceful shutdown** - обработка сигналов SIGTERM/SIGINT/Ctrl+C

### 🔧 Текущие задачи

7. 🔧 **Storage интеграция** - временно отключена, нужно исправить compilation errors:
   - Создать недостающий schema.rs
   - Настроить SQLX для offline mode
   - Исправить type mismatches в models
   
8. 🔧 **Background daemon mode** - реализовать полноценный daemon:
   - PID file management
   - Process isolation
   - Proper daemonization

### 📋 Будущие задачи

9. 📋 **Мониторинг и health checks** - добавить метрики и диагностику
10. 🧪 **Написать тесты** - unit и integration тесты для новой функциональности
11. 📝 **Обновить документацию** - README, примеры использования
12. 🚀 **Production deployment** - systemd service, Docker integration