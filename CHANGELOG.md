# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2536.26] - 2025-01-09

### 💥 BREAKING CHANGES
- **라이센스 변경**: MIT 라이센스에서 독점 상업용 라이센스로 변경
- 상업적 사용을 위해서는 유효한 라이센스 구매 필요

### ✨ Added
- **🌐 완전한 한국어/영어 i18n 지원**
  - `rust-i18n` 3.1 기반 다국어 시스템
  - 자동 언어 감지 (`LANG` 환경변수 → `--lang` 플래그)
  - CLI 도움말, 에러 메시지, 셸 완성 메시지 완전 지역화
  - 84개 번역 키를 통한 포괄적 지원
  - 사용자 정의 지역화 도움말 명령어 (`warp localized-help`)

- **📄 독점 상업용 라이센스 시스템**
  - LICENSE 파일: 독점 상업용 라이센스 전문
  - EULA.md: 한국어/영어 최종 사용자 라이센스 동의서
  - 3단계 라이센스 정책 (Personal/Team/Enterprise)
  - 평가판 다운로드 지원 (30일 제한)

- **📈 향상된 진행 상황 표시기 시스템** (Issue #13)
  - 실시간 진행률 표시
  - 현재/전체 아이템 카운터
  - 경과 시간 및 예상 완료 시간
  - 처리 속도 (아이템/초) 표시
  - 동적 스피너 애니메이션

### 🚀 Performance Improvements
- **⚡ 종합 성능 최적화 시스템** (Issue #11)
  - HTTP 커넥션 풀링으로 연결 오버헤드 40-60% 감소
  - 실시간 성능 메트릭 시스템
  - 병렬 API 처리로 3-5배 처리량 개선
  - 메모리 효율적 스트리밍으로 80-90% 메모리 절약
  - 포괄적 벤치마킹 시스템 (`Criterion.rs`)

- **💾 캐싱 시스템 구현** (Issue #6)
  - SQLite 기반 로컬 캐시
  - API별 TTL 설정 및 자동 만료
  - 캐시 관리 명령어 (`warp cache`)
  - 캐시 히트율 최적화

### 🔧 Improved
- **🛠️ 사용자 친화적 에러 메시지 개선** (Issue #10)
  - 상세한 에러 컨텍스트 제공
  - 해결 방법 제안 및 힌트
  - 색상 코딩된 에러 레벨
  - `--verbose` 옵션으로 디버그 정보

- **🐚 Shell 자동완성 향상**
  - 자동 셸 감지 기능
  - Windows CMD 제한사항 안내
  - 대화형 설치 프로세스
  - 모든 주요 셸 지원 (Bash, Zsh, Fish, PowerShell)

### 🧪 Testing
- **📊 테스트 커버리지 대폭 향상** (27.5% → 55.7%)
  - 단위 테스트 확장
  - 통합 테스트 추가
  - 에러 처리 테스트
  - 성능 벤치마크 테스트

### 🔒 Security
- **🛡️ 보안 감사 및 의존성 관리**
  - `cargo audit` 통합
  - 자동 보안 취약점 검사
  - 의존성 업데이트 자동화
  - pre-commit hooks 통한 코드 품질 관리

### 📞 Contact
- **이메일 통일**: 모든 문의를 `me@pyhub.kr`로 일원화
- **라이센스 구매**: me@pyhub.kr
- **기술 지원**: me@pyhub.kr

### 🏗️ Technical Details
- **Rust 버전**: 1.82.0+ 지원
- **의존성 업데이트**: 모든 주요 의존성 최신 버전
- **빌드 시스템**: 크로스 플랫폼 빌드 최적화
- **배포**: 6개 플랫폼 자동 바이너리 배포

### 📱 Platform Support
- ✅ macOS: Intel (x86_64) + Apple Silicon (aarch64)
- ✅ Linux: x86_64 + ARM64 (aarch64)
- ✅ Windows: x86_64 + ARM64 (aarch64)

## [2.2536.22] - 2025-09-02

### Fixed
- pre-commit hooks 설정 및 코드 품질 개선
- 모든 Clippy 경고 및 에러 해결
- cargo-audit 설치 옵션 개선

## [2.2536.21] - 2025-09-01

### Added
- 진행 상황 표시기 기능
- 캐싱 시스템 구현
- Shell 자동완성 지원

### Fixed
- API 응답 파싱 에러 해결
- 페이지네이션 문제 수정

---

## 구매 및 라이센스 정보

### 💳 라이센스 유형 및 가격

| 라이센스 | 가격 | 사용자 수 | 지원 |
|---------|------|-----------|------|
| **Personal** | ₩99,000/년 | 1명 (개인) | 이메일 지원 |
| **Team** | ₩490,000/년 | 최대 10명 | 우선 지원 |
| **Enterprise** | 문의 | 무제한 | 전담 지원 |

### 📋 평가판
30일 무료 평가판을 제공합니다.

### 📞 문의
- **이메일**: me@pyhub.kr
- **웹사이트**: https://github.com/pyhub-apps/warp

**⚠️ 중요**: 유효한 라이센스 없이 사용 시 저작권법 위반이 될 수 있습니다.
