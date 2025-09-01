# 설치 가이드

Warp CLI를 시스템에 설치하는 방법을 안내합니다.

## 📋 시스템 요구사항

### 최소 요구사항
- **Rust**: 1.70 이상
- **메모리**: 512MB RAM
- **디스크**: 10MB 여유 공간
- **네트워크**: 인터넷 연결 (API 호출용)

### 지원 운영체제
- ✅ Linux (x86_64, aarch64)
- ✅ macOS (Intel, Apple Silicon)
- ✅ Windows 10/11

## 🚀 설치 방법

### 방법 1: Cargo를 통한 설치 (추천)

가장 간단한 설치 방법입니다.

```bash
# Rust와 Cargo가 설치되어 있지 않다면:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Warp 설치
cargo install warp
```

#### 설치 확인
```bash
warp --version
```

### 방법 2: 소스 코드에서 빌드

최신 개발 버전을 사용하거나 직접 수정하려는 경우:

```bash
# 1. 저장소 클론
git clone https://github.com/pyhub-apps/warp.git
cd warp

# 2. 빌드
cargo build --release

# 3. 바이너리 설치 (선택사항)
# Linux/macOS
sudo cp target/release/warp /usr/local/bin/
chmod +x /usr/local/bin/warp

# Windows (PowerShell 관리자 권한)
Copy-Item target\release\warp.exe C:\Windows\System32\
```

### 방법 3: 사전 빌드된 바이너리 다운로드

GitHub Releases에서 운영체제에 맞는 바이너리를 다운로드합니다.

#### Linux (x86_64)
```bash
# 최신 릴리스 다운로드
wget https://github.com/pyhub-apps/warp/releases/latest/download/warp-linux-x86_64.tar.gz

# 압축 해제
tar -xzf warp-linux-x86_64.tar.gz

# 설치
sudo mv warp /usr/local/bin/
chmod +x /usr/local/bin/warp
```

#### macOS (Intel)
```bash
# 최신 릴리스 다운로드
curl -L https://github.com/pyhub-apps/warp/releases/latest/download/warp-macos-x86_64.tar.gz -o warp.tar.gz

# 압축 해제
tar -xzf warp.tar.gz

# 설치
sudo mv warp /usr/local/bin/
chmod +x /usr/local/bin/warp

# macOS 보안 설정 (필요 시)
xattr -d com.apple.quarantine /usr/local/bin/warp
```

#### macOS (Apple Silicon)
```bash
# 최신 릴리스 다운로드
curl -L https://github.com/pyhub-apps/warp/releases/latest/download/warp-macos-aarch64.tar.gz -o warp.tar.gz

# 압축 해제
tar -xzf warp.tar.gz

# 설치
sudo mv warp /usr/local/bin/
chmod +x /usr/local/bin/warp

# macOS 보안 설정 (필요 시)
xattr -d com.apple.quarantine /usr/local/bin/warp
```

#### Windows
1. [Releases 페이지](https://github.com/pyhub-apps/warp/releases)에서 `warp-windows-x86_64.zip` 다운로드
2. 압축 해제
3. `warp.exe`를 원하는 위치로 이동 (예: `C:\Program Files\warp\`)
4. 시스템 PATH에 추가:
   - 시스템 속성 → 고급 → 환경 변수
   - 시스템 변수에서 Path 선택 → 편집
   - 새로 만들기 → `C:\Program Files\warp\` 추가

## 🔧 설치 후 설정

### 1. 설치 확인
```bash
# 버전 확인
warp --version

# 도움말 확인
warp --help
```

### 2. 설정 초기화
```bash
# 설정 파일 생성
warp config init

# 설정 파일 위치 확인
warp config path
```

### 3. API 키 설정
```bash
# API 키 설정 (https://open.law.go.kr 에서 발급)
warp config set law.key YOUR_API_KEY

# 설정 확인
warp config get law.key
```

### 4. 첫 검색 테스트
```bash
# 간단한 검색으로 설치 확인
warp law "민법" --size 1
```

## 🐛 설치 문제 해결

### Rust/Cargo가 설치되지 않음
```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 환경 변수 적용
source $HOME/.cargo/env
```

### 권한 오류 (Linux/macOS)
```bash
# sudo 없이 설치하려면 사용자 디렉토리 사용
mkdir -p ~/.local/bin
cp target/release/warp ~/.local/bin/

# PATH에 추가 (.bashrc 또는 .zshrc)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Windows에서 실행 오류
- Visual C++ 재배포 가능 패키지가 필요할 수 있습니다
- [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist) 설치

### macOS 보안 경고
```bash
# "개발자를 확인할 수 없음" 오류 시
xattr -d com.apple.quarantine /usr/local/bin/warp
```

## 🔄 업데이트

### Cargo로 설치한 경우
```bash
cargo install warp --force
```

### 소스에서 빌드한 경우
```bash
cd warp
git pull
cargo build --release
sudo cp target/release/warp /usr/local/bin/
```

## 🗑️ 제거

### Cargo로 설치한 경우
```bash
cargo uninstall warp
```

### 수동 설치한 경우
```bash
# Linux/macOS
sudo rm /usr/local/bin/warp

# 설정 파일 제거 (선택사항)
rm -rf ~/.config/warp
```

## 📚 다음 단계

- [사용자 가이드](user-guide.md) - 기본 사용법 익히기
- [사용 예제](examples.md) - 실제 사용 시나리오 확인
- [문제 해결](troubleshooting.md) - 일반적인 문제 해결