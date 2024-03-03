# Sway

[![构建](https://github.com/FuelLabs/sway/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/sway/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/forc?label=最新版本)](https://crates.io/crates/forc)
[![文档](https://docs.rs/forc/badge.svg)](https://docs.rs/forc/)
[![discord](https://img.shields.io/badge/在-discord上聊天-橙色?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

Sway是为Fuel区块链开发的一种语言。它深受Rust的启发，旨在将现代语言开发和性能引入到区块链生态系统中。

## 文档

有关用户文档，包括安装发布版本的信息，请参阅Sway Book：<https://fuellabs.github.io/sway/latest/>。

有关Sway标准库文档，请参阅：<https://fuellabs.github.io/sway/master/std/>

还可以查看Sway编程语言的技术参考：<https://fuellabs.github.io/sway/master/reference/>

## 从源代码构建

此部分用于开发Sway编译器和工具链。要开发合同并使用Sway，请参阅上面的文档部分。

### 依赖关系

Sway是用Rust构建的。首先，请按照<https://www.rust-lang.org/tools/install>上的说明安装Rust工具链。然后配置您的Rust工具链以使用Rust“stable”：

```sh
rustup default stable

如果尚未完成，请通过将以下行添加到~/.profile并重新启动shell会话来将Cargo二进制目录添加到您的PATH中。

export PATH="${HOME}/.cargo/bin:${PATH}"

构建Forc
克隆存储库并构建Sway工具链：

git clone git@github.com:FuelLabs/sway.git
cd sway
cargo build

确认Sway工具链已成功构建：

cargo run --bin forc -- --help

参与Sway开发
我们欢迎对Sway的贡献！

请参阅Sway书中的Contributing To Sway(https://fuellabs.github.io/sway/master/book/reference/contributing_to_sway.html) 部分，以获取开始的指南和说明。
