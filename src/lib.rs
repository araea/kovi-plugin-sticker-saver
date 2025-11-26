//! kovi-plugin-sticker-saver
//!
//! 这是一个贴心的辅助插件，专为手机 QQ 用户设计。
//! 当用户【引用】别人的表情包并发送指令（如“表情转图片”或“收”）时，
//! 插件会将表情包作为普通图片发送回来，方便用户保存到手机相册。

// =============================
//          Modules
// =============================

mod config {
    use kovi::toml;
    use kovi::utils::{load_toml_data, save_toml_data};
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    pub static CONFIG: std::sync::OnceLock<Arc<RwLock<Config>>> = std::sync::OnceLock::new();

    pub fn get() -> Arc<RwLock<Config>> {
        CONFIG.get().cloned().expect("Config not initialized")
    }

    const DEFAULT_CONFIG: &str = r#"
# 插件开关
enabled = true

# 触发指令列表
# 支持多个别名，越短越方便手机输入
commands = ["表情转图片", "收", "偷", "存表情"]

# 指令前缀
# 示例：["/", "."]
# 如果留空 []，则不需要前缀直接发送指令即可
prefixes = []

# 是否在发送图片后撤回用户的指令消息？
# true: 保持群聊版面整洁
# false: 不撤回
recall_command = false
"#;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Config {
        pub enabled: bool,
        pub commands: Vec<String>,
        pub prefixes: Vec<String>,
        pub recall_command: bool,

        #[serde(skip)]
        config_path: PathBuf,
    }

    impl Config {
        pub fn load(data_dir: PathBuf) -> Arc<RwLock<Self>> {
            if !data_dir.exists() {
                std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
            }
            let config_path = data_dir.join("config.toml");

            let default: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
            let mut config = load_toml_data(default, config_path.clone()).unwrap_or_else(|_| {
                let c: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
                c
            });

            config.config_path = config_path;

            Arc::new(RwLock::new(config))
        }

        pub fn save(&self) {
            let _ = save_toml_data(self, &self.config_path);
        }
    }
}

mod utils {
    use kovi::MsgEvent;
    use std::sync::Arc;

    /// 解析指令，判断是否命中配置的命令
    pub fn parse_command<'a>(text: &'a str, prefixes: &[String], commands: &[String]) -> bool {
        let text = text.trim();

        // 如果有前缀配置，先去前缀
        let clean_text = if !prefixes.is_empty() {
            let mut found = None;
            // 排序前缀，优先匹配长前缀
            let mut sorted_prefixes = prefixes.to_vec();
            sorted_prefixes.sort_by_key(|b| std::cmp::Reverse(b.len()));

            for p in sorted_prefixes {
                if text.starts_with(&p) {
                    found = Some(&text[p.len()..]);
                    break;
                }
            }
            match found {
                Some(t) => t.trim(),
                None => return false, // 配置了前缀但没匹配到，直接忽略
            }
        } else {
            text
        };

        // 匹配指令
        commands.contains(&clean_text.to_string())
    }

    /// 获取引用消息的消息ID
    pub fn get_reply_id(event: &Arc<MsgEvent>) -> Option<i32> {
        // 遍历消息段，寻找 reply 类型
        for seg in event.message.iter() {
            if seg.type_ == "reply" {
                if let Some(id_val) = seg.data.get("id") {
                    if let Some(id_str) = id_val.as_str() {
                        return id_str.parse::<i32>().ok();
                    }
                    if let Some(id_int) = id_val.as_i64() {
                        return Some(id_int as i32);
                    }
                }
            }
        }
        None
    }

    /// 提取消息中的所有图片URL
    /// 支持 image 类型，部分 mface (如果是转成 image 格式的话)
    pub fn extract_image_urls(message_segments: &Vec<kovi::serde_json::Value>) -> Vec<String> {
        let mut urls = Vec::new();
        for seg in message_segments {
            // OneBot v11 标准图片类型
            if let Some(type_) = seg.get("type").and_then(|t| t.as_str()) {
                if type_ == "image" {
                    if let Some(data) = seg.get("data") {
                        if let Some(url) = data.get("url").and_then(|u| u.as_str()) {
                            urls.push(url.to_string());
                        }
                    }
                }
            }
        }
        urls
    }
}

// =============================
//      Main Plugin Logic
// =============================

use kovi::PluginBuilder;

#[kovi::plugin]
async fn main() {
    let bot = PluginBuilder::get_runtime_bot();
    let data_dir = bot.get_data_path();

    // 初始化配置
    let config_lock = config::Config::load(data_dir.clone());
    config::CONFIG.set(config_lock.clone()).ok();

    PluginBuilder::on_msg(move |event| {
        let bot = bot.clone();
        let config_lock = config_lock.clone();

        async move {
            // 1. 获取纯文本内容（用于判断指令）
            let text = match event.borrow_text() {
                Some(t) => t,
                None => return, // 没文本肯定不是指令
            };

            // 2. 读取配置
            let (commands, prefixes, recall_cmd, enabled) = {
                let cfg = config_lock.read().unwrap();
                (
                    cfg.commands.clone(),
                    cfg.prefixes.clone(),
                    cfg.recall_command,
                    cfg.enabled,
                )
            };

            if !enabled {
                return;
            }

            // 3. 判断是否是插件指令
            if utils::parse_command(text, &prefixes, &commands) {
                // 4. 获取引用的消息 ID
                let reply_id = match utils::get_reply_id(&event) {
                    Some(id) => id,
                    None => {
                        event.reply("❌ 请【引用】你想要保存的表情包，然后发送此指令。");
                        return;
                    }
                };

                // 5. 获取原消息内容
                match bot.get_msg(reply_id).await {
                    Ok(res) => {
                        // 提取 message 字段（包含消息段数组）
                        if let Some(msg_data) = res.data.get("message").and_then(|v| v.as_array()) {
                            // 6. 提取图片链接
                            let urls = utils::extract_image_urls(msg_data);

                            if urls.is_empty() {
                                event.reply(
                                    "⚠️ 检测不到图片或表情，可能是非常规表情格式（如商城表情）。",
                                );
                            } else {
                                // 7. 构建回复消息
                                let mut reply_msg = kovi::Message::new();
                                // 可选：回复时引用用户的指令，提示来源
                                reply_msg = reply_msg.add_reply(event.message_id);
                                reply_msg = reply_msg.add_text("✅ 图片提取成功：\n");

                                for url in urls {
                                    reply_msg = reply_msg.add_image(&url);
                                }

                                event.reply(reply_msg);

                                // 8. 如果配置了撤回指令，且在群里
                                if recall_cmd && event.group_id.is_some() {
                                    bot.delete_msg(event.message_id);
                                }
                            }
                        } else {
                            event.reply("❌ 获取原消息失败，消息可能已过期。");
                        }
                    }
                    Err(_) => {
                        event.reply("❌ 获取引用消息失败，API 请求错误。");
                    }
                }
            }
        }
    });

    // 插件卸载或停止时保存配置
    PluginBuilder::drop(move || {
        let config_lock = config::get();
        async move {
            let config = {
                let guard = config_lock.read().unwrap();
                guard.clone()
            };
            config.save();
        }
    });
}
