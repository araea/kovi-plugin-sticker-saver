kovi-plugin-sticker-saver
=========================

[<img alt="github" src="https://img.shields.io/badge/github-araea/kovi__plugin__sticker__saver-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/araea/kovi-plugin-sticker-saver)
[<img alt="crates.io" src="https://img.shields.io/crates/v/kovi-plugin-sticker-saver.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/kovi-plugin-sticker-saver)

Kovi 的表情包提取插件。专为解决手机 QQ 用户无法直接保存表情包原图的痛点设计。

## 特性

- 📱 **手机党福音** - 完美解决手机端无法保存表情包到相册的问题
- ⚡ **简单高效** - 仅需【引用】表情包并发送指令即可获取原图
- 🛠️ **高度自定义** - 支持自定义触发指令（如：收、偷、转图片）
- 🧹 **自动清理** - 支持提取成功后自动撤回指令，保持群聊版面整洁
- 🖼️ **批量提取** - 支持一次性提取引用消息中的多张图片

## 前置

1. 创建 Kovi 项目
2. 执行 `cargo kovi add sticker-saver`
3. 在 `src/main.rs` 中添加 `kovi_plugin_sticker_saver`

## 快速开始

1. **发现想要的表情**：在群聊或私聊中看到想要保存的表情包。
2. **引用并发送指令**：长按该表情包选择 **引用**，输入指令 `收` (默认指令) 并发送。
3. **保存图片**：机器人会立即回复该表情包的原图，点击图片保存即可。

## 配置

资源目录：`data/kovi-plugin-sticker-saver/*`

> 首次运行时自动生成 `config.toml`。

### `config.toml`

```toml
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
```

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `enabled` | bool | 插件总开关 | `true` |
| `commands` | Vec<String> | 触发插件的指令列表 | `["表情转图片", "收", ...]` |
| `prefixes` | Vec<String> | 指令前缀（可选） | `[]` |
| `recall_command` | bool | 机器人回复后是否撤回用户的指令 | `false` |

## 常见问题

**Q: 发送指令后机器人回复“请引用你想要保存的表情包”？**  
A: 你必须使用 QQ 的 **引用 (Reply)** 功能，单纯发送指令机器人不知道你要获取哪张图片。

**Q: 为什么有的表情包无法提取？**  
A: 插件支持绝大多数图片和 GIF 格式的表情。部分商城付费表情或特殊 XML 卡片表情可能无法提取。

## 致谢

- [Kovi](https://kovi.threkork.com/)

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
