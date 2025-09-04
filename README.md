# Warp CLI - Korean Legal Information CLI (Rust)

한국 법령 정보를 터미널에서 손쉽게 검색할 수 있는 CLI 도구입니다.

## ✨ 특징

- 🚀 **고성능**: Rust로 구현되어 빠른 실행 속도와 낮은 메모리 사용량
- 📚 **5개 법령 API 지원**:
  - 국가법령정보센터 (NLIC)
  - 자치법규정보시스템 (ELIS)
  - 판례 (PREC)
  - 행정규칙 (ADMRUL)
  - 법령해석례 (EXPC)
- 🔍 **통합 검색**: 모든 API를 동시에 검색하는 병렬 처리
- 🎯 **고급 필터링**: 법령 종류, 부처, 날짜, 상태별 정밀 검색
- 📋 **필터 프리셋**: 자주 사용하는 검색 조건을 저장하고 재사용
- 🎨 **6가지 출력 형식**: Table, JSON, Markdown, CSV, HTML, HtmlSimple
- ⚡ **비동기 처리**: Tokio 기반 병렬 API 호출
- 🔄 **재시도 로직**: 지수 백오프를 통한 안정적인 API 호출
- 📊 **실시간 성능 모니터링**: 대시보드, 히스토리 분석, 다양한 리포트 형식 (JSON/CSV/HTML)
- 🔒 **보안**: API 키를 안전하게 관리 (파일 권한 0600)

## 📚 문서

- **[한국어 문서](docs/ko/)** - 상세한 한국어 가이드
  - [설치 가이드](docs/ko/installation.md) - 시스템별 설치 방법
  - [사용자 가이드](docs/ko/user-guide.md) - 전체 기능 설명
  - [사용 예제](docs/ko/examples.md) - 실제 사용 시나리오
  - [문제 해결](docs/ko/troubleshooting.md) - 일반적인 문제 해결
  - [FAQ](docs/ko/faq.md) - 자주 묻는 질문

## 📦 설치

자세한 설치 방법은 [설치 가이드](docs/ko/installation.md)를 참조하세요.

### 빠른 설치

```bash
# 라이센스 구매 후 정식 버전 다운로드
# 구매 문의: me@pyhub.kr

# 평가판 다운로드 (30일 제한)
wget https://github.com/pyhub-apps/warp/releases/latest/download/warp-trial.tar.gz
tar -xzf warp-trial.tar.gz
cd warp-trial

# 또는 라이센스 구매자는 소스에서 빌드 가능
git clone https://github.com/pyhub-apps/warp.git
cd warp
cargo build --release
```

### Shell 자동완성 설치

빌드 후 자동완성을 설치하면 명령어와 옵션을 쉽게 입력할 수 있습니다:

```bash
# 자동 설치 (현재 셸 감지)
./install-completions.sh

# 특정 셸 설치
./install-completions.sh --bash
./install-completions.sh --zsh
./install-completions.sh --fish
./install-completions.sh --powershell

# 모든 셸에 설치
./install-completions.sh --all

# 수동 생성 (셸 자동 감지)
warp completions              # 현재 셸을 자동으로 감지하여 대화형으로 진행

# 수동 생성 (특정 셸 지정)
warp completions bash > ~/.bash_completion.d/warp.bash
warp completions zsh > ~/.zsh/completions/_warp
warp completions fish > ~/.config/fish/completions/warp.fish
warp completions powershell > warp.ps1
```

> **참고**: Windows 명령 프롬프트(CMD)는 자동완성을 지원하지 않습니다.
> PowerShell 사용을 권장합니다.

## 🚀 빠른 시작

### 1. API 키 발급

