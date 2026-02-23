# RsOcr

ddddocr 的 Rust 实现，基于 ONNX Runtime 进行验证码识别。支持多个 ONNX 模型和字符集，可识别简单文本验证码。

## 编译和运行

### 编译

```bash
cargo build --release
```

ONNX Runtime 已**静态链接**到可执行文件中，编译产物为独立 exe，无需额外的动态库文件。首次编译会从网络下载 ONNX Runtime 静态库（需网络连接）。

### 运行

```bash
./target/release/captcha-ocr [OPTIONS]
```

## 参数说明

| 参数 | 说明 | 默认值 |
|------|:----:|:------:|
| `-m <path>` | ONNX 模型路径 | `./common.onnx` |
| `-c <path>` | 字符集文件路径 | 内置 charset3.json |
| `-i <path>` | 识别单张图片 | - |
| `-d <path>` | 识别目录下所有图片 | - |
| `-b <base64>` |       识别转为base64字符串的图片       |         -          |
| `-f`          | 显示文件名（格式：filename -> result） |    不显示文件名    |
| `-h` | 显示帮助信息 | - |

**说明**：不指定 `-i` 或 `-d` 时，默认扫描当前目录下的所有图片。

## 支持的模型和字符集

### 推荐配置（新模型）

```bash
./target/release/captcha-ocr -m common.onnx -i image.jpg
```

- **模型**：`common.onnx`（更新更强）
- **字符集**：`charset3.json`（默认，无需指定）
- **识别精度**：更高，可处理粘连字符（如 45WJ）

### 传统配置（旧模型）

```bash
./target/release/captcha-ocr -m common_old.onnx -c charset.json -i image.jpg
```

- **模型**：`common_old.onnx`（轻量级）
- **字符集**：`charset.json`（旧字符集）
- **识别精度**：较低，不可处理粘连字符（识别为 45w）

## 使用示例

```bash
# 推荐：识别单张图片（默认使用新模型 common.onnx 和 charset3.json<已内置> ）
./target/release/captcha-ocr -i image.jpg

# 推荐：识别目录，显示文件名
./target/release/captcha-ocr -f -d ./images/

# 传统：使用旧模型识别，指定旧字符集
./target/release/captcha-ocr -m common_old.onnx -c charset.json -i image.jpg

# 扫描当前目录（默认使用新模型）
./target/release/captcha-ocr
```

```
PS E:\coding\Rust\captcha> .\captcha-ocr.exe -b "data:image/jpg;base64,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"

输出：
45WJ
```



## 识别效果对比

### 新模型 (common.onnx + charset3.json<已内置>)

| 图片 | 识别结果 |
|:----:|:-------:|
| ![45WJ](./README/00e78300cff5a79b6592e2abcb0b4b8b.jpg) | **45WJ** ✓ |
| ![9yKL](./README/00aab55810b3e0827b101876957df5bf.jpg) | **9yKL** ✓ |

### 旧模型 (common_old.onnx + charset.json)

| 图片 | 识别结果 |
|:----:|:-------:|
| ![45WJ](./README/00e78300cff5a79b6592e2abcb0b4b8b.jpg) | **45w** （缺 J） |
| ![9yKL](./README/00aab55810b3e0827b101876957df5bf.jpg) | **9yKL** |

## 支持的图片格式

- JPEG / JPG
- PNG
- BMP
- GIF
- base64

## 其他

- 首次编译需要下载 ONNX Runtime 静态库
- 生成的 exe 大小约 15-25MB（包含静态链接的 ONNX Runtime）
- 旧模型和新模型使用不同的字符集，混合使用会导致乱码

