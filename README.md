# RsOcr

ddddocr 的 Rust 实现，基于 ONNX Runtime 进行验证码识别。

## 原理

- 加载 ddddocr 的 `common_old.onnx` 模型
- 图像预处理：缩放到高度 64px、转灰度、归一化到 [0, 1]
- ONNX 推理 + CTC 解码（argmax + 去重）
- 字符集（8210 个字符）在编译时嵌入二进制，运行时无需额外文件

### ONNX Runtime

运行时需要 ONNX Runtime 动态库和模型文件。

| 平台 | 库文件 | 获取方式 |
|------|--------|---------|
| Windows | `onnxruntime.dll`, `onnxruntime_providers_shared.dll` | 从 Python 的 `onnxruntime` 包中复制，或从 [GitHub Releases](https://github.com/microsoft/onnxruntime/releases) 下载 |
| Linux | `libonnxruntime.so` | 同上，或 `apt install libonnxruntime-dev` |

将动态库放在以下任一位置：
- 可执行文件同目录
- 系统库路径（Linux: `LD_LIBRARY_PATH`，Windows: `PATH`）

### 模型文件

将 `common_old.onnx` 放在当前工作目录或可执行文件同目录，也可通过 `-m` 参数指定路径。

## 用法

```
captcha-ocr [OPTIONS]
```

### 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-i <path>` | 识别单张图片 | - |
| `-d <path>` | 识别目录下所有图片 | - |
| `-m <path>` | 指定 ONNX 模型路径 | `./common_old.onnx` |
| `-h` | 显示帮助 | - |

不指定 `-i` 或 `-d` 时，默认扫描当前目录下的所有图片。

### 示例

```bash
# 识别单张图片
captcha-ocr -i captcha.jpg

# 识别目录下所有图片
captcha-ocr -d ./images/

# 指定模型路径
captcha-ocr -m /path/to/common_old.onnx -d ./images/

# 不加参数，识别当前目录下所有图片
captcha-ocr
```

### 输出

有时可能识别不准确，如下：

![00aab55810b3e0827b101876957df5bf](./README/00aab55810b3e0827b101876957df5bf.jpg) ![00afce3506303922d506ddfbc52ee9d4](./README/00afce3506303922d506ddfbc52ee9d4.jpg) ![00e78300cff5a79b6592e2abcb0b4b8b](./README/00e78300cff5a79b6592e2abcb0b4b8b.jpg)

```
00aab55810b3e0827b101876957df5bf.jpg -> 9yKL
00afce3506303922d506ddfbc52ee9d4.jpg -> nVwa
00e78300cff5a79b6592e2abcb0b4b8b.jpg -> 45w

共识别 3 张验证码
```

仅能识别这种简单的验证码

## 支持的图片格式

jpg / jpeg / png / bmp / gif

## 项目结构

```
rust_version/
├── Cargo.toml
├── common_old.onnx          # ONNX 模型（需自行放置）
├── charset.json              # 字符集数据（编译时嵌入）
├── README.md
└── src/
    ├── charset.rs            # 字符集加载
    └── main.rs               # 预处理、推理、CTC 解码、CLI
```