[https://open.law.go.kr](https://open.law.go.kr)에서 무료 API 키를 발급받으세요.

### 2. API 키 설정

```bash
# 설정 초기화
warp config init

# 공통 API 키 설정 (모든 API에 기본값으로 사용)
warp config set law.key YOUR_API_KEY

# API별 개별 키 설정 (선택사항)
warp config set law.nlic.key YOUR_NLIC_KEY   # 국가법령정보센터 전용
warp config set law.elis.key YOUR_ELIS_KEY   # 자치법규정보시스템 전용 (별도 키 필요)
warp config set law.prec.key YOUR_PREC_KEY   # 판례 전용
warp config set law.admrul.key YOUR_ADMRUL_KEY # 행정규칙 전용
warp config set law.expc.key YOUR_EXPC_KEY   # 법령해석례 전용
```

> **참고**: ELIS(자치법규정보시스템)는 별도의 API 키가 필요할 수 있습니다. API별
> 키가 설정되지 않은 경우 공통 키(`law.key`)가 자동으로 사용됩니다.

### 3. 법령 검색

```bash
# 기본 검색
warp law "민법"
warp ordinance "서울"
warp precedent "음주운전"
warp admrule "개인정보"
warp interpretation "건축법"

# 통합 검색 (모든 API 동시 검색)
warp search "개인정보보호"
```

## 📖 사용법

더 자세한 사용법은 [사용자 가이드](docs/ko/user-guide.md)와
[사용 예제](docs/ko/examples.md)를 참조하세요.

### 검색 명령어

#### 법령 검색 (국가법령정보센터)

```bash
# 간단한 검색
warp law "도로교통법"

# 상세 옵션
warp law "민법" --page 2 --size 20 --format json

# 상세보기
warp law detail LAW_ID

# 개정 이력
warp law history LAW_ID
```

#### 자치법규 검색

```bash
warp ordinance "서울" --size 10
warp ordinance detail ORDINANCE_ID
```

#### 판례 검색

```bash
warp precedent "음주운전" --court "대법원" --date-from 20240101
warp precedent detail CASE_ID
```

### 📊 성능 모니터링 (신규!)

Warp CLI에서 제공하는 실시간 성능 대시보드로 API 호출 성능을 모니터링할 수 있습니다.

#### 실시간 대시보드
```bash
# 성능 대시보드 표시
warp metrics show

# 실시간 모니터링 (5초마다 갱신)
warp metrics show --refresh 5s

# 상세 분석 포함
warp metrics show --details --window 1h
```

#### 히스토리 분석
```bash
# 최근 24시간 성능 히스토리
warp metrics history --hours 24

# 특정 API 성능 분석
warp metrics history --api nlic --days 7

# API별 필터링
warp metrics history --api elis,prec --hours 12
```

#### 성능 리포트
```bash
# JSON 형식 리포트 생성
warp metrics report --output-format json

# CSV 형식으로 내보내기
warp metrics report --output-format csv

# HTML 리포트 생성
warp metrics report --output-format html > 성능리포트.html
```

#### 상세 분석
```bash
# 캐시 성능 확인
warp metrics cache

# 연결 풀 상태
warp metrics pools

# 응답시간 백분위수 분석
warp metrics latency --percentiles 50,95,99
```

**예시 출력:**
```
📊 Warp CLI 성능 대시보드 (2024-09-04 16:26:52)
────────────────────────────────────────────────────────────

🚀 시스템 상태
├─ 전체 상태: ✅ 정상
├─ 가동시간: 2h 15m 30s
├─ 메모리: 45.2 MB / 512 MB [████████░░░░] 8.8%
└─ 측정 구간: Last5Minutes

⚡ API 성능 (최근 5분)
┌─────────┬────────┬─────────┬──────────┬──────────┐
│ API     │ 요청수  │ 성공률   │ 평균시간  │ 캐시율    │
├─────────┼────────┼─────────┼──────────┼──────────┤
│ NLIC    │ 234    │ 99.1% ✅ │ 187ms    │ 73.2% 🎯 │
│ ELIS    │ 156    │ 97.4% ✅ │ 245ms    │ 65.8% ⚠️  │
│ PREC    │ 89     │ 95.5% ✅ │ 312ms    │ 52.1% ⚠️  │
└─────────┴────────┴─────────┴──────────┴──────────┘

📈 응답시간 분포
p50: 198ms ████████████████████ 100%
p95: 567ms ██████████░░░░░░░░░░  60%
p99: 891ms ██████░░░░░░░░░░░░░░  30%
```

#### 행정규칙 검색

```bash
warp admrule "개인정보" --department "행정안전부"
```

#### 법령해석례 검색

```bash
warp interpretation "건축법" --size 5
```

#### 통합 검색

```bash
# 모든 소스에서 검색
warp search "개인정보" --source all

# 특정 소스만 검색
warp search "민법" --source nlic,elis

# 출력 형식 지정
warp search "도로교통법" --format markdown
```

#### 🎯 고급 필터링 검색

```bash
# 법령 종류별 필터링
warp search "민법" --law-type 법률,시행령,시행규칙

# 부처별 필터링
warp search "개인정보" --department 개인정보보호위원회,법무부

# 날짜 범위 필터링
warp search "데이터" --from 20230101 --to 20241231
warp search "개인정보" --recent-days 30

# 상태별 필터링
warp search "민법" --status 시행중,폐지

# 정규표현식 검색
warp search --regex "개인정보.*보호" --title-only

# 최소 관련도 점수
warp search "민법" --min-score 0.8

# 복합 필터 조합
warp search "개인정보" \
    --law-type 법률 \
    --department 개인정보보호위원회 \
    --recent-days 30 \
    --status 시행중
```

#### 📋 필터 프리셋 관리

```bash
# 필터 프리셋 저장
warp filter save privacy-laws \
    --query "개인정보" \
    --law-type 법률 \
    --department 개인정보보호위원회 \
    --status 시행중

# 저장된 프리셋 목록
warp filter list

# 프리셋 상세 정보
warp filter show privacy-laws

# 프리셋으로 검색하기
warp search --filter privacy-laws

# 프리셋과 추가 옵션 조합
warp search --filter privacy-laws --recent-days 7

# 프리셋 삭제
warp filter delete privacy-laws
```

### 출력 형식

```bash
# 테이블 형식 (기본)
warp law "민법"

# JSON 형식 (프로그래밍 연동용)
warp law "민법" --format json | jq '.'

# Markdown 형식 (문서 작성용)
warp law "민법" --format markdown > laws.md

# CSV 형식 (Excel 호환, BOM 포함)
warp law "민법" --format csv > laws.csv

# HTML 형식 (웹 페이지용)
warp law "민법" --format html > laws.html

# HTML Simple (HTML 조각)
warp law "민법" --format html-simple
```

### 설정 관리

```bash
# 설정 파일 경로 확인
warp config path

# 설정 값 확인
warp config get law.key

# 설정 값 변경
warp config set law.key NEW_API_KEY
```

## ⚙️ 설정 파일

Warp는 TOML 형식의 설정 파일을 사용합니다 (`~/.pyhub/warp/config.toml`):

```toml
[law]
key = "YOUR_API_KEY"  # 공통 API 키

[law.nlic]
# key = "NLIC_KEY"   # 국가법령정보센터 전용 (선택)

[law.elis]
# key = "ELIS_KEY"   # 자치법규정보시스템 전용 (선택)

# ... 기타 API 설정
```

> **참고**: 기존 YAML 설정 파일(`config.yaml`)은 자동으로 TOML로
> 마이그레이션됩니다.

## 🏗️ 프로젝트 구조

```
warp/
├── docs/                    # 📚 문서
│   └── ko/                  # 한국어 문서
│       ├── README.md        # 문서 메인
│       ├── installation.md  # 설치 가이드
│       ├── user-guide.md    # 사용자 가이드
│       ├── examples.md      # 사용 예제
│       ├── troubleshooting.md # 문제 해결
│       └── faq.md           # FAQ
├── src/
│   ├── main.rs              # 진입점
│   ├── lib.rs               # 라이브러리 루트
│   ├── error.rs             # 에러 처리
│   ├── api/                 # API 클라이언트
│   │   ├── mod.rs           # API 모듈 정의
│   │   ├── client.rs        # 공통 클라이언트 인터페이스
│   │   ├── types.rs         # 공통 타입 정의
│   │   ├── nlic.rs          # 국가법령정보센터 클라이언트
│   │   ├── elis.rs          # 자치법규정보시스템 클라이언트
│   │   ├── prec.rs          # 판례 클라이언트
│   │   ├── admrul.rs        # 행정규칙 클라이언트
│   │   └── expc.rs          # 법령해석례 클라이언트
│   ├── cli/                 # CLI 명령어
│   │   ├── mod.rs           # CLI 모듈 정의
│   │   ├── args.rs          # 명령어 인자 정의
│   │   └── commands/        # 명령어 구현
│   │       ├── law.rs       # 법령 명령어
│   │       ├── ordinance.rs # 자치법규 명령어
│   │       ├── precedent.rs # 판례 명령어
│   │       ├── admrule.rs   # 행정규칙 명령어
│   │       ├── interpretation.rs # 법령해석례 명령어
│   │       ├── search.rs    # 통합 검색 명령어
│   │       ├── config.rs    # 설정 명령어
│   │       └── version.rs   # 버전 명령어
│   ├── config/              # 설정 관리
│   │   └── mod.rs           # 설정 파일 처리
│   └── output/              # 출력 포맷터
│       ├── mod.rs           # 출력 모듈 정의
│       └── formatter.rs     # 포맷터 구현
├── tests/                   # 테스트
├── locales/                 # 국제화 파일 (준비 중)
├── Cargo.toml               # 프로젝트 설정
├── Cargo.lock               # 의존성 잠금 파일
└── README.md                # 프로젝트 문서
```

## 🛠️ 기술 스택

- **언어**: Rust 2021 Edition
- **CLI 프레임워크**: Clap 4.5
- **비동기 런타임**: Tokio 1.40
- **HTTP 클라이언트**: Reqwest 0.12
- **직렬화**: Serde, Serde JSON/YAML
- **테이블 출력**: Comfy-table 7.1
- **색상 출력**: Colored 2.1
- **CSV 처리**: CSV 1.3

## 🧪 개발

### 테스트 실행

```bash
# 모든 테스트
cargo test

# 특정 테스트
cargo test test_api_type

# 통합 테스트
cargo test --test '*'
```

### 개발 모드 실행

```bash
# 디버그 모드
cargo run -- law "민법"

# 상세 로그 출력
RUST_LOG=debug cargo run -- law "민법" --verbose
```

### 크로스 컴파일

```bash
# Windows
cargo build --release --target x86_64-pc-windows-gnu

# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu
```

### 벤치마크 실행

프로젝트에는 성능 측정을 위한 벤치마크 도구가 포함되어 있습니다:

```bash
# 모든 벤치마크 실행
cargo bench

# API 성능 벤치마크
cargo bench --bench api_performance

# 향상된 성능 벤치마크 (캐싱, 배치 처리 등)
cargo bench --bench enhanced_performance

# 특정 벤치마크만 실행
cargo bench --bench api_performance single_search
```

벤치마크는 다음 항목들을 측정합니다:

- API 클라이언트 생성 성능
- 검색 요청 처리 시간
- 캐시 히트/미스 비율
- 병렬 처리 성능
- 배치 처리 효율성

## 📊 현재 상태

### ✅ 구현 완료

- 5개 API 클라이언트 (NLIC, ELIS, PREC, ADMRUL, EXPC)
- 7개 CLI 명령어 (law, ordinance, precedent, admrule, interpretation, search,
  config)
- 6개 출력 형식 (Table, JSON, Markdown, CSV, HTML, HtmlSimple)
- 통합 검색 (병렬 API 호출)
- 설정 관리 시스템
- 재시도 로직 (지수 백오프)
- 에러 처리 개선
- **캐싱 시스템** (다층 캐시 지원, 압축 최적화)
- **진행 상황 표시기** (실시간 검색 진행률 표시)
- **성능 최적화** (연결 풀링, 적응형 배치 처리)
- **개발자 도구** (벤치마크 및 성능 측정 지원)

### 🚧 알려진 이슈

- API 응답이 단일 결과일 때 파싱 에러 (배열/객체 처리)
- ELIS API는 별도 API 키 필요
- ADMRUL, EXPC API 일부 검색어 결과 없음
- 페이지네이션 2페이지 이상 조회 시 결과 없음

### 📝 개발 예정

- [ ] 국제화 (i18n) 지원
- [x] Shell 자동완성 (완료 - 자동 감지 포함)
- [x] 캐싱 시스템 (완료 - 다층 캐시, 압축 최적화)
- [x] 진행 상황 표시기 (완료 - 실시간 진행률 표시)
- [ ] 일괄 처리 기능
- [ ] 검색 필터 고급 옵션

## 🤝 기여

프로젝트에 기여를 환영합니다!

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 라이선스 및 구매

**WARP CLI는 상업용 소프트웨어입니다.** 사용하려면 유효한 라이센스가 필요합니다.

### 💳 라이센스 유형 및 가격

| 라이센스       | 가격        | 사용자 수  | 지원        |
| -------------- | ----------- | ---------- | ----------- |
| **Personal**   | ₩99,000/년  | 1명 (개인) | 이메일 지원 |
| **Team**       | ₩490,000/년 | 최대 10명  | 우선 지원   |
| **Enterprise** | 문의        | 무제한     | 전담 지원   |

### 📋 평가판

30일 무료 평가판을 제공합니다. 평가판 다운로드:

```bash
# 평가판 다운로드 (30일 제한)
wget https://github.com/pyhub-apps/warp/releases/latest/download/warp-trial.tar.gz
```

### 📞 라이센스 구매 문의

- **이메일**: me@pyhub.kr
- **기술지원**: me@pyhub.kr
- **웹사이트**: https://github.com/pyhub-apps/warp

### 📜 라이센스 조건

본 소프트웨어는 독점 상업용 라이센스로 보호됩니다. 자세한 내용은 다음을
참조하세요:

- [LICENSE](LICENSE) - 라이센스 전문
- [EULA.md](EULA.md) - 최종 사용자 라이센스 동의서

**⚠️ 중요**: 유효한 라이센스 없이 사용 시 저작권법 위반이 될 수 있습니다.

## 🔗 관련 프로젝트

- [Go 버전](https://github.com/pyhub-apps/warp-golang)
- [Python 버전](https://github.com/pyhub-kr/pyhub-warp-cli) (원본)

## 👥 만든 사람

- PyHub Korea Community
- Contributors

## 📞 문의

- GitHub Issues:
  [https://github.com/pyhub-apps/warp/issues](https://github.com/pyhub-apps/warp/issues)
- Email: me@pyhub.kr
